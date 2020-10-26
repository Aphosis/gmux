use super::{File, Pool, Repository, Result, Settings};
use git2::{Cred, RemoteCallbacks};
use std::io::prelude::*;

pub type CheckoutResult = Result<Checkout>;

pub struct CheckoutManager;

impl CheckoutManager {
    pub fn checkout(settings: &mut Settings) -> Result<Checkout> {
        let pool = Pool::from_current(settings)?;

        let data = CheckoutManager::build_checkout_worker_data(&pool)?;

        // FIXME: multi thread this part.
        // https://github.com/rust-lang/git2-rs/issues/329
        for repository in &data.clone {
            CheckoutManager::clone_repository(&pool, repository)?;
        }

        for branch in &data.branches {
            let git = git2::Repository::open(branch.repository.full_path(&pool.root))?;

            let next_rev = git.revparse_single(&branch.next)?;
            let next_tree = next_rev.peel_to_tree()?;

            let mut index = git.index()?;
            index.read_tree(&next_tree)?;

            git.checkout_index(Some(&mut index), None)?;
        }

        for file in pool.files {
            let path = file.full_path(&pool.root);
            let mut writer = std::fs::File::create(&path)?;

            let content = match &file.content {
                Some(content) => content,
                None => continue,
            };

            if path.is_file() {
                match file.checksum {
                    Some(checksum) => {
                        if File::checksum(&path)? == checksum {
                            continue;
                        }
                    }
                    None => (),
                }
            }

            write!(writer, "{}", content)?;
        }

        Ok(data)
    }

    fn build_checkout_worker_data(pool: &Pool) -> CheckoutResult {
        let mut clone = Vec::new();
        let mut branches = Vec::new();
        for repository in &pool.repositories {
            if !repository.full_path(&pool.root).is_dir() {
                clone.push(repository.clone());
            } else {
                let existing_repository =
                    Repository::from_path(&pool.root, repository.full_path(&pool.root))?;
                if existing_repository.branch != repository.branch {
                    let current = existing_repository.branch.clone();
                    let next = repository.branch.clone();
                    branches.push(BranchCheckout {
                        repository: repository.clone(),
                        current,
                        next,
                    })
                }
            }
        }
        let data = Checkout { clone, branches };

        Ok(data)
    }

    // https://docs.rs/git2/0.13.12/git2/build/struct.RepoBuilder.html
    fn clone_repository(pool: &Pool, repository: &Repository) -> Result<()> {
        let into = repository.full_path(&pool.root);

        // TODO: Add HTTPS options ?

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.expect("Could not guess username from URL."))
        });

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        builder.clone(&repository.fetch.url, &into)?;
        Ok(())
    }
}

#[derive(Debug)]
/// A checkout command worker data.
pub struct Checkout {
    /// Missing repositories to clone with the checkout command.
    pub clone: Vec<Repository>,
    /// List of branches to checkout.
    pub branches: Vec<BranchCheckout>,
}

#[derive(Debug)]
/// A branch checkout worker data.
pub struct BranchCheckout {
    /// Repository on which the checkout has to be performed.
    pub repository: Repository,
    /// Previous branch name.
    pub current: String,
    /// Name of the branch to checkout.
    pub next: String,
}
