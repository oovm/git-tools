//! commit 历史遍历与 message 对象层改写。

mod history;
mod lint;
mod map;
mod rewrite;

pub use crate::error::{Error, Result};
pub use history::{head_ref_name, open, resolve_ref_tip, resolve_rev};
pub use lint::{LintIssue, duplicate_subjects, lint_message};
pub use map::{MapEntry, export_map, parse_map, resolve_map};
pub use rewrite::{PlannedChange, collect_commits, dry_run_plan, full_message, move_ref, plan_rewrite, short};
