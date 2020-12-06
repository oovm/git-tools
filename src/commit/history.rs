//! commit 对象读写、历史范围遍历与引用更新。

use std::path::Path;

use gix::{
    Commit, ObjectId, Repository,
    refs::{
        Target,
        transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog},
    },
    revision::walk::Sorting,
    traverse::commit::simple::CommitTimeOrder,
};

use crate::error::{Result, ResultExt, message, validation};

use super::identity::{copy_signature, same_contributor};

/// 从路径发现 git 工作区并打开仓库。
pub fn open(path: &Path) -> Result<Repository> {
    Ok(gix::discover(path).or_raise(|| message!("discover git repository"))?)
}

/// 从当前工作目录向上发现 git 仓库并打开。
pub fn open_here() -> Result<Repository> {
    open(&std::env::current_dir().or_raise(|| message!("read current directory"))?)
}

/// 将 revision 字符串解析为对象 OID（如 `HEAD`、`34e1e665^`）。
pub fn resolve_rev(repo: &Repository, rev: &str) -> Result<ObjectId> {
    Ok(repo.rev_parse_single(rev).or_raise(|| message!("resolve revision"))?.detach())
}

/// 解析分支或符号引用名，得到其 tip 的 OID。
pub fn resolve_ref_tip(repo: &Repository, ref_name: &str) -> Result<ObjectId> {
    if ref_name == "HEAD" {
        return resolve_rev(repo, "HEAD");
    }
    let name = if ref_name.starts_with("refs/") { ref_name.to_string() } else { format!("refs/heads/{}", ref_name) };
    let reference = repo.find_reference(&name).or_raise(|| message!("find git reference"))?;
    Ok(reference.id().detach())
}

/// 返回当前 HEAD 指向的引用全名（ detached HEAD 时报错）。
pub fn head_ref_name(repo: &Repository) -> Result<String> {
    let head = repo.head().or_raise(|| message!("read HEAD reference"))?;
    if head.is_detached() {
        return Err(validation("HEAD is detached; pass an explicit --ref"));
    }
    Ok(head.name().as_bstr().to_string())
}

/// 沿 first-parent 链从 `tip` 追溯到无父 commit（root）。
pub fn find_root_commit(repo: &Repository, tip: ObjectId) -> Result<ObjectId> {
    let mut current = tip;
    loop {
        let commit = read_commit(repo, current)?;
        let parents = commit_parents(&commit);
        if parents.is_empty() {
            return Ok(current);
        }
        current = parents[0];
    }
}

/// 收集 `exclusive_base..tip` 范围内的 commit OID，按 commit 时间从旧到新。
pub fn commits_in_range(repo: &Repository, exclusive_base: ObjectId, tip: ObjectId) -> Result<Vec<ObjectId>> {
    let mut ids = Vec::new();
    for info in repo
        .rev_walk([tip])
        .with_pruned([exclusive_base])
        .sorting(Sorting::ByCommitTime(CommitTimeOrder::OldestFirst))
        .all()
        .or_raise(|| message!("configure revision walk"))?
    {
        ids.push(info.or_raise(|| message!("walk commit history"))?.id().detach());
    }
    Ok(ids)
}

/// 读取并解析 commit 对象。
pub fn read_commit(repo: &Repository, oid: ObjectId) -> Result<Commit<'_>> {
    let object = repo.find_object(oid).or_raise(|| message!("find commit object"))?;
    Ok(object.into_commit())
}

/// 返回 commit 的全部父 commit OID。
pub fn commit_parents(commit: &Commit<'_>) -> Vec<ObjectId> {
    commit.parent_ids().map(|id| id.detach()).collect()
}

/// 返回 commit 的完整 message 文本（含 subject 与 body）。
pub fn commit_message(commit: &Commit<'_>) -> String {
    commit.message_raw_sloppy().to_string().trim_end().to_string()
}

/// 返回 commit message 的第一行 subject。
pub fn commit_subject(commit: &Commit<'_>) -> String {
    commit_message(commit).lines().next().unwrap_or("").trim().to_string()
}

