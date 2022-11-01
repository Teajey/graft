#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { spawn } from "child_process";
import { fileURLToPath } from "url";
import { dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const cargoDir = path.dirname("$HOME" + ".cargo");

// check if directory exists
if (fs.existsSync(cargoDir)) {
  //   console.log("Cargo found.");
} else {
  console.error(
    "`cargo` does not appear to be installed ($HOME/.cargo not found). This package requires Rust: https://www.rust-lang.org/"
  );
  process.exit(1);
}

console.log(`Compiling rust dependency. This may take a moment...`);

let proc = spawn("cargo", ["build", __dirname, "--release"]);

proc.stdout.on("data", (data) => console.log(data.toString()));

proc.stderr.on("data", (data) => console.error(data.toString()));

proc.on("cargo build returned an error:", (error) => {
  console.error(`error: ${error.message}`);
});
