//! 只读 git 子进程与 tag 区间解析。

use std::{path::Path, process::Command};

use crate::{
    changelog::gitmoji::{leading_gitmoji, section_for_gitmoji, strip_gitmoji},
    error::{Result, validation, validation_with_input},
};

/// 单条 commit 摘要（来自 `git log --format`）。
#[derive(Debug, Clone)]
pub struct CommitEntry {
    /// Full commit hash.
    pub hash: String,
    /// Author email.
    pub email: String,
    /// Author name.
    pub author: String,
    /// Subject line (first line of commit message).
    pub subject: String,
    /// Subject with gitmoji stripped.
    pub body: String,
    /// Leading gitmoji if present.
    pub gitmoji: Option<&'static str>,
    /// Release section derived from gitmoji.
    pub section: crate::changelog::gitmoji::Section,
}

/// 在仓库根目录执行 git 并返回 stdout（trim 尾部空白）。
pub fn git_output(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|err| validation(format!("spawn git: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() { format!("git {} failed", args.join(" ")) } else { stderr };
        return Err(validation_with_input("git command failed", detail));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
}

/// 列出 `v*` tag（`version:refname` 排序）。
pub fn list_version_tags(repo_root: &Path) -> Result<Vec<String>> {
    let out = git_output(repo_root, &["tag", "-l", "v*", "--sort=version:refname"])?;
    Ok(if out.is_empty() { Vec::new() } else { out.lines().map(str::to_string).collect() })
}

/// 校验 ref 指向 commit。
pub fn verify_commit_ref(repo_root: &Path, reference: &str) -> Result<()> {
    git_output(repo_root, &["rev-parse", "--verify", &format!("{reference}^{{commit}}")])?;
    Ok(())
}

/// 去掉可选 `v` 前缀。
pub fn normalize_version(version: &str) -> String {
    version.trim_start_matches('v').to_string()
}

/// 返回 `vX.Y.Z` tag 名。
pub fn version_tag(version: &str) -> String {
    format!("v{}", normalize_version(version))
}

/// 查找目标版本的上一个 `v*` tag。
pub fn previous_version_tag(repo_root: &Path, version: &str) -> Result<Option<String>> {
    let target = version_tag(version);
    let tags = list_version_tags(repo_root)?;
    if let Some(index) = tags.iter().position(|tag| tag == &target) {
        if index > 0 {
            return Ok(Some(tags[index - 1].clone()));
        }
        return Ok(None);
    }

    let normalized = normalize_version(version);
    let parts: Vec<&str> = normalized.split('.').collect();
    if parts.len() != 3 {
        return Ok(None);
    }
    let numbers: Option<Vec<u32>> = parts.iter().map(|part| part.parse().ok()).collect();
    let Some(numbers) = numbers
    else {
        return Ok(None);
    };
    if numbers[2] > 0 {
        let prev = format!("{}.{}.{}", numbers[0], numbers[1], numbers[2] - 1);
        let tag = version_tag(&prev);
        if tags.iter().any(|existing| existing == &tag) {
            return Ok(Some(tag));
        }
    }
    Ok(None)
}

/// 列出 tag 与短 hash（`--tags` 子命令输出）。
pub fn format_tag_list(repo_root: &Path) -> Result<String> {
    let tags = list_version_tags(repo_root)?;
    if tags.is_empty() {
        return Ok("(no v* tags)".to_string());
    }
    let mut lines = Vec::new();
    for tag in tags {
        let short = git_output(repo_root, &["rev-parse", "--short", &tag])?;
        lines.push(format!("{tag}\t{short}"));
    }
    Ok(lines.join("\n"))
}

/// 解析 tag 区间内的非 merge commit。
pub fn collect_commits(repo_root: &Path, from_ref: Option<&str>, to_ref: &str) -> Result<Vec<CommitEntry>> {
    let range = match from_ref {
        Some(from) => format!("{from}..{to_ref}"),
        None => to_ref.to_string(),
    };
    let out = git_output(repo_root, &["log", &range, "--no-merges", "--format=%H%x1f%ae%x1f%an%x1f%s"])?;
    if out.is_empty() {
        return Ok(Vec::new());
    }
    let mut commits = Vec::new();
    for line in out.lines() {
        let mut parts = line.split('\x1f');
        let hash = parts.next().unwrap_or_default().to_string();
        let email = parts.next().unwrap_or_default().to_string();
        let author = parts.next().unwrap_or_default().to_string();
        let subject = parts.next().unwrap_or_default().to_string();
        let gitmoji = leading_gitmoji(&subject);
        let section = section_for_gitmoji(gitmoji);
        commits.push(CommitEntry {
            hash,
            email,
            author,
            subject: subject.clone(),
            body: strip_gitmoji(&subject),
            gitmoji,
            section,
        });
    }
    Ok(commits)
}

/// 解析 release 区间（`--version` 或 `--from` / `--to`）。
pub fn resolve_range(
    repo_root: &Path,
    version: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<(String, Option<String>, String)> {
    if let Some(version) = version {
        let tag = version_tag(version);
        verify_commit_ref(repo_root, &tag)?;
        let prev = previous_version_tag(repo_root, version)?;
        return Ok((normalize_version(version), prev, tag));
    }

    let Some(to) = to
    else {
        return Err(crate::error::validation("need --version=X.Y.Z or --to=REF (optional --from=REF)"));
    };

    let to_ref = if to.starts_with('v') || is_hex_oid(to) { to.to_string() } else { version_tag(to) };
    verify_commit_ref(repo_root, &to_ref)?;
    if let Some(from_ref) = from {
        verify_commit_ref(repo_root, from_ref)?;
    }

    let version_label = regex_version_label(&to_ref).unwrap_or_else(|| to_ref.trim_start_matches('v').to_string());
    Ok((version_label, from.map(str::to_string), to_ref))
}

fn is_hex_oid(value: &str) -> bool {
    value.len() >= 7 && value.len() <= 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn regex_version_label(reference: &str) -> Option<String> {
    static VERSION: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = VERSION.get_or_init(|| regex::Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap());
    re.captures(reference).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// 从 `origin` remote URL 解析 `owner/repo`。
pub fn detect_github_repo(repo_root: &Path) -> Option<String> {
    let url = git_output(repo_root, &["remote", "get-url", "origin"]).ok()?;
    parse_github_repo_from_remote(&url)
}

/// 从 remote URL 解析 `owner/repo`（不访问 git）。
pub fn parse_github_remote_repo(url: &str) -> Option<String> {
    parse_github_repo_from_remote(url)
}

fn parse_github_repo_from_remote(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        return normalize_repo_path(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        return normalize_repo_path(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("ssh://git@github.com/") {
        return normalize_repo_path(rest);
    }
    None
}

fn normalize_repo_path(path: &str) -> Option<String> {
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    if path.contains(':') || path.is_empty() || !path.contains('/') {
        return None;
    }
    Some(path.to_string())
}
