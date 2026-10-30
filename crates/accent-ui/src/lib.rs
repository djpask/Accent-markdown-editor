pub mod ui;

use accent_session::{Action, Session};
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub file_path: Option<PathBuf>,
}

pub struct AccentApp {
    pub session: Session,
    frame_count: u32,
    sync_cursor_to_widget: bool,
}

impl AccentApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: LaunchOptions) -> Self {
        ui::theme::configure_fonts(&cc.egui_ctx);

        let mut session = Session::default();
        if let Some(fp) = opts.file_path {
            session.apply(Action::OpenFile(fp));
        }

        Self {
            session,
            frame_count: 0,
            sync_cursor_to_widget: true,
        }
    }
}

impl eframe::App for AccentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Borrow-Checker Rule (AGENTS.md Section 3):
        // Treat Session as read-only during UI rendering, collect Actions into a frame-local Vec,
        // and drain/apply all actions at the end of update().
        let atlas_before = ctx.fonts(|f| f.font_image_size());
        let mut actions = Vec::new();
        ui::render_ui(
            ctx,
            &self.session,
            &mut actions,
            &mut self.sync_cursor_to_widget,
        );
        let atlas_after = ctx.fonts(|f| f.font_image_size());
        let had_actions = !actions.is_empty();
        for action in actions {
            if matches!(
                action,
                Action::MoveCursor(_)
                    | Action::ApplyFormat(_)
                    | Action::NewDocument
                    | Action::OpenFile(_)
                    | Action::OpenTutorial
                    | Action::ConfirmRecoverySave
                    | Action::DiscardRecovery
                    | Action::FindNext
                    | Action::FindPrev
                    | Action::InsertText(_)
                    | Action::ReplaceNext
            ) {
                self.sync_cursor_to_widget = true;
            }
            self.session.apply(action);
        }

        // Phase 4: Poll < 300ms debounced Write-Ahead Recovery Journal (WAL) flush
        let now = Instant::now();
        let _ = self.session.poll_journal_flush(now);
        if let Some(wait) = self.session.time_until_next_journal_flush(now) {
            ctx.request_repaint_after(
                wait.max(Duration::from_millis(16))
                    .min(Duration::from_millis(250)),
            );
        }

        if had_actions || atlas_before != atlas_after || self.frame_count <= 3 {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.session.on_clean_exit();
    }
}

pub fn parse_cli_args(args: &[String]) -> LaunchOptions {
    let mut opts = LaunchOptions::default();
    for arg in args {
        if !arg.starts_with('-') && opts.file_path.is_none() {
            opts.file_path = Some(PathBuf::from(arg));
        }
    }
    opts
}

pub const OFFICIAL_ICON_SVG: &str = include_str!("../resources/icon.svg");

pub fn load_app_icon() -> Option<egui::IconData> {
    let (width, height, rgba, _png) = accent_mermaid::rasterize_svg(OFFICIAL_ICON_SVG, 0.25)?;
    Some(egui::IconData {
        rgba,
        width,
        height,
    })
}

pub fn run_app(opts: LaunchOptions) -> eframe::Result {
    // If DISPLAY=:99 is requested or running headless, force X11 on :99 with software GL
    let want_xvfb_99 = std::env::var("DISPLAY").as_deref() == Ok(":99")
        || (std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err());
    if want_xvfb_99 && std::path::Path::new("/tmp/.X11-unix/X99").exists() {
        unsafe {
            std::env::set_var("DISPLAY", ":99");
            std::env::set_var("WAYLAND_DISPLAY", "");
            std::env::set_var("WINIT_UNIX_BACKEND", "x11");
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        }
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Accent")
        .with_inner_size([1200.0, 1100.0])
        .with_min_inner_size([600.0, 400.0])
        .with_decorations(true);
    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Accent",
        native_options,
        Box::new(move |cc| Ok(Box::new(AccentApp::new(cc, opts)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use accent_session::ThemeMode;

    #[test]
    fn test_cli_args_parser() {
        let args = vec!["doc/esempio_accent.md".to_string()];
        let opts = parse_cli_args(&args);
        assert_eq!(opts.file_path, Some(PathBuf::from("doc/esempio_accent.md")));
    }

    #[test]
    fn test_theme_palettes_match_sampled_hex_values() {
        let light = ui::theme::AccentPalette::for_mode(ThemeMode::Light);
        assert_eq!(light.bg_canvas, egui::Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(light.fg_primary, egui::Color32::from_rgb(0x2e, 0x34, 0x36));

        let sepia = ui::theme::AccentPalette::for_mode(ThemeMode::Sepia);
        assert_eq!(sepia.bg_canvas, egui::Color32::from_rgb(0xf9, 0xf3, 0xe9));
        assert_eq!(sepia.fg_primary, egui::Color32::from_rgb(0x4f, 0x39, 0x15));

        let dark = ui::theme::AccentPalette::for_mode(ThemeMode::Dark);
        assert_eq!(dark.bg_canvas, egui::Color32::from_rgb(0x2b, 0x2b, 0x2f));
        assert_eq!(dark.fg_primary, egui::Color32::from_rgb(0xee, 0xee, 0xee));
    }

    #[test]
    fn test_official_accent_logo_svg_and_png_generation() {
        let (w, h, _rgba, png_bytes) = accent_mermaid::rasterize_svg(OFFICIAL_ICON_SVG, 1.0)
            .expect("Failed to rasterize official Accent SVG icon");
        assert_eq!(w, 512);
        assert_eq!(h, 512);
        assert!(png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png_bytes.len() > 10_000);

        let brand_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../img/brand");
        let _ = std::fs::create_dir_all(&brand_dir);
        std::fs::write(brand_dir.join("accent_logo.png"), &png_bytes)
            .expect("Failed to write img/brand/accent_logo.png");
    }
}
