use std::{
    io,
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

    // pub fn search_from_paths(
    //     &self,
    //     path_iter: impl Iterator<Item = Result<String, io::Error>>,
    // ) -> Vec<FilterResult> {
    //     let mut found: Vec<FilterResult> = Vec::new();
    //     let mut skip_count: u32 = 0;

    //     for path_result in path_iter {
    //         let path_str = path_result.unwrap();
    //         let path = Path::new(&path_str);
    //         self.user_msg(&format!("{}", path.to_str().unwrap()));

    //         let filter_result = match self.filter_file(path) {
    //             Some(result) => result,
    //             None => {
    //                 if self.verbose {
    //                     eprintln!("Skipping {}", path.to_str().unwrap());
    //                 }
    //                 continue;
    //             }
    //         };
    //         if filter_result.selected {
    //             if self.verbose {
    //                 eprintln!("{}\t{}", filter_result.distance, path.to_string_lossy());
    //             }
    //             found.push(filter_result);
    //         }
    //     }

    //     self.user_msg(&format!(
    //         "Found {} images near target location",
    //         found.len()
    //     ));
    //     if self.sort_by_distance {
    //         found.sort_by(|a, b| {
    //             a.distance
    //                 .partial_cmp(&b.distance)
    //                 .unwrap()
    //                 .then_with(|| a.path.to_string_lossy().cmp(&b.path.to_string_lossy()))
    //         });
    //     }
    //     for f in found.iter() {
    //         self.user_msg(&format!("{}", f.distance));
    //     }
    //     found
    // }

    // pub fn search_from_dir(&self, dir: &str) -> Vec<FilterResult> {
    //     let search_dir = Path::new(dir);

    //     let mut found: Vec<(f64, String)> = Vec::new();

    //     let paths = search_files(search_dir);
    //     if self.verbose {
    //         eprintln!("file count: {}", paths.len());
    //     }
    //     let mut skip_count: u32 = 0;

    //     // Make it look like stdin iterator to reuse the same method
    //     let path_iter = paths.iter().map(|x| Ok(x.to_string_lossy().to_string()));

    //     self.search_from_paths(path_iter)
    // }

    // FIXME: return Option<bool> for missing attributes
    fn filter_file(&self, path: &Path) -> Option<FilterResult> {
        let (lat, lon) = match read_exif(path) {
            Some(loc) => loc,
            None => {
                return None;
            }
        };
        // println!("lat: {}", lat);
        // println!("lon: {}", lon);

        let dist = compute_distance(self.target_loc, (lat, lon));

        // println!("path: {:?}", path);
        // println!("distance: {}", dist);

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

// pub fn search_from_dir(
//     target_loc: (f64, f64),
//     radius: f64,
//     dir: &str,
//     early_stop_count: isize,
//     sort_by_distance: bool,
//     verbose: bool,
// ) {
//     let search_dir = Path::new(dir);

//     let mut found: Vec<(f64, String)> = Vec::new();

//     let files = search_files(search_dir);
//     if verbose {
//         eprintln!("file count: {}", files.len());
//     }
//     let mut skip_count: u32 = 0;
//     for file in files.iter() {
//         let (lat, lon) = match read_exif(file) {
//             Some(loc) => loc,
//             None => {
//                 skip_count += 1;
//                 if verbose {
//                     eprintln!("Skipping {}", file.to_str().unwrap());
//                 }
//                 continue;
//             }
//         };
//         // println!("lat: {}", lat);
//         // println!("lon: {}", lon);

//         let dist = compute_distance(target_loc, (lat, lon));

//         // println!("file: {:?}", file);
//         // println!("distance: {}", dist);

//         if dist <= radius {
//             if verbose {
//                 eprintln!("{}\t{}", dist, file.to_string_lossy());
//             } else if !sort_by_distance {
//                 println!("{}", file.to_string_lossy());
//             }
//             found.push((dist, file.to_string_lossy().to_string()));
//             if early_stop_count == <usize as TryInto<isize>>::try_into(found.len()).unwrap() {
//                 break;
//             }
//         }
//     }

//     if sort_by_distance {
//         found.sort_by(|(a0, a1), (b0, b1)| a0.partial_cmp(&b0).unwrap().then_with(|| a1.cmp(&b1)));
//         if verbose {
//             eprintln!("Sorted by distance:");
//             for f in found.iter() {
//                 eprintln!("{}\t{}", f.0, f.1);
//             }
//         } else {
//             for f in found.iter() {
//                 println!("{}", f.1);
//             }
//         }
//     }

//     if verbose {
//         eprintln!("Skip count: {}", skip_count);
//     }
// }

pub struct FilterResult {
    pub path: PathBuf,
    pub distance: f64,
    selected: bool,
    // add time info
}

// pub fn search_from_paths(
//     path_iter: impl Iterator<Item = Result<String, io::Error>>,
//     target_loc: (f64, f64),
//     radius: f64,
//     dir: &str,
//     early_stop_count: isize,
//     sort_by_distance: bool,
//     verbose: bool,
// ) -> Vec<FilterResult> {
//     let mut found: Vec<FilterResult> = Vec::new();
//     let mut skip_count: u32 = 0;

//     for path_result in path_iter {
//         let path_str = path_result.unwrap();
//         let path = Path::new(&path_str);
//         if verbose {
//             eprintln!("{}", path.to_str().unwrap());
//         }

//         let filter_result = match filter_file(path, target_loc, radius) {
//             Some(result) => result,
//             None => {
//                 if verbose {
//                     eprintln!("Skipping {}", path.to_str().unwrap());
//                 }
//                 continue;
//             }
//         };
//         if filter_result.selected {
//             if verbose {
//                 eprintln!("{}\t{}", filter_result.distance, path.to_string_lossy());
//             }
//             found.push(filter_result);
//         }
//     }

//     if verbose {
//         eprintln!("Found {} images near target location", found.len());
//     }
//     if sort_by_distance {
//         found.sort_by(|a, b| {
//             a.distance
//                 .partial_cmp(&b.distance)
//                 .unwrap()
//                 .then_with(|| a.path.to_string_lossy().cmp(&b.path.to_string_lossy()))
//         });
//     }
//     if verbose {
//         for f in found.iter() {
//             eprintln!("{}", f.distance);
//         }
//     }
//     found
// }

// // FIXME: return Option<bool> for missing attributes
// fn filter_file(path: &Path, target_loc: (f64, f64), radius: f64) -> Option<FilterResult> {
//     let (lat, lon) = match read_exif(path) {
//         Some(loc) => loc,
//         None => {
//             return None;
//         }
//     };
//     // println!("lat: {}", lat);
//     // println!("lon: {}", lon);

//     let dist = compute_distance(target_loc, (lat, lon));

//     // println!("path: {:?}", path);
//     // println!("distance: {}", dist);

//     Some(FilterResult {
//         path: path.to_path_buf(),
//         selected: dist <= radius,
//         distance: dist,
//     })
// }

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

// fn search_files(src_root: &Path) -> Vec<PathBuf> {
//     globwalk::glob(src_root.join("**/*.{jpg,mp4}").to_str().unwrap())
//         .unwrap()
//         .map(|item| item.unwrap())
//         .map(|item| item.into_path())
//         .collect()
// }
