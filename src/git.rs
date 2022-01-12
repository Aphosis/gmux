use super::{Pool, Result, Settings};
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Output};

#[derive(Debug)]
enum LogLevel {
    Info,
    Error,
}

#[derive(Debug)]
struct OutputLog {
    level: LogLevel,
    message: String,
}

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
        let messages: Vec<Option<String>> = self
            .pool
            .repositories
            .par_iter()
            .map(|repository| {
                let path = repository.full_path(&self.pool.root);
                let path = match path.to_str() {
                    None => return None,
                    Some(path) => path,
                };

                let mut repository_args = vec!["-C".into(), path.into()];
                repository_args.append(&mut args.clone());

                let exe = self.executable.clone();
                let header = format!("- {}\n", repository.path.join(&repository.name).display());

                let output_result = Command::new(exe).args(repository_args).output();

                if let Some(output_log) = GitCaller::log_from_output_results(
                    output_result,
                    filter.clone(),
                    exclude_filter.clone(),
                ) {
                    let colored_header = match output_log.level {
                        LogLevel::Error => header.red(),
                        LogLevel::Info => header.blue(),
                    };
                    let message = format!("{}\n{}\n", colored_header, output_log.message.trim());
                    Some(message)
                } else {
                    None
                }
            })
            .collect();

        for message in messages {
            if let Some(msg) = message {
                std::io::stdout()
                    .write_all(&msg.as_bytes())
                    .expect("Could not write command output to stdout.");
            }
        }

        Ok(())
    }
    fn log_from_output_results(
        output_results: std::io::Result<Output>,
        filter: Option<String>,
        exclude_filter: Option<String>,
    ) -> Option<OutputLog> {
        match output_results {
            Ok(output) => {
                if !output.stderr.is_empty() {
                    Some(OutputLog {
                        message: String::from_utf8(output.stderr)
                            .expect("Could not decode error message"),
                        level: LogLevel::Error,
                    })
                } else if !output.stdout.is_empty() {
                    if let Some(ref pattern) = exclude_filter {
                        let re = Regex::new(&pattern).ok()?;
                        let out = std::str::from_utf8(&output.stdout).ok()?;
                        if re.is_match(out) {
                            return None;
                        }
                    }

                    if let Some(ref pattern) = filter {
                        let re = Regex::new(&pattern).ok()?;
                        let out = std::str::from_utf8(&output.stdout).ok()?;
                        if !re.is_match(out) {
                            return None;
                        }
                    }

                    Some(OutputLog {
                        message: String::from_utf8(output.stdout)
                            .expect("Could not decode git message"),
                        level: LogLevel::Info,
                    })
                } else {
                    return None;
                }
            }
            Err(err) => Some(OutputLog {
                message: err.to_string(),
                level: LogLevel::Error,
            }),
        }
    }
}
