use clap::Shell;
use std::env;

include!("src/app.rs");

const APP_NAME: &'static str = "gmux";

fn main() {
    let out_dir = match env::var_os("OUT_DIR") {
        None => return,
        Some(out_dir) => out_dir,
    };

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
}
