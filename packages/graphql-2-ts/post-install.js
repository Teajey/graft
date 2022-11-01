#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { exec } from "child_process";
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

console.log(`Compiling graphql-2-ts 0.1.0 ...`);
exec(`cargo build ${__dirname} --release`, (error, stdout, stderr) => {
  console.log(stdout);
  if (error || stderr) {
    console.error(error || stderr);
  } else {
    console.log("install finished!");
  }
});
