use rayon::prelude::*;
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

pub fn scan_for_repos(watched_folders: &[String]) -> Vec<PathBuf> {
    watched_folders.par_iter().flat_map(|folder| {
        let path = Path::new(folder);
        if !path.exists() {
            return Vec::new();
        }

        let mut repos = Vec::new();
        let mut it = WalkDir::new(path).into_iter();
        
        loop {
            let entry = match it.next() {
                None => break,
                Some(Ok(e)) => e,
                Some(Err(_)) => continue,
            };
            
            let name = entry.file_name().to_string_lossy();
            
            // Prune folders that are notoriously large
            if entry.file_type().is_dir() && (
                name == "node_modules" || name == "target" || name == ".next" || 
                name == "dist" || name == "build" || name == "vendor" || name == ".cache"
            ) {
                it.skip_current_dir();
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
