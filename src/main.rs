mod commands;
mod store;

use commands::Command;
use store::Store;
use std::str::FromStr;
use std::io;
use std::collections::HashMap;

fn main() {
    let mut store = Store {
        store: HashMap::new(),
    };

    loop {
        print!("> ");
        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");

        match Command::from_str(&command.trim()) {
            Ok(command) => {
                match command {
                    Command::SET(key, value) => {
                        println!("Detected values {:?}-{:?}", key, value);
                        store.store.insert(key, value);
                    },
                    Command::GET(key) => {
                         match store.store.get(&key) {
                            Some(value) => println!("Value for key {:?}: {:?}", key, value),
                            None => println!("The key does not exists")
                        }
                        
                    },
                    _ => println!("Not implemented yet"),
                }
            },
            Err(_) => println!("Not parsed, command not available"),
        }
    }
}
