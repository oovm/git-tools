//! release 参考 changelog：tag 区间、gitmoji 分组、贡献者头像墙。

pub mod author;
pub mod git_cmd;
pub mod gitmoji;
pub mod render;

pub use author::{
    GithubAuthor, display_login, fetch_github_user_by_login, github_from_noreply_email, load_author_map,
    lookup_github_user_by_email, parse_author_entry, search_github_user_by_email,
};
pub use git_cmd::{
    CommitEntry, collect_commits, detect_github_repo, format_tag_list, list_version_tags, normalize_version,
    parse_github_remote_repo, previous_version_tag, resolve_range, verify_commit_ref, version_tag,
};
pub use render::{collect_contributors_from_commits, group_commits, render_contributor_wall, render_reference};
