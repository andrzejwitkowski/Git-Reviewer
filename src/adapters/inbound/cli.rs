use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "git-reviewer")]
#[command(about = "Local Git diff reviewer")]
pub struct CliArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long)]
    pub port: Option<u16>,
}
