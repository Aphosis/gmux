use super::{File, Pool, Repository, Result, Settings};
use walkdir::WalkDir;

pub struct DiscoveryManager;

const GIT_DIR: &'static str = ".git";

impl DiscoveryManager {
    pub fn discover(pool: &Pool) -> Result<Discovery> {
        let mut repositories = Vec::new();
        let mut files = Vec::new();

        let mut iterator = WalkDir::new(pool.root.clone()).into_iter();

        loop {
            let entry = match iterator.next() {
                None => break,
                Some(Err(err)) => return Err(err.into()),
                Some(Ok(entry)) => entry,
            };

            let is_excluded = pool
                .exclude_patterns
                .iter()
                .any(|pattern| pattern.matches_path(entry.path()));

            if is_excluded {
                if entry.file_type().is_dir() {
                    iterator.skip_current_dir();
                }
                continue;
            }

            // TODO: Should we do something about symlinks ?
            if entry.file_type().is_dir() {
                let git_dir = entry.path().join(GIT_DIR);
                if git_dir.is_dir() {
                    let repository = Repository::from_path(&pool.root, entry.path().to_owned())?;
                    repositories.push(repository);

                    iterator.skip_current_dir();
                }
            } else if entry.file_type().is_file() {
                let file = File::from_path(&pool.root, entry.path().to_owned())?;
                files.push(file);
            }
        }

        Ok(Discovery {
            repositories,
            files,
        })
    }

    pub fn discover_current(settings: &Settings) -> Result<Pool> {
        let mut pool = Pool::from_current(settings)?;
        let discovery = DiscoveryManager::discover(&pool)?;
        pool.repositories = discovery.repositories;
        pool.files = discovery.files;
        pool.save(settings)?;
        Ok(pool)
    }
}

/// A collection of repositories and files found by inspecting a root directory.
pub struct Discovery {
    /// List of repositories managed by this pool.
    pub repositories: Vec<Repository>,
    /// List of files managed by this pool.
    pub files: Vec<File>,
}
