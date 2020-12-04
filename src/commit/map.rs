//! commit message 映射：JSON 解析与在 commit 范围内解析 OID。

use std::{collections::HashMap, fs, path::Path};

use gix::ObjectId;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ResultExt, message, validation, validation_with_input};

/// JSON 映射条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    /// commit hash 前缀或完整 OID。
    pub hash: String,
    /// 完整 commit message。
    pub message: String,
}

/// JSON 映射文件（`entries` 数组形式）。
#[derive(Debug, Deserialize, Serialize)]
struct MapDocument {
    #[serde(default = "default_version")]
    version: u32,
    entries: Vec<MapEntry>,
}

fn default_version() -> u32 {
    1
}

/// 解析 JSON 映射文件。
///
/// 支持两种形状：
///
/// - 文档：`{ "version": 1, "entries": [ { "hash": "...", "message": "..." } ] }`
/// - 扁平对象：`{ "abc12345": "message", ... }`
pub fn parse_map(path: &Path) -> Result<Vec<(String, String)>> {
    let text = fs::read_to_string(path).or_raise(|| message!("read reword map json"))?;
    if let Ok(document) = serde_json::from_str::<MapDocument>(&text) {
        return normalize_map_entries(document.entries);
    }
    let flat: HashMap<String, String> = serde_json::from_str(&text).or_raise(|| message!("parse reword map json"))?;
    let entries = flat.into_iter().map(|(hash, message)| MapEntry { hash, message }).collect();
    normalize_map_entries(entries)
}

/// 导出 JSON 映射模板。
pub fn export_map(repo: &gix::Repository, exclusive_base: ObjectId, tip: ObjectId, path: &Path) -> Result<()> {
    use super::rewrite::{collect_commits, full_message};

    let chain = collect_commits(repo, exclusive_base, tip)?;
    let entries = chain
        .into_iter()
        .map(|oid| {
            let message = full_message(repo, oid)?;
            Ok(MapEntry { hash: oid.to_string(), message })
        })
        .collect::<Result<Vec<_>>>()?;
    let document = MapDocument { version: 1, entries };
    let text = serde_json::to_string_pretty(&document).or_raise(|| message!("serialize reword map json"))?;
    fs::write(path, text).or_raise(|| message!("write reword map json"))?;
    Ok(())
}

/// 将 hash 前缀条目解析为范围内的唯一 `ObjectId` → message 映射。
///
/// 前缀必须在 `commits_in_range` 中唯一匹配，否则报错。
pub fn resolve_map(entries: Vec<(String, String)>, commits_in_range: &[ObjectId]) -> Result<HashMap<ObjectId, String>> {
    let mut resolved = HashMap::new();
    for (prefix, message) in entries {
        let matches: Vec<ObjectId> =
            commits_in_range.iter().filter(|oid| oid.to_string().starts_with(&prefix)).copied().collect();
        if matches.is_empty() {
            return Err(validation_with_input("hash prefix not found in range", prefix));
        }
        if matches.len() > 1 {
            return Err(validation_with_input("ambiguous hash prefix in range", prefix));
        }
        let oid = matches[0];
        if resolved.contains_key(&oid) {
            return Err(validation_with_input("duplicate map entry for commit hash", prefix));
        }
        resolved.insert(oid, message);
    }
    Ok(resolved)
}

fn normalize_map_entries(entries: Vec<MapEntry>) -> Result<Vec<(String, String)>> {
    if entries.is_empty() {
        return Err(validation("no commit entries in reword map json"));
    }
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let hash = entry.hash.trim();
        if !is_commit_hash(hash) {
            return Err(validation_with_input("expected commit hash", hash));
        }
        let message = entry.message.trim();
        if message.is_empty() {
            return Err(validation_with_input("missing message for commit hash", hash));
        }
        out.push((hash.to_ascii_lowercase(), message.to_string()));
    }
    Ok(out)
}

fn is_commit_hash(hash: &str) -> bool {
    (8..=40).contains(&hash.len()) && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
}
