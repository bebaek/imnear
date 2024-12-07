use std::{
    path::{Path, PathBuf},
    process::Command,
};

use geo::{Distance, Haversine, Point};

pub fn run(
    target_loc: (f64, f64),
    radius: f64,
    dir: &str,
    early_stop_count: isize,
    sort_by_distance: bool,
    verbose: bool,
) {
    let search_dir = Path::new(dir);

    let mut found: Vec<(f64, String)> = Vec::new();

    let files = search_files(search_dir);
    if verbose {
        eprintln!("file count: {}", files.len());
    }
    let mut skip_count: u32 = 0;
    for file in files.iter() {
        let (lat, lon) = match read_exif(file) {
            Some(loc) => loc,
            None => {
                skip_count += 1;
                if verbose {
                    eprintln!("Skipping {}", file.to_str().unwrap());
                }
                continue;
            }
        };
        // println!("lat: {}", lat);
        // println!("lon: {}", lon);

        let dist = compute_distance(target_loc, (lat, lon));

        // println!("file: {:?}", file);
        // println!("distance: {}", dist);

        if dist <= radius {
            if verbose {
                eprintln!("{}\t{}", dist, file.to_string_lossy());
            } else if !sort_by_distance {
                println!("{}", file.to_string_lossy());
            }
            found.push((dist, file.to_string_lossy().to_string()));
            if early_stop_count == <usize as TryInto<isize>>::try_into(found.len()).unwrap() {
                break;
            }
        }
    }

    if sort_by_distance {
        found.sort_by(|(a0, a1), (b0, b1)| a0.partial_cmp(&b0).unwrap().then_with(|| a1.cmp(&b1)));
        if verbose {
            eprintln!("Sorted by distance:");
            for f in found.iter() {
                eprintln!("{}\t{}", f.0, f.1);
            }
        } else {
            for f in found.iter() {
                println!("{}", f.1);
            }
        }
    }

    if verbose {
        eprintln!("Skip count: {}", skip_count);
    }
}

fn read_exif(path: &Path) -> Option<(f64, f64)> {
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

// fn compute_distance(lat0: f64, lon0: f64, lat1: f64, lon1: f64) -> f64 {
fn compute_distance((lat0, lon0): (f64, f64), (lat1, lon1): (f64, f64)) -> f64 {
    let loc0 = Point::new(lat0, lon0);
    let loc1 = Point::new(lat1, lon1);
    let dist = Haversine::distance(loc0, loc1);
    dist
}

fn search_files(src_root: &Path) -> Vec<PathBuf> {
    globwalk::glob(src_root.join("**/*.{jpg,mp4}").to_str().unwrap())
        .unwrap()
        .map(|item| item.unwrap())
        .map(|item| item.into_path())
        .collect()
}
