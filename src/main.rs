mod commands;
mod store;

use commands::Command;
use store::Store;
use std::io::{self, Write};
use std::str::FromStr;

fn main() {
    let mut store = Store::new();

    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read command");

        match Command::from_str(input.trim()) {
            Ok(command) => match command {
                Command::SET(key, value) => {
                    store.set(key, value);
                    println!("OK");
                }
                Command::GET(key) => match store.get(&key) {
                    Some(value) => println!("{}", value),
                    None => eprintln!("(nil)"),
                },
                Command::DELETE(key) => match store.delete(&key) {
                    Some(_) => println!("OK"),
                    None => eprintln!("Key not found"),
                },
                Command::EXISTS(key) => {
                    if store.exists(&key) {
                        println!("(integer) 1");
                    } else {
                        println!("(integer) 0");
                    }
                }
            },
            Err(_) => eprintln!("Unknown command"),
        }
    }
}
