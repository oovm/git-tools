//! Git 工具库：对象库 blob 统计与 commit message 对象层改写。
//!
//! 模块按能力划分：[`repo`](repo) 仓库访问、[`object`](object) 对象库、[`commit`](commit) 提交历史。

/// commit 历史遍历与 message 改写。
pub mod commit;
/// 对象库遍历与 blob 统计。
pub mod object;
/// 仓库发现与通用 OID 工具。
pub mod repo;
