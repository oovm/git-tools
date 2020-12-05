# ⏱️ git-retime

Spread commit **author** and **committer** timestamps across a date range by rewriting commit objects and pointing a new branch at the result.

Implemented with **[gix](https://github.com/GitoxideLabs/gitoxide)** (pure Rust, no `libgit2`).

No interactive rebase. The current branch is left unchanged. By default the result is written to `time-travel`.

Unlike `git commit --amend --date=…`, **name and email are never replaced** with your local `user.name` / `user.email`. Only timestamps change.

## 🚀 Build

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-retime
```

## 📖 Usage

Range retime — commits in `(commit..tip]`:

```bash
git-retime --repo /path/to/repo 2a990148 2019-01-01
git-retime 2a990148 2019-01-01 --end-date 2019-06-01 --branch dev-time-travel
git-retime 2a990148 2019-03-22T09:00:00 --end-date 2019-06-01T18:00:00
```

Root retime — all commits from repository root through `tip` (inclusive), like `git rebase -i --root`:

```bash
git-retime root 2019-01-01
git-retime root 2019-03-22T09:00:00 --message "🎂 Project initialized!" --branch time-travel
```

### Datetime format

- `YYYY-MM-DD` — start/end of day at `00:00:00`
- `YYYY-MM-DDTHH:MM:SS` or `YYYY-MM-DD HH:MM:SS` — exact wall-clock bounds for the random window

Timestamps are chosen as unique Unix seconds in `[start, end)` and written to **both** author and committer.

### Arguments

| Mode | Positional | Meaning |
| --- | --- | --- |
| range | `commit` | Range start. `(commit..tip]` is retimed. The start commit is unchanged. |
| range / root | `start_date` | Random window start |
| root | — | Use subcommand `root` instead of `commit` |
| both | `--end-date` | Window end. Defaults to `start + number of commits in range` days |
| both | `--branch` | Branch to create or force-update. Default `time-travel` |
| both | `--tip` | Range end revision. Default `HEAD` |
| root | `--message` | Optional new message for the root commit |

## 🌳 Object reuse

Trees, blobs, and commit messages are reused (except root `--message`). Only author/committer timestamps change (plus parent relinking when ancestors are rewritten).

## 🧪 Tests

```bash
cargo test -p git-tools
```
