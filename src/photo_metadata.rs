use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PhotoMetadata {
    pub coordinates: Option<(f64, f64)>,
}

impl Default for PhotoMetadata {
    fn default() -> Self {
        Self { coordinates: None }
    }
}
