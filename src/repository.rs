use super::{Error, Result};
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A repository data model.
pub struct Repository {
    /// Directory name.
    /// It is used when cloning with a different directory name.
    pub name: String,
    /// Parent directory.
    /// This path is relative to the current pool root.
    pub path: PathBuf,
    /// Current, local branch.
    pub branch: String,
    /// List of remotes.
    pub remotes: Vec<Remote>,
    /// Fetch remote.
    pub fetch: Remote,
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.join(&self.name).display())
    }
}

impl Repository {
    pub fn full_path(&self, root: &Path) -> PathBuf {
        root.join(self.path.join(self.name.clone()))
    }

    pub fn from_path(root: &PathBuf, path: PathBuf) -> Result<Repository> {
        if !path.is_dir() {
            return Err(Error::RepositoryDoesNotExists { path }.into());
        }

        let git = git2::Repository::open(path.clone())?;

        // FIXME: Better exception handling.
        let name = String::from(
            path.file_name()
                .expect("Could not get file name.")
                .to_str()
                .expect("Could not convert OS string."),
        );
        let path = path
            .parent()
            .expect("Could not get parent path.")
            .strip_prefix(root)?
            .to_owned();

        let branch = String::from(git.head()?.name().expect("Could not parse branch name."));

        let mut remotes = Vec::new();

        for name in git.remotes()?.iter() {
            if let Some(name) = name {
                let remote = Repository::find_remote(&git, name)?;
                remotes.push(remote);
            }
        }

        let branch_remote = git.branch_upstream_remote(&branch)?;
        let branch_remote = branch_remote
            .as_str()
            .expect("Could not parse fetch remote name.");
        let fetch = Repository::find_remote(&git, branch_remote)?;

        let repository = Repository {
            name,
            path,
            branch,
            remotes,
            fetch,
        };
        Ok(repository)
    }

    fn find_remote(git: &git2::Repository, name: &str) -> Result<Remote> {
        let remote = git.find_remote(name)?;
        let name = String::from(remote.name().expect("Could not find remote name."));
        let url = String::from(remote.url().expect("Could not find remote URL."));
        Ok(Remote { name, url })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A remote repository.
pub struct Remote {
    /// Local name of the remote.
    pub name: String,
    /// URL of the remote.
    pub url: String,
}

impl fmt::Display for Remote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Represent a file outside of a repository, but still managed by rit.
pub struct File {
    /// Path to the file.
    /// This path is relative to the current pool root.
    pub path: PathBuf,
    /// File content as raw text.
    pub content: Option<String>,
    /// Hash of the file content.
    pub checksum: Option<u32>,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl File {
    pub fn full_path(&self, root: &Path) -> PathBuf {
        root.join(self.path.clone())
    }
    pub fn from_path(root: &PathBuf, path: PathBuf) -> Result<File> {
        if !path.is_file() {
            return Err(Error::FileDoesNotExists { path }.into());
        }

        let metadata = std::fs::metadata(path.clone())?;
        let (content, checksum) = match metadata.len() {
            0 => {
                let mut reader = std::fs::File::open(path.clone())?;

                let mut content = String::new();
                reader.read_to_string(&mut content)?;

                let mut hasher = Hasher::new();
                hasher.update(content.as_bytes());
                let checksum = hasher.finalize();

                (Some(content), Some(checksum))
            }
            _ => (None, None),
        };

        let path = path.strip_prefix(root)?.to_owned();

        Ok(File {
            path,
            content,
            checksum,
        })
    }

    pub fn checksum(path: &PathBuf) -> Result<u32> {
        let mut reader = std::fs::File::open(path.clone())?;

        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        Ok(hasher.finalize())
    }
}
