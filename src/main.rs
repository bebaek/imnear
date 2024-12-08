use atty::Stream;
use clap::Parser;
use std::io;

fn main() {
    let args = Cli::parse();

    let searcher = imnear::Searcher::new(
        (args.lat, args.lon),
        args.radius,
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

fn is_stdin_piped() -> bool {
    !atty::is(Stream::Stdin)
}
