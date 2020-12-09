//! 对象层改写 commit author/committer 时间并写入新分支。

use std::collections::BTreeSet;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime};
use gix::{ObjectId, Repository};
use rand::Rng;
use tracing::instrument;

use crate::error::{Result, validation};

use super::{
    history::{
        commit_message, commit_parents, commits_in_range, find_root_commit, read_commit, set_branch_tip,
        write_commit_with_signatures,
    },
    identity::{copy_signature, retime_signature},
};

/// 范围 retime：改写 `(commit..tip]`。
#[derive(Debug, Clone)]
pub struct RetimeOptions {
    /// 范围起点 commit（改写 `(commit..tip]`，不含 `commit` 本身）。
    pub commit: String,
    /// 随机时间分布起始时刻（`YYYY-MM-DD` 或 ISO datetime）；缺省为范围起点 commit 的 author 时间。
    pub start_date: Option<String>,
    /// 随机时间分布结束时刻；缺省为 `start_date + commit 数量` 天。
    pub end_date: Option<String>,
    /// 新分支名；缺省为 `time-travel`。
    pub branch: Option<String>,
    /// 范围终点 revision；缺省为 `HEAD`。
    pub tip: String,
}

/// 从 root 起的 retime：改写 `[root..tip]`（含 root）。
#[derive(Debug, Clone)]
pub struct RetimeRootOptions {
    /// 随机时间分布起始时刻（`YYYY-MM-DD` 或 ISO datetime）；缺省为范围起点 commit 的 author 时间。
    pub start_date: Option<String>,
    /// 随机时间分布结束时刻；缺省为 `start_date + commit 数量` 天。
    pub end_date: Option<String>,
    /// 新分支名；缺省为 `time-travel`。
    pub branch: Option<String>,
    /// 范围终点 revision；缺省为 `HEAD`。
    pub tip: String,
    /// 可选：改写 root commit message（对应旧 `git-root`）。
    pub message: Option<String>,
}

/// retime 完成后写入的分支名与新 tip。
#[derive(Debug, Clone)]
pub struct RetimeSummary {
    /// 创建或更新的分支名。
    pub branch: String,
    /// 新分支 tip OID。
    pub new_tip: ObjectId,
    /// 改写的 commit 数量。
    pub rewritten: usize,
}

/// 解析 `YYYY-MM-DD` 为当天 00:00:00。
pub fn parse_date(input: &str) -> Result<NaiveDateTime> {
    let date = NaiveDate::parse_from_str(input, "%Y-%m-%d").map_err(|_| validation("date parse failed"))?;
    Ok(date.and_time(NaiveTime::MIN))
}

/// 解析日期或 ISO datetime（`YYYY-MM-DD`、`YYYY-MM-DDTHH:MM:SS`、`YYYY-MM-DD HH:MM:SS`）。
pub fn parse_datetime(input: &str) -> Result<NaiveDateTime> {
    const FORMATS: &[&str] = &["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"];
    for format in FORMATS {
        if let Ok(value) = NaiveDateTime::parse_from_str(input, format) {
            return Ok(value);
        }
    }
    parse_date(input)
}

/// 将 commit 的 author 时间转为 UTC naive datetime（用作默认窗口起点）。
pub fn commit_author_datetime(repo: &Repository, oid: ObjectId) -> Result<NaiveDateTime> {
    let commit = read_commit(repo, oid)?;
    let decoded = commit.decode().map_err(|err| validation(err.to_string()))?;
    let seconds = decoded.author().time.seconds;
    DateTime::from_timestamp(seconds, 0)
        .map(|value| value.naive_utc())
        .ok_or_else(|| validation("commit author timestamp out of range"))
}

/// 在 `[start, end)` 内生成 `count` 个不重复 Unix 秒时间戳（升序）。
pub fn random_timestamps(count: usize, start: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<i64>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let start_secs = start.and_utc().timestamp();
    let end_secs = end.and_utc().timestamp();
    if end_secs <= start_secs {
        return Err(validation("end datetime must be after start datetime"));
    }
    let mut rng = rand::thread_rng();
    let mut stamps = BTreeSet::new();
    while stamps.len() < count {
        stamps.insert(rng.gen_range(start_secs..end_secs));
    }
    Ok(stamps.into_iter().collect())
}

