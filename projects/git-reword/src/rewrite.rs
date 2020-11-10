use std::collections::HashMap;
use std::path::Path;

use gix::{ObjectId, Repository};

use crate::error::{Result, RewordError};
use crate::repo::{
    commit_message, commit_parents, commit_subject, commits_in_range, read_commit, synthetic_oid,
    write_commit,
};

#[derive(Debug, Clone)]
pub struct PlannedChange {
    pub old_oid: ObjectId,
    pub old_subject: String,
    pub new_subject: String,
    pub parents_relinked: bool,
    pub message_changed: bool,
}

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

    let mut substitution: HashMap<ObjectId, ObjectId> = HashMap::new();
    let mut changes = Vec::new();

    for old_oid in chain {
        let commit = read_commit(repo, old_oid)?;
        let old_parents = commit_parents(&commit);
        let new_parents: Vec<ObjectId> = old_parents
            .iter()
            .map(|parent| substitution.get(parent).copied().unwrap_or(*parent))
            .collect();

        let old_message = commit_message(&commit);
        let new_message = updates
            .get(&old_oid)
            .cloned()
            .unwrap_or_else(|| old_message.clone());
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

    let new_tip = substitution
        .get(&tip)
        .copied()
        .ok_or_else(|| RewordError::msg("failed to resolve new tip"))?;
    Ok((changes, new_tip))
}

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
        let new_parents: Vec<ObjectId> = old_parents
            .iter()
            .map(|parent| substitution.get(parent).copied().unwrap_or(*parent))
            .collect();

        let old_message = commit_message(&commit);
        let new_message = updates
            .get(&old_oid)
            .cloned()
            .unwrap_or_else(|| old_message.clone());
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
        } else {
            substitution.insert(old_oid, old_oid);
        }
    }
    Ok(changes)
}

pub fn full_message(repo: &Repository, oid: ObjectId) -> Result<String> {
    Ok(commit_message(&read_commit(repo, oid)?))
}

pub fn export_map(repo: &Repository, exclusive_base: ObjectId, tip: ObjectId, path: &Path) -> Result<()> {
    let chain = commits_in_range(repo, exclusive_base, tip)?;
    let mut out = String::new();
    out.push_str(&format!(
        "# Hash-keyed reword map ({} commit(s)). Delete unchanged blocks before rewrite.\n\n",
        chain.len()
    ));
    for oid in chain {
        let message = full_message(repo, oid)?;
        out.push_str(&format!("{}\n{}\n\n---\n\n", oid, message));
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub use crate::repo::{commits_in_range as collect_commits, short, update_ref as move_ref};
