module.exports.fetchJson = async function (url, options) {
  const { default: fetch } = await import("node-fetch");
  const optionsObj = Object.fromEntries(options);
  const resp = await fetch(url, optionsObj);
  return await resp.json();
};
