#!/usr/bin/env node

import { exec } from "child_process";
import { fileURLToPath } from "url";
import path, { dirname } from "path";
import { cwd } from "node:process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const controller =
  typeof AbortController !== "undefined"
    ? new AbortController()
    : { abort: () => {}, signal: undefined };
const { signal } = controller;

exec(
  path.join(__dirname, `target/release/graphql-2-ts ${cwd()}`),
  { signal },
  (error, stdout, stderr) => {
    stdout && console.log(stdout);
    stderr && console.error(stderr);
    if (error !== null) {
      console.log(`exec error: ${error}`);
    }
  }
);

process.on("SIGTERM", () => {
  controller.abort();
});
