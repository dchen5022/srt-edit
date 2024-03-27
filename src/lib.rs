use regex::Regex;
use std::error::Error;

#[derive(Debug)]
pub struct Config {
    pub input_filepath: String,
    pub output_filepath: String,
    pub offset_ms: i32,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 4 {
            return Err("Not enough arguments");
        }

        let input_filepath = args[1].clone();
        let output_filepath = args[2].clone();
        let offset_ms = match args[3].clone().parse() {
            Ok(num) => num,
            Err(_) => return Err("Error parsing offset"),
        };

        Ok(Config { input_filepath, output_filepath, offset_ms })
    }
}

#[derive(Debug, PartialEq)]
pub struct Timestamp {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub milliseconds: u32,
}

impl Timestamp {
    pub fn new(str: &str) -> Result<Self, ParseError> {
        let re = Regex::new(r"(\d{2}):(\d{2}):(\d{2}),(\d{3})").unwrap();

        let captures = match re.captures(str) {
            Some(captures) => captures,
            None => return Err(ParseError::new("Timestamp doesn't match .srt timestamp format: ".to_string() + str)),
        };

        let hours = match captures.get(1) {
            Some(hours_match) => match hours_match.as_str().parse() {
                Ok(hours) => hours,
                Err(err) => return Err(ParseError::new(
                    format!("Malformed hours field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing hours field in timestamp: ".to_string() + str
            )),
        };

        let minutes = match captures.get(2) {
            Some(minutes_match) => match minutes_match.as_str().parse() {
                Ok(minutes) => minutes,
                Err(err) => return Err(ParseError::new(
                    format!("Malformed minutes field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing minutes field in timestamp: ".to_string() + str
            )),
        };

        let seconds = match captures.get(3) {
            Some(seconds_match) => match seconds_match.as_str().parse() {
                Ok(seconds) => seconds,
                Err(err) => return Err(ParseError::new(
                    format!("Malformed seconds field in timestamp: {} - {}", str, err)
                )),
            },
            None => return Err(ParseError::new(
                "Missing seconds field in timestamp: ".to_string() + str
            )),
        };

        let milliseconds = match captures.get(4) {
            Some(milliseconds_match) => match milliseconds_match.as_str().parse() {
                Ok(milliseconds) => milliseconds,
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

}

#[derive(Debug)]
pub struct ParseError {
    pub error_message: String
}

impl ParseError {
    pub fn new(error_message: String) -> Self {
        ParseError {
            error_message
        }
    }
}

pub fn run() {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_valid_timestamp() {
        let valid_timestamp_str = "16:54:12,590";
        let timestamp = Timestamp::new(valid_timestamp_str).unwrap();

        assert_eq!(16, timestamp.hours);
        assert_eq!(54, timestamp.minutes);
        assert_eq!(12, timestamp.seconds);
        assert_eq!(590, timestamp.milliseconds);
    }

    #[test]
    fn apply_offest() {
        let mut timestamp = Timestamp::new("16:54:12,590").unwrap();
        timestamp.apply_offset_ms(100);
        
        assert_eq!(16, timestamp.hours);
        assert_eq!(54, timestamp.minutes);
        assert_eq!(12, timestamp.seconds);
        assert_eq!(690, timestamp.milliseconds);

        timestamp = Timestamp::new("10:59:59,900").unwrap();
        timestamp.apply_offset_ms(500);
        assert_eq!(11, timestamp.hours);
        assert_eq!(0, timestamp.minutes);
        assert_eq!(0, timestamp.seconds);
        assert_eq!(400, timestamp.milliseconds);
    }
}