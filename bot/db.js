const { MongoClient, ServerApiVersion } = require("mongodb");
const uri = process.env.MONGO_URI;
// Create a MongoClient with a MongoClientOptions object to set the Stable API version

let client = null;

async function init() {
  console.log("initializing db");
  client = new MongoClient(uri, {
    serverApi: {
      version: ServerApiVersion.v1,
      strict: true,
      deprecationErrors: true,
    },
  });

  try {
    await client.connect();
    await client.db("admin").command({ ping: 1 });
    return client;
  } catch (err) {
    console.log(err);
  }

  return null;
}

// { user, status, added_at, completed_at, files_id[]}
async function saveRequest(job) {}

function findTorrent(hash) {
  return client.db("db").collection("torrents").findOne({ hash });
}

// { initiator, hash, file_id[], created_at }
async function saveTorrent({ initiator, hash }) {
  const torrent = {
    initiator,
    hash,
    files: [],
    status: "downloading",
    created_at: new Date(),
  };

  await client
    .db("db")
    .collection("torrents")
    .insertOne(torrent, (err, res) => {
      if (err) {
        console.log(err);
        return;
      }

      console.log(res);
    });
}

async function setIsUploading(hash, name) {
  return await client
    .db("db")
    .collection("torrents")
    .updateOne(
      { hash, "files.name": name },
      {
        $set: {
          "files.$.status": "uploading",
        },
      }
    );
}

async function updateTorrent(hash, payload) {
  let ok = await client.db("db").collection("torrents").updateOne({ hash }, { $set: payload });
  console.log(ok);
}

async function addFileToTorrent(hash, updater) {
  return await client.db("db").collection("torrents").updateOne({ hash }, updater);
}

function torrent() {
  return client.db("db").collection("torrents");
}

async function findIncompleteUploads() {
  return await client
    .db("db")
    .collection("torrents")
    .find({ status: "upload", "files.file_id": null, "files.status": null })
    .limit(1)
    .toArray();
}

module.exports = {
  init,
  saveTorrent,
  saveRequest,
  updateTorrent,
  addFileToTorrent,
  findTorrent,
  torrent,
  setIsUploading,
  findIncompleteUploads,
};
