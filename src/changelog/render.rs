//! release 参考稿 Markdown 渲染。

use std::collections::{BTreeMap, HashMap};

use crate::changelog::{
    author::{GithubAuthor, avatar_url, contributor_key, display_login, merge_authors, profile_url, resolve_github_author},
    git_cmd::CommitEntry,
    gitmoji::{Section, all_sections, section_meta},
};

/// 按 gitmoji 分组 commit。
pub fn group_commits(commits: &[CommitEntry]) -> BTreeMap<Section, Vec<&CommitEntry>> {
    let mut groups = BTreeMap::new();
    for section in all_sections() {
        groups.insert(section, Vec::new());
    }
    for commit in commits {
        groups.entry(commit.section).or_default().push(commit);
    }
    groups
}

/// 从 commit 列表收集去重后的 GitHub 贡献者。
pub fn collect_contributors_from_commits(commits: &[CommitEntry], map: &HashMap<String, GithubAuthor>) -> Vec<GithubAuthor> {
    let mut by_key = BTreeMap::new();
    for commit in commits {
        let Some(author) = resolve_github_author(&commit.email, map)
        else {
            continue;
        };
        let Some(key) = contributor_key(&author)
        else {
            continue;
        };
        by_key
            .entry(key)
            .and_modify(|existing: &mut GithubAuthor| *existing = merge_authors(existing, &author))
            .or_insert(author);
    }
    let mut contributors: Vec<GithubAuthor> = by_key.into_values().collect();
    contributors.sort_by_key(display_login);
    contributors
}

/// 单条 commit bullet：`- subject (@user)`。
pub fn commit_bullet(commit: &CommitEntry, map: &HashMap<String, GithubAuthor>) -> String {
    format!("- {} ({})", commit.body, author_mention(&commit.email, &commit.author, map))
}

/// 渲染 Contributors 头像墙 HTML 片段。
pub fn render_contributor_wall(contributors: &[GithubAuthor], default_repo: &str) -> String {
    if contributors.is_empty() {
        return "(none — no GitHub login from commit email; add `documentation/maintenance/author-github.json` if needed)"
            .to_string();
    }
    if contributors.len() > 12 {
        let users: Vec<&str> = contributors.iter().filter_map(|author| author.login.as_deref()).collect();
        if users.is_empty() {
            return "(none — contributors have numeric ids only; use the table layout below 12 people or add `login` to author-github.json)".to_string();
        }
        return format!(
            "<a href=\"https://github.com/{default_repo}/graphs/contributors\">\n  <img src=\"https://contrib.rocks/image?users={}&columns=9\" alt=\"Contributors\" />\n</a>",
            users.join(",")
        );
    }

    let columns = contributors.len().min(6);
    let mut rows = Vec::new();
    for chunk in contributors.chunks(columns) {
        let cells: Vec<String> = chunk
            .iter()
            .map(|contributor| {
                let login = display_login(contributor);
                let href = profile_url(contributor);
                let src = avatar_url(contributor);
                format!(
                    "<td align=\"center\"><a href=\"{href}\"><img src=\"{src}\" width=\"64\" height=\"64\" alt=\"@{login}\"/><br /><sub><b>{login}</b></sub></a></td>"
                )
            })
            .collect();
        rows.push(format!("<tr>\n{}\n</tr>", cells.join("\n")));
    }
    format!("<table>\n<tbody>\n{}\n</tbody>\n</table>", rows.join("\n"))
}

/// 渲染完整 reference markdown。
pub fn render_reference(
    version: &str,
    from_ref: Option<&str>,
    to_ref: &str,
    groups: &BTreeMap<Section, Vec<&CommitEntry>>,
    contributors: &str,
    commit_count: usize,
    map: &HashMap<String, GithubAuthor>,
) -> String {
    let range_label = match from_ref {
        Some(from) => format!("{from}..{to_ref}"),
        None => to_ref.to_string(),
    };
    let mut lines = vec![
        format!("# Reference: v{version} (`{range_label}`)"),
        String::new(),
        format!(
            "> Commit index for drafting `documentation/maintenance/releases/v{version}.md`. Temporary file — do not commit or publish as the GitHub Release body."
        ),
        String::new(),
    ];

    for section in all_sections() {
        let (heading, empty) = section_meta(section);
        lines.push(heading.to_string());
        lines.push(String::new());
        let items = groups.get(&section).map_or(&[] as &[_], Vec::as_slice);
        if items.is_empty() {
            lines.push(empty.to_string());
            lines.push(String::new());
        }
        else {
            for commit in items {
                lines.push(commit_bullet(commit, map));
            }
            lines.push(String::new());
        }
    }

    lines.push("## 👥 Contributors".to_string());
    lines.push(String::new());
    lines.push(contributors.to_string());
    lines.push(String::new());
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push(format!("{commit_count} commit(s) in range."));
    lines.push(String::new());
    lines.join("\n")
}

/// Markdown 作者提及（链接或 `@name`）。
pub fn author_mention(email: &str, author_name: &str, map: &HashMap<String, GithubAuthor>) -> String {
    if let Some(github) = resolve_github_author(email, map) {
        let label = format!("@{}", display_login(&github));
        return format!("[{label}]({})", profile_url(&github));
    }
    let name = author_name.trim();
    if !name.is_empty() {
        return format!("@{}", name.replace(' ', ""));
    }
    if let Some(local) = email.split('@').next().map(str::trim).filter(|part| !part.is_empty()) {
        return format!("@{local}");
    }
    "@unknown".to_string()
}
