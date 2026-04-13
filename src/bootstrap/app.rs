use crate::adapters::inbound::cli::CliArgs;
use crate::bootstrap::http_server;
use crate::bootstrap::repo_target;
use crate::error::AppError;
use clap::Parser;

pub async fn run() -> Result<(), AppError> {
    let args = CliArgs::parse();
    let target = repo_target::resolve(args.path)?;

    http_server::serve(&target, args.port).await?;

    Ok(())
}
