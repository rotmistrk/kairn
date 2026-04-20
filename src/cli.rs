// Command-line argument parsing.

use std::path::PathBuf;
use std::process;

use clap::Parser;

/// kairn — A TUI IDE oriented around Kiro AI
#[derive(Parser, Debug)]
#[command(name = "kairn", version, about, long_about = None)]
pub struct Cli {
    /// Project directory to open
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Config file to load (overrides .kairnrc search)
    #[arg(short = 'C', long = "config")]
    pub config: Option<PathBuf>,

    /// Arguments to pass to kiro-cli
    #[arg(last = true)]
    pub kiro_args: Vec<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Resolve the project path to an absolute path.
    pub fn resolve_path(&self) -> PathBuf {
        std::fs::canonicalize(&self.path).unwrap_or_else(|e| {
            eprintln!("kairn: {}: {e}", self.path.display());
            process::exit(66); // EX_NOINPUT
        })
    }
}
