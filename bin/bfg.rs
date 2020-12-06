//! `bfg` 命令行入口。

use clap::Parser;
use git_tools::{Result, ResultExt, message, object::Cleaner, repo::find_git_root};

/// 命令行参数。
#[derive(Parser)]
#[command(name = "bfg", about = "List largest blob objects in a git repository (pure Rust / gix)")]
struct Cli {
    /// 输出的最大 blob 数量
    #[arg(long, default_value_t = 100)]
    top: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = find_git_root(std::env::current_dir().or_raise(|| message!("read current directory"))?)?;
    let mut cleaner = Cleaner::new(&root)?;
    cleaner.collect_info()?;
    cleaner.largest_objects(cli.top);
    Ok(())
}
