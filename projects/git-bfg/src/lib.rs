//! 扫描 git 对象库中的 blob，并按大小列出排名。
//!
//! 基于 [gix](https://github.com/GitoxideLabs/gitoxide) 实现，不依赖 libgit2。

#![doc = include_str!("../Readme.md")]

mod cleaner;
mod errors;

pub use cleaner::{BlobFormat, BlobItem, Cleaner, find_git_root};
pub use errors::{CleanerError, Result};
