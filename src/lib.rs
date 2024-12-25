use std::{
    path::{Path, PathBuf},
    process::Command,
};

use geo::{Distance, Haversine, Point};

pub use cache::Cache;
pub use photo_metadata::PhotoMetadata;

pub mod cache;
pub mod photo_metadata;

pub struct Searcher {
    radius: f64,
    target_loc: (f64, f64),
    early_stop_count: isize,
    sort_by_distance: bool,
    verbose: bool,
    cache: Cache,
}

impl Searcher {
    pub fn new(
        radius: f64,
        target_loc: (f64, f64),
        early_stop_count: isize,
        sort_by_distance: bool,
        verbose: bool,
        cache: Cache,
    ) -> Searcher {
        Searcher {
            radius,
            target_loc,
            early_stop_count,
            sort_by_distance,
            verbose,
            cache,
        }
    }

    pub fn filter_by_path_str(&self, path_str: &str) -> Option<FilterResult> {
        let path = Path::new(&path_str);
        self.user_msg(&format!("{}", path.to_str().unwrap()));

        let filter_result = match self.filter_file(path) {
            Some(result) => result,
            None => {
                self.user_msg(&format!("Skipping {}", path.to_str().unwrap()));
                return None;
            }
        };
        if filter_result.selected {
            self.user_msg(&format!(
                "{}\t{}",
                filter_result.distance,
                path.to_string_lossy()
            ));
            Some(filter_result)
        } else {
            None
        }
    }

    pub fn print_result(&self, mut found: Vec<FilterResult>) {
        if self.sort_by_distance {
            found.sort_by(|a, b| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap()
                    .then_with(|| a.path.to_string_lossy().cmp(&b.path.to_string_lossy()))
            });
        }

        self.user_msg(&format!("Found {} images", found.len()));
        for f in found.iter() {
            self.user_msg(&format!("{}\t{}", f.distance, f.path.to_string_lossy()));
        }
        for f in found.iter() {
            println!("{}", f.path.to_string_lossy());
        }
    }

    fn filter_file(&self, path: &Path) -> Option<FilterResult> {
        let ext = path.extension().unwrap().to_ascii_lowercase();

        // Read from cache or file
        let key = self.path_to_key(path);
        let path_str = path.to_string_lossy();
        let cache_read: Option<PhotoMetadata> = self.cache.read_into(&key);
        let (lat, lon) = match cache_read {
            Some(metadata) => metadata.coordinates.unwrap(),
            // Cache miss
            None => {
                self.user_msg(&format!("Exif cache miss"));
                let (lat, lon) = match ext.to_str() {
                    Some("jpg") => match read_exif_kamadak(path) {
                        Ok(Some(coords)) => coords,
                        Ok(None) => {
                            self.user_msg(&format!(
                                "Found no coordinates by primary parser in {}",
                                path_str
                            ));
                            // TODO: cache none coords
                            return None;
                        }
                        // FIXME: Log error msg
                        Err(_) => match read_exif_exiftool(path) {
                            Some(coords) => coords,
                            _ => {
                                self.user_msg(&format!(
                                    "Found no coordinates by secondary parser in {}",
                                    path_str
                                ));
                                return None;
                            }
                        },
                    },
                    Some("mp4") => match read_exif_exiftool(path) {
                        Some(coords) => coords,
                        _ => {
                            self.user_msg(&format!(
                                "Found no coordinates by secondary parser {}",
                                path_str
                            ));
                            return None;
                        }
                    },
                    _ => {
                        self.user_msg(&format!("Unsupported type for {}", path_str));
                        return None;
                    }
                };
                let md = PhotoMetadata {
                    coordinates: Some((lat, lon)),
                };
                self.cache.write_from(&key, &md);
                (lat, lon)
            }
        };

        let dist = compute_distance(self.target_loc, (lat, lon));

        Some(FilterResult {
            path: path.to_path_buf(),
            selected: dist <= self.radius,
            distance: dist,
        })
    }

    fn user_msg(&self, msg: &str) {
        if self.verbose {
            eprintln!("{}", msg);
        }
    }

    fn path_to_key(&self, path: &Path) -> String {
        path.to_string_lossy()
            .replace("/", "__")
            .replace("\\", "__")
    }
}

pub fn visit_paths(src_root: &str) -> impl Iterator<Item = String> {
    let root_path = Path::new(src_root);
    globwalk::glob(root_path.join("**/*.{jpg,mp4}").to_str().unwrap())
        .unwrap()
        .map(|item| item.unwrap())
        .map(|item| item.into_path().to_string_lossy().to_string())
}

pub struct FilterResult {
    pub path: PathBuf,
    pub distance: f64,
    selected: bool,
    // add time info
}

