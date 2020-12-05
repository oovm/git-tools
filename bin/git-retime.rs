//! `git-retime` 命令行入口：在对象层随机分布 commit 时间并写入新分支。

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use git_tools::{
    Result,
    commit::{RetimeOptions, RetimeRootOptions, open, run_retime, run_retime_root, short},
};

/// 将 commit author/committer 时间随机分布到日期区间，结果写入新分支。
///
/// 只改时间戳，不改动 author/committer 的 name/email（不会像 `git commit --amend` 那样写入本地用户）。
#[derive(Parser)]
#[command(name = "git-retime", about = "Spread commit timestamps across a date range (pure Rust / gix)")]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    /// git 仓库路径（默认为当前目录）
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,

    /// 范围起点 commit（`(commit..tip]`，不含起点本身）
    commit: Option<String>,

    /// 随机时间窗口起点（`YYYY-MM-DD` 或 `YYYY-MM-DDTHH:MM:SS`）
    start_date: Option<String>,

    /// 随机时间窗口终点；缺省为 `start + commit 数量` 天
    #[arg(short, long, value_name = "END")]
    end_date: Option<String>,

    /// 输出分支名；缺省为 `time-travel`
    #[arg(short, long, value_name = "BRANCH")]
    branch: Option<String>,

    /// 范围终点 revision；缺省为 `HEAD`
    #[arg(long, default_value = "HEAD")]
    tip: String,
}

/// 子命令。
#[derive(Subcommand)]
enum Command {
    /// 从 root 到 `--tip` 改写全部 commit 时间（含 root），可选改写 root message
    Root {
        /// 随机时间窗口起点（`YYYY-MM-DD` 或 ISO datetime）
        start_date: String,

        /// 随机时间窗口终点
        #[arg(short, long, value_name = "END")]
        end_date: Option<String>,

        /// 输出分支名；缺省为 `time-travel`
        #[arg(short, long, value_name = "BRANCH")]
        branch: Option<String>,

        /// 范围终点 revision；缺省为 `HEAD`
        #[arg(long, default_value = "HEAD")]
        tip: String,

        /// 改写 root commit message（对应旧 `git-root`）
        #[arg(short, long)]
        message: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = open(&cli.repo)?;

    let summary = match cli.command {
        Some(Command::Root { start_date, end_date, branch, tip, message }) => {
            run_retime_root(&repo, &RetimeRootOptions { start_date, end_date, branch, tip, message })?
        }
        None => {
            let commit = cli.commit.ok_or_else(|| git_tools::validation("missing commit hash for range retime"))?;
            let start_date = cli.start_date.ok_or_else(|| git_tools::validation("missing start datetime"))?;
            run_retime(&repo, &RetimeOptions { commit, start_date, end_date: cli.end_date, branch: cli.branch, tip: cli.tip })?
        }
    };

    println!("retimed {} commit(s) on branch {}; tip {}", summary.rewritten, summary.branch, short(summary.new_tip));
    Ok(())
}
