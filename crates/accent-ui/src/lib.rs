pub mod ui;

use accent_session::{
    Action, ActiveOverlay, LineWidthMode, Locale, MarkdownDialect, PreviewLayout, Session,
    ThemeMode,
};
use eframe::egui;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub theme: Option<ThemeMode>,
    pub locale: Option<Locale>,
    pub dialect: Option<MarkdownDialect>,
    pub line_width: Option<LineWidthMode>,
    pub prose_font: Option<String>,
    pub code_font: Option<String>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub overlay: Option<ActiveOverlay>,
    pub preview_layout: Option<PreviewLayout>,
    pub view_while_typing: Option<bool>,
    pub cursor_offset: Option<usize>,
    pub insert_text: Option<String>,
    pub focus_mode: bool,
    pub hemingway_mode: bool,
    pub shot_path: Option<PathBuf>,
}

pub struct AccentApp {
    pub session: Session,
    shot_path: Option<PathBuf>,
    frame_count: u32,
    shot_requested: bool,
    sync_cursor_to_widget: bool,
}

impl AccentApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: LaunchOptions) -> Self {
        ui::theme::configure_fonts(&cc.egui_ctx);

        let mut session = Session::default();
        if let Some(t) = opts.theme {
            session.apply(Action::SetTheme(t));
        }
        if let Some(l) = opts.locale {
            session.apply(Action::SetLocale(l));
        }
        if let Some(d) = opts.dialect {
            session.apply(Action::SetDialect(d));
        }
        if let Some(lw) = opts.line_width {
            session.apply(Action::SetLineWidth(lw));
        }
        if let Some(pf) = opts.prose_font {
            session.apply(Action::SetProseFont(pf));
        }
        if let Some(cf) = opts.code_font {
            session.apply(Action::SetCodeFont(cf));
        }
        if let Some(fs) = opts.font_size {
            session.apply(Action::SetFontSize(fs));
        }
        if let Some(lh) = opts.line_height {
            session.apply(Action::SetLineHeight(lh));
        }
        if let Some(layout) = opts.preview_layout {
            session.apply(Action::SetPreviewMode {
                active: true,
                layout,
            });
        }
        if let Some(vwt) = opts.view_while_typing {
            session.apply(Action::SetViewWhileTyping(vwt));
        }
        if let Some(cur) = opts.cursor_offset {
            session.apply(Action::MoveCursor(cur));
        }
        if let Some(txt) = opts.insert_text {
            session.apply(Action::InsertText(txt));
        }
        if let Some(o) = opts.overlay {
            session.apply(Action::SetOverlay(o));
        }
        if opts.focus_mode {
            session.apply(Action::ToggleFocusMode);
        }
        if opts.hemingway_mode {
            session.apply(Action::ToggleHemingway);
        }

        Self {
            session,
            shot_path: opts.shot_path,
            frame_count: 0,
            shot_requested: false,
            sync_cursor_to_widget: true,
        }
    }
}

