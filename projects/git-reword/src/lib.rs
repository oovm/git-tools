//! 在 git 对象层改写 commit message，无需 interactive rebase。
//!
//! 基于 [gix](https://github.com/GitoxideLabs/gitoxide) 实现：写入新 commit 对象、
//! 按需 relink 父链，最后更新分支 ref。

/// 错误类型与 `Result` 别名。
pub mod error;
/// commit message 规范校验。
pub mod lint;
/// hash 前缀映射文件的解析与解析。
pub mod map;
/// 仓库访问、rev 解析与对象读写。
pub mod repo;
/// 改写规划、dry-run 与映射导出。
pub mod rewrite;
