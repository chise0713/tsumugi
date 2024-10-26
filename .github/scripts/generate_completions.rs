#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! clap = { version = "4.5.17", features = ["derive"] }
//! clap_complete = "4.5.33"
//! ```

use std::{env, fs};

use clap::CommandFactory as _;
use clap_complete::{generate_to, Shell};
#[path = "../../src/app.rs"]
mod app;

fn main() {
    let pkgname = "tsumugi";
    let completions = env::current_dir()
        .unwrap()
        .join("target/")
        .join("completions/");
    fs::create_dir_all(&completions).unwrap();
    let mut app = app::App::command();
    for shell in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ] {
        generate_to(shell, &mut app, pkgname, &completions).unwrap();
    }
    println!("{}", completions.to_string_lossy())
}
