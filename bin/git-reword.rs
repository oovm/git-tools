//! `git-reword` 命令行入口：导出 JSON 映射、对象层改写。

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use git_tools::{
    Result,
    commit::{
        collect_commits, dry_run_plan, export_map, move_ref, open, parse_map, plan_rewrite, resolve_map, resolve_ref_tip,
        resolve_rev, short,
    },
    validation,
};

/// 全局参数与子命令。
#[derive(Parser)]
#[command(name = "git-reword", about = "Rewrite commit messages at the git object layer (pure Rust / gix)")]
struct Cli {
    /// git 仓库路径（默认为当前目录）
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    #[command(subcommand)]
    command: Command,
}

/// 子命令：改写或导出映射模板。
#[derive(Subcommand)]
enum Command {
    /// 按映射改写 commit 并更新分支 ref（无 interactive rebase）
    Rewrite {
        #[arg(long)]
        base: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
        /// JSON 映射文件路径
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// 导出 `base..ref` 的 JSON 映射模板
    Export {
        #[arg(long)]
        base: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
        #[arg(long, default_value = "reword.pending.json")]
        path: PathBuf,
    },
}

fn normalize_ref_name(repo: &gix::Repository, ref_name: &str) -> Result<String> {
    if ref_name == "HEAD" {
        return git_tools::commit::head_ref_name(repo);
    }
    if ref_name.starts_with("refs/") {
        return Ok(ref_name.to_string());
    }
    Ok(format!("refs/heads/{}", ref_name))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo = open(&cli.repo)?;

    match cli.command {
        Command::Rewrite { base, r#ref, path, dry_run } => {
            let exclusive_base = resolve_rev(&repo, &base)?;
            let tip = resolve_ref_tip(&repo, &r#ref)?;
            let chain = collect_commits(&repo, exclusive_base, tip)?;
            let entries = parse_map(&path)?;
            let updates = resolve_map(entries, &chain)?;

            if dry_run {
                let changes = dry_run_plan(&repo, exclusive_base, tip, &updates)?;
                if changes.is_empty() {
                    println!("dry-run: no object rewrites needed");
                    return Ok(());
                }
                println!("dry-run: {} commit object(s) would be rewritten on {}\n", changes.len(), r#ref);
                for change in &changes {
                    println!("{}  {}", short(change.old_oid), change.old_subject);
                    println!("     ->  {}", change.new_subject);
                    if change.parents_relinked && !change.message_changed {
                        println!("     (parent chain relink only)");
                    }
                    println!();
                }
                return Ok(());
            }

            let old_tip = tip;
            let (changes, new_tip) = plan_rewrite(&repo, exclusive_base, tip, &updates)?;
            if changes.is_empty() {
                return Err(validation("no object rewrites performed"));
            }

            let ref_name = normalize_ref_name(&repo, &r#ref)?;
            move_ref(&repo, &ref_name, new_tip, old_tip)?;

            println!("rewrote {} commit object(s); {} -> {}", changes.len(), short(old_tip), short(new_tip));
            for change in &changes {
                println!("  {}  {}", short(change.old_oid), change.new_subject);
            }
        }
        Command::Export { base, r#ref, path } => {
            let exclusive_base = resolve_rev(&repo, &base)?;
            let tip = resolve_ref_tip(&repo, &r#ref)?;
            let count = collect_commits(&repo, exclusive_base, tip)?.len();
            export_map(&repo, exclusive_base, tip, &path)?;
            println!("export: wrote {} commit entry(ies) to {}", count, path.display());
        }
    }

    Ok(())
}
