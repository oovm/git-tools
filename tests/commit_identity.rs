#![doc = "reword / retime 贡献者身份保留集成测试。"]

use git_tools::commit::{commit_parents, copy_signature, open_here, read_commit, same_contributor, write_commit};

#[test]
fn write_commit_preserves_author_and_committer() {
    let repo = open_here().expect("discover git repository from cwd");
    let head = repo.head_id().expect("read HEAD").detach();
    let commit = read_commit(&repo, head).expect("read HEAD commit");
    let decoded = commit.decode().expect("decode commit");
    let source_author = copy_signature(decoded.author().into());
    let source_committer = copy_signature(decoded.committer().into());
    let parents = commit_parents(&commit);

    let new_oid = write_commit(&repo, &commit, &parents, "identity preservation smoke test\n").expect("write commit");
    let rewritten = read_commit(&repo, new_oid).expect("read rewritten commit");
    let rewritten = rewritten.decode().expect("decode rewritten commit");
    let new_author = copy_signature(rewritten.author().into());
    let new_committer = copy_signature(rewritten.committer().into());

    assert!(same_contributor(&source_author, &new_author));
    assert!(same_contributor(&source_committer, &new_committer));
    assert_eq!(new_author.time.seconds, source_author.time.seconds);
    assert_eq!(new_committer.time.seconds, source_committer.time.seconds);
}
