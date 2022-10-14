#!/usr/bin/env node

import { exec } from "child_process";

const controller =
  typeof AbortController !== "undefined"
    ? new AbortController()
    : { abort: () => {}, signal: undefined };
const { signal } = controller;

exec("./target/release/graphql-2-ts", { signal }, (error, stdout, stderr) => {
  stdout && console.log(stdout);
  stderr && console.error(stderr);
  if (error !== null) {
    console.log(`exec error: ${error}`);
  }
});

process.on("SIGTERM", () => {
  controller.abort();
});
