use clap::Parser;
use polygraph::{analyze_path, clones::CloneConfig, config::ReportConfig, output};
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

    /// Enable structural (Type-1/2/3) clone detection via CST comparison.
    /// Off by default: output and performance are unchanged without this flag.
    #[arg(long)]
    clones: bool,

    /// Override the clone similarity threshold (0.0-1.0) from polygraph.toml
    #[arg(long)]
    clone_threshold: Option<f64>,

    /// Override the minimum function size (in lines) considered for clone comparison
    #[arg(long)]
    clone_min_lines: Option<usize>,
}

fn main() {
    let args = Args::parse();

    let config_path = args
        .config
        .unwrap_or_else(|| std::path::PathBuf::from("config/polygraph.toml"));
    let config = ReportConfig::load_from_path(&config_path);

    let results = match analyze_path(&args.path, args.include_closures, args.clones) {
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

    let mut clone_config = CloneConfig::default();
    if let Some(threshold) = config.clone_similarity_threshold {
        clone_config.similarity_threshold = threshold;
    }
    if let Some(min_lines) = config.clone_min_lines {
        clone_config.min_lines = min_lines;
    }
    if let Some(threshold) = args.clone_threshold {
        clone_config.similarity_threshold = threshold;
    }
    if let Some(min_lines) = args.clone_min_lines {
        clone_config.min_lines = min_lines;
    }

    let clones = if args.clones {
        polygraph::clones::compute_clones(&results, &clone_config)
    } else {
        Vec::new()
    };

    let formatter = output::get_formatter(&args.format, config);
    println!("{}", formatter.format(&results, &clusters, &clones));
}
