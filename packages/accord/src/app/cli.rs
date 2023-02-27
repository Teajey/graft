use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Mode {
    Typescript,
    All,
}

#[derive(Parser)]
#[command(name = "Graft", author, version, about)]
pub struct Base {
    #[clap(subcommand)]
    pub mode: Option<Mode>,
    pub working_directory: Option<PathBuf>,
    #[arg(short, long = "config")]
    pub config_location: Option<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
