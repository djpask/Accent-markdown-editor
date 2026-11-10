pub mod ui;

use accent_session::{Action, Session};
use accent_storage::{
    FocusedFileWatcher, IpcRequest, SingleInstanceGuard, SingleInstanceOutcome,
};
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
    single_instance: Option<SingleInstanceGuard>,
    file_watcher: Option<FocusedFileWatcher>,
    last_watched_path: Option<PathBuf>,
    last_fullscreen: bool,
}

impl AccentApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: LaunchOptions) -> Self {
        Self::with_single_instance(cc, opts, None)
    }

    pub fn with_single_instance(
        cc: &eframe::CreationContext<'_>,
        opts: LaunchOptions,
        single_instance: Option<SingleInstanceGuard>,
    ) -> Self {
        ui::theme::configure_fonts(&cc.egui_ctx);

        if let Some(ref guard) = single_instance {
            let egui_ctx = cc.egui_ctx.clone();
            guard.set_waker(move || {
                egui_ctx.request_repaint();
            });
        }

        let mut session = Session::default();
        if let Some(fp) = opts.file_path {
            session.apply(Action::OpenFile(fp));
        }

        let file_watcher = FocusedFileWatcher::new().ok();
        if let Some(ref watcher) = file_watcher {
            let egui_ctx = cc.egui_ctx.clone();
            watcher.set_waker(move || {
                egui_ctx.request_repaint();
            });
            watcher.set_watched_file(session.current_file_path.clone());
        }
        let last_watched_path = session.current_file_path.clone();
        let last_fullscreen = session.fullscreen;

        Self {
            session,
            frame_count: 0,
            sync_cursor_to_widget: true,
            single_instance,
            file_watcher,
            last_watched_path,
            last_fullscreen,
        }
    }
}

impl eframe::App for AccentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        if self.frame_count <= 3 {
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }

        // Process any incoming single-instance IPC requests from secondary `accent [file.md]` launches
        if let Some(ref guard) = self.single_instance {
            let ipc_requests = guard.drain_requests();
            if !ipc_requests.is_empty() {
                for req in ipc_requests {
                    if let IpcRequest::OpenFile(path) = req {
                        self.session.apply(Action::OpenFile(path));
                        self.sync_cursor_to_widget = true;
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.request_repaint();
            }
        }

        // Check OS-driven file modification notification or window focus regain (zero continuous polling)
        let watcher_notified = self
            .file_watcher
            .as_ref()
            .is_some_and(|w| w.take_change_notification());
        let window_gained_focus = ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::WindowFocused(true)))
        });
        if watcher_notified || window_gained_focus {
            self.session.apply(Action::CheckExternalFileModification);
        }

        // Borrow-Checker Rule (AGENTS.md Section 3):
        // Treat Session as read-only during UI rendering, collect Actions into a frame-local Vec,
        // and drain/apply all actions at the end of update().
        let rendered_tab_id = self
            .session
            .tabs
            .get(self.session.active_tab_index)
            .map(|t| t.tab_id);
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
            let current_tab_id = self
                .session
                .tabs
                .get(self.session.active_tab_index)
                .map(|t| t.tab_id);
            if matches!(
                action,
                Action::UpdateBufferText(_) | Action::SetSelection(_, _) | Action::MoveCursor(_)
            ) && current_tab_id != rendered_tab_id
            {
                // Never apply buffer or cursor mutations from a previous tab's widget onto a newly switched tab
                continue;
            }
            if matches!(
                action,
                Action::MoveCursor(_)
                    | Action::ApplyFormat(_)
                    | Action::NewDocument
                    | Action::NewTab
                    | Action::SwitchTab(_)
                    | Action::CloseTab(_)
                    | Action::NextTab
                    | Action::PrevTab
                    | Action::Undo
                    | Action::Redo
                    | Action::OpenFile(_)
                    | Action::OpenTutorial
                    | Action::ConfirmRecoverySave
                    | Action::DiscardRecovery
                    | Action::ReloadFromDisk
                    | Action::FindNext
                    | Action::FindPrev
                    | Action::InsertText(_)
                    | Action::ReplaceNext
            ) {
                self.sync_cursor_to_widget = true;
            }
            if matches!(
                action,
                Action::SaveDocument | Action::SaveAsFile(_) | Action::ConfirmRecoverySave
            ) {
                if let Some(ref watcher) = self.file_watcher {
                    watcher.suppress_notifications_for(180);
                }
            }
            self.session.apply(action);
        }

        // Update OS file watcher target whenever the active tab's file path changes
        if self.session.current_file_path != self.last_watched_path {
            self.last_watched_path = self.session.current_file_path.clone();
            if let Some(ref watcher) = self.file_watcher {
                watcher.set_watched_file(self.last_watched_path.clone());
            }
        }

        // Synchronize fullscreen viewport command when toggled via menu or F11
        if self.session.fullscreen != self.last_fullscreen {
            self.last_fullscreen = self.session.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.session.fullscreen));
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

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.session.on_clean_exit();
        self.file_watcher = None;
        self.single_instance = None;
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

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const OFFICIAL_ICON_SVG: &str = include_str!("../resources/icon.svg");

pub fn load_app_icon() -> Option<egui::IconData> {
    let (width, height, rgba, _png) = accent_mermaid::rasterize_svg(OFFICIAL_ICON_SVG, 0.25)?;
    Some(egui::IconData {
        rgba,
        width,
        height,
    })
}

