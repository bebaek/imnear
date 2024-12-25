use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use sanitize_filename::sanitize;
use serde::{de::DeserializeOwned, Serialize};
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
        let path = self.path.join(sanitize(key));
        if !path.exists() {
            let file = File::create(path).unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &json).unwrap();
        } else {
            panic!("Cache file exists: {}", path.to_string_lossy());
        }
    }

    pub fn read(&self, key: &str) -> Option<Value> {
        let path = self.path.join(sanitize(key));
        if path.exists() {
            let contents = fs::read_to_string(path).unwrap();
            let json: Value = serde_json::from_str(&contents).unwrap();
            Some(json)
        } else {
            None
        }
    }

    pub fn write_from(&self, key: &str, value: impl Serialize) {
        let path = self.path.join(sanitize(key));
        if !path.exists() {
            let ser = serde_json::to_string(&value).unwrap();
            fs::write(path, ser).unwrap();
        } else {
            panic!("Cache file exists: {}", path.to_string_lossy());
        }
    }

    pub fn read_into<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.path.join(sanitize(key));
        if path.exists() {
            let contents = fs::read_to_string(path).unwrap();
            let des: T = serde_json::from_str(&contents).unwrap();
            Some(des)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Debug)]
    struct Data {
        coordinates: Option<(f64, f64)>,
    }

    #[test]
    fn write_read() {
        let data = Data {
            coordinates: Some((1.2, 3.4)),
        };
        let dir = tempdir().unwrap();
        let cache_path = dir.path();
        let cache = Cache::new(cache_path);
        let key = "test-key";

        cache.write_from(key, &data);

        let saved_str = fs::read_to_string(cache_path.join(key)).unwrap();
        assert_eq!(saved_str, r#"{"coordinates":[1.2,3.4]}"#);

        let des: Data = cache.read_into(key).unwrap();

        assert_eq!(data.coordinates, des.coordinates);
    }
}
