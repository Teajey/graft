#!/usr/bin/env node

import { spawn, spawnSync } from "child_process";
import { fileURLToPath } from "url";
import { dirname } from "path";
import { argv, cwd, chdir } from "node:process";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

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

const {
  argv: { mode = "run", verbose = 0 },
} = yargs(hideBin(argv))
  .option("verbose", {
    alias: "v",
    count: true,
    type: "number",
    description: "0 = no cargo output, 1 = cargo stderr, 2 = cargo stdout",
  })
  .option("mode", {
    type: "string",
    description: "cargo subcommand",
  })
  .version(false)
  .help()
  .strict();

if (mode === "build") {
  console.log(`Compiling rust dependency. This may take a moment...`);
}

let proc = spawn("cargo", [mode, "--release", scriptDir, "--", targetDir]);

if (verbose >= 2) {
  proc.stdout.on("data", (data) => console.log(data.toString()));
}

if (verbose >= 1) {
  proc.stderr.on("data", (data) => console.error(data.toString()));
}

proc.on("error", (error) => {
  console.error(`cargo returned an error: ${error.message}`);
  process.exit(1);
});
