//! hash 前缀映射文件：解析与在 commit 范围内解析 OID。

use std::{collections::HashMap, fs, path::Path};

use gix::ObjectId;

use crate::error::{Result, ResultExt, message, validation, validation_with_input};

/// 映射块首行 hash 的正则（8–40 位十六进制）。
const HASH_LINE: &str = r"^[0-9a-fA-F]{8,40}$";

/// 解析 hash 映射文件：块之间用仅含 `---` 的一行分隔。
///
/// 每个块第一行非空、非 `#` 开头的内容为 commit hash 前缀，其余为完整 message。
pub fn parse_map_file(path: &Path) -> Result<Vec<(String, String)>> {
    let text = fs::read_to_string(path).or_raise(|| message!("read reword map file"))?;
    let mut blocks = Vec::new();
    for block in text.replace("\r\n", "\n").split("\n---\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let mut hash_line = "";
        let mut message_start = 0usize;
        let mut started = false;
        for (index, line) in block.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if !started {
                hash_line = trimmed;
                message_start = index + 1;
                started = true;
                break;
            }
        }
        if !started {
            continue;
        }
        let re = regex::Regex::new(HASH_LINE).expect("hash regex");
        if !re.is_match(hash_line) {
            return Err(validation_with_input("expected commit hash", hash_line));
        }
        let message = block.lines().skip(message_start).collect::<Vec<_>>().join("\n").trim().to_string();
        if message.is_empty() {
            return Err(validation_with_input("missing message for commit hash", hash_line));
        }
        blocks.push((hash_line.to_lowercase(), message));
    }
    if blocks.is_empty() {
        return Err(validation("no commit blocks in map file"));
    }
    Ok(blocks)
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
