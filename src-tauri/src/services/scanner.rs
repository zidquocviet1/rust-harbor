use rayon::prelude::*;
use walkdir::WalkDir;
use std::path::{Path, PathBuf};
use crate::config::AppConfig;
use glob::Pattern;

pub fn scan_for_repos(config: &AppConfig) -> Vec<PathBuf> {
    let exclusion_patterns: Vec<Pattern> = config.exclusion_patterns
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    config.watched_folders.par_iter().flat_map(|folder| {
        let path = Path::new(folder);
        if !path.exists() {
            return Vec::new();
        }

        let mut repos = Vec::new();
        let mut it = WalkDir::new(path)
            .max_depth(config.max_depth as usize)
            .into_iter();
        
        loop {
            let entry = match it.next() {
                None => break,
                Some(Ok(e)) => e,
                Some(Err(_)) => continue,
            };
            
            let name = entry.file_name().to_string_lossy();
            let relative_path = entry.path().strip_prefix(path).unwrap_or(entry.path());
            
            // Check exclusion patterns
            // We check if any part of the path matches the exclusion patterns
            let is_excluded = exclusion_patterns.iter().any(|p| {
                p.matches_path(relative_path) || p.matches(&name)
            });

            if is_excluded {
                if entry.file_type().is_dir() {
                    it.skip_current_dir();
                }
                continue;
            }

            // Check if this directory is a git repo
            let dot_git = entry.path().join(".git");
            if dot_git.exists() {
                repos.push(entry.path().to_path_buf());
                // Once we find a root repo, don't look inside for submodules/nested repos
                it.skip_current_dir();
            }
        }
        repos
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use crate::config::AppConfig;

    #[test]
    fn test_scan_respects_max_depth() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        
        // Setup: root/depth1/depth2/repo/ (repo is at depth 3)
        let repo_path = root.join("depth1").join("depth2").join("repo");
        fs::create_dir_all(&repo_path.join(".git")).unwrap();

        let mut config = AppConfig::default();
        config.watched_folders = vec![root.to_string_lossy().to_string()];
        
        // max_depth 2 should NOT find it
        config.max_depth = 2;
        let repos = scan_for_repos(&config);
        assert!(repos.is_empty(), "Should not find repo at depth 3 when max_depth is 2");

        // max_depth 3 should find it
        config.max_depth = 3;
        let repos = scan_for_repos(&config);
        assert_eq!(repos.len(), 1, "Should find repo at depth 3 when max_depth is 3");
    }

    #[test]
    fn test_scan_respects_exclusion_patterns() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        
        // Setup: 
        // root/my_repo/.git
        // root/node_modules/some_repo/.git (should be excluded)
        let repo1 = root.join("my_repo");
        let repo2 = root.join("node_modules").join("some_repo");
        
        fs::create_dir_all(&repo1.join(".git")).unwrap();
        fs::create_dir_all(&repo2.join(".git")).unwrap();

        let mut config = AppConfig::default();
        config.watched_folders = vec![root.to_string_lossy().to_string()];
        config.exclusion_patterns = vec!["**/node_modules/**".to_string()];
        config.max_depth = 5;

        let repos = scan_for_repos(&config);
        
        assert_eq!(repos.len(), 1, "Should only find 1 repo, excluding node_modules");
        assert_eq!(repos[0].file_name().unwrap(), "my_repo");
    }
}
