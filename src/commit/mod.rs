//! commit 历史遍历与 message 对象层改写。

mod history;
mod map;
mod retime;
mod rewrite;

pub use crate::error::{Error, Result};
pub use history::{head_ref_name, open, open_here, resolve_ref_tip, resolve_rev};
pub use map::{MapEntry, export_map, parse_map, resolve_map};
pub use retime::{
    RetimeOptions, RetimeRootOptions, RetimeSummary, parse_date, parse_datetime, plan_retime, random_timestamps, run_retime,
    run_retime_root,
};
pub use rewrite::{PlannedChange, collect_commits, dry_run_plan, full_message, move_ref, plan_rewrite, short};
