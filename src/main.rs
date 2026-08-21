mod cli;
mod config;
mod db;
mod file_walk;

use clap::Parser;
use cli::{Cli, Commands};
use log::LevelFilter;
use std::io::Write;

fn main() {
    let args = Cli::parse();

    //Setup program logger
    env_logger::builder()
        .format(|buff, record| writeln!(buff, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Auto) //For security, logging user input should be sanitized or use WriteStyle::never
        .init();

    println!("{:#?}", args);
    match &args.command {
        Commands::Compare(compare_args) => {
            compare_args.run();
        }
        Commands::Backup(backup_args) => {
            backup_args.run();
        }
    }
}
