//! commit 历史遍历与 message 对象层改写。

mod history;
mod identity;
mod map;
mod retime;
mod rewrite;

pub use crate::error::{Error, Result};
pub use history::{commit_parents, head_ref_name, open, open_here, read_commit, resolve_ref_tip, resolve_rev, write_commit};
pub use identity::{copy_signature, retime_signature, same_contributor};
pub use map::{MapEntry, export_map, parse_map, resolve_map};
pub use retime::{
    RetimeOptions, RetimeRootOptions, RetimeSummary, commit_author_datetime, parse_date, parse_datetime, plan_retime,
    random_timestamps, run_retime, run_retime_root,
};
pub use rewrite::{PlannedChange, collect_commits, dry_run_plan, full_message, move_ref, plan_rewrite, short};
