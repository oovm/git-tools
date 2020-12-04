# ⏱️ git-retime

Spread commit **author** and **committer** timestamps across a date range by rewriting commit objects and pointing a new branch at the result.

Implemented with **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no `libgit2`).

No interactive rebase. The current branch is left unchanged. By default the result is written to `time-travel`.

## 🚀 Build

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-retime
```

## 📖 Usage

```bash
git-retime --repo /path/to/repo 2a990148 2019-01-01
git-retime 2a990148 2019-01-01 --end-date 2019-06-01 --branch dev-time-travel
```

Arguments:

- `commit` — range start. Commits in `(commit..tip]` are retimed. The start commit itself is unchanged.
- `start_date` — first day of the random window (`YYYY-MM-DD`).
- `--end-date` — last day window bound. Defaults to `start_date + number of commits in range`.
- `--branch` — branch to create or force-update. Defaults to `time-travel`.
- `--tip` — range end revision. Defaults to `HEAD`.

## 🌳 Object reuse

Trees, blobs, and commit messages are reused. Only author/committer timestamps change (plus parent relinking when ancestors are rewritten).

## 🧪 Tests

```bash
cargo test -p git-tools
```
