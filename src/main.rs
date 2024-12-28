use atty::Stream;
use clap::Parser;
use directories::ProjectDirs;
use std::{fs, io};

use imnear::Cache;
mod geocode;

fn main() -> Result<(), String> {
    let args = Cli::parse();

    // Get geocode cache
    let cache_dir = ProjectDirs::from("", "", "imnear")
        .expect("Cannot find app cache dir")
        .cache_dir()
        .to_path_buf();
    let geocode_cache_dir = cache_dir.join("nominatim");
    fs::create_dir_all(&geocode_cache_dir).expect("Error creating geocode cache dir");
    let geocode_cache = Cache::new(&geocode_cache_dir);

    // Get exif cache
    let exif_cache_dir = cache_dir.join("exif");
    fs::create_dir_all(&exif_cache_dir).expect("Error creating exif cache dir");
    let exif_cache = Cache::new(&exif_cache_dir);

    // Use address if provided
    let coords = match args.address {
        Some(addr) => match geocode::locate(&addr, geocode_cache) {
            Some(coords) => Some(coords),
            _ => return Err(format!("Found no location info for {}", &addr)),
        },
        None => None,
    };
    let (lat, lon) = match coords {
        Some((lat, lon)) => {
            if args.verbose {
                eprintln!("Found coordinates: {}, {}", lat, lon)
            }
            (lat, lon)
        }
        None => (
            args.lat.expect("Latitude is missing"),
            args.lon.expect("Longitude is missing"),
        ),
    };

    // Search
    let searcher = imnear::Searcher::new(
        args.radius,
        (lat, lon),
        args.early_stop_count,
        args.sort_by_distance,
        args.verbose,
        exif_cache,
    );
    let found: Vec<_> = if is_stdin_piped() {
        io::stdin()
            .lines()
            .filter_map(|line| searcher.filter_by_path_str(&line.unwrap()))
            .collect()
    } else {
        imnear::visit_paths(&args.dir)
            .filter_map(|path| searcher.filter_by_path_str(&path))
            .collect()
    };

    searcher.print_result(found);

    Ok(())
}

/// Search photos near a geographic location
#[derive(Parser)]
struct Cli {
    /// Latitude of the target location
    #[arg(long, allow_negative_numbers = true)]
    lat: Option<f64>,
    /// Longitude of the target location
    #[arg(long, allow_negative_numbers = true)]
    lon: Option<f64>,
    /// Address or search words
    #[arg(long)]
    address: Option<String>,
    /// Directory/folder to search from
    #[arg(short, long, default_value_t = String::from("."))]
    dir: String,
    #[arg(short, long, default_value_t = -1)]
    early_stop_count: isize,
    #[arg(short, long, action)]
    sort_by_distance: bool,
    #[arg(short, long, action)]
    verbose: bool,
    /// Max distance from the target location
    radius: f64,
}

fn is_stdin_piped() -> bool {
    !atty::is(Stream::Stdin)
}
