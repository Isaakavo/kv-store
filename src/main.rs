mod commands;
mod store;

use commands::Command;
use std::io::{self, Write};
use std::str::FromStr;
use store::Store;

use crate::store::StoreError;

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
                Command::KEYS => store.keys(),
                Command::SAVE => match store.save_to_disk() {
                    Ok(()) => println!("Saved to disk"),
                    Err(e) => eprintln!("Could not save to disk: {e}"),
                },
                Command::LOAD => match store.load_from_disk() {
                    Ok(values) => println!("{values}"),
                    Err(e) => eprint!("Could not read from disk {e}"),
                },
                Command::CLEAR => match store.clear() {
                    Ok(()) => println!("OK"),
                    Err(StoreError::EmptyStore) => eprintln!("The store is empty, could not clear"),
                    Err(StoreError::Io(io_erro)) => eprintln!("Could not open the file {io_erro}"),
                },
                Command::EXIT => {
                    println!("GoodBye!");
                    return ();
                }
            },
            Err(_) => eprintln!("Unknown command"),
        }
    }
}
