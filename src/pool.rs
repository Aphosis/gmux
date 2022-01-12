use super::{Error, File, Repository, Result, Settings};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use serde_yaml::{from_reader, to_writer};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
/// A collection of repositories and files to manage.
pub struct Pool {
    /// Pool name.
    /// It is a user facing label.
    pub label: String,
    /// Root of the repositories tree.
    /// Every repository contained in this pool will be relative to
    /// this root path.
    pub root: PathBuf,
    /// Exclusion patterns.
    /// This is used when discovering the pool repositories
    /// from its root.
    pub excludes: Vec<String>,
    /// List of repositories managed by this pool.
    pub repositories: Vec<Repository>,
    /// List of files managed by this pool.
    pub files: Vec<File>,

    #[serde(skip_serializing, skip_deserializing)]
    pub exclude_patterns: Vec<Pattern>,
}

impl fmt::Display for Pool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Pool {
    pub fn list(settings: &Settings) -> Result<Vec<Pool>> {
        let iterator = settings.store_full_path().read_dir()?;

        // TODO: Make an iterator from this method.
        let mut pools = Vec::new();

        for entry in iterator {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let label = String::from(
                entry
                    .path()
                    .file_stem()
                    .expect("Could not extract label from pool file.")
                    .to_str()
                    .expect("Could not parse label"),
            );
            let pool = Pool::from_label(&settings, label)?;
            pools.push(pool);
        }

        Ok(pools)
    }
    pub fn create(settings: &mut Settings, label: String, root: Option<PathBuf>) -> Result<Self> {
        if Pool::path(settings, &label).is_file() {
            return Err(Error::PoolAlreadyExists { label }.into());
        }

        let root = match root {
            Some(path) => path,
            None => std::env::current_dir()?,
        };
        let excludes = Vec::new();
        let repositories = Vec::new();
        let files = Vec::new();
        let exclude_patterns = Vec::new();

        let pool = Pool {
            label,
            root,
            excludes,
            repositories,
            files,
            exclude_patterns,
        };

        pool.save(settings)?;
        pool.set_as_current(settings)?;

        Ok(pool)
    }

    pub fn from_label(settings: &Settings, label: String) -> Result<Self> {
        let pool_path = Pool::path(settings, &label);

        if !pool_path.is_file() {
            return Err(Error::PoolDoesNotExists { label }.into());
        }

        let reader = std::fs::File::open(pool_path)?;

        let mut pool: Pool = from_reader(reader)?;

        let exclude_patterns = pool
            .excludes
            .iter()
            .map(|pattern| Pattern::new(&pattern).expect("Invalid exclude patterns"))
            .collect();

        pool.exclude_patterns = exclude_patterns;

        return Ok(pool);
    }

    pub fn from_current(settings: &Settings) -> Result<Self> {
        if let Some(label) = &settings.current {
            return Pool::from_label(settings, String::from(label));
        }
        Err(Error::NoCurrentPoolSet.into())
    }

    pub fn set_as_current(&self, settings: &mut Settings) -> Result<()> {
        settings.current = Some(self.label.clone());
        settings.save()?;

        Ok(())
    }

    pub fn set_current(settings: &mut Settings, label: String) -> Result<()> {
        let _ = Pool::from_label(settings, label.clone())?;
        settings.current = Some(label);
        settings.save()?;

        Ok(())
    }

    pub fn set_label(&mut self, settings: &Settings, label: String) -> Result<()> {
        let existing = Pool::from_label(&settings, label.clone());
        if existing.is_ok() {
            return Err(Error::PoolAlreadyExists { label }.into());
        }
        self.label = label;
        self.save(settings)?;
        Ok(())
    }

    pub fn set_root(&mut self, settings: &Settings, root: PathBuf) -> Result<()> {
        self.root = root;
        self.save(settings)?;
        Ok(())
    }

    pub fn add_exclude(&mut self, settings: &Settings, pattern: String) -> Result<()> {
        self.excludes.push(pattern);
        self.save(settings)?;
        Ok(())
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        let writer = std::fs::File::create(Pool::path(settings, &self.label))?;

        to_writer(writer, &self)?;
        Ok(())
    }
    fn path(settings: &Settings, label: &String) -> PathBuf {
        settings.store_full_path().join(label).with_extension("yml")
    }
}
