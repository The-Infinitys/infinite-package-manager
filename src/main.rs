use infinite_package_manager::utils::args::Args;
use clap::Parser;
use env_logger;
use log;

fn main() {
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

    log::info!("Application started.");
    log::debug!("Parsed arguments: {:#?}", args);

    println!("{:#?}", args);
}
