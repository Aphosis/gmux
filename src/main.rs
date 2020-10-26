use log::{debug, error, info};
use rit::{
    Application, ApplicationCommands, CheckoutManager, DiscoveryManager, GitCaller, GitCommand,
    Pool, PoolCommands, Settings,
};
use std::io::prelude::*;
use structopt::StructOpt;

fn report_error(err: Box<dyn std::error::Error>) {
    debug!("{:#?}", err.source());
    error!("{}", err);
}

fn main() {
    let app = Application::from_args();
    let mut settings = Settings::load().map_err(report_error).unwrap();

    pretty_env_logger::init();

    match app.command {
        ApplicationCommands::Pool(subcommand) => match subcommand.command {
            None => match Pool::from_current(&settings) {
                Ok(pool) => match std::io::stdout().write_all(format!("{}", pool.label).as_bytes())
                {
                    Ok(()) => (),
                    Err(err) => report_error(err.into()),
                },
                Err(err) => report_error(err),
            },
            Some(pool_command) => match pool_command {
                PoolCommands::List => match Pool::list(&settings) {
                    Ok(pools) => match std::io::stdout().write_all(
                        format!(
                            "{}",
                            pools
                                .into_iter()
                                .map(|pool| pool.label)
                                .collect::<Vec<String>>()
                                .join("\n")
                        )
                        .as_bytes(),
                    ) {
                        Ok(()) => (),
                        Err(err) => report_error(err.into()),
                    },
                    Err(err) => report_error(err),
                },
                PoolCommands::New { label, root } => match Pool::create(&mut settings, label, root)
                {
                    Ok(pool) => info!("New pool {} created.", pool),
                    Err(err) => report_error(err),
                },
                PoolCommands::Set { label } => {
                    match Pool::set_current(&mut settings, label.clone()) {
                        Ok(_) => info!("Default pool set to {}.", label),
                        Err(err) => report_error(err),
                    }
                }
                PoolCommands::Move { root } => match Pool::from_current(&settings) {
                    Ok(mut pool) => match pool.set_root(&settings, root) {
                        Ok(_) => info!("Moved pool {} to {}.", &pool.label, &pool.root.display()),
                        Err(err) => report_error(err),
                    },
                    Err(err) => report_error(err),
                },
                PoolCommands::Rename { label } => match Pool::from_current(&settings) {
                    Ok(mut pool) => match pool.set_label(&settings, label) {
                        Ok(_) => info!("Renamed current pool to {}.", &pool.label),
                        Err(err) => report_error(err),
                    },
                    Err(err) => report_error(err),
                },
                PoolCommands::Exclude { pattern } => match Pool::from_current(&settings) {
                    Ok(mut pool) => match pool.add_exclude(&settings, pattern.clone()) {
                        Ok(_) => info!("Added exclusion rule {} to {}", pattern, &pool.label),
                        Err(err) => report_error(err),
                    },
                    Err(err) => report_error(err),
                },
                PoolCommands::Checkout => match CheckoutManager::checkout(&mut settings) {
                    Ok(checkout) => info!(
                        "Checked out {} branches and {} repositories.",
                        checkout.branches.len(),
                        checkout.clone.len()
                    ),
                    Err(err) => report_error(err),
                },
                PoolCommands::Discover => match DiscoveryManager::discover_current(&settings) {
                    Ok(pool) => info!("Discovered pool {} files and repositories.", pool),
                    Err(err) => report_error(err),
                },
            },
        },
        ApplicationCommands::Command {
            exclude_filter,
            filter,
            command,
        } => match command {
            GitCommand::Command(args) => match GitCaller::new(&settings) {
                Ok(git) => {
                    git.call(args, filter, exclude_filter).unwrap();
                    ()
                }
                Err(err) => {
                    report_error(err);
                }
            },
        },
    }
}
