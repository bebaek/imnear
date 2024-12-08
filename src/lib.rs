use std::{
    path::{Path, PathBuf},
    process::Command,
};

use geo::{Distance, Haversine, Point};

pub struct Searcher {
    target_loc: (f64, f64),
    radius: f64,
    early_stop_count: isize,
    sort_by_distance: bool,
    verbose: bool,
}

impl Searcher {
    pub fn new(
        target_loc: (f64, f64),
        radius: f64,
        early_stop_count: isize,
        sort_by_distance: bool,
        verbose: bool,
    ) -> Searcher {
        Searcher {
            target_loc,
            radius,
            early_stop_count,
            sort_by_distance,
            verbose,
        }
    }

    pub fn filter_by_path_str(&self, path_str: &str) -> Option<FilterResult> {
        let path = Path::new(&path_str);
        self.user_msg(&format!("{}", path.to_str().unwrap()));

        let filter_result = match self.filter_file(path) {
            Some(result) => result,
            None => {
                if self.verbose {
                    eprintln!("Skipping {}", path.to_str().unwrap());
                }
                return None;
            }
        };
        if filter_result.selected {
            if self.verbose {
                eprintln!("{}\t{}", filter_result.distance, path.to_string_lossy());
            }
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
            println!("{}", f.path.to_string_lossy());
        }
    }

    fn filter_file(&self, path: &Path) -> Option<FilterResult> {
        let read_exif: fn(&Path) -> Option<(f64, f64)> = match path.extension().unwrap().to_str() {
            Some("jpg") => read_exif_kamadak,
            // Very slow fallback
            Some("mp4") => read_exif_exiftool,
            _ => return None,
        };

        let (lat, lon) = match read_exif(path) {
            Some(loc) => loc,
            None => return None,
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

fn read_exif_kamadak(path: &Path) -> Option<(f64, f64)> {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    // Handle error?
    let exif = match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => exif,
        Err(e) => {
            // Happened once for an unknown reason
            eprintln!(
                "Error reading from file {}: {:?}",
                path.to_string_lossy(),
                e
            );
            return None;
        }
    };
    let lat: f64;
    let lon: f64;

    let lat_ref = match &exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY) {
        Some(field) => field.display_value(),
        _ => return None,
    };

    if let exif::Value::Rational(lat_rational) = &exif
        .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
        .unwrap()
        .value
    {
        let sign = if lat_ref.to_string() == "W" { -1 } else { 1 };
        lat = sign as f64
            * (lat_rational[0].to_f64()
                + lat_rational[1].to_f64() / 60.0
                + lat_rational[2].to_f64() / 60.0 / 60.0)
    } else {
        return None;
    };

    let lon_ref = &exif
        .get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)
        .unwrap()
        .display_value();

    if let exif::Value::Rational(lon_rational) = &exif
        .get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
        .unwrap()
        .value
    {
        let sign = if lon_ref.to_string() == "W" { -1 } else { 1 };
        lon = sign as f64
            * (lon_rational[0].to_f64()
                + lon_rational[1].to_f64() / 60.0
                + lon_rational[2].to_f64() / 60.0 / 60.0)
    } else {
        return None;
    };

    Some((lat, lon))
}

fn compute_distance((lat0, lon0): (f64, f64), (lat1, lon1): (f64, f64)) -> f64 {
    let loc0 = Point::new(lat0, lon0);
    let loc1 = Point::new(lat1, lon1);
    let dist = Haversine::distance(loc0, loc1);
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_read_exif() {
        let path = Path::new("samples/sample.jpg");

        let loc_exiftool = read_exif_exiftool(path).unwrap();
        let loc_kamadak = read_exif_kamadak(path).unwrap();

        assert_eq!(loc_exiftool, loc_kamadak);
    }
}
