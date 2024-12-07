use clap::Parser;

/// Search photos near a geographic location
#[derive(Parser)]
struct Cli {
    /// Latitude of the target location
    #[arg(allow_negative_numbers = true)]
    lat: f64,
    /// Longitude of the target location
    #[arg(allow_negative_numbers = true)]
    lon: f64,
    /// Max distance from the target location
    radius: f64,
    /// Directory/folder to search from
    #[arg(short, long, default_value_t = String::from("."))]
    dir: String,
    #[arg(short, long, default_value_t = -1)]
    early_stop_count: isize,
    #[arg(short, long, action)]
    sort_by_distance: bool,
    #[arg(short, long, action)]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    // imnear::run((args.lat, args.lon), args.radius, &args.dir);
    imnear::run(
        (args.lat, args.lon),
        args.radius,
        &args.dir,
        args.early_stop_count,
        args.sort_by_distance,
        args.verbose,
    );
}
