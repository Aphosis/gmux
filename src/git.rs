use super::{Pool, Result, Settings};
use std::path::PathBuf;
use std::process::Command;

pub struct GitCaller {
    pub executable: PathBuf,
    pub pool: Pool,
}

impl GitCaller {
    pub fn new(settings: &Settings) -> Result<Self> {
        let executable = settings.executable.clone();
        let pool = Pool::from_current(&settings)?;
        Ok(GitCaller { executable, pool })
    }
    pub fn call(&self, args: Vec<String>) -> Result<()> {
        for repository in &self.pool.repositories {
            let path = repository.full_path(&self.pool.root);
            let path = match path.to_str() {
                None => continue,
                Some(path) => path,
            };

            let mut repository_args = vec!["-C".into(), path.into()];
            repository_args.append(&mut args.clone());

            let status = Command::new(&self.executable)
                .args(repository_args)
                .status()?;

            if !status.success() {
                return Err("Process failed.".into());
            }
        }
        Ok(())
    }
}