impl eframe::App for AccentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Handle incoming screenshot event if --shot was requested
        let events = ctx.input(|i| i.raw.events.clone());
        for event in events {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(path) = &self.shot_path {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let pixels = image.as_raw();
                    let w = image.width() as u32;
                    let h = image.height() as u32;
                    let _ = image::save_buffer(path, pixels, w, h, image::ColorType::Rgba8);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        // Borrow-Checker Rule (AGENTS.md Section 3):
        // Treat Session as read-only during UI rendering, collect Actions into a frame-local Vec,
        // and drain/apply all actions at the end of update().
        let mut actions = Vec::new();
        ui::render_ui(
            ctx,
            &self.session,
            &mut actions,
            &mut self.sync_cursor_to_widget,
        );
        let had_actions = !actions.is_empty();
        for action in actions {
            if matches!(
                action,
                Action::MoveCursor(_)
                    | Action::ApplyFormat(_)
                    | Action::NewDocument
                    | Action::OpenFile(_)
                    | Action::OpenTutorial
                    | Action::FindNext
                    | Action::FindPrev
                    | Action::InsertText(_)
                    | Action::ReplaceNext
            ) {
                self.sync_cursor_to_widget = true;
            }
            self.session.apply(action);
        }
        if had_actions {
            ctx.request_repaint();
        }

        if self.shot_path.is_some() {
            if self.frame_count >= 2 && !self.shot_requested {
                self.shot_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            ctx.request_repaint();
        }
    }
}

pub fn parse_cli_args(args: &[String]) -> LaunchOptions {
    let mut opts = LaunchOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--shot" if i + 1 < args.len() => {
                opts.shot_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--theme" if i + 1 < args.len() => {
                opts.theme = ThemeMode::from_str_name(&args[i + 1]);
                i += 2;
            }
            "--locale" if i + 1 < args.len() => {
                opts.locale = Locale::from_code(&args[i + 1]);
                i += 2;
            }
            "--dialect" if i + 1 < args.len() => {
                opts.dialect = MarkdownDialect::from_str_name(&args[i + 1]);
                i += 2;
            }
            "--width" if i + 1 < args.len() => {
                opts.line_width = LineWidthMode::from_str_name(&args[i + 1]);
                i += 2;
            }
            "--prose-font" if i + 1 < args.len() => {
                opts.prose_font = Some(args[i + 1].clone());
                i += 2;
            }
            "--code-font" if i + 1 < args.len() => {
                opts.code_font = Some(args[i + 1].clone());
                i += 2;
            }
            "--font-size" if i + 1 < args.len() => {
                opts.font_size = args[i + 1].parse().ok();
                i += 2;
            }
            "--line-height" if i + 1 < args.len() => {
                opts.line_height = args[i + 1].parse().ok();
                i += 2;
            }
            "--preview" if i + 1 < args.len() => {
                opts.preview_layout = PreviewLayout::from_str_name(&args[i + 1]);
                i += 2;
            }
            "--live" | "--view-while-typing" => {
                opts.view_while_typing = Some(true);
                i += 1;
            }
            "--cursor" if i + 1 < args.len() => {
                opts.cursor_offset = args[i + 1].parse().ok();
                i += 2;
            }
            "--insert" if i + 1 < args.len() => {
                opts.insert_text = Some(args[i + 1].clone());
                i += 2;
            }
            "--overlay" if i + 1 < args.len() => {
                opts.overlay = match args[i + 1].to_ascii_lowercase().as_str() {
                    "open" => Some(ActiveOverlay::OpenMenu),
                    "save" => Some(ActiveOverlay::SaveMenu),
                    "preview" => Some(ActiveOverlay::PreviewMenu),
                    "main" => Some(ActiveOverlay::MainMenu),
                    "stats" => Some(ActiveOverlay::StatsMenu),
                    "width" | "linewidth" => Some(ActiveOverlay::LineWidthMenu),
                    "prefs" | "preferences" => Some(ActiveOverlay::PreferencesDialog),
                    _ => Some(ActiveOverlay::None),
                };
                i += 2;
            }
            "--focus" => {
                opts.focus_mode = true;
                i += 1;
            }
            "--hemingway" => {
                opts.hemingway_mode = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    opts
}

pub fn run_app(opts: LaunchOptions) -> eframe::Result {
    // If DISPLAY=:99 is requested or running headless/screenshot mode, force X11 on :99 with software GL
    let want_xvfb_99 = std::env::var("DISPLAY").as_deref() == Ok(":99")
        || (std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err())
        || opts.shot_path.is_some();
    if want_xvfb_99 && std::path::Path::new("/tmp/.X11-unix/X99").exists() {
        unsafe {
            std::env::set_var("DISPLAY", ":99");
            std::env::set_var("WAYLAND_DISPLAY", "");
            std::env::set_var("WINIT_UNIX_BACKEND", "x11");
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Accent")
            .with_inner_size([1200.0, 1100.0])
            .with_min_inner_size([600.0, 400.0])
            .with_decorations(false),
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

    #[test]
    fn test_cli_args_parser() {
        let args = vec![
            "--theme".to_string(),
            "dark".to_string(),
            "--locale".to_string(),
            "en".to_string(),
            "--dialect".to_string(),
            "glfm".to_string(),
            "--width".to_string(),
            "wide".to_string(),
            "--preview".to_string(),
            "halfwidth".to_string(),
            "--live".to_string(),
            "--cursor".to_string(),
            "375".to_string(),
            "--overlay".to_string(),
            "preview".to_string(),
            "--shot".to_string(),
            "img/Accent_screenshots/accent_shot.png".to_string(),
        ];
        let opts = parse_cli_args(&args);
        assert_eq!(opts.theme, Some(ThemeMode::Dark));
        assert_eq!(opts.locale, Some(Locale::En));
        assert_eq!(opts.dialect, Some(MarkdownDialect::Glfm));
        assert_eq!(opts.line_width, Some(LineWidthMode::Wide));
        assert_eq!(opts.preview_layout, Some(PreviewLayout::HalfWidth));
        assert_eq!(opts.view_while_typing, Some(true));
        assert_eq!(opts.cursor_offset, Some(375));
        assert_eq!(opts.overlay, Some(ActiveOverlay::PreviewMenu));
        assert_eq!(
            opts.shot_path,
            Some(PathBuf::from("img/Accent_screenshots/accent_shot.png"))
        );
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
}
