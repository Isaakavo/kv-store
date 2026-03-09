mod commands;

use commands::Command;
use std::str::FromStr;
use std::io;

fn main() {
    
    loop {
        print!("> ");
        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");


        match Command::from_str(&command.trim()) {
            Ok(command) => println!("Parsed command {:?}", command),
            Err(_) => println!("Not parsed"),
        }
    }
}
