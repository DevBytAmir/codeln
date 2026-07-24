//! `CodeLn` entry point: parse args, resolve config, walk the tree, print results.

mod cli;
mod config;
mod counter;
mod filter;
mod init;
mod languages;
mod output;

use anyhow::Result;
use clap::Parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = match cli::Args::try_parse() {
        Ok(args) => args,
        Err(e) => e.exit(),
    };

    if args.init {
        return init::generate_config_file();
    }

    let config = config::Config::load(&args)?;
    let filter = filter::FileFilter::new(&config);
    let results = counter::count_lines(&config.path, &filter, &config)?;
    output::print_results(&results, &config)?;

    Ok(())
}
