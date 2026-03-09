use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    SET,
    GET,
    DELETE,
    EXISTS,
}

#[derive(Debug)]
pub struct ParseCommandError;

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set" => Ok(Command::SET),
            "get" => Ok(Command::GET),
            "delete" => Ok(Command::DELETE),
            "exists" => Ok(Command::EXISTS),
            _ => Err(ParseCommandError),
        }
    }
}