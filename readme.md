# 🛠️ git-tools

Rust git utilities built on **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no libgit2).

## 🧰 Tools

| Binary       | Library module | Purpose                                                                |
|--------------|----------------|------------------------------------------------------------------------|
| `bfg`        | `object`       | Scan the object database and list the largest blobs                    |
| `git-reword` | `commit`       | Rewrite commit messages at the object layer without interactive rebase |

## 🚀 Install

Install a binary from the Git repository (no local clone required):

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin bfg
cargo install --git https://github.com/oovm/git-tools.git --bin git-reword
```

Track the `dev` branch while the crate is pre-release:

```bash
cargo install --git https://github.com/oovm/git-tools.git --branch dev --bin bfg
cargo install --git https://github.com/oovm/git-tools.git --branch dev --bin git-reword
```

## 🛠️ Development

Requires the pinned toolchain in [`rust-toolchain.toml`](rust-toolchain.toml) (nightly + `rustfmt` + `clippy`).

```bash
git clone https://github.com/oovm/git-tools.git
cd git-tools
cargo build --release
cargo test
```

## 📦 Layout

```text
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
```

## 🧪 CI

GitHub Actions runs `cargo fmt --check`, `cargo build --release`, and `cargo test --release` on Ubuntu, macOS, and
Windows.

## 📄 License

MPL-2.0
