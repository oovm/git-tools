//! CLI 追踪与诊断（`tracing` + `miette`）。

use std::fmt;

use miette::Diagnostic;

use crate::error::{Error, Result};

/// 从 `RUST_LOG` 初始化 `tracing-subscriber`（默认过滤级别：`warn`）。
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr).with_target(true))
        .with(filter)
        .init();
}

/// CLI 侧包装，使 `gix-error` 经 miette 渲染。
#[derive(Debug, Diagnostic)]
#[diagnostic(code(git_tools::error))]
struct CliReport {
    message: String,
}

impl fmt::Display for CliReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for CliReport {}

fn format_error(err: &Error) -> String {
    err.iter_errors().map(ToString::to_string).collect::<Vec<_>>().join("\n  caused by: ")
}

/// 用 miette 将 `err` 写入 stderr。
pub fn report_error(err: Error) {
    eprintln!("{:?}", miette::Report::new(CliReport { message: format_error(&err) }));
}

/// 运行 CLI 入口：初始化 tracing，成功 exit `0`，失败经 miette 报告后 exit `1`。
pub fn main<F>(run: F) -> !
where
    F: FnOnce() -> Result<()>,
{
    init_tracing();
    match run() {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            report_error(err);
            std::process::exit(1);
        }
    }
}
