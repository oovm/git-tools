//! commit 历史遍历与 message 对象层改写。

mod history;
mod lint;
mod map;
mod rewrite;

pub use crate::error::{Error, Result};
pub use history::{head_ref_name, open, resolve_ref_tip, resolve_rev};
pub use lint::{LintIssue, duplicate_subjects, lint_message};
pub use map::{parse_map_file, resolve_map};
pub use rewrite::{PlannedChange, collect_commits, dry_run_plan, export_map, full_message, move_ref, plan_rewrite, short};
