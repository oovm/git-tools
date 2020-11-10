use std::path::Path;

use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::refs::Target;
use gix::revision::walk::Sorting;
use gix::traverse::commit::simple::CommitTimeOrder;
use gix::{Commit, ObjectId, Repository};

use crate::error::{Result, RewordError};

pub fn open(path: &Path) -> Result<Repository> {
    gix::discover(path).map_err(RewordError::from)
}

pub fn resolve_rev(repo: &Repository, rev: &str) -> Result<ObjectId> {
    Ok(repo.rev_parse_single(rev)?.detach())
}

pub fn resolve_ref_tip(repo: &Repository, ref_name: &str) -> Result<ObjectId> {
    if ref_name == "HEAD" {
        return resolve_rev(repo, "HEAD");
    }
    let name = if ref_name.starts_with("refs/") {
        ref_name.to_string()
    } else {
        format!("refs/heads/{}", ref_name)
    };
    let reference = repo.find_reference(&name)?;
    Ok(reference.id().detach())
}

pub fn head_ref_name(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    if head.is_detached() {
        return Err(RewordError::msg("HEAD is detached; pass an explicit --ref"));
    }
    Ok(head.name().as_bstr().to_string())
}

/// Commits reachable from `tip` but not from `exclusive_base`, oldest first.
pub fn commits_in_range(repo: &Repository, exclusive_base: ObjectId, tip: ObjectId) -> Result<Vec<ObjectId>> {
    let mut ids = Vec::new();
    for info in repo
        .rev_walk([tip])
        .with_pruned([exclusive_base])
        .sorting(Sorting::ByCommitTime(CommitTimeOrder::OldestFirst))
        .all()?
    {
        ids.push(info?.id().detach());
    }
    Ok(ids)
}

pub fn read_commit(repo: &Repository, oid: ObjectId) -> Result<Commit<'_>> {
    let object = repo.find_object(oid)?;
    Ok(object.into_commit())
}

pub fn commit_parents(commit: &Commit<'_>) -> Vec<ObjectId> {
    commit.parent_ids().map(|id| id.detach()).collect()
}

pub fn commit_message(commit: &Commit<'_>) -> String {
    commit.message_raw_sloppy().to_string().trim_end().to_string()
}

pub fn commit_subject(commit: &Commit<'_>) -> String {
    commit_message(commit)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn write_commit(
    repo: &Repository,
    commit: &Commit<'_>,
    parents: &[ObjectId],
    message: &str,
) -> Result<ObjectId> {
    let decoded = commit.decode()?;
    let commit_obj = gix::objs::Commit {
        tree: decoded.tree(),
        parents: parents.iter().copied().collect(),
        author: decoded.author().into(),
        committer: decoded.committer().into(),
        message: message.into(),
        encoding: decoded.encoding.map(|encoding| encoding.to_owned()),
        extra_headers: decoded
            .extra_headers
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.as_ref().to_owned()))
            .collect(),
    };
    Ok(repo.write_object(&commit_obj)?.detach())
}

pub fn update_ref(repo: &Repository, ref_name: &str, new_tip: ObjectId, old_tip: ObjectId) -> Result<()> {
    let name: gix::refs::FullName = ref_name
        .try_into()
        .map_err(|err: gix::validate::reference::name::Error| RewordError::msg(err.to_string()))?;
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: format!("git-reword: rewrite {}..{}", short(old_tip), short(new_tip)).into(),
            },
            expected: PreviousValue::MustExistAndMatch(Target::Object(old_tip)),
            new: Target::Object(new_tip),
        },
        name,
        deref: false,
    })?;
    Ok(())
}

pub fn short(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}

pub fn synthetic_oid(seed: ObjectId) -> ObjectId {
    let mut bytes = seed.as_bytes().to_vec();
    bytes[19] ^= 0xff;
    ObjectId::from_bytes_or_panic(&bytes)
}
