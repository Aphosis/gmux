use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub enum Error {
    PoolAlreadyExists { label: String },
    PoolDoesNotExists { label: String },
    RepositoryDoesNotExists { path: PathBuf },
    FileDoesNotExists { path: PathBuf },
    InvalidSettingsFile,
    NoCurrentPoolSet,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::PoolAlreadyExists { label } => write!(f, "Pool '{}' already exists.", &label),
            Error::PoolDoesNotExists { label } => write!(f, "Pool '{}' does not exists.", &label),
            Error::RepositoryDoesNotExists { path } => write!(f, "Repository '{}' does not exists.", &path.display()),
            Error::FileDoesNotExists { path } => write!(f, "File '{}' does not exists.", &path.display()),
            Error::InvalidSettingsFile => {
                write!(f, "Settings file is invalid, do you have a home folder ?")
            }
            Error::NoCurrentPoolSet => {
                write!(f, "No pool is currently set, create one using `gmux pool new` or set an existing one with `gmux pool set`.")
            }
        }
    }
}

impl std::error::Error for Error {}
