use clap::Parser;
use std::process;
use srt_edit::Config;

fn main() {
    let config = Config::parse();

    if let Err(err) = srt_edit::run(config) {
        eprintln!("Application erorr: {err}");
        process::exit(1);
    };
}



