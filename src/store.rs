use std::collections::HashMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Write};

pub struct Store {
    file_name: String,
    data: HashMap<String, String>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    EmptyStore,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "store I/O error: {e}"),
            StoreError::EmptyStore => write!(f, "cannot save: store contains no entries"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(e) => Some(e),
            StoreError::EmptyStore => None,
        }
    }
}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

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

    pub fn save_to_disk(&self) -> Result<(), StoreError> {
        if self.data.is_empty() {
            return Err(StoreError::EmptyStore);
        }

        // Write to a temp file first so that a crash mid-write never leaves
        // a corrupt or partial store file behind.
        let tmp_path = format!("{}.tmp", self.file_name);

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?; // io::Error converts to StoreError::Io via From

        let mut writer = BufWriter::new(file);

        for (key, value) in &self.data {
            writeln!(writer, "{key}: {value}")?;
        }

        // Flush BufWriter's internal buffer to the OS before renaming.
        writer.flush()?;

        // Atomic rename: the final file is either the old version or the fully
        // written new one — never a half-written hybrid.
        fs::rename(&tmp_path, &self.file_name)?;

        Ok(())
    }
}
