#!/usr/bin/env node

import { spawn } from "child_process";
import { fileURLToPath } from "url";
import { dirname } from "path";
import { cwd } from "node:process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

let proc = spawn("cargo", [
  "run",
  __dirname,
  "--release",
  "--quiet",
  "--",
  cwd(),
]);

proc.stdout.on("data", (data) => console.log(data.toString()));

proc.stderr.on("data", (data) => console.error(data.toString()));

proc.on("rust bin returned an error:", (error) => {
  console.error(`error: ${error.message}`);
});
