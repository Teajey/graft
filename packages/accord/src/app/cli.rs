use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "Accord", author, version, about)]
pub struct Base {
    pub working_directory: Option<String>,
    #[arg(short, long = "config")]
    pub config_location: Option<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
