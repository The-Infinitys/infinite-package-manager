use clap::Parser;
use clap::Subcommand;

use crate::modules::error::UpmError;
#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    Get,
    Post,
}

impl Cli {
    pub fn execute(&self)->Result<(),UpmError>{
        Ok(())
    }

}