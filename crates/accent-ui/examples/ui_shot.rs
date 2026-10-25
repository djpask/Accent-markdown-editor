use accent_ui::{parse_cli_args, run_app};
use std::path::PathBuf;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = parse_cli_args(&args);
    if opts.shot_path.is_none() {
        opts.shot_path = Some(PathBuf::from("img/Accent_screenshots/accent_shot.png"));
    }
    run_app(opts)
}
