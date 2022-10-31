#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { exec } from "child_process";

const cargoDir = path.dirname("$HOME" + ".cargo");

// check if directory exists
if (fs.existsSync(cargoDir)) {
  //   console.log("Cargo found.");
} else {
  console.error(
    "`cargo` is not installed. This package requires Rust: https://www.rust-lang.org/"
  );
  process.exit(1);
}

const features = process.env.npm_config_features
  ? `--features ${process.env.npm_config_features.replace(",", " ")}`
  : "";

console.log(`Compiling graphql-2-ts 0.1.0 ${features} ...`);
exec(`cargo build --release ${features}`, (error, stdout, stderr) => {
  console.log(stdout);
  if (error || stderr) {
    console.error(error || stderr);
  } else {
    console.log("install finished!");
  }
});
