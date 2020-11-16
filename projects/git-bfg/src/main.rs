//! `bfg` 命令行入口。

use std::path::PathBuf;

use clap::Parser;
use git_bfg::{Cleaner, Result, find_git_root};

/// 命令行参数。
#[derive(Parser)]
#[command(name = "bfg", about = "List largest blob objects in a git repository (pure Rust / gix)")]
struct Cli {
    /// 仓库路径；省略时向上查找最近的 `.git` 目录
    #[arg(long)]
    repo: Option<PathBuf>,

    /// 输出的最大 blob 数量
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
