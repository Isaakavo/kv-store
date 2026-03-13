use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;

pub struct Store {
    file_name: String,
    data: HashMap<String, String>,
}

pub struct SaveToFileErr;

impl Store {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            file_name: "store.txt".to_string(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn delete(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn keys(&self) {
        if self.data.is_empty() {
            println!("The store is empty");
            return;
        }

        for key in self.data.keys() {
            println!("{key}");
        }
    }

    fn create_file(&self, content: &[u8])-> Result<usize, std::io::Error> {
        match File::create_new("store.txt") {
            Ok(mut file) => {
                return Ok(file.write(content)?);
            }
            Err(error) => Err(error),
        }
    }

    pub fn save_to_disk(&self) -> Result<bool, SaveToFileErr>{

        let mut content = "".to_string();

        for key in self.data.keys() {
            let value = self.get(key).expect("key doesnt exists");
            let key_value = format!("{}: {}\n", key, value);
            content = content.to_owned() + &key_value;
        }

        if content.is_empty() {
            return Err(SaveToFileErr)
        }

        let content = content.as_bytes();


        match fs::exists(self.file_name.clone()) {
            Ok(bol) => {
                if bol {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .open(self.file_name.clone())
                        .expect("Cannot open file");

                    file.write(content);
                    return Ok(true);
                }

                return match  self.create_file(content){
                    Ok(val) => {
                        if val > 0 {
                            return Ok(true)
                        }

                        return  Ok(false);
                    },
                    Err(err) => Err(SaveToFileErr)
                };
            }
            Err(e) => match self.create_file(content) {
                    Ok(val) => {
                        if val > 0 {
                            return Ok(true)
                        }

                        return  Ok(false);
                    },
                    Err(err) => Err(SaveToFileErr)
                }
        }
    }
}
