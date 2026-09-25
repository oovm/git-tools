# 📋 git-change-logs

Draft **reference** release changelogs from git tags and resolve GitHub user ids from commit emails.

Replaces per-repo `scripts/change-logs.mjs`. Output is a commit index for hand-editing
`documentation/maintenance/releases/vX.Y.Z.md` — not the published GitHub Release body.

## 🚀 Build

```bash
cargo install --git https://github.com/oovm/git-tools.git --bin git-change-logs
```

## 📖 Render reference changelog

Run from the repository working tree (or any subdirectory):

```bash
git-change-logs --version 0.0.3
git-change-logs --from v0.0.2 --to v0.0.3
git-change-logs --version 0.0.3 --write
git-change-logs --tags
```

`--write` creates `documentation/maintenance/releases/vX.Y.Z.reference.md` (typically gitignored).

Each commit becomes one bullet:

```text
- <subject without gitmoji> (@user)
```

Commits are grouped by gitmoji into Features / Bug Fixes / Breaking / Other. A contributor avatar wall is appended when GitHub logins are known.

### Options

| Flag | Purpose |
|------|---------|
| `--version=X.Y.Z` | Range: previous `v*` tag .. `vX.Y.Z` |
| `--from=REF` | Explicit range start (exclusive) |
| `--to=REF` | Explicit range end |
| `--write` | Write `vX.Y.Z.reference.md` |
| `--tags` | List `v*` tags with short hashes |
| `--repo owner/name` | contrib.rocks repo (default: parse `origin`) |
| `--author-map PATH` | Email → GitHub map (default: `documentation/maintenance/author-github.json`) |
| `--releases-dir PATH` | Output directory for `--write` |

## 🔍 Lookup GitHub user by email

Resolve numeric id and login for `author-github.json`:

```bash
git-change-logs lookup --email aster@vers.site
git-change-logs lookup --email aster@vers.site --map documentation/maintenance/author-github.json
git-change-logs lookup --login oovm
git-change-logs lookup --email someone@example.com --fetch --github-token "$GITHUB_TOKEN"
```

Stdout is JSON:

```json
{
  "id": 17541209,
  "login": "oovm"
}
```

Resolution order for `--email`:

1. GitHub noreply address (`12345+login@users.noreply.github.com`)
2. Local `author-github.json` entry
3. GitHub Search API (`in:email`) when `GITHUB_TOKEN` is set
4. `--fetch` fills missing numeric id via `GET /users/{login}`

Map file shape:

```json
{
  "email@example": {
    "id": 12345,
    "login": "handle"
  }
}
```

Prefer numeric `id` for stable avatar URLs.

## 🧪 Tests

```bash
cargo test changelog
```
