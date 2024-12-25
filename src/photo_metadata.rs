use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PhotoMetadata {
    pub coordinates: Option<(f64, f64)>,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::cache;

//     #[test]
//     fn serde() {
//         let md = PhotoMetadata {
//             coordinates: Some((1.2, 3.4)),
//         };
//         let ser = serde_json::to_string(&md).unwrap();

//         assert_eq!(ser, "{\"coordinates\":[1.2,3.4]}");

//         let des: PhotoMetadata = serde_json::from_str(&ser).unwrap();

//         assert_eq!(des.coordinates, Some((1.2, 3.4)));
//     }
// }
