use std::path::PathBuf;

use clap::{Args, Parser};
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Commands {
    /// File to analyze
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Whether to include stopwords
    #[arg(short = 'a', long)]
    pub analyze_stopwords: bool,

    /// List the top N words
    #[arg(short = 't', long, value_name = "N", value_parser = 1..10000)]
    pub top: Option<i64>,
    /// Lists the bottom N words
    /// WARNING: sorting is non deterministic, so multiple rare words will be random on every call
    #[arg(long, value_name = "N", value_parser = 1..10000)]
    pub bottom: Option<i64>,

    /// Show various statistics about diversity
    #[arg(short = 'd', long)]
    pub diversity: bool,

    /// Path to write to
    #[arg(long, short = 'o')]
    pub out: Option<PathBuf>,

    /// Whether to include words in the export
    #[arg(short, long, requires("out"))]
    pub write_words: bool,
}