// Much slower than using rust lib
fn read_exif_exiftool(path: &Path) -> Option<(f64, f64)> {
    let output = Command::new("exiftool")
        .arg("-json")
        .arg(path)
        .output()
        .unwrap();
    let json = String::from_utf8_lossy(&output.stdout);
    // println!("exif out: {}", &json);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let lat: String;
    if let Some(lat_str) = value[0]["GPSLatitude"].as_str() {
        lat = str::replace(lat_str, " deg", "°");
    } else {
        return None;
    }
    let lon = str::replace(
        value[0]["GPSLongitude"]
            .as_str()
            .expect("found no longitude"),
        " deg",
        "°",
    );

    let lat_f: f64 = latlon::parse_lat(lat).unwrap();
    let lon_f: f64 = latlon::parse_lng(lon).unwrap();
    Some((lat_f, lon_f))
}

fn read_exif_kamadak(path: &Path) -> Result<Option<(f64, f64)>, exif::Error> {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    // Latitude
    let lat_ref = match &exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY) {
        Some(field) => field.display_value(),
        _ => return Ok(None),
    };
    let lat = match &exif
        .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
        .unwrap()
        .value
    {
        exif::Value::Rational(lat_rational) => coord_rational_to_f64(lat_rational, lat_ref),
        exif::Value::SRational(lat_rational) => coord_srational_to_f64(lat_rational, lat_ref),
        _ => return Ok(None),
    };

    // Longitude
    let lon_ref = match &exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY) {
        Some(field) => field.display_value(),
        _ => return Ok(None),
    };
    let lon = match &exif
        .get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
        .unwrap()
        .value
    {
        exif::Value::Rational(lon_rational) => coord_rational_to_f64(lon_rational, lon_ref),
        exif::Value::SRational(lon_rational) => coord_srational_to_f64(lon_rational, lon_ref),
        _ => return Ok(None),
    };

    Ok(Some((lat, lon)))
}

// FIXME: refactor rational vs srational
fn coord_rational_to_f64<T: ToString>(coord_rational: &Vec<exif::Rational>, coord_ref: T) -> f64 {
    let sign = if coord_ref.to_string() == "W" { -1 } else { 1 };
    sign as f64
        * (coord_rational[0].to_f64()
            + coord_rational[1].to_f64() / 60.0
            + coord_rational[2].to_f64() / 60.0 / 60.0)
}

fn coord_srational_to_f64<T: ToString>(coord_rational: &Vec<exif::SRational>, coord_ref: T) -> f64 {
    let sign = if coord_ref.to_string() == "W" { -1 } else { 1 };
    sign as f64
        * (coord_rational[0].to_f64()
            + coord_rational[1].to_f64() / 60.0
            + coord_rational[2].to_f64() / 60.0 / 60.0)
}

fn compute_distance((lat0, lon0): (f64, f64), (lat1, lon1): (f64, f64)) -> f64 {
    let loc0 = Point::new(lon0, lat0);
    let loc1 = Point::new(lon1, lat1);
    let dist = Haversine::distance(loc0, loc1);
    dist
}

fn json_to_coords(json_response: serde_json::Value) -> (f64, f64) {
    let coords = &json_response["coordinates"];
    (coords[0].as_f64().unwrap(), coords[1].as_f64().unwrap())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn compare_read_exif() {
        let path = Path::new("samples/sample.jpg");

        let loc_exiftool = read_exif_exiftool(path).unwrap();
        let loc_kamadak = read_exif_kamadak(path).unwrap().unwrap();

        assert_eq!(loc_exiftool, loc_kamadak);
    }

    #[test]
    fn test_json_to_coords() {
        let json = serde_json::json!({"coordinates": [1.2, 3.4]});
        let coords = json_to_coords(json);

        assert_eq!(coords, (1.2, 3.4));
    }

    // Smoke test yet
    #[test]
    fn filter_path() {
        let radius = 10000.0;
        let target_loc = (10.0, 10.0);
        let early_stop_count = 10;
        let sort_by_distance = true;
        let verbose = true;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        let exif_cache_dir = temp_path.join("exif");
        fs::create_dir_all(&exif_cache_dir).expect("Error creating exif cache dir");
        let exif_cache = Cache::new(&exif_cache_dir);

        let searcher = Searcher::new(
            radius,
            target_loc,
            early_stop_count,
            sort_by_distance,
            verbose,
            exif_cache,
        );

        // Read from photo
        let photo_dir = temp_path.join("photos");
        fs::create_dir_all(&photo_dir).expect("Error creating temp photos dir");
        let sample_photo_path = photo_dir.join("sample.jpg");
        fs::copy("samples/sample.jpg", &sample_photo_path).unwrap();
        searcher.filter_by_path_str(sample_photo_path.to_str().unwrap());

        // Read from cache
        fs::remove_file(&sample_photo_path).unwrap();
        searcher.filter_by_path_str(sample_photo_path.to_str().unwrap());
    }
}
