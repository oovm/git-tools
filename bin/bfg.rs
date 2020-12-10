//! `bfg` 命令行入口。

use clap::Parser;
use git_tools::{
    Result, ResultExt, diag, message,
    object::{Cleaner, print_largest_blobs},
    repo::find_git_root,
};

/// 命令行参数。
#[derive(Parser)]
#[command(name = "bfg", about = "List largest blob objects in a git repository (pure Rust / gix)")]
struct Cli {
    /// Number of largest blobs to list
    #[arg(long, default_value_t = 100)]
    top: usize,
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    tracing::info!(top = cli.top, "starting blob scan");
    let root = find_git_root(std::env::current_dir().or_raise(|| message!("read current directory"))?)?;
    tracing::debug!(root = ?root, "discovered git repository");
    let mut cleaner = Cleaner::new(&root)?;
    cleaner.collect_info()?;
    let ranking = cleaner.rank_largest_blobs();
    print_largest_blobs(&ranking.summary, &ranking.ranked, &ranking.corrupt, cli.top);
    Ok(())
}

fn main() {
    diag::main(run);
}
