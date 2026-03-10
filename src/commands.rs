use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    SET(String, String),
    GET(String),
    DELETE,
    EXISTS,
}

#[derive(Debug)]
pub struct ParseCommandError;

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        match parts.next() {
            Some("set") => {
                let key = parts.next().ok_or(ParseCommandError)?;
                let value = parts.next().ok_or(ParseCommandError)?;
                Ok(Command::SET(key.to_string(), value.to_string()))
            }
            Some("get") => {
                let key = parts.next().ok_or(ParseCommandError)?;
                Ok(Command::GET(key.to_string()))
            },
            Some("delete") => Ok(Command::DELETE),
            Some("exists") => Ok(Command::EXISTS),
            _ => Err(ParseCommandError),
        }
    }
}