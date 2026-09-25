#![doc = "changelog 作者解析与 gitmoji 分组单元测试。"]

use git_tools::changelog::{
    display_login, github_from_noreply_email, parse_author_entry,
};
use git_tools::changelog::gitmoji::{leading_gitmoji, section_for_gitmoji, strip_gitmoji, Section};
use serde_json::json;

#[test]
fn noreply_with_numeric_id() {
    let author = github_from_noreply_email("17541209+oovm@users.noreply.github.com").unwrap();
    assert_eq!(author.id, Some(17541209));
    assert_eq!(author.login.as_deref(), Some("oovm"));
}

#[test]
fn noreply_login_only() {
    let author = github_from_noreply_email("oovm@users.noreply.github.com").unwrap();
    assert_eq!(author.id, None);
    assert_eq!(author.login.as_deref(), Some("oovm"));
}

#[test]
fn parse_author_map_entry_object() {
    let entry = parse_author_entry(&json!({ "id": 17541209, "login": "oovm" })).unwrap();
    assert_eq!(entry.id, Some(17541209));
    assert_eq!(display_login(&entry), "oovm");
}

#[test]
fn gitmoji_section_and_strip() {
    let subject = "✨ Add `git-change-logs` binary";
    assert_eq!(leading_gitmoji(subject), Some("✨"));
    assert_eq!(strip_gitmoji(subject), "Add `git-change-logs` binary");
    assert_eq!(section_for_gitmoji(leading_gitmoji(subject)), Section::Features);
}

#[test]
fn detect_github_repo_from_https() {
    use git_tools::changelog::parse_github_remote_repo;
    assert_eq!(
        parse_github_remote_repo("https://github.com/valkyrie-language/valkyrie.rs.git"),
        Some("valkyrie-language/valkyrie.rs".to_string())
    );
}
