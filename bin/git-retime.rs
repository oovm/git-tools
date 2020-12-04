//! `git-retime` 命令行入口：在对象层随机分布 commit 时间并写入新分支。

use std::path::PathBuf;

use clap::Parser;

use git_tools::{
    Result,
    commit::{RetimeOptions, open, run_retime, short},
};

/// 将 `(commit..HEAD]` 上的 author/committer 时间随机分布到日期区间，结果写入新分支。
#[derive(Parser)]
#[command(name = "git-retime", about = "Spread commit timestamps across a date range (pure Rust / gix)")]
struct Cli {
    /// git 仓库路径（默认为当前目录）
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    /// 范围起点 commit hash 或 revision
    commit: String,

    /// 随机时间分布起始日期（`YYYY-MM-DD`）
    start_date: String,

    /// 随机时间分布结束日期；缺省为 `start_date + commit 数量` 天
    #[arg(short, long, value_name = "END")]
    end_date: Option<String>,

    /// 输出分支名；缺省为 `time-travel`
    #[arg(short, long, value_name = "BRANCH")]
    branch: Option<String>,

    /// 范围终点 revision；缺省为 `HEAD`
    #[arg(long, default_value = "HEAD")]
    tip: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = open(&cli.repo)?;
    let summary = run_retime(
        &repo,
        &RetimeOptions {
            commit: cli.commit,
            start_date: cli.start_date,
            end_date: cli.end_date,
            branch: cli.branch,
            tip: cli.tip,
        },
    )?;
    println!(
        "retimed {} commit(s) on branch {}; tip {}",
        summary.rewritten,
        summary.branch,
        short(summary.new_tip)
    );
    Ok(())
}
