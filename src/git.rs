use super::{Pool, Result, Settings};
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use std::fmt;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::mpsc::channel;

#[derive(Debug)]
struct CommandOutput {
    header: String,
    output: std::io::Result<Output>,
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
        let (sender, receiver) = channel();

        self.pool
            .repositories
            .par_iter()
            .for_each_with(sender, |s, repository| {

                let path = repository.full_path(&self.pool.root);
                let path = match path.to_str() {
                    None => return (),
                    Some(path) => path,
                };

                let mut repository_args = vec!["-C".into(), path.into()];
                repository_args.append(&mut args.clone());

                let exe = self.executable.clone();
                let header = format!("- {}\n", repository.path.join(&repository.name).display());

                let output = Command::new(exe).args(repository_args).output();

                s.send(CommandOutput { header, output })
                    .expect("Cannot send git command output to channel receiver.");

            });

        for command_output in receiver {
            match command_output.output {
                Ok(output) => {
                    let filtered_output = FilteredOutput::from(output, &filter, &exclude_filter);

                    if filtered_output.is_empty() {
                        continue;
                    }

                    let message = format!("{}\n{}", command_output.header.blue(), filtered_output);

                    std::io::stdout()
                        .write_all(&message.as_bytes())
                        .expect("Could not write command output to stdout.");
                }
                Err(_) => (),
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FilteredOutput {
    out: Option<String>,
    err: Option<String>,
}

impl FilteredOutput {
    fn from(output: Output, filter: &Option<String>, exclude: &Option<String>) -> Self {
        let out = match FilteredOutput::bytes_to_string(&output.stdout) {
            Some(message) => FilteredOutput::filter(message, filter, exclude),
            None => None,
        };
        let err = match FilteredOutput::bytes_to_string(&output.stderr) {
            Some(message) => FilteredOutput::filter(message, filter, exclude),
            None => None,
        };
        FilteredOutput { out, err }
    }

    fn is_empty(&self) -> bool {
        self.out.is_none() && self.err.is_none()
    }

    fn bytes_to_string(bytes: &Vec<u8>) -> Option<String> {
        if bytes.is_empty() {
            return None;
        }
        match std::str::from_utf8(bytes) {
            Ok(string) => Some(String::from(string)),
            Err(_) => None,
        }
    }

    fn filter(
        message: String,
        filter: &Option<String>,
        exclude: &Option<String>,
    ) -> Option<String> {
        match exclude {
            Some(pattern) => {
                if FilteredOutput::is_match(&message, &pattern) {
                    return None;
                }
            }
            None => (),
        }
        match filter {
            Some(pattern) => {
                if !FilteredOutput::is_match(&message, &pattern) {
                    return None;
                }
            }
            None => (),
        }
        Some(message)
    }

    fn is_match(message: &String, pattern: &String) -> bool {
        let re =
            Regex::new(pattern).expect("Could not convert match pattern to regular expression.");
        re.is_match(message)
    }
}

impl fmt::Display for FilteredOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "");
        }

        let mut message = String::from("");

        if let Some(out) = &self.out {
            message.push_str(&format!("{}\n", out.trim()));
        }

        if let Some(err) = &self.err {
            message.push_str(&format!("{}\n", err.red().trim()));
        }

        write!(f, "{}", message)
    }
}
