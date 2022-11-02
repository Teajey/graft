#!/usr/bin/env node

import { spawn, spawnSync } from "child_process";
import { fileURLToPath } from "url";
import { dirname } from "path";
import { argv, cwd, chdir } from "node:process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const scriptDir = __dirname;
const targetDir = cwd();
chdir(scriptDir);

let { status } = spawnSync("cargo", ["-h"]);

if (status !== 0) {
  console.error(
    "`cargo -h` returned a non-zero exit status, which probably means Rust isn't properly installed. This package requires Rust: https://www.rust-lang.org/"
  );
  process.exit(1);
}

const [, , mode = "run"] = argv;

if (mode === "build") {
  console.log(`Compiling rust dependency. This may take a moment...`);
}

let proc = spawn("cargo", [mode, "--release", scriptDir, "--", targetDir]);

proc.stdout.on("data", (data) => console.log(data.toString()));

proc.stderr.on("data", (data) => console.error(data.toString()));

proc.on("error", (error) => {
  console.error(`cargo returned an error: ${error.message}`);
  process.exit(1);
});
