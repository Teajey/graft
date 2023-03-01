const fs = require("fs");
const glob = require("glob");

module.exports.fetchJson = async function (url, noSsl, options) {
  const { default: fetch } = await import("node-fetch");
  let optionsObj = Object.fromEntries(options);
  if (noSsl) {
    const { Agent } = await import("https");
    const agent = new Agent({ rejectUnauthorized: false });
    optionsObj = {
      ...optionsObj,
      agent,
    };
  }
  const resp = await fetch(url, optionsObj);
  return await resp.json();
};

module.exports.readFileToString = function (path) {
  return fs.readFileSync(path, { encoding: "utf8" });
};
