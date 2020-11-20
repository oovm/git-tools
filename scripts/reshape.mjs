#!/usr/bin/env node
/**
 * Reshape git-tools src/ by capability, not by CLI tool name.
 *
 *   repo.rs              shared discovery and OID helpers
 *   object/{blob,inventory,error}.rs   ODB blob inventory
 *   commit/{history,rewrite,lint,map,error}.rs   commit message rewrite
 *
 * Usage:
 *   node scripts/reshape.mjs
 *   node scripts/reshape.mjs --dry-run
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const dryRun = process.argv.includes("--dry-run");

/** @param {string} rel */
function abs(rel) {
    return path.join(root, rel);
}

/** @param {string} rel */
function exists(rel) {
    return fs.existsSync(abs(rel));
}

/** @param {string} rel */
function ensureDir(rel) {
    const dir = abs(rel);
    if (!dryRun && !fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

/**
 * @param {string} rel
 * @param {string} content
 */
function writeFile(rel, content) {
    if (dryRun) {
        console.log(`write ${rel}`);
        return;
    }
    ensureDir(path.dirname(rel));
    fs.writeFileSync(abs(rel), content.endsWith("\n") ? content : `${content}\n`, "utf8");
}

/**
 * @param {string} rel
 * @returns {string}
 */
function readText(rel) {
    return fs.readFileSync(abs(rel), "utf8");
}

/**
 * @param {string[]} candidates
 * @returns {string}
 */
function readFirst(candidates) {
    const rel = candidates.find((candidate) => exists(candidate));
    if (!rel) {
        throw new Error(`missing source: ${candidates.join(" | ")}`);
    }
    return readText(rel);
}

/** @param {string} rel */
function removePath(rel) {
    const target = abs(rel);
    if (!fs.existsSync(target)) {
        return;
    }
    if (dryRun) {
        console.log(`remove ${rel}`);
        return;
    }
    fs.rmSync(target, { recursive: true, force: true });
}

function transformRewordModule(text) {
    return text
        .replace(/\buse crate::error/g, "use super::error")
        .replace(/\buse crate::\{/g, "use super::{")
        .replace(/\buse super::repo::/g, "use super::history::")
        .replace(/\bpub use super::repo::/g, "pub use super::history::");
}

function transformInventory(text) {
    let out = text.replace(/^mod blob_item;\s*\n/m, "");
    if (!out.startsWith("//!")) {
        out = `//! 遍历对象库、统计 blob 并输出大小排名。\n\n${out}`;
    }
    return out
        .replace(/\buse super::Result;/g, "use super::error::Result;")
        .replace(/\bsuper::CleanerError\b/g, "super::error::InventoryError")
        .replace(/\bpub fn find_git_root\b[\s\S]*?^}/m, "")
        .replace(/\buse std::\{\s*collections::HashSet,\s*path::\{Path, PathBuf\},\s*\};/, "use std::{collections::HashSet, path::Path};")
        .replace(/\buse super::blob_item::/g, "use super::blob::");
}

function writeRootCargoToml() {
    writeFile(
        "Cargo.toml",
        `[package]
name = "git-tools"
version = "0.4.0"
authors = ["Aster <192607617@qq.com>"]
description = "Git utilities built on gix: blob scanner and commit message reword"
readme = "Readme.md"
license = "MPL-2.0"
edition = "2021"

[lib]
name = "git_tools"
path = "src/lib.rs"

[[bin]]
name = "bfg"
path = "bin/bfg.rs"

[[bin]]
name = "git-reword"
path = "bin/git-reword.rs"

[dependencies]
byte-unit = "5.1"
clap = { version = "4.5", features = ["derive"] }
gix = { version = "0.68", default-features = false, features = ["revision", "max-performance-safe"] }
regex = "1.11"
thiserror = "2.0"

[lints.rust]
missing_docs = "deny"

[lints.rustdoc]
missing_crate_level_docs = "deny"

[profile.release]
lto = true
panic = "abort"
`,
    );
}

function writeRootLib() {
    writeFile(
        "src/lib.rs",
        `//! Git 工具库：对象库 blob 统计与 commit message 对象层改写。
//!
//! 模块按能力划分：[\`repo\`](repo) 仓库访问、[\`object\`](object) 对象库、[\`commit\`](commit) 提交历史。

/// 仓库发现与通用 OID 工具。
pub mod repo;
/// 对象库遍历与 blob 统计。
pub mod object;
/// commit 历史遍历与 message 改写。
pub mod commit;
`,
    );
}

function writeRepo() {
    writeFile(
        "src/repo.rs",
        `//! 仓库发现与通用 OID 工具。

use std::path::PathBuf;

use gix::ObjectId;

use crate::object::{InventoryError, Result};

/// 从 \`start\` 向上查找包含 \`.git\` 的工作区根目录。
pub fn find_git_root(start: PathBuf) -> Result<PathBuf> {
    let mut path = start;
    loop {
        if path.join(".git").exists() {
            return Ok(path);
        }
        if !path.pop() {
            return Err(InventoryError::msg("no \`.git\` directory found in ancestors"));
        }
    }
}

/// 将 OID 格式化为 8 位十六进制前缀。
pub fn short(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
`,
    );
}

function writeObjectMod() {
    writeFile(
        "src/object/mod.rs",
        `//! 对象库遍历与 blob 统计。

mod blob;
mod error;
mod inventory;

pub use blob::{BlobFormat, BlobItem};
pub use error::{InventoryError, Result};
pub use inventory::Cleaner;
`,
    );
}

function writeObjectError(source) {
    writeFile(
        "src/object/error.rs",
        source
            .replace(/CleanerError/g, "InventoryError")
            .replace(/`git-bfg`/g, "`object`")
            .replace(/`git-bfg` 结果别名。/g, "`object` 模块结果别名。"),
    );
}

function writeCommitMod() {
    writeFile(
        "src/commit/mod.rs",
        `//! commit 历史遍历与 message 对象层改写。

mod error;
mod history;
mod lint;
mod map;
mod rewrite;

pub use error::{Result, RewordError};
pub use history::{head_ref_name, open, resolve_ref_tip, resolve_rev};
pub use lint::{duplicate_subjects, lint_message, LintIssue};
pub use map::{parse_map_file, resolve_map};
pub use rewrite::{
    collect_commits, dry_run_plan, export_map, full_message, move_ref, plan_rewrite, short, PlannedChange,
};
`,
    );
}

function writeCommitHistory(source) {
    let out = source.replace(/^\/\/!.*\n/m, "//! commit 对象读写、历史范围遍历与引用更新。\n\n");
    out = out.replace(/\buse super::error::/g, "use crate::commit::error::");
    out = out.replace(/\bRewordError::/g, "crate::commit::error::RewordError::");
    out = out.replace(/\bResult</g, "crate::commit::error::Result<");
    out = out.replace(/\bpub fn short\b[\s\S]*?^}\n\n/m, "");
    out = out.replace(/(?<!crate::repo::)\bshort\(/g, "crate::repo::short(");
    writeFile("src/commit/history.rs", out);
}

function writeCommitRewrite(source) {
    let out = transformRewordModule(source);
    out = out.replace(/\bsuper::repo::/g, "super::history::");
    out = out.replace(/pub use super::history::short;/, "pub use crate::repo::short;");
    writeFile("src/commit/rewrite.rs", out);
}

function writeBfgBin(source) {
    writeFile(
        "bin/bfg.rs",
        source
            .replace(/\bgit_tools::bfg::/g, "git_tools::object::")
            .replace(
                /use git_tools::object::\{Cleaner, Result, find_git_root\};/,
                "use git_tools::object::{Cleaner, Result};\nuse git_tools::repo::find_git_root;",
            ),
    );
}

function writeRewordBin(source) {
    let out = source
        .replace(/\bgit_tools::reword::/g, "git_tools::commit::")
        .replace(
            /use git_tools::commit::\{[\s\S]*?\};/,
            `use git_tools::commit::{
    collect_commits, dry_run_plan, duplicate_subjects, export_map, full_message, head_ref_name, lint_message,
    move_ref, open, parse_map_file, plan_rewrite, resolve_map, resolve_ref_tip, resolve_rev, short, Result, RewordError,
};`,
        );
    writeFile("bin/git-reword.rs", out);
}

function writeReadme() {
    writeFile(
        "Readme.md",
        `# 🛠️ git-tools

Rust git utilities built on **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no libgit2).

## 🧰 Tools

| Binary | Library module | Purpose |
| --- | --- | --- |
| \`bfg\` | \`object\` | Scan the object database and list the largest blobs |
| \`git-reword\` | \`commit\` | Rewrite commit messages at the object layer without interactive rebase |

## 🚀 Quick start

Requires the pinned toolchain in [\`rust-toolchain.toml\`](rust-toolchain.toml) (nightly + \`rustfmt\` + \`clippy\`).

\`\`\`bash
cargo build --release
cargo test
\`\`\`

Install binaries from this crate:

\`\`\`bash
cargo install --path . --bin bfg
cargo install --path . --bin git-reword
\`\`\`

## 📦 Layout

\`\`\`text
git-tools/
  src/
    repo.rs       # repository discovery and OID helpers
    object/       # ODB blob inventory
    commit/       # commit history and message rewrite
  bin/
    bfg.rs
    git-reword.rs
  documentation/
    bfg.md
    reword.md
\`\`\`

## 🧪 CI

GitHub Actions runs \`cargo fmt --check\`, \`cargo build --release\`, and \`cargo test --release\` on Ubuntu, macOS, and Windows.

## 📄 License

MPL-2.0
`,
    );
}

function removeLegacyLayout() {
    removePath("src/bfg");
    removePath("src/reword");
    removePath("projects/git-bfg");
    removePath("projects/git-reword");
}

function main() {
    console.log(dryRun ? "dry-run capability reshape" : "reshape git-tools by capability");

    ensureDir("src/object");
    ensureDir("src/commit");
    ensureDir("bin");
    ensureDir("tests");
    ensureDir("documentation");

    const blobSource = readFirst([
        "src/object/blob.rs",
        "src/bfg/blob_item.rs",
        "projects/git-bfg/src/cleaner/blob_item.rs",
    ]);
    const inventorySource = readFirst([
        "src/object/inventory.rs",
        "src/bfg/cleaner.rs",
        "src/bfg/cleaner/mod.rs",
        "projects/git-bfg/src/cleaner/mod.rs",
    ]);
    const objectErrorSource = readFirst([
        "src/object/error.rs",
        "src/bfg/error.rs",
        "src/bfg/errors.rs",
        "projects/git-bfg/src/errors.rs",
    ]);
    const commitErrorSource = readFirst([
        "src/commit/error.rs",
        "src/reword/error.rs",
        "projects/git-reword/src/error.rs",
    ]);
    const lintSource = readFirst(["src/commit/lint.rs", "src/reword/lint.rs", "projects/git-reword/src/lint.rs"]);
    const mapSource = readFirst(["src/commit/map.rs", "src/reword/map.rs", "projects/git-reword/src/map.rs"]);
    const historySource = readFirst([
        "src/commit/history.rs",
        "src/reword/repo.rs",
        "projects/git-reword/src/repo.rs",
    ]);
    const rewriteSource = readFirst([
        "src/commit/rewrite.rs",
        "src/reword/rewrite.rs",
        "projects/git-reword/src/rewrite.rs",
    ]);
    const bfgBinSource = readFirst(["bin/bfg.rs", "projects/git-bfg/src/main.rs"]);
    const rewordBinSource = readFirst(["bin/git-reword.rs", "projects/git-reword/src/main.rs"]);

    writeRootCargoToml();
    writeRootLib();
    writeRepo();
    writeObjectMod();
    writeFile("src/object/blob.rs", blobSource);
    writeObjectError(objectErrorSource);
    writeFile("src/object/inventory.rs", transformInventory(inventorySource));
    writeCommitMod();
    writeFile("src/commit/error.rs", commitErrorSource);
    writeFile("src/commit/lint.rs", lintSource);
    writeFile("src/commit/map.rs", transformRewordModule(mapSource));
    writeCommitHistory(historySource);
    writeCommitRewrite(rewriteSource);
    writeBfgBin(bfgBinSource);
    writeRewordBin(rewordBinSource);

    if (exists("tests/bfg_smoke.rs")) {
        writeFile("tests/bfg_smoke.rs", readText("tests/bfg_smoke.rs"));
    } else if (exists("projects/git-bfg/tests/main.rs")) {
        writeFile("tests/bfg_smoke.rs", readText("projects/git-bfg/tests/main.rs"));
    }

    if (exists("documentation/bfg.md")) {
        writeFile("documentation/bfg.md", readText("documentation/bfg.md"));
    } else if (exists("projects/git-bfg/Readme.md")) {
        writeFile("documentation/bfg.md", readText("projects/git-bfg/Readme.md"));
    }

    if (exists("documentation/reword.md")) {
        writeFile("documentation/reword.md", readText("documentation/reword.md"));
    } else if (exists("projects/git-reword/Readme.md")) {
        writeFile("documentation/reword.md", readText("projects/git-reword/Readme.md"));
    }

    writeReadme();
    removeLegacyLayout();

    const projectsDir = abs("projects");
    if (!dryRun && fs.existsSync(projectsDir) && fs.readdirSync(projectsDir).length === 0) {
        fs.rmdirSync(projectsDir);
    }

    console.log(dryRun ? "dry-run complete" : "reshape complete");
}

main();
