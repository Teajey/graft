#!/usr/bin/env node

import { exec } from "child_process";
import { fileURLToPath } from "url";
import { dirname } from "path";
import { cwd } from "node:process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const controller =
  typeof AbortController !== "undefined"
    ? new AbortController()
    : { abort: () => {}, signal: undefined };
const { signal } = controller;

exec(
  `cargo run ${__dirname} --release --quiet -- ${cwd()}`,
  { signal },
  (_, stdout, stderr) => {
    stdout && console.log(stdout);
    stderr && console.error(stderr);
  }
);

process.on("SIGTERM", () => {
  controller.abort();
});
