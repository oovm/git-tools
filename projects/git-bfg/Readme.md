# 🧹 git-bfg

Scan a git object database and list the largest **blob** objects.

Implemented with **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no `libgit2`).

## 🚀 Build

```bash
cargo build -p git-bfg --release
```

## 📖 Usage

From the repository root (walks up to find `.git`):

```bash
bfg
```

Explicit path and result limit:

```bash
bfg --repo /path/to/repo --top 50
```

## 📤 Example output

```text
Found 328 blob(s) and 282 tree(s) (total blob size 42.1 MB)
Top 100 largest blob(s):
  1 |  15.78 MB | 623199237e74b78e4c9b94c0d07d918d22eb6e19 | Binary
  2 |  14.94 MB | c1d834f148f74215221551317327a554aed3b7f0 | Binary
  3 |   9.61 MB | 65a99bc359f6be38b3c223b6d47c376937f0883b | Binary
```

Trees, commits, and tags are counted but only blobs are ranked. Text vs binary uses the usual NUL-byte heuristic.

## 🧪 Tests

```bash
cargo test -p git-bfg
```
