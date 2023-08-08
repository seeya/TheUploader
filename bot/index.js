const fs = require("fs");
const path = require("path");
const http = require("http");
const https = require("https");
const TelegramBot = require("node-telegram-bot-api");
const {
  init: initDB,
  saveTorrent,
  updateTorrent,
  findTorrent,
  addFileToTorrent,
  setIsUploading,
  torrent,
  saveRequest,
  findIncompleteUploads,
} = require("./db");
const { execSync } = require("child_process");
const { client } = require("./telegram_client");

function magnetToTorrent(hash, cb) {
  const stream = fs.createWriteStream(`${process.env.TORRENT_JOB_FOLDER}/${hash}.torrent`);
  const url = `https://itorrents.org/torrent/${hash}.torrent`;

  const req = https.get(url, (res) => {
    res.pipe(stream);
    res.on("end", cb);
    res.on("error", (err) => cb(err));
  });

  req.end();
}

function linkToTorrent(url, cb) {
  const now = Date.now();
  const stream = fs.createWriteStream(`${process.env.TORRENT_JOB_FOLDER}/${now}.torrent`);

  const req = https.get(url, (res) => {
    if (res.statusCode === 302 || res.statusCode === 301) {
      const newUrl = res.headers.location;
      linkToTorrent(newUrl);
    } else {
      res.pipe(stream);
      res.on("end", cb);
      res.on("error", (err) => cb(err));
    }
  });

  req.end();
}

http
  .createServer((req, res) => {
    console.log(req.url);
    const splitted = req.url.split("/");

    if (splitted[splitted.length - 2] === "job") {
      const hash = splitted[splitted.length - 1].toUpperCase();
      console.log("hash done", hash);

      let body = "";
      req.on("data", (chunk) => (body += chunk));
      req.on("end", async () => {
        // Check if local folder path exists
        const basePath = body.replaceAll('"', "").replaceAll("'", "");
        let files = [];

        if (fs.existsSync(basePath)) {
          try {
            fs.readdirSync(basePath).forEach((fileName) => {
              const fullPath = path.join(basePath, fileName);

              const stats = fs.statSync(fullPath);
              if (stats.size > 1000 && stats.size < 4e9) {
                console.log(fileName, stats.size);
                if (!stats.isDirectory()) {
                  files.push({
                    name: fileName,
                    file_id: null,
                    size: stats.size,
                  });
                }
              } else {
              }
            });
          } catch (err) {
            files.push({
              name: basePath.split("/").pop(),
              file_id: null,
            });
          }
        } else {
          await updateTorrent(hash, {
            status: "failed",
            completed_at: new Date(),
          });
          return;
        }

        let status = files.length == 0 ? "failed" : "upload";
        console.log("Creating jobs to upload", files);
        await updateTorrent(hash, {
          status,
          files,
          base_path: basePath.split("/").pop(),
          completed_at: new Date(),
        });
      });
    }

    res.end("Hello World");
  })
  .listen(process.env.PORT || 3000, async () => {
    console.log("listening on port 3000");
    const dbClient = await initDB();

    initBot(dbClient);
    await client.login();

    // Important step: we need to get the chat list before we can send messages
    await client.invoke({
      _: "getChats",
      chat_list: { _: "chatListMain" },
      limit: 100,
    });

    (async () => {
      while (true) {
        const incomplete = await findIncompleteUploads();
        console.log(`Finding Incomplete Jobs: ${incomplete.length}`);

        for (let i = 0; i < incomplete.length; i++) {
          const job = incomplete[i];
          const { hash, files, base_path } = job;
          const rootPath = process.env.TORRENT_DONE_PATH;

          for (let f = 0; f < files.length; f++) {
            const file = files[f];
            const fullPath = `${rootPath}/${base_path}/${file.name}`;

            if (file.file_id != null) continue;

            try {
              console.log(`Uploading ${file.name}`);
              await client.invoke({
                _: "sendMessage",
                chat_id: process.env.SEND_TO,
                input_message_content: {
                  _: "inputMessageDocument",
                  document: {
                    _: "inputFileLocal",
                    path: fullPath,
                  },
                },
                caption: {
                  _: "formattedText",
                  text: hash,
                  entities: null,
                },
              });

              await setIsUploading(hash, file.name);
            } catch (err) {
              console.log("failed to upload", err);
            }
          }
        }

        await sleep(100 * 1000);
      }
    })();
  });

function initBot(dbClient) {
  console.log("Initializing Bot");

  const bot = new TelegramBot(process.env.TELEGRAM_TOKEN, { polling: true });

  bot.onText(/magnet:/, (msg, match) => {
    const { from, text } = msg;
    const { id, first_name, username } = from;

    const hash = text.split("urn:btih:")[1].split("&")[0].toUpperCase();
    console.log(`Magnet request, downloading: ${hash}`);
    magnetToTorrent(hash, async (err) => {
      if (err) {
        console.log("failed to download magnet torrent", err);
        return;
      }

      let exists = await findTorrent(hash);
      if (exists) {
        console.log("already exists, just forward");
      } else {
        await saveTorrent({
          initiator: {
            id,
            first_name,
            username,
          },
          hash,
        });

        bot.sendMessage(id, "Torrent added to queue");
      }
    });
  });

  bot.onText(/\.torrent$/, (msg, match) => {
    const { from, text } = msg;
    const { id, first_name, username } = from;
    console.log(`Torrent link found, downloading: ${text}`);
    if (text.indexOf("http" !== -1)) {
      linkToTorrent(text, (err) => {
        console.log("failed to download torrent", err);
      });
    }
  });

  bot.onText(/^http*/, (msg, match) => {
    const { from, text } = msg;
    const { id, first_name, username } = from;
    console.log(`Torrent link found, downloading: ${text}`);
    if (text.indexOf("http" !== -1)) {
      linkToTorrent(text, (err) => {
        console.log("failed to download torrent", err);
      });
    }
  });

  bot.on("document", async (msg) => {
    const { file_id, mime_type } = msg.document;
    if (mime_type == "application/x-bittorrent") {
      try {
        // let r = await bot.downloadFile(file_id, process.env.TORRENT_JOB_FOLDER);
        // bot.sendMessage(msg.from.id, "Torrent added to queue");
        // console.log(r);
        // await saveTorrent({
        //   initiator: {
        //     id,
        //     first_name,
        //     username,
        //   },
        //   hash,
        // });
        bot.sendMessage(msg.from.id, "Sorry, no torrent files allowed for now.");
      } catch (err) {
        console.log(err);
      }
    } else {
      fileHandler(msg);
    }
  });
  bot.on("audio", fileHandler);
}

async function fileHandler(msg) {
  const { from, document, audio, caption } = msg;
  if (from.id != parseInt(process.env.ADMIN_ID)) {
    console.log("unknown user sending us file");
    return;
  }

  const { file_name, mime_type, file_id, file_unique_id, file_size } = document ? document : audio;

  console.log(`Saving: ${file_name} - ${file_id}`);

  await torrent().updateOne(
    {
      // hash: caption,
      "files.name": file_name,
      "files.file_id": null,
    },
    {
      $set: {
        "files.$.file_id": file_id,
        "files.$.file_unique_id": file_unique_id,
        "files.$.mime_type": mime_type,
        "files.$.file_size": file_size,
        "files.$.status": "success",
      },
    }
  );
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
