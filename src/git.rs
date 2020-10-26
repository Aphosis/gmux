use super::{Pool, Result, Settings};
use colored::*;
use regex::Regex;
use std::io::prelude::*;
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
    pub fn call(
        &self,
        args: Vec<String>,
        filter: Option<String>,
        exclude_filter: Option<String>,
    ) -> Result<()> {
        for repository in &self.pool.repositories {
            let path = repository.full_path(&self.pool.root);
            let path = match path.to_str() {
                None => continue,
                Some(path) => path,
            };

            let mut repository_args = vec!["-C".into(), path.into()];
            repository_args.append(&mut args.clone());

            let output = Command::new(&self.executable)
                .args(repository_args)
                .output()?;

            let header = format!("- {}\n", repository.path.join(&repository.name).display());

            if !output.stderr.is_empty() {
                let header = header.red().bytes().collect::<Vec<u8>>();
                std::io::stdout().write_all(&header)?;
                std::io::stdout().write_all(&output.stderr)?;
            } else if !output.stdout.is_empty() {
                if let Some(ref pattern) = exclude_filter {
                    let re = Regex::new(&pattern)?;
                    let out = std::str::from_utf8(&output.stdout)?;
                    if re.is_match(out) {
                        continue;
                    }
                }

                if let Some(ref pattern) = filter {
                    let re = Regex::new(&pattern)?;
                    let out = std::str::from_utf8(&output.stdout)?;
                    if !re.is_match(out) {
                        continue;
                    }
                }

                let header = header.blue().bytes().collect::<Vec<u8>>();
                std::io::stdout().write_all(&header)?;
                std::io::stdout().write_all(&output.stdout)?;
            } else {
                continue;
            };
        }
        Ok(())
    }
}
