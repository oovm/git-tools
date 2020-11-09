use std::collections::HashMap;
use std::fs;
use std::path::Path;

use gix::ObjectId;

use crate::error::{Result, RewordError};

const HASH_LINE: &str = r"^[0-9a-fA-F]{8,40}$";

/// Parse hash-keyed message blocks separated by a line containing only `---`.
pub fn parse_map_file(path: &Path) -> Result<Vec<(String, String)>> {
    let text = fs::read_to_string(path)?;
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
            return Err(RewordError::msg(format!(
                "expected commit hash, got {:?}",
                hash_line
            )));
        }
        let message = block
            .lines()
            .skip(message_start)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        if message.is_empty() {
            return Err(RewordError::msg(format!("missing message for {}", hash_line)));
        }
        blocks.push((hash_line.to_lowercase(), message));
    }
    if blocks.is_empty() {
        return Err(RewordError::msg("no commit blocks in map file"));
    }
    Ok(blocks)
}

pub fn resolve_map(
    entries: Vec<(String, String)>,
    commits_in_range: &[ObjectId],
) -> Result<HashMap<ObjectId, String>> {
    let mut resolved = HashMap::new();
    for (prefix, message) in entries {
        let matches: Vec<ObjectId> = commits_in_range
            .iter()
            .filter(|oid| oid.to_string().starts_with(&prefix))
            .copied()
            .collect();
        if matches.is_empty() {
            return Err(RewordError::msg(format!(
                "hash prefix not found in range: {}",
                prefix
            )));
        }
        if matches.len() > 1 {
            return Err(RewordError::msg(format!(
                "ambiguous hash prefix in range: {}",
                prefix
            )));
        }
        let oid = matches[0];
        if resolved.contains_key(&oid) {
            return Err(RewordError::msg(format!("duplicate map entry for {}", prefix)));
        }
        resolved.insert(oid, message);
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_hash_blocks() {
        let dir = std::env::temp_dir().join("git-reword-test-map");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("map.txt");
        fs::write(
            &path,
            "abc12345\n✨ Subject line\n\nBody line.\n\n---\n\nabcdef01\n🐛 Fix `thing`\n",
        )
        .unwrap();

        let entries = parse_map_file(&path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "abc12345");
    }
}
