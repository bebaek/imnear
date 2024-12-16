use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use serde_json::Value;

// FIXME: implement evict
pub struct Cache {
    path: PathBuf,
}

impl Cache {
    pub fn new(path: &Path) -> Cache {
        fs::create_dir_all(&path).expect("Error creating cache dir");
        Cache {
            path: path.to_path_buf(),
        }
    }

    pub fn write(&self, key: &str, json: Value) {
        // FIXME: sanitize key for filename
        let path = self.path.join(key);
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &json).unwrap();
    }

    pub fn read(&self, key: &str) -> Option<Value> {
        let path = self.path.join(key);
        if path.exists() {
            let contents = fs::read_to_string(path).unwrap();
            let json: Value = serde_json::from_str(&contents).unwrap();
            eprintln!("cache read: {:?}", json);
            Some(json)
        } else {
            None
        }
    }
}
