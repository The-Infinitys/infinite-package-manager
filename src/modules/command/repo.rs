use crate::Error;
use crate::modules::compats::repo::apt::{self, RepoInfo};
use std::fs;
use std::path::PathBuf;

pub fn list_apt_repos() -> Result<Vec<RepoInfo>, Error> {
    let mut all_repos: Vec<RepoInfo> = Vec::new();

    let paths = [
        PathBuf::from("/etc/apt/sources.list"),
        // You might need to glob for files in sources.list.d/
        // For simplicity, we'll just check a common one for now.
        PathBuf::from("/etc/apt/sources.list.d/ubuntu.sources"),
    ];

    for path in paths.iter() {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match apt::parse_repo_info(&content) {
                    Ok(repos) => all_repos.extend(repos),
                    Err(e) => eprintln!("Error parsing {}: {}", path.display(), e),
                },
                Err(e) => eprintln!("Error reading {}: {}", path.display(), e),
            }
        }
    }

    Ok(all_repos)
}

pub fn run() -> Result<(), Error> {
    println!("Repo command executed.");

    match list_apt_repos() {
        Ok(repos) => {
            if repos.is_empty() {
                println!("No APT repositories found.");
            } else {
                println!("Found APT repositories:");
                for repo in repos {
                    println!("{:#?}", repo);
                }
            }
        }
        Err(e) => eprintln!("Failed to list APT repositories: {}", e),
    }

    Ok(())
}
