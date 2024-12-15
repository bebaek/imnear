use atty::Stream;
use clap::Parser;
use directories::ProjectDirs;
use std::{fs, io};

pub mod cache;
pub mod geocode;

fn main() {
    let args = Cli::parse();

    // Get cache
    let mut path = ProjectDirs::from("", "", "imnear")
        .expect("Cannot find app cache dir")
        .cache_dir()
        .to_path_buf();
    path.push("nominatim");
    fs::create_dir_all(&path).expect("Error creating cache dir");
    let geocode_cache = cache::Cache::new(&path);

    // Use address if provided
    let coords = match args.address {
        Some(addr) => geocode::locate(&addr, geocode_cache),
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

    let searcher = imnear::Searcher::new(
        args.radius,
        (lat, lon),
        args.early_stop_count,
        args.sort_by_distance,
        args.verbose,
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