/// Generates a multi-resolution Windows `.ico` file (`256x256`, `64x64`, `32x32`, `16x16` PNG frames)
/// directly from the official Accent SVG logo.
pub fn generate_windows_ico() -> Option<Vec<u8>> {
    let scales = [
        (0u8, 0.5_f32),     // 256x256 (0 in ICONDIRENTRY means 256px)
        (64u8, 0.125_f32),  // 64x64
        (32u8, 0.0625_f32), // 32x32
        (16u8, 0.03125_f32), // 16x16
    ];
    let mut png_frames = Vec::with_capacity(scales.len());
    for (dim_byte, scale) in scales {
        let (_w, _h, _rgba, png_bytes) = accent_mermaid::rasterize_svg(OFFICIAL_ICON_SVG, scale)?;
        png_frames.push((dim_byte, png_bytes));
    }

    let count = png_frames.len() as u16;
    let mut ico = Vec::new();
    // ICONDIR header (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes()); // idReserved = 0
    ico.extend_from_slice(&1u16.to_le_bytes()); // idType = 1 (ICO)
    ico.extend_from_slice(&count.to_le_bytes()); // idCount

    let mut data_offset: u32 = 6 + (16 * png_frames.len() as u32);
    for (dim_byte, png_bytes) in &png_frames {
        let size = png_bytes.len() as u32;
        ico.push(*dim_byte); // bWidth
        ico.push(*dim_byte); // bHeight
        ico.push(0); // bColorCount
        ico.push(0); // bReserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // wPlanes = 1
        ico.extend_from_slice(&32u16.to_le_bytes()); // wBitCount = 32
        ico.extend_from_slice(&size.to_le_bytes()); // dwBytesInRes
        ico.extend_from_slice(&data_offset.to_le_bytes()); // dwImageOffset
        data_offset += size;
    }

    for (_dim_byte, png_bytes) in png_frames {
        ico.extend_from_slice(&png_bytes);
    }
    Some(ico)
}

pub fn run_app(opts: LaunchOptions) -> eframe::Result {
    // Clear any inherited XDG/X11 startup notification ID so desktop compositors
    // never leave the mouse pointer in the 15-second spinning "busy" state.
    unsafe {
        std::env::remove_var("DESKTOP_STARTUP_ID");
    }

    // Enforce Single-Instance execution: if another Accent instance is already running,
    // forward the requested file (or a focus request) to it and exit immediately.
    let lock_path = accent_storage::default_instance_lock_path();
    let single_instance =
        match SingleInstanceGuard::acquire_or_forward(&lock_path, opts.file_path.as_deref()) {
            Ok(SingleInstanceOutcome::ForwardedToExisting) => {
                return Ok(());
            }
            Ok(SingleInstanceOutcome::Primary(guard)) => Some(guard),
            Err(_) => None,
        };

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
        .with_app_id("accent")
        .with_title("Accent")
        .with_inner_size([1200.0, 1100.0])
        .with_min_inner_size([600.0, 400.0])
        .with_decorations(false)
        .with_transparent(true);
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
        Box::new(move |cc| {
            Ok(Box::new(AccentApp::with_single_instance(
                cc,
                opts,
                single_instance,
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use accent_session::{ColorPalettePreset, ThemeMode};

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
    fn test_ergonomic_palette_presets() {
        let ed_light = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::Light,
        );
        assert_eq!(ed_light.bg_canvas, egui::Color32::from_rgb(0xfa, 0xf9, 0xf5));
        assert_eq!(ed_light.accent_orange, egui::Color32::from_rgb(0x2b, 0x6c, 0xb0));

        let ed_sepia = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::Sepia,
        );
        assert_eq!(ed_sepia.bg_canvas, egui::Color32::from_rgb(0xf4, 0xef, 0xe6));
        assert_eq!(ed_sepia.accent_orange, egui::Color32::from_rgb(0x2c, 0x6e, 0x63));

        let ed_dark = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::Dark,
        );
        assert_eq!(ed_dark.bg_canvas, egui::Color32::from_rgb(0x19, 0x1c, 0x22));

        let nordic_sepia = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::NordicForest,
            ThemeMode::Sepia,
        );
        assert_eq!(nordic_sepia.bg_canvas, egui::Color32::from_rgb(0xf3, 0xea, 0xd3));

        let velvet_dark = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::VelvetDawn,
            ThemeMode::Dark,
        );
        assert_eq!(velvet_dark.bg_canvas, egui::Color32::from_rgb(0x1f, 0x1d, 0x29));
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

        let ico_bytes = generate_windows_ico().expect("Failed to generate Windows .ico");
        assert!(&ico_bytes[0..4] == &[0x00, 0x00, 0x01, 0x00]);
        std::fs::write(brand_dir.join("accent_logo.ico"), &ico_bytes)
            .expect("Failed to write img/brand/accent_logo.ico");
    }

    #[test]
    fn test_version_system_consistency() {
        assert_eq!(VERSION, "0.8.10");
        assert_eq!(accent_core::VERSION, VERSION);
        let version_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../VERSION");
        let file_ver = std::fs::read_to_string(version_file)
            .expect("Failed to read root VERSION file");
        assert_eq!(file_ver.trim(), VERSION);
    }
}
