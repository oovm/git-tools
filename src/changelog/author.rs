//! GitHub 作者解析：noreply 邮箱、本地映射与 GitHub API 查询。

use std::{collections::HashMap, path::Path};

use regex::Regex;
use serde_json::Value;

use crate::error::{Result, validation, validation_with_input};

/// GitHub 用户标识（numeric id 优先用于稳定头像 URL）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubAuthor {
    /// GitHub numeric user id.
    pub id: Option<u64>,
    /// GitHub login handle.
    pub login: Option<String>,
}

/// 从 `author-github.json` 加载 email → GitHub 映射（email 键小写）。
pub fn load_author_map(path: &Path) -> HashMap<String, GithubAuthor> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text.trim_start_matches('\u{feff}').to_string(),
        Err(_) => return HashMap::new(),
    };
    let raw: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => return HashMap::new(),
    };
    let Some(object) = raw.as_object()
    else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (email, entry) in object {
        if let Some(parsed) = parse_author_entry(entry) {
            out.insert(email.trim().to_lowercase(), parsed);
        }
    }
    out
}

/// 解析映射文件中的单条 entry（字符串 login、数字 id 或 `{ id, login }` 对象）。
pub fn parse_author_entry(value: &Value) -> Option<GithubAuthor> {
    match value {
        Value::String(login) => {
            let login = login.trim();
            if login.is_empty() { None } else { Some(GithubAuthor { id: None, login: Some(login.to_string()) }) }
        }
        Value::Number(number) => number.as_u64().map(|id| GithubAuthor { id: Some(id), login: None }),
        Value::Object(object) => {
            let login = object.get("login").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
            let id = object.get("id").and_then(Value::as_u64);
            if login.is_some() || id.is_some() { Some(GithubAuthor { id, login: login.map(str::to_string) }) } else { None }
        }
        _ => None,
    }
}

/// 解析 GitHub noreply 邮箱为 `{ id, login }`。
pub fn github_from_noreply_email(email: &str) -> Option<GithubAuthor> {
    static WITH_ID: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static LOGIN_ONLY: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let with_id = WITH_ID.get_or_init(|| Regex::new(r"(?i)^(\d+)\+([^@+]+)@users\.noreply\.github\.com$").unwrap());
    let login_only = LOGIN_ONLY.get_or_init(|| Regex::new(r"(?i)^([^@+]+)@users\.noreply\.github\.com$").unwrap());

    if let Some(caps) = with_id.captures(email) {
        let id = caps.get(1)?.as_str().parse().ok()?;
        let login = caps.get(2)?.as_str().to_string();
        return Some(GithubAuthor { id: Some(id), login: Some(login) });
    }
    if let Some(caps) = login_only.captures(email) {
        let login = caps.get(1)?.as_str().to_string();
        return Some(GithubAuthor { id: None, login: Some(login) });
    }
    None
}

/// 合并 noreply 解析与本地映射，得到 GitHub 作者信息。
pub fn resolve_github_author(email: &str, map: &HashMap<String, GithubAuthor>) -> Option<GithubAuthor> {
    if let Some(from_noreply) = github_from_noreply_email(email) {
        return Some(from_noreply);
    }
    map.get(&email.trim().to_lowercase()).cloned()
}

/// 贡献者去重键。
pub fn contributor_key(author: &GithubAuthor) -> Option<String> {
    if let Some(id) = author.id {
        return Some(format!("id:{id}"));
    }
    author.login.as_ref().map(|login| format!("login:{login}"))
}

/// 展示用 login（无 login 时用 `user-{id}`）。
pub fn display_login(author: &GithubAuthor) -> String {
    if let Some(login) = &author.login {
        return login.clone();
    }
    if let Some(id) = author.id {
        return format!("user-{id}");
    }
    "unknown".to_string()
}

/// GitHub profile URL。
pub fn profile_url(author: &GithubAuthor) -> String {
    if let Some(login) = &author.login {
        return format!("https://github.com/{login}");
    }
    if let Some(id) = author.id {
        return format!("https://github.com/user/{id}");
    }
    "#".to_string()
}

