use clap::IntoApp;
use clap_complete::{generate_to, shells};
use std::env;
use std::io::Error;
use std::path::Path;

include!("src/app.rs");

const APP_NAME: &'static str = "gmux";

fn main() -> Result<(), Error> {
    // HACK: It's very much a hack to expose completion files as artifacts.
    // `cargo` does not support installing completions yet, but package
    // managers could, so it's desirable to generate them.
    // When the `--out-dir` flag goes out of its experimental state,
    // this won't be needed anymore as packagers will be able to specify
    // the output directory and access the completion files directly.
    // Or we could use nightly right now and forget about all this.
    let manifest_dir = match env::var_os("CARGO_MANIFEST_DIR") {
        None => {
            println!("cargo:warning=Could not find manifest dir, aborting completions generation.");
            return Ok(());
        }
        Some(manifest_dir) => manifest_dir,
    };

    let profile = match env::var_os("PROFILE") {
        None => {
            println!("cargo:warning=Could not find profile, aborting completions generation.");
            return Ok(());
        }
        Some(profile) => profile,
    };

    let target_dir = Path::new(&manifest_dir).join("target").join(profile);

    let mut app = Application::into_app();

    let path = generate_to(shells::Bash, &mut app, APP_NAME, &target_dir)?;
    println!(
        "cargo:warning=bash completion file is generated: {:?}",
        path
    );
    let path = generate_to(shells::Zsh, &mut app, APP_NAME, &target_dir)?;
    println!("cargo:warning=zsh completion file is generated: {:?}", path);
    let path = generate_to(shells::Fish, &mut app, APP_NAME, &target_dir)?;
    println!(
        "cargo:warning=fish completion file is generated: {:?}",
        path
    );
    let path = generate_to(shells::PowerShell, &mut app, APP_NAME, &target_dir)?;
    println!(
        "cargo:warning=powershell completion file is generated: {:?}",
        path
    );
    let path = generate_to(shells::Elvish, &mut app, APP_NAME, &target_dir)?;
    println!(
        "cargo:warning=elvish completion file is generated: {:?}",
        path
    );

    Ok(())
}
