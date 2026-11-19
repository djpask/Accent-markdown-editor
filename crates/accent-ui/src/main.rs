#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use accent_ui::{parse_cli_args, run_app};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opts = parse_cli_args(&args);
    if let Err(err) = run_app(opts) {
        eprintln!("Accent fatal error: {err}");
        std::process::exit(1);
    }
}
