use clap::Parser;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::process;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Parser)]
pub struct Config {
    pub input_filepath: std::path::PathBuf,
    pub output_filepath: std::path::PathBuf,
    #[arg(allow_negative_numbers = true)]
    pub offset_ms: i32,
}

#[derive(Debug, PartialEq)]
pub struct Timestamp {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub milliseconds: u32,
}

impl Timestamp {
    // !TODO add constraints for values (<60 for mins and secs)
    pub fn build(str: &str) -> Result<Self, ParseError> {
        let re = Regex::new(r"^(\d{2}):(\d{2}):(\d{2}),(\d{3})$").unwrap();

        let captures = match re.captures(str) {
            Some(captures) => captures,
            None => return Err(ParseError::new("Timestamp doesn't match .srt timestamp format: ".to_string() + str)),
        };

        let hours = match captures.get(1) {
            Some(hours_match) => match hours_match.as_str().parse() {
                Ok(hours) => match hours {
                    h if h <= 99 => h,
                    _ => return Err(ParseError::new(
                        format!("Malformed hours field in timestamp: {}", str)
                    ))
                }
                Err(err) => return Err(ParseError::new(
                    format!("Error parsing hours field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing hours field in timestamp: ".to_string() + str
            )),
        };

        let minutes = match captures.get(2) {
            Some(minutes_match) => match minutes_match.as_str().parse() {
                Ok(minutes) => match minutes {
                    m if m <= 59 => m,
                    _ => return Err(ParseError::new(
                        format!("Malformed minutes field in timestamp: {}", str)
                    ))
                },
                Err(err) => return Err(ParseError::new(
                    format!("Error parsing minutes field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing minutes field in timestamp: ".to_string() + str
            )),
        };

        let seconds = match captures.get(3) {
            Some(seconds_match) => match seconds_match.as_str().parse() {
                Ok(seconds) => match seconds {
                    sec if sec <= 59 => sec,
                    _ => return Err(ParseError::new(
                        format!("Malformed seconds field in timestamp: {}", str)
                    ))
                },
                Err(err) => return Err(ParseError::new(
                    format!("Error parsing seconds field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing seconds field in timestamp: ".to_string() + str
            )),
        };

        let milliseconds = match captures.get(4) {
            Some(milliseconds_match) => match milliseconds_match.as_str().parse() {
                Ok(milliseconds) =>  match milliseconds {
                    ms if ms <= 999 => ms,
                    _ => return Err(ParseError::new(
                        format!("Malformed milliseconds field in timestamp: {}", str) 
                    )) 
                },
                Err(err) => return Err(ParseError::new(
                    format!("Malformed milliseconds field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing milliseconds field in timestamp: ".to_string() + str
            )),
        };

        Ok(Timestamp {
            hours,
            minutes,
            seconds,
            milliseconds
        })

    }

    pub fn apply_offset_ms(&mut self, offset_ms: i32) {
        let total_milliseconds: i32 = (self.hours as i32 * 3_600_000) +
                            (self.minutes as i32 * 60_000) +
                            (self.seconds as i32 * 1000) +
                            self.milliseconds as i32 +
                            offset_ms;
        if total_milliseconds <= 0 {
            self.hours = 0;
            self.minutes = 0;
            self.seconds = 0;
            self.milliseconds = 0;
            return;
        }
        self.hours = (total_milliseconds/ 3_600_000) as u32;
        let remaining_milliseconds = total_milliseconds % 3_600_000;
        self.minutes = (remaining_milliseconds / 60_000) as u32;
        let remaining_milliseconds = remaining_milliseconds % 60_000;
        self.seconds = (remaining_milliseconds / 1_000) as u32;
        self.milliseconds = (remaining_milliseconds % 1_000) as u32;
    }

    pub fn to_string(&self) -> String {
        format!("{:02}:{:02}:{:02},{:03}", self.hours, self.minutes, self.seconds, self.milliseconds)
    }

}

#[derive(Debug)]
pub struct ParseError {
    pub error_message: String
}

impl ParseError {
    pub fn new(error_message: String) -> Self {
        ParseError { error_message }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let input_filepath = &config.input_filepath;
    let output_filepath = &config.output_filepath;

    let contents = fs::read_to_string(input_filepath).unwrap_or_else(|err| {
        eprintln!("Couldn't read file: {}, {}", input_filepath.to_str().expect("Input filepath should be valid String"), err);
        process::exit(1);
    });

    let input_lines: Vec<&str> = contents.lines().collect();

    let re = Regex::new(r"^(\d{2}:\d{2}:\d{2},\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2},\d{3})$").unwrap();

    let mut output_content: Vec<String> = Vec::with_capacity(input_lines.len());
    for line in contents.lines() {
        if let Some(captures) = re.captures(line) {
            let start_timestamp_str = captures.get(1).unwrap().as_str();
            let mut start_timestamp = Timestamp::build(start_timestamp_str).unwrap();
            let end_timestamp_str = captures.get(2).unwrap().as_str();
            let mut end_timestamp = Timestamp::build(end_timestamp_str).unwrap();

            start_timestamp.apply_offset_ms(config.offset_ms);
            end_timestamp.apply_offset_ms(config.offset_ms);

            let offset_line = format!("{} --> {}", start_timestamp.to_string(), end_timestamp.to_string()); 
            output_content.push(offset_line);

        } else {
            output_content.push(line.to_string());
        }
    }

    let mut output_file = File::create(output_filepath)?;
    let output_string = output_content.join("\n");
    output_file.write_all(output_string.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_valid_timestamp() {
        let valid_timestamp_str = "01:54:12,590";
        let timestamp = Timestamp::build(valid_timestamp_str).unwrap();

        assert_eq!(1, timestamp.hours);
        assert_eq!(54, timestamp.minutes);
        assert_eq!(12, timestamp.seconds);
        assert_eq!(590, timestamp.milliseconds);
        assert_eq!("01:54:12,590", timestamp.to_string());
    }

    #[test]
    fn parse_invalid_timestamp() {
        let negative_hours_str = "-15:01:01,000";
        assert!(Timestamp::build(negative_hours_str).is_err());

        let overflow_hours = "100:01:01,000";
        assert!(Timestamp::build(overflow_hours).is_err());

        let overflow_mins = "000:60:01,000";
        assert!(Timestamp::build(overflow_mins).is_err());

        let overflow_secs = "000:00:60,000";
        assert!(Timestamp::build(overflow_secs).is_err());

        let missing_hrs = ":00:00,000";
        assert!(Timestamp::build(missing_hrs).is_err());
    }

    #[test]
    fn apply_offest() {
        let mut timestamp = Timestamp::build("16:54:12,590").unwrap();
        timestamp.apply_offset_ms(100);
        
        assert_eq!(16, timestamp.hours);
        assert_eq!(54, timestamp.minutes);
        assert_eq!(12, timestamp.seconds);
        assert_eq!(690, timestamp.milliseconds);

        timestamp = Timestamp::build("10:59:59,900").unwrap();
        timestamp.apply_offset_ms(500);
        assert_eq!(11, timestamp.hours);
        assert_eq!(0, timestamp.minutes);
        assert_eq!(0, timestamp.seconds);
        assert_eq!(400, timestamp.milliseconds);
    }
}