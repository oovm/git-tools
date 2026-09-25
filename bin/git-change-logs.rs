//! `git-change-logs` 命令行入口：tag 区间参考稿与 GitHub 作者查询。

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use tracing::info;

use git_tools::{
    Result,
    changelog::{
        collect_commits, collect_contributors_from_commits, detect_github_repo, fetch_github_user_by_login, format_tag_list,
        group_commits, load_author_map, lookup_github_user_by_email, render_contributor_wall, render_reference, resolve_range,
    },
    diag,
    repo::find_git_root,
    validation,
};

const DEFAULT_AUTHOR_MAP: &str = "documentation/maintenance/author-github.json";
const DEFAULT_RELEASES_DIR: &str = "documentation/maintenance/releases";

/// 全局参数与子命令。
#[derive(Parser)]
#[command(
    name = "git-change-logs",
    about = "Draft release reference changelogs from git tags and resolve GitHub authors by email"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    render: RenderOptions,
}

/// 子命令。
#[derive(Subcommand)]
enum Command {
    /// Look up GitHub user id/login from an email or login handle
    Lookup(LookupOptions),
}

/// 参考稿渲染选项（默认行为，与 legacy `change-logs.mjs` 对齐）。
#[derive(Parser)]
struct RenderOptions {
    /// Semver release: previous v* tag .. vX.Y.Z
    #[arg(long)]
    version: Option<String>,
    /// Explicit range start (exclusive)
    #[arg(long)]
    from: Option<String>,
    /// Explicit range end (tip tag or commit)
    #[arg(long)]
    to: Option<String>,
    /// Write documentation/maintenance/releases/vX.Y.Z.reference.md
    #[arg(long)]
    write: bool,
    /// List version tags
    #[arg(long)]
    tags: bool,
    /// GitHub owner/repo for contrib.rocks (default: parse from origin)
    #[arg(long)]
    repo: Option<String>,
    /// Path to author-github.json (default: documentation/maintenance/author-github.json)
    #[arg(long)]
    author_map: Option<PathBuf>,
    /// Releases directory (default: documentation/maintenance/releases)
    #[arg(long)]
    releases_dir: Option<PathBuf>,
}

/// GitHub 作者查询选项。
#[derive(Parser)]
struct LookupOptions {
    /// Author email to resolve
    #[arg(long, conflicts_with = "login")]
    email: Option<String>,
    /// GitHub login to resolve numeric id
    #[arg(long, conflicts_with = "email")]
    login: Option<String>,
    /// Path to author-github.json
    #[arg(long)]
    map: Option<PathBuf>,
    /// GitHub token for search-by-email and higher rate limits
    #[arg(long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,
    /// Fetch missing numeric id from GitHub API when login is known
    #[arg(long)]
    fetch: bool,
}

fn default_author_map_path(repo_root: &Path) -> PathBuf {
    repo_root.join(DEFAULT_AUTHOR_MAP)
}

fn default_releases_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(DEFAULT_RELEASES_DIR)
}

fn resolve_github_repo(repo_root: &Path, override_repo: Option<&str>) -> Result<String> {
    if let Some(repo) = override_repo {
        return Ok(repo.to_string());
    }
    detect_github_repo(repo_root).ok_or_else(|| validation("could not detect GitHub repo from origin; pass --repo owner/name"))
}

fn run_lookup(repo_root: &Path, options: LookupOptions) -> Result<()> {
    let map_path = options.map.unwrap_or_else(|| default_author_map_path(repo_root));
    let map = load_author_map(&map_path);
    let token = options.github_token.as_deref();

    let author = if let Some(email) = options.email {
        lookup_github_user_by_email(&email, &map, token, options.fetch)?
            .ok_or_else(|| validation(format!("no GitHub user found for email `{email}`")))?
    }
    else if let Some(login) = options.login {
        fetch_github_user_by_login(&login, token)?
    }
    else {
        return Err(validation("lookup requires --email or --login"));
    };

    let mut json = serde_json::Map::new();
    if let Some(id) = author.id {
        json.insert("id".to_string(), serde_json::Value::Number(id.into()));
    }
    if let Some(login) = author.login {
        json.insert("login".to_string(), serde_json::Value::String(login));
    }
    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(json)).map_err(|err| validation(err.to_string()))?);
    Ok(())
}

fn run_render(repo_root: &Path, options: RenderOptions) -> Result<()> {
    if options.tags {
        println!("{}", format_tag_list(repo_root)?);
        return Ok(());
    }

    let github_repo = resolve_github_repo(repo_root, options.repo.as_deref())?;
    let author_map_path = options.author_map.unwrap_or_else(|| default_author_map_path(repo_root));
    let releases_dir = options.releases_dir.unwrap_or_else(|| default_releases_dir(repo_root));
    let author_map = load_author_map(&author_map_path);

    let (version, from_ref, to_ref) =
        resolve_range(repo_root, options.version.as_deref(), options.from.as_deref(), options.to.as_deref())?;

    info!(version = %version, from = ?from_ref, to = %to_ref, github_repo = %github_repo, "change-logs render");

    let commits = collect_commits(repo_root, from_ref.as_deref(), &to_ref)?;
    let groups = group_commits(&commits);
    let contributors = collect_contributors_from_commits(&commits, &author_map);
    let contributor_wall = render_contributor_wall(&contributors, &github_repo);
    let notes =
        render_reference(&version, from_ref.as_deref(), &to_ref, &groups, &contributor_wall, commits.len(), &author_map);

    let range_label = match from_ref.as_deref() {
        Some(from) => format!("{from}..{to_ref}"),
        None => to_ref.clone(),
    };
    eprintln!("change-logs: {range_label} — {} commit(s)", commits.len());

    if options.write {
        std::fs::create_dir_all(&releases_dir).map_err(|err| validation(err.to_string()))?;
        let out_path = releases_dir.join(format!("v{version}.reference.md"));
        std::fs::write(&out_path, &notes).map_err(|err| validation(err.to_string()))?;
        let relative = out_path.strip_prefix(repo_root).unwrap_or(&out_path);
        eprintln!("change-logs: wrote {}", relative.display());
    }

    print!("{notes}");
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let repo_root = find_git_root(std::env::current_dir().map_err(|err| validation(err.to_string()))?)?;

    match cli.command {
        Some(Command::Lookup(options)) => run_lookup(&repo_root, options),
        None => run_render(&repo_root, cli.render),
    }
}

fn main() {
    diag::main(run);
}
