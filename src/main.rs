use anyhow::anyhow;
use clap::Parser;
use git2::{Oid, Repository, Signature};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Automatically create git tags for Cargo (Cargo.toml), JavaScript (package.json), and Python (pyproject.toml) packages.
#[derive(clap::Parser)]
struct AutoTagArgs {
    /// Print the tags to be created but do not create them.
    #[clap(long)]
    dry_run: bool,
    /// The commit SHA to create the tag for.
    /// 
    /// Uses HEAD by default.
    #[clap(long)]
    commit: Option<String>,
    #[clap(long)]
    git_user_email: String,
    #[clap(long)]
    git_user_name: String,
    /// Directories to search for packages.
    #[clap(default_value = ".")]
    paths: Vec<PathBuf>,
}

fn main() -> Result<(), anyhow::Error> {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let args = AutoTagArgs::parse();

    for arg in &args.paths {
        for entry in WalkDir::new(arg) {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    println!("cannot access file: {}", err);
                    continue;
                }
            };

            if entry
                .path()
                .file_name()
                .map(|f| f == "Cargo.toml")
                .unwrap_or(false)
            {
                if let Err(err) = process_cargo_toml(&args, entry.path(), &repo) {
                    println!("failed to process {:?}: {}", entry.path(), err);
                }
            } else if entry
                .path()
                .file_name()
                .map(|f| f == "package.json")
                .unwrap_or(false)
            {
                if let Err(err) = process_package_json(&args, entry.path(), &repo) {
                    println!("failed to process {:?}: {}", entry.path(), err);
                }
            } else if entry
                .path()
                .file_name()
                .map(|f| f == "pyproject.toml")
                .unwrap_or(false)
            {
                if let Err(err) = process_pyproject_toml(&args, entry.path(), &repo) {
                    println!("failed to process {:?}: {}", entry.path(), err);
                }
            }
        }
    }

    Ok(())
}

fn process_package_json(
    args: &AutoTagArgs,
    path: &Path,
    repo: &Repository,
) -> Result<(), anyhow::Error> {
    let json_str = std::fs::read_to_string(path)?;
    let package_json: serde_json::Value = serde_json::from_str(&json_str)?;

    if let Some(true) = package_json["autoTag"]["enabled"].as_bool() {
        let name = package_json["name"]
            .as_str()
            .ok_or_else(|| anyhow!("package name not found"))?
            .replace('@', "")
            .replace('/', "__");

        let version = package_json["version"]
            .as_str()
            .ok_or_else(|| anyhow!("package version not found"))?;

        let tag_name = format!("release-{name}-{version}");
        create_tag(args, &name, version, &tag_name, repo)?;
    }

    Ok(())
}

fn process_cargo_toml(
    args: &AutoTagArgs,
    path: &Path,
    repo: &Repository,
) -> Result<(), anyhow::Error> {
    let toml_str = std::fs::read_to_string(path)?;

    let cargo_toml: toml::Value = toml::from_str(&toml_str)?;

    let auto_tag = cargo_toml
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("auto-tag"))
        .and_then(|tag| tag.get("enabled"))
        .and_then(|auto_tag| auto_tag.as_bool());

    if let Some(true) = auto_tag {
        let name = cargo_toml
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(|name| name.as_str())
            .ok_or_else(|| anyhow!("package name not found"))?;

        let version = cargo_toml
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(|version| version.as_str())
            .ok_or_else(|| anyhow!("package version not found"))?;

        let tag_name = format!("release-{name}-{version}");
        create_tag(args, name, version, &tag_name, repo)?;
    }

    Ok(())
}

fn process_pyproject_toml(
    args: &AutoTagArgs,
    path: &Path,
    repo: &Repository,
) -> Result<(), anyhow::Error> {
    let toml_str = std::fs::read_to_string(path)?;

    let pyproject_toml: toml::Value = toml::from_str(&toml_str)?;

    let auto_tag = pyproject_toml
        .get("tool")
        .and_then(|package| package.get("auto-tag"))
        .and_then(|tag| tag.get("enabled"))
        .and_then(|auto_tag| auto_tag.as_bool());

    if let Some(true) = auto_tag {
        let name = pyproject_toml
            .get("tool")
            .and_then(|tool| tool.get("poetry"))
            .and_then(|poetry| poetry.get("name"))
            .and_then(|name| name.as_str())
            .ok_or_else(|| anyhow!("package name not found"))?;

        let version = pyproject_toml
            .get("tool")
            .and_then(|tool| tool.get("poetry"))
            .and_then(|poetry| poetry.get("version"))
            .and_then(|version| version.as_str())
            .ok_or_else(|| anyhow!("package version not found"))?;

        let tag_name = format!("release-{name}-{version}");
        create_tag(args, name, version, &tag_name, repo)?;
    }

    Ok(())
}

fn create_tag(
    args: &AutoTagArgs,
    name: &str,
    version: &str,
    tag_name: &str,
    repo: &Repository,
) -> Result<(), anyhow::Error> {
    if !repo.tag_names(Some(tag_name))?.is_empty() {
        println!(r#"tag "{}" already exists, skipping..."#, tag_name);
        return Ok(());
    }

    let tag_message = format!("automatic release tag of {} ({})", name, version);

    let git_user = &args.git_user_name;
    let git_email = &args.git_user_email;

    let commit = if let Some(sha) = &args.commit {
        repo.find_commit(Oid::from_str(sha)?)?
    } else {
        repo.head()?.peel_to_commit()?
    };

    let commit_sha = commit.id();

    if args.dry_run {
        println!(
            r#"would create tag "{tag_name}" for "{commit_sha}" with message "{tag_message}" as {git_user} ({git_email})"#
        );
        return Ok(());
    }

    repo.tag(
        tag_name,
        commit.as_object(),
        &Signature::now(git_user, git_email)?,
        &tag_message,
        false,
    )?;

    println!(r#"created tag "{}""#, tag_name);

    Ok(())
}
