//! 仓库发现与通用 OID 工具。

use std::path::PathBuf;

use gix::ObjectId;

use crate::error::{Result, validation};

/// 从 `start` 向上查找包含 `.git` 的工作区根目录。
pub fn find_git_root(start: PathBuf) -> Result<PathBuf> {
    let mut path = start;
    loop {
        if path.join(".git").exists() {
            return Ok(path);
        }
        if !path.pop() {
            return Err(validation("no `.git` directory found in ancestors"));
        }
    }
}

/// 将 OID 格式化为 8 位十六进制前缀。
pub fn short(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
