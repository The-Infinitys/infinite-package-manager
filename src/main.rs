use clap::Parser;
use union_package_manager::modules::cmd::Cli;
use union_package_manager::modules::error::UpmError;
fn main() -> Result<(), UpmError> {
    let cmd = Cli::parse();
    cmd.execute()
}
