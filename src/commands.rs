use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    SET(String, String),
    GET(String),
    DELETE(String),
    EXISTS(String),
    KEYS,
    SAVE,
    LOAD,
    CLEAR,
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
            }
            Some("delete") => {
                let key = parts.next().ok_or(ParseCommandError)?;
                Ok(Command::DELETE(key.to_string()))
            }
            Some("exists") => {
                let key = parts.next().ok_or(ParseCommandError)?;
                Ok(Command::EXISTS(key.to_string()))
            }
            Some("keys") => Ok(Command::KEYS),
            Some("save") => Ok(Command::SAVE),
            Some("load") => Ok(Command::LOAD),
            Some("clear") => Ok(Command::CLEAR),
            _ => Err(ParseCommandError),
        }
    }
}
