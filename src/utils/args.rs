/// src/utils/args.rs
/// this file is used for configure command-line interfaces

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = clap::ArgAction::Count, global = true, help = "Increase verbosity level (e.g., -v, -vv, -vvv)", conflicts_with = "quiet")]
    pub verbose: u8,
    #[arg(short, long, global = true, help = "Suppress all output")]
    pub quiet: bool,
	#[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Debug, Parser)]
pub enum Subcommands {
    /// Manage licenses
    License,
    /// Display manual pages
    Manual,
    /// Manage packages
    Pkg,
    /// Manage repositories
    Repo,
}
