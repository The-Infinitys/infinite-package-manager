use infinite_package_manager::utils::args::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);
}
