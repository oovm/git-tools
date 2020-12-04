#![warn(missing_docs)]

//! Git 工具库：对象库 blob 统计与 commit message 对象层改写。
//!
//! 模块按能力划分：[`repo`](repo) 仓库访问、[`object`](object) 对象库、[`commit`](commit) 提交历史与改写。

/// commit 历史遍历与 message 改写。
pub mod commit;
/// 统一错误类型（`gix-error`）。
pub mod error;
/// 对象库遍历与 blob 统计。
pub mod object;
/// 仓库发现与通用 OID 工具。
pub mod repo;

pub use error::{Error, Exn, Message, OptionExt, Result, ResultExt, ensure, message, validation, validation_with_input};
