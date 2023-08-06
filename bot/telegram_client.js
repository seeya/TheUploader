const tdl = require("tdl");
const client = tdl.createClient({
  apiId: process.env.API_ID,
  apiHash: process.env.API_HASH,
});

client.on("error", console.error);

// client.on("update", (update) => {
//   console.log("Got update:", update);
// });

module.exports = {
  client,
};
