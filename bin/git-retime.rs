//! `git-retime` 命令行入口：在对象层随机分布 commit 时间并写入新分支。

use clap::{Parser, Subcommand};

use git_tools::{
    Result,
    commit::{RetimeOptions, RetimeRootOptions, open_here, run_retime, run_retime_root, short},
};

/// 将 commit author/committer 时间随机分布到日期区间，结果写入新分支。
///
/// 只改时间戳，不改动 author/committer 的 name/email（不会像 `git commit --amend` 那样写入本地用户）。
#[derive(Parser)]
#[command(name = "git-retime", about = "Spread commit timestamps across a date range (pure Rust / gix)")]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Range start commit (`(commit..tip]` is retimed; its author time is the default window start)
    commit: Option<String>,

    /// Random window start (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS`)
    #[arg(short, long, value_name = "START")]
    start_date: Option<String>,

    /// Random window end; defaults to `start + number of commits in range` days
    #[arg(short, long, value_name = "END")]
    end_date: Option<String>,

    /// Output branch name; defaults to `time-travel`
    #[arg(short, long, value_name = "BRANCH")]
    branch: Option<String>,

    /// Range end revision; defaults to `HEAD`
    #[arg(long, default_value = "HEAD")]
    tip: String,
}

/// 子命令。
#[derive(Subcommand)]
enum Command {
    /// Retime all commits from repository root through `tip` (inclusive)
    Root {
        /// Random window start; defaults to the root commit author time
        #[arg(short, long, value_name = "START")]
        start_date: Option<String>,

        /// Random window end
        #[arg(short, long, value_name = "END")]
        end_date: Option<String>,

        /// Output branch name; defaults to `time-travel`
        #[arg(short, long, value_name = "BRANCH")]
        branch: Option<String>,

        /// Range end revision; defaults to `HEAD`
        #[arg(long, default_value = "HEAD")]
        tip: String,

        /// Optional new message for the root commit
        #[arg(short, long)]
        message: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = open_here()?;

    let summary = match cli.command {
        Some(Command::Root { start_date, end_date, branch, tip, message }) => {
            run_retime_root(&repo, &RetimeRootOptions { start_date, end_date, branch, tip, message })?
        }
        None => {
            let commit = cli.commit.ok_or_else(|| git_tools::validation("missing commit hash for range retime"))?;
            run_retime(
                &repo,
                &RetimeOptions { commit, start_date: cli.start_date, end_date: cli.end_date, branch: cli.branch, tip: cli.tip },
            )?
        }
    };

    println!("retimed {} commit(s) on branch {}; tip {}", summary.rewritten, summary.branch, short(summary.new_tip));
    Ok(())
}
