use std::env;
use std::process;
use srt_edit::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(err) = srt_edit::run(config) {
        eprintln!("Application erorr: {err}");
        process::exit(1);
    };
}



