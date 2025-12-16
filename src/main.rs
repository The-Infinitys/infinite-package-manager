use clap::Parser;
use union_package_manager::modules::cmd::Cli;
use union_package_manager::modules::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cmd = Cli::parse();
    colored::control::set_override(true);
    cmd.execute().await
}
