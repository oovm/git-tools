//! `git-reword` 命令行入口：导出映射、lint、对象层改写。

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use git_tools::commit::{
    Result, RewordError, collect_commits, dry_run_plan, duplicate_subjects, export_map, full_message, head_ref_name,
    lint_message, move_ref, open, parse_map_file, plan_rewrite, resolve_map, resolve_ref_tip, resolve_rev, short,
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

/// 子命令：改写、lint 或导出映射模板。
#[derive(Subcommand)]
enum Command {
    /// 按映射改写 commit 并更新分支 ref（无 interactive rebase）
    Rewrite {
        /// exclusive base：从 tip 可达但不在该 OID 祖先链上的 commit 会被改写
        #[arg(long)]
        base: String,
        /// 要更新的分支 ref，如 `refs/heads/dev` 或 `dev`
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
        /// hash 前缀 → message 映射文件
        #[arg(long)]
        map: PathBuf,
        /// 仅打印计划，不写对象、不更新 ref
        #[arg(long)]
        dry_run: bool,
    },
    /// lint `base..ref` 范围内现有 commit message
    LintLog {
        #[arg(long)]
        base: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
    },
    /// lint 映射文件中的 message 是否落在指定范围内
    LintMap {
        #[arg(long)]
        base: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
        #[arg(long)]
        map: PathBuf,
    },
    /// 导出 `base..ref` 的 hash 映射模板
    Export {
        #[arg(long)]
        base: String,
        #[arg(long, default_value = "HEAD")]
        r#ref: String,
        #[arg(long, default_value = "reword.pending.txt")]
        out: PathBuf,
    },
}

/// 将 `HEAD` / 短分支名规范化为完整 ref 名。
fn normalize_ref_name(repo: &gix::Repository, ref_name: &str) -> Result<String> {
    if ref_name == "HEAD" {
        return head_ref_name(repo);
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
        Command::Rewrite { base, r#ref, map, dry_run } => {
            let exclusive_base = resolve_rev(&repo, &base)?;
            let tip = resolve_ref_tip(&repo, &r#ref)?;
            let chain = collect_commits(&repo, exclusive_base, tip)?;
            let entries = parse_map_file(&map)?;
            let updates = resolve_map(entries, &chain)?;

            for (oid, message) in &updates {
                for issue in lint_message(message, &short(*oid)) {
                    eprintln!("lint {}: {}", issue.label, issue.detail);
                    std::process::exit(1);
                }
            }

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
                return Err(RewordError::msg("no object rewrites performed"));
            }

            let ref_name = normalize_ref_name(&repo, &r#ref)?;
            move_ref(&repo, &ref_name, new_tip, old_tip)?;

            println!("rewrote {} commit object(s); {} -> {}", changes.len(), short(old_tip), short(new_tip));
            for change in &changes {
                println!("  {}  {}", short(change.old_oid), change.new_subject);
            }
        }
        Command::LintLog { base, r#ref } => {
            let exclusive_base = resolve_rev(&repo, &base)?;
            let tip = resolve_ref_tip(&repo, &r#ref)?;
            let chain = collect_commits(&repo, exclusive_base, tip)?;
            let chain_len = chain.len();
            let mut failed = 0usize;
            let mut subjects = Vec::new();
            for oid in &chain {
                let message = full_message(&repo, *oid)?;
                let subject = message.lines().next().unwrap_or("").to_string();
                subjects.push((short(*oid), subject.clone()));
                for issue in lint_message(&message, &short(*oid)) {
                    failed += 1;
                    eprintln!("\n{}  {}", short(*oid), subject);
                    eprintln!("  - {}", issue.detail);
                }
            }
            for line in duplicate_subjects(&subjects) {
                failed += 1;
                eprintln!("\n{}", line);
            }
            if failed > 0 {
                eprintln!("\nlint-log: {} issue(s) in {} commit(s)", failed, chain_len);
                std::process::exit(1);
            }
            println!("lint-log: {} commit(s) OK", chain_len);
        }
        Command::LintMap { base, r#ref, map } => {
            let exclusive_base = resolve_rev(&repo, &base)?;
            let tip = resolve_ref_tip(&repo, &r#ref)?;
            let chain = collect_commits(&repo, exclusive_base, tip)?;
            let entries = parse_map_file(&map)?;
            let updates = resolve_map(entries, &chain)?;
            let mut failed = 0usize;
            for (oid, message) in &updates {
                for issue in lint_message(message, &short(*oid)) {
                    failed += 1;
                    eprintln!("lint {}: {}", short(*oid), issue.detail);
                }
            }
            if failed > 0 {
                std::process::exit(1);
            }
            println!("lint-map: {} block(s) OK", updates.len());
        }
        Command::Export { base, r#ref, out } => {
            let exclusive_base = resolve_rev(&repo, &base)?;
            let tip = resolve_ref_tip(&repo, &r#ref)?;
            let count = collect_commits(&repo, exclusive_base, tip)?.len();
            export_map(&repo, exclusive_base, tip, &out)?;
            println!("export: wrote {} commit block(s) to {}", count, out.display());
        }
    }

    Ok(())
}
