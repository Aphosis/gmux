use clap::Shell;
use std::env;
use std::fs;
use std::path::Path;

include!("src/app.rs");

const APP_NAME: &'static str = "gmux";

fn main() {
    let out_dir = match env::var_os("OUT_DIR") {
        None => {
            println!("cargo:warning=Could not find output dir, aborting completions generation.");
            return;
        }
        Some(out_dir) => out_dir,
    };

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
            return;
        }
        Some(manifest_dir) => manifest_dir,
    };

    let profile = match env::var_os("PROFILE") {
        None => {
            println!("cargo:warning=Could not find profile, aborting completions generation.");
            return;
        }
        Some(profile) => profile,
    };

    let target_dir = Path::new(&manifest_dir).join("target").join(profile);

    let mut app = Application::clap();

    app.gen_completions(
        APP_NAME,    // We need to specify the bin name manually
        Shell::Bash, // Then say which shell to build completions for
        &out_dir,    // Then say where write the completions to
    );
    app.gen_completions(APP_NAME, Shell::Zsh, &out_dir);
    app.gen_completions(APP_NAME, Shell::PowerShell, &out_dir);
    app.gen_completions(APP_NAME, Shell::Fish, &out_dir);
    app.gen_completions(APP_NAME, Shell::Elvish, &out_dir);

    let completions = vec![
        format!("{}.bash", APP_NAME),
        format!("_{}", APP_NAME),
        format!("_{}.ps1", APP_NAME),
        format!("{}.fish", APP_NAME),
        format!("{}.elv", APP_NAME),
    ];

    for filename in completions {
        match fs::copy(
            Path::new(&out_dir).join(&filename),
            target_dir.join(&filename),
        ) {
            Ok(_) => (),
            Err(err) => {
                println!("cargo:warning=Error while copying completions: {}", err);
                return;
            }
        }
    }
}
