//! 对象层改写规划：按范围自旧向新 relink 父链并写入新 commit。

use std::{collections::HashMap, path::Path};

use gix::{ObjectId, Repository};

use crate::{
    error::{Result, RewordError},
    repo::{commit_message, commit_parents, commit_subject, commits_in_range, read_commit, synthetic_oid, write_commit},
};

/// 单次计划中的 commit 变更摘要。
#[derive(Debug, Clone)]
pub struct PlannedChange {
    /// 原 commit OID。
    pub old_oid: ObjectId,
    /// 改写前的 subject。
    pub old_subject: String,
    /// 改写后的 subject。
    pub new_subject: String,
    /// 是否因祖先被改写而仅 relink 父指针。
    pub parents_relinked: bool,
    /// message 是否与原文不同。
    pub message_changed: bool,
}

/// 按 `updates` 映射在 `exclusive_base..tip` 上规划并执行对象改写，返回变更列表与新 tip。
///
/// 自旧向新遍历：若 message 或父 OID 变化则写新 commit，否则复用原 OID。
pub fn plan_rewrite(
    repo: &Repository,
    exclusive_base: ObjectId,
    tip: ObjectId,
    updates: &HashMap<ObjectId, String>,
) -> Result<(Vec<PlannedChange>, ObjectId)> {
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    if chain.is_empty() {
        return Err(RewordError::msg("no commits in rewrite range"));
    }

    // old_oid -> new_oid，供后续 commit relink 父链
    let mut substitution: HashMap<ObjectId, ObjectId> = HashMap::new();
    let mut changes = Vec::new();

    for old_oid in chain {
        let commit = read_commit(repo, old_oid)?;
        let old_parents = commit_parents(&commit);
        let new_parents: Vec<ObjectId> =
            old_parents.iter().map(|parent| substitution.get(parent).copied().unwrap_or(*parent)).collect();

        let old_message = commit_message(&commit);
        let new_message = updates.get(&old_oid).cloned().unwrap_or_else(|| old_message.clone());
        let message_changed = new_message.trim() != old_message.trim();
        let parents_changed = new_parents != old_parents;

        let old_subject = commit_subject(&commit);
        let new_subject = new_message.lines().next().unwrap_or("").trim().to_string();

        if message_changed || parents_changed {
            changes.push(PlannedChange {
                old_oid,
                old_subject,
                new_subject,
                parents_relinked: parents_changed,
                message_changed,
            });
        }

        if !message_changed && !parents_changed {
            substitution.insert(old_oid, old_oid);
            continue;
        }

        let new_oid = write_commit(repo, &commit, &new_parents, new_message.trim())?;
        substitution.insert(old_oid, new_oid);
    }

    let new_tip = substitution.get(&tip).copied().ok_or_else(|| RewordError::msg("failed to resolve new tip"))?;
    Ok((changes, new_tip))
}

/// 不写入对象库，仅规划哪些 commit 会被改写（dry-run）。
pub fn dry_run_plan(
    repo: &Repository,
    exclusive_base: ObjectId,
    tip: ObjectId,
    updates: &HashMap<ObjectId, String>,
) -> Result<Vec<PlannedChange>> {
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    let mut substitution: HashMap<ObjectId, ObjectId> = HashMap::new();
    let mut changes = Vec::new();

    for old_oid in chain {
        let commit = read_commit(repo, old_oid)?;
        let old_parents = commit_parents(&commit);
        let new_parents: Vec<ObjectId> =
            old_parents.iter().map(|parent| substitution.get(parent).copied().unwrap_or(*parent)).collect();

        let old_message = commit_message(&commit);
        let new_message = updates.get(&old_oid).cloned().unwrap_or_else(|| old_message.clone());
        let message_changed = new_message.trim() != old_message.trim();
        let parents_changed = new_parents != old_parents;

        if message_changed || parents_changed {
            let old_subject = commit_subject(&commit);
            let new_subject = new_message.lines().next().unwrap_or("").trim().to_string();
            changes.push(PlannedChange {
                old_oid,
                old_subject,
                new_subject,
                parents_relinked: parents_changed,
                message_changed,
            });
            substitution.insert(old_oid, synthetic_oid(old_oid));
        }
        else {
            substitution.insert(old_oid, old_oid);
        }
    }
    Ok(changes)
}

/// 读取指定 commit 的完整 message。
pub fn full_message(repo: &Repository, oid: ObjectId) -> Result<String> {
    Ok(commit_message(&read_commit(repo, oid)?))
}

/// 导出 `exclusive_base..tip` 范围内各 commit 的 hash 映射模板文件。
pub fn export_map(repo: &Repository, exclusive_base: ObjectId, tip: ObjectId, path: &Path) -> Result<()> {
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    let mut out = String::new();
    out.push_str(&format!("# Hash-keyed reword map ({} commit(s)). Delete unchanged blocks before rewrite.\n\n", chain.len()));
    for oid in chain {
        let message = full_message(repo, oid)?;
        out.push_str(&format!("{}\n{}\n\n---\n\n", oid, message));
    }
    std::fs::write(path, out)?;
    Ok(())
}

/// 收集 `exclusive_base..tip` 范围内的 commit OID（自旧到新）。
pub use crate::repo::commits_in_range as collect_commits;
/// 格式化 OID 为 8 位前缀。
pub use crate::repo::short;
/// 更新分支引用 tip。
pub use crate::repo::update_ref as move_ref;
