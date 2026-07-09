use polygraph::{analyze_path, config::ReportConfig, output};
use clap::Parser;
use std::process;

#[derive(Parser)]
#[command(name = "polygraph", version)]
struct Args {
    /// Path to a file or directory to analyze
    path: std::path::PathBuf,

    /// Output format: markdown, pretty, or json
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// Include closures and lambda expressions in the analysis
    #[arg(long)]
    include_closures: bool,

    /// Path to a polygraph.toml configuration file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();

    let config_path = args
        .config
        .unwrap_or_else(|| std::path::PathBuf::from("config/polygraph.toml"));
    let config = ReportConfig::load_from_path(&config_path);

    let results = match analyze_path(&args.path, args.include_closures) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    for file in &results {
        if let Some(ref err) = file.error {
            eprintln!("Error parsing {}: {}", file.path.display(), err);
        }
    }

    let clusters = polygraph::duplicates::compute_duplicates(&results);
    let formatter = output::get_formatter(&args.format, config);
    println!("{}", formatter.format(&results, &clusters));
}
