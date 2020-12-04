//! 对象层改写 commit author/committer 时间并写入新分支。

use std::collections::BTreeSet;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use gix::{ObjectId, Repository, actor::Signature};
use rand::Rng;

use crate::error::{Result, validation};

use super::history::{
    commit_message, commit_parents, commits_in_range, read_commit, set_branch_tip, write_commit_with_signatures,
};

/// 范围 retime：改写 `(commit..tip]`。
#[derive(Debug, Clone)]
pub struct RetimeOptions {
    /// 范围起点 commit（改写 `(commit..tip]`，不含 `commit` 本身）。
    pub commit: String,
    /// 随机时间分布起始日期（`YYYY-MM-DD`）。
    pub start_date: String,
    /// 随机时间分布结束日期；缺省为 `start_date + commit 数量` 天。
    pub end_date: Option<String>,
    /// 新分支名；缺省为 `time-travel`。
    pub branch: Option<String>,
    /// 范围终点 revision；缺省为 `HEAD`。
    pub tip: String,
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

/// 在 `[start, end)` 内生成 `count` 个不重复 Unix 秒时间戳（升序）。
pub fn random_timestamps(count: usize, start: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<i64>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let start_secs = start.and_utc().timestamp();
    let end_secs = end.and_utc().timestamp();
    if end_secs <= start_secs {
        return Err(validation("end date must be after start date"));
    }
    let mut rng = rand::thread_rng();
    let mut stamps = BTreeSet::new();
    while stamps.len() < count {
        stamps.insert(rng.gen_range(start_secs..end_secs));
    }
    Ok(stamps.into_iter().collect())
}

/// 将 `exclusive_base..tip` 上的 commit 依次赋予新时间戳并写入对象库。
pub fn plan_retime(repo: &Repository, exclusive_base: ObjectId, tip: ObjectId, timestamps: &[i64]) -> Result<ObjectId> {
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    if chain.is_empty() {
        return Err(validation("no commits in retime range"));
    }
    if chain.len() != timestamps.len() {
        return Err(validation(format!(
            "need {} timestamp(s) for retime range but got {}",
            chain.len(),
            timestamps.len()
        )));
    }

    let mut substitution = std::collections::HashMap::new();
    for (index, old_oid) in chain.into_iter().enumerate() {
        let commit = read_commit(repo, old_oid)?;
        let old_parents = commit_parents(&commit);
        let new_parents: Vec<ObjectId> =
            old_parents.iter().map(|parent| substitution.get(parent).copied().unwrap_or(*parent)).collect();

        let decoded = commit.decode().map_err(|err| validation(err.to_string()))?;
        let seconds = timestamps[index];
        let author = retime_signature(decoded.author().into(), seconds);
        let committer = retime_signature(decoded.committer().into(), seconds);
        let message = commit_message(&commit);
        let new_oid = write_commit_with_signatures(repo, &commit, &new_parents, &message, author, committer)?;
        substitution.insert(old_oid, new_oid);
    }

    substitution.get(&tip).copied().ok_or_else(|| validation("failed to resolve new tip after retime"))
}

/// 按 [`RetimeOptions`] 解析范围、生成时间戳、改写对象并创建分支。
pub fn run_retime(repo: &Repository, options: &RetimeOptions) -> Result<RetimeSummary> {
    use super::history::{resolve_ref_tip, resolve_rev};

    let exclusive_base = resolve_rev(repo, &options.commit)?;
    let tip = if options.tip == "HEAD" {
        resolve_rev(repo, "HEAD")?
    } else {
        resolve_ref_tip(repo, &options.tip)?
    };
    let commit_count = commits_in_range(repo, exclusive_base, tip)?.len();
    let start = parse_date(&options.start_date)?;
    let end = match &options.end_date {
        Some(value) => parse_date(value)?,
        None => start + Duration::days(commit_count as i64),
    };
    let timestamps = random_timestamps(commit_count, start, end)?;
    let branch = options.branch.clone().unwrap_or_else(|| "time-travel".to_string());
    let new_tip = plan_retime(repo, exclusive_base, tip, &timestamps)?;
    set_branch_tip(repo, &branch, new_tip)?;
    Ok(RetimeSummary { branch, new_tip, rewritten: commit_count })
}

fn retime_signature(mut signature: Signature, seconds: i64) -> Signature {
    signature.time.seconds = seconds;
    signature
}