/// 将 commit 链依次赋予新时间戳；仅改 author/committer 的 `time`，保留 name/email/offset。
pub fn plan_retime_chain(
    repo: &Repository,
    chain: &[ObjectId],
    timestamps: &[i64],
    root_message: Option<&str>,
) -> Result<ObjectId> {
    if chain.is_empty() {
        return Err(validation("no commits in retime range"));
    }
    if chain.len() != timestamps.len() {
        return Err(validation(format!("need {} timestamp(s) for retime range but got {}", chain.len(), timestamps.len())));
    }

    let tip = *chain.last().expect("non-empty chain");
    let mut substitution = std::collections::HashMap::new();
    for (index, old_oid) in chain.iter().copied().enumerate() {
        let commit = read_commit(repo, old_oid)?;
        let old_parents = commit_parents(&commit);
        let new_parents: Vec<ObjectId> =
            old_parents.iter().map(|parent| substitution.get(parent).copied().unwrap_or(*parent)).collect();

        let decoded = commit.decode().map_err(|err| validation(err.to_string()))?;
        let seconds = timestamps[index];
        let author = retime_signature(copy_signature(decoded.author().into()), seconds);
        let committer = retime_signature(copy_signature(decoded.committer().into()), seconds);
        let message = if index == 0 {
            root_message.map(str::to_string).unwrap_or_else(|| commit_message(&commit))
        }
        else {
            commit_message(&commit)
        };
        let new_oid = write_commit_with_signatures(repo, &commit, &new_parents, &message, author, committer)?;
        substitution.insert(old_oid, new_oid);
    }

    substitution.get(&tip).copied().ok_or_else(|| validation("failed to resolve new tip after retime"))
}

/// 将 `exclusive_base..tip` 上的 commit 依次赋予新时间戳。
pub fn plan_retime(repo: &Repository, exclusive_base: ObjectId, tip: ObjectId, timestamps: &[i64]) -> Result<ObjectId> {
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    plan_retime_chain(repo, &chain, timestamps, None)
}

/// 按 [`RetimeOptions`] 解析范围、生成时间戳、改写对象并创建分支。
#[instrument(skip(repo, options), fields(commit = %options.commit, tip = %options.tip))]
pub fn run_retime(repo: &Repository, options: &RetimeOptions) -> Result<RetimeSummary> {
    let exclusive_base = super::history::resolve_rev(repo, &options.commit)?;
    let tip = resolve_tip(repo, &options.tip)?;
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    let start = resolve_start(repo, options.start_date.as_deref(), exclusive_base)?;
    let end = resolve_end(options.end_date.as_deref(), start, chain.len())?;
    let timestamps = random_timestamps(chain.len(), start, end)?;
    let branch = options.branch.clone().unwrap_or_else(|| "time-travel".to_string());
    let new_tip = plan_retime_chain(repo, &chain, &timestamps, None)?;
    set_branch_tip(repo, &branch, new_tip)?;
    Ok(RetimeSummary { branch, new_tip, rewritten: chain.len() })
}

/// 从 root 到 `tip`（含 root）retime 并创建分支。
#[instrument(skip(repo, options), fields(tip = %options.tip))]
pub fn run_retime_root(repo: &Repository, options: &RetimeRootOptions) -> Result<RetimeSummary> {
    let tip = resolve_tip(repo, &options.tip)?;
    let root = find_root_commit(repo, tip)?;
    let mut chain = commits_in_range(repo, root, tip)?;
    chain.insert(0, root);
    let start = resolve_start(repo, options.start_date.as_deref(), root)?;
    let end = resolve_end(options.end_date.as_deref(), start, chain.len())?;
    let timestamps = random_timestamps(chain.len(), start, end)?;
    let branch = options.branch.clone().unwrap_or_else(|| "time-travel".to_string());
    let new_tip = plan_retime_chain(repo, &chain, &timestamps, options.message.as_deref())?;
    set_branch_tip(repo, &branch, new_tip)?;
    Ok(RetimeSummary { branch, new_tip, rewritten: chain.len() })
}

fn resolve_tip(repo: &Repository, tip: &str) -> Result<ObjectId> {
    use super::history::{resolve_ref_tip, resolve_rev};
    if tip == "HEAD" { resolve_rev(repo, "HEAD") } else { resolve_ref_tip(repo, tip) }
}

fn resolve_start(repo: &Repository, start_date: Option<&str>, fallback_commit: ObjectId) -> Result<NaiveDateTime> {
    match start_date {
        Some(value) => parse_datetime(value),
        None => commit_author_datetime(repo, fallback_commit),
    }
}

fn resolve_end(end_date: Option<&str>, start: NaiveDateTime, commit_count: usize) -> Result<NaiveDateTime> {
    match end_date {
        Some(value) => parse_datetime(value),
        None => Ok(start + Duration::days(commit_count as i64)),
    }
}
