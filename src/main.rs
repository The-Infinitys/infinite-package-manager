use clap::Parser;
use env_logger;
use infinite_package_manager::utils::args::{Args, Subcommands};
use log;
use infinite_package_manager::Error;
use infinite_package_manager::command;

fn main()->Result<(), Error> {
    let args = Args::parse();

    let mut log_builder = env_logger::Builder::new();

    if args.quiet {
        log_builder.filter_level(log::LevelFilter::Off);
    } else {
        match args.verbose {
            0 => log_builder.filter_level(log::LevelFilter::Info),
            1 => log_builder.filter_level(log::LevelFilter::Debug),
            _ => log_builder.filter_level(log::LevelFilter::Trace),
        };
    }

    log_builder.init();
    match &args.command {
    	Subcommands::License => command::license(),
        Subcommands::Manual => command::manual(),
        Subcommands::Pkg => command::pkg(),
        Subcommands::Repo => command::repo(),
    }?;
    Ok(())
}
