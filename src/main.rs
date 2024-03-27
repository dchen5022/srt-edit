use regex::Regex;
use std::env;
use std::fs;
use std::process;
use std::error::Error;
use srt_edit::Config;
use srt_edit::{Timestamp, ParseError};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(err) = run(config) {
        eprintln!("Application erorr: {err}");
        process::exit(1);
    };
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let input_filepath = &config.input_filepath;

    let contents = fs::read_to_string(input_filepath).unwrap_or_else(|err| {
        eprintln!("Couldn't read file: {input_filepath}, {err}");
        process::exit(1);
    });

    let re = Regex::new(r"^(\d{2}:\d{2}:\d{2},\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2},\d{3})$").unwrap();
    for line in contents.lines() {
        if let Some(captures) = re.captures(line) {
            let start_timestamp = captures.get(1).unwrap().as_str();
            let end_timestamp = captures.get(2).unwrap().as_str();
            println!("Start timestamp: {}", start_timestamp);
            println!("End timestamp: {}", end_timestamp);
        }
    }

    Ok(())
}

