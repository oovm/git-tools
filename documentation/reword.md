# ✏️ git-reword

Rewrite commit messages by **writing new commit objects** and updating a branch ref.

Implemented with **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no `libgit2`).

No interactive rebase and no `GIT_EDITOR` index drift. Only mapped commits (plus descendants that need parent relinking)
are rewritten.

## 🚀 Build

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-reword
```

## 📖 Workflow

Export a JSON map template:

```bash
git-reword export --base 34e1e665^ --ref dev --path reword.pending.json
```

Dry-run, then apply:

```bash
git-reword rewrite --base 34e1e665^ --ref dev --path reword.pending.json --dry-run
git-reword rewrite --base 34e1e665^ --ref dev --path reword.pending.json
```

Run from the repository working tree (or any subdirectory). The tool discovers `.git` like `git` itself.

## 📄 Map format

Canonical export shape:

```json
{
  "version": 1,
  "entries": [
    {
      "hash": "abc12345deadbeef...",
      "message": "✨ Subject\n\nBody."
    }
  ]
}
```

A flat object `{ "abc12345": "message" }` is also accepted for hand-edited maps.

## 🌳 Object reuse

Trees and blobs are reused. When an ancestor is rewritten, descendants get new commit objects with relinked parents even
if their message is unchanged.

## 🧪 Tests

```bash
cargo test -p git-tools
```