/// 头像 URL（优先 numeric id）。
pub fn avatar_url(author: &GithubAuthor) -> String {
    if let Some(id) = author.id {
        return format!("https://avatars.githubusercontent.com/u/{id}?s=100");
    }
    if let Some(login) = &author.login {
        return format!("https://github.com/{login}.png?s=100");
    }
    String::new()
}

/// 合并两位贡献者信息（补全缺失的 id / login）。
pub fn merge_authors(existing: &GithubAuthor, incoming: &GithubAuthor) -> GithubAuthor {
    GithubAuthor { id: existing.id.or(incoming.id), login: existing.login.clone().or_else(|| incoming.login.clone()) }
}

/// 通过 GitHub REST API 按 login 查询 numeric id。
pub fn fetch_github_user_by_login(login: &str, token: Option<&str>) -> Result<GithubAuthor> {
    let url = format!("https://api.github.com/users/{login}");
    let mut request = ureq::get(&url).set("Accept", "application/vnd.github+json").set("User-Agent", "git-tools");
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request.call().map_err(|err| validation_with_input("GitHub API request failed", err.to_string()))?;
    if response.status() == 404 {
        return Err(validation_with_input("GitHub user not found", login));
    }
    if !(200..300).contains(&response.status()) {
        return Err(validation_with_input(
            "GitHub API returned an error status",
            format!("{} {}", response.status(), response.status_text()),
        ));
    }
    let body: Value = response
        .into_json()
        .map_err(|err| validation(format!("parse GitHub API JSON: {err}")))?;
    let id = body.get("id").and_then(Value::as_u64);
    let api_login = body.get("login").and_then(Value::as_str).map(str::to_string);
    Ok(GithubAuthor { id, login: api_login.or_else(|| Some(login.to_string())) })
}

/// 通过 GitHub Search API 按 email 查询用户（需 token，且受 GitHub 隐私策略限制）。
pub fn search_github_user_by_email(email: &str, token: &str) -> Result<Option<GithubAuthor>> {
    let query = format!("{} in:email", email);
    let url = format!("https://api.github.com/search/users?q={}", urlencoding::encode(&query));
    let response = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "git-tools")
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|err| validation_with_input("GitHub search API request failed", err.to_string()))?;
    if !(200..300).contains(&response.status()) {
        return Err(validation_with_input(
            "GitHub search API returned an error status",
            format!("{} {}", response.status(), response.status_text()),
        ));
    }
    let body: Value = response
        .into_json()
        .map_err(|err| validation(format!("parse GitHub search JSON: {err}")))?;
    let items = body.get("items").and_then(Value::as_array);
    let Some(items) = items
    else {
        return Ok(None);
    };
    let first = items.first().and_then(Value::as_object);
    let Some(first) = first
    else {
        return Ok(None);
    };
    let id = first.get("id").and_then(Value::as_u64);
    let login = first.get("login").and_then(Value::as_str).map(str::to_string);
    if id.is_none() && login.is_none() {
        return Ok(None);
    }
    Ok(Some(GithubAuthor { id, login }))
}

/// 按 email 解析 GitHub 用户：noreply → 本地映射 → 可选 API 补全 / 搜索。
pub fn lookup_github_user_by_email(
    email: &str,
    map: &HashMap<String, GithubAuthor>,
    token: Option<&str>,
    fetch: bool,
) -> Result<Option<GithubAuthor>> {
    if let Some(from_noreply) = github_from_noreply_email(email) {
        return Ok(Some(enrich_author(from_noreply, token, fetch)?));
    }
    if let Some(mapped) = map.get(&email.trim().to_lowercase()).cloned() {
        return Ok(Some(enrich_author(mapped, token, fetch)?));
    }
    if let Some(token) = token {
        if let Some(found) = search_github_user_by_email(email, token)? {
            return Ok(Some(enrich_author(found, Some(token), fetch)?));
        }
    }
    Ok(None)
}

fn enrich_author(author: GithubAuthor, token: Option<&str>, fetch: bool) -> Result<GithubAuthor> {
    if !fetch && author.id.is_some() && author.login.is_some() {
        return Ok(author);
    }
    if !fetch && author.id.is_some() {
        return Ok(author);
    }
    if let Some(login) = &author.login {
        if author.id.is_none() || fetch {
            let fetched = fetch_github_user_by_login(login, token)?;
            return Ok(merge_authors(&author, &fetched));
        }
    }
    Ok(author)
}
