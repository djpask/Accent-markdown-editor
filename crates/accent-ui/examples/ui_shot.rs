use accent_ui::{parse_cli_args, run_app};

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opts = parse_cli_args(&args);
    run_app(opts)
}
