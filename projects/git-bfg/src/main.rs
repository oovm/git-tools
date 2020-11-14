#![doc = include_str!("../Readme.md")]

mod cleaner;
mod errors;

use std::path::PathBuf;

use clap::Parser;

pub use cleaner::{Cleaner, find_git_root};
pub use errors::{CleanerError, Result};

#[derive(Parser)]
#[command(name = "bfg", about = "List largest blob objects in a git repository (pure Rust / gix)")]
struct Cli {
    /// Git repository path (defaults to nearest ancestor with `.git`)
    #[arg(long)]
    repo: Option<PathBuf>,

    /// Number of largest blobs to print
    #[arg(long, default_value_t = 100)]
    top: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = match cli.repo {
        Some(path) => path,
        None => find_git_root(std::env::current_dir()?)?,
    };
    let mut cleaner = Cleaner::new(&root)?;
    cleaner.collect_info()?;
    cleaner.largest_objects(cli.top);
    Ok(())
}
