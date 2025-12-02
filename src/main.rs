use clap::Parser;
use union_package_manager::modules::cmd::Cli;
use union_package_manager::modules::error::UpmError;
fn main() -> Result<(), UpmError> {
    let cmd = Cli::parse();
    colored::control::set_override(true);
    cmd.execute()
}
