const tdl = require("tdl");
const client = tdl.createClient({
  apiId: process.env.API_ID,
  apiHash: process.env.API_HASH,
});

client.on("error", console.error);

client.on("update", (update) => {
  if (update._ == "updateFile") {
    const { local, remote } = update.file;
    let fileName = local.path.split("/").pop();
    console.log(`${fileName} - ${remote.uploaded_size} / ${local.downloaded_size}`);

    if (remote.is_uploading_completed) {
      console.log(`Done!`);
    }
  }
});
// client.on("update", (update) => {
//   console.log("Got update:", update);
// });

module.exports = {
  client,
};
