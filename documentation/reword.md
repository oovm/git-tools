# ✏️ git-reword

Rewrite commit messages by **writing new commit objects** and updating a branch ref.

Implemented with **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no `libgit2`).

No interactive rebase and no `GIT_EDITOR` index drift. Only mapped commits (plus descendants that need parent relinking) are rewritten.

## 🚀 Build

```bash
cargo build -p git-reword --release
```

## 📖 Workflow

Export a hash-keyed map template:

```bash
git-reword export --repo /path/to/repo --base 34e1e665^ --ref dev
```

Lint existing messages or a pending map:

```bash
git-reword lint-log --repo /path/to/repo --base 34e1e665^ --ref dev
git-reword lint-map --repo /path/to/repo --base 34e1e665^ --map reword.pending.txt
```

Dry-run, then apply:

```bash
git-reword rewrite --repo /path/to/repo --base 34e1e665^ --ref dev --map reword.pending.txt --dry-run
git-reword rewrite --repo /path/to/repo --base 34e1e665^ --ref dev --map reword.pending.txt
```

## 🌳 Object reuse

Trees and blobs are reused. When an ancestor is rewritten, descendants get new commit objects with relinked parents even if their message is unchanged.

## 🧪 Tests

```bash
cargo test -p git-reword
```
