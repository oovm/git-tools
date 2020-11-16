# 🛠️ git-tools

Rust workspace of git utilities built on **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no libgit2).

## 🧰 Tools

| Crate                                         | Binary       | Purpose                                                                |
|-----------------------------------------------|--------------|------------------------------------------------------------------------|
| [`git-bfg`](projects/git-bfg/Readme.md)       | `bfg`        | Scan the object database and list the largest blobs                    |
| [`git-reword`](projects/git-reword/Readme.md) | `git-reword` | Rewrite commit messages at the object layer without interactive rebase |

## 🚀 Quick start

Requires the pinned toolchain in [`rust-toolchain.toml`](rust-toolchain.toml) (nightly + `rustfmt` + `clippy`).

```bash
cargo build --release
cargo test
```

Install a single binary:

```bash
cargo install --path projects/git-bfg
cargo install --path projects/git-reword
```

## 📦 Workspace layout

```text
git-tools/
  rust-toolchain.toml   # nightly toolchain pin
  rustfmt.toml
  projects/
    git-bfg/            # blob size scanner
    git-reword/         # hash-keyed commit message rewrite
```

## 🧪 CI

GitHub Actions runs `cargo fmt --check`, `cargo build --release`, and `cargo test --release` on Ubuntu, macOS, and
Windows.

## 📄 License

MPL-2.0