/// 写入新 commit 对象：复用 tree 与 author/committer 身份，替换 parents 与 message。
pub fn write_commit(repo: &Repository, commit: &Commit<'_>, parents: &[ObjectId], message: &str) -> Result<ObjectId> {
    let decoded = commit.decode().or_raise(|| message!("decode commit object"))?;
    let commit_obj = gix::objs::Commit {
        tree: decoded.tree(),
        parents: parents.iter().copied().collect(),
        author: copy_signature(decoded.author().into()),
        committer: copy_signature(decoded.committer().into()),
        message: message.into(),
        encoding: decoded.encoding.map(|encoding| encoding.to_owned()),
        extra_headers: decoded
            .extra_headers
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.as_ref().to_owned()))
            .collect(),
    };
    Ok(repo.write_object(&commit_obj).or_raise(|| message!("write commit object"))?.detach())
}

/// 写入新 commit 对象：替换 parents、message 与 author/committer 签名。
///
/// 调用方须保证 `author` / `committer` 的 name 与 email 与源 commit 一致（`retime` 仅改时间戳）。
pub fn write_commit_with_signatures(
    repo: &Repository,
    commit: &Commit<'_>,
    parents: &[ObjectId],
    message: &str,
    author: gix::actor::Signature,
    committer: gix::actor::Signature,
) -> Result<ObjectId> {
    let decoded = commit.decode().or_raise(|| message!("decode commit object"))?;
    let source_author = copy_signature(decoded.author().into());
    let source_committer = copy_signature(decoded.committer().into());
    if !same_contributor(&author, &source_author) || !same_contributor(&committer, &source_committer) {
        return Err(validation("author or committer identity must match the source commit"));
    }
    let commit_obj = gix::objs::Commit {
        tree: decoded.tree(),
        parents: parents.iter().copied().collect(),
        author,
        committer,
        message: message.into(),
        encoding: decoded.encoding.map(|encoding| encoding.to_owned()),
        extra_headers: decoded
            .extra_headers
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.as_ref().to_owned()))
            .collect(),
    };
    Ok(repo.write_object(&commit_obj).or_raise(|| message!("write commit object"))?.detach())
}

/// 创建或强制更新本地分支 tip（等价于 `git branch -f`）。
pub fn set_branch_tip(repo: &Repository, branch: &str, tip: ObjectId) -> Result<()> {
    let ref_name = if branch.starts_with("refs/") { branch.to_string() } else { format!("refs/heads/{}", branch) };
    let name: gix::refs::FullName =
        ref_name.try_into().map_err(|err: gix::validate::reference::name::Error| validation(err.to_string()))?;
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: format!("git-retime: update {} to {}", branch, crate::repo::short(tip)).into(),
            },
            expected: PreviousValue::Any,
            new: Target::Object(tip),
        },
        name,
        deref: false,
    })
    .or_raise(|| message!("update git branch"))?;
    Ok(())
}

/// 将引用 `ref_name` 的 tip 从 `old_tip` 更新为 `new_tip`（带 reflog）。
pub fn update_ref(repo: &Repository, ref_name: &str, new_tip: ObjectId, old_tip: ObjectId) -> Result<()> {
    let name: gix::refs::FullName =
        ref_name.try_into().map_err(|err: gix::validate::reference::name::Error| validation(err.to_string()))?;
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: format!("git-reword: rewrite {}..{}", crate::repo::short(old_tip), crate::repo::short(new_tip)).into(),
            },
            expected: PreviousValue::MustExistAndMatch(Target::Object(old_tip)),
            new: Target::Object(new_tip),
        },
        name,
        deref: false,
    })
    .or_raise(|| message!("update git reference"))?;
    Ok(())
}

/// dry-run 时生成与真实 OID 不同的占位 OID，避免误写对象库。
pub fn synthetic_oid(seed: ObjectId) -> ObjectId {
    let mut bytes = seed.as_bytes().to_vec();
    bytes[19] ^= 0xff;
    ObjectId::from_bytes_or_panic(&bytes)
}
