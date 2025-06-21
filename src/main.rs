use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ghq_root: Option<PathBuf>,
    
    #[arg(long)]
    json: bool,
    
    #[arg(long)]
    deny: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClaudeSettings {
    permissions: Option<Permissions>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Permissions {
    allow: Option<Vec<String>>,
    deny: Option<Vec<String>>,
}


#[derive(Debug)]
struct ProjectInfo {
    permissions: Vec<String>,
    deny_permissions: Vec<String>,
    error: Option<String>,
}

impl ProjectInfo {
    fn new(permissions: Vec<String>, deny_permissions: Vec<String>) -> Self {
        Self {
            permissions,
            deny_permissions,
            error: None,
        }
    }

    fn with_error(error: String) -> Self {
        Self {
            permissions: vec![],
            deny_permissions: vec![],
            error: Some(error),
        }
    }
}

fn get_ghq_root(override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(path);
    }

    let output = Command::new("ghq")
        .arg("root")
        .output()
        .context("Failed to execute 'ghq root' command")?;

    if !output.status.success() {
        anyhow::bail!("ghq command failed");
    }

    let root_str = String::from_utf8(output.stdout)
        .context("Failed to parse ghq root output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(root_str))
}

fn find_claude_settings(ghq_root: &Path) -> Vec<PathBuf> {
    WalkDir::new(ghq_root)
        .max_depth(4) // Limit depth to github.com/user/repo/.claude
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            entry.file_type().is_dir() && entry.file_name() == ".claude"
        })
        .filter_map(|claude_dir| {
            let settings_file = claude_dir.path().join("settings.local.json");
            if settings_file.exists() {
                Some(settings_file)
            } else {
                None
            }
        })
        .collect()
}

fn extract_permissions(settings_path: &Path, _ghq_root: &Path) -> ProjectInfo {
    let content = match std::fs::read_to_string(settings_path) {
        Ok(content) => content,
        Err(e) => {
            return ProjectInfo::with_error(format!("Failed to read file: {}", e));
        }
    };

    let settings: ClaudeSettings = match serde_json::from_str(&content) {
        Ok(settings) => settings,
        Err(e) => {
            return ProjectInfo::with_error(format!("Failed to parse JSON: {}", e));
        }
    };

    let allow_permissions = settings
        .permissions
        .as_ref()
        .and_then(|p| p.allow.as_ref())
        .cloned()
        .unwrap_or_default();
    
    let deny_permissions = settings
        .permissions
        .as_ref()
        .and_then(|p| p.deny.as_ref())
        .cloned()
        .unwrap_or_default();

    ProjectInfo::new(allow_permissions, deny_permissions)
}

fn display_results(projects: &[ProjectInfo], json_output: bool, show_deny: bool) {
    let all_permissions: HashSet<String> = projects
        .iter()
        .filter(|p| p.error.is_none())
        .flat_map(|p| {
            if show_deny {
                p.deny_permissions.iter()
            } else {
                p.permissions.iter()
            }
        })
        .cloned()
        .collect();

    let mut sorted_permissions: Vec<_> = all_permissions.into_iter().collect();
    sorted_permissions.sort();

    if json_output {
        let output = if show_deny {
            serde_json::json!({
                "permissions": {
                    "deny": sorted_permissions
                }
            })
        } else {
            serde_json::json!({
                "permissions": {
                    "allow": sorted_permissions
                }
            })
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        for permission in sorted_permissions {
            println!("{}", permission);
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let ghq_root = get_ghq_root(args.ghq_root).context("Failed to determine ghq root directory")?;

    let settings_files = find_claude_settings(&ghq_root);

    if settings_files.is_empty() {
        println!("No .claude/settings.local.json files found");
        return Ok(());
    }

    let projects: Vec<ProjectInfo> = settings_files
        .iter()
        .map(|path| extract_permissions(path, &ghq_root))
        .collect();

    display_results(&projects, args.json, args.deny);

    Ok(())
}
