pub mod ui;

use accent_mermaid::MermaidTheme;
use accent_session::{Action, ColorPalettePreset, Session, ThemeMode};
use accent_storage::{
    FocusedFileWatcher, IpcRequest, SingleInstanceGuard, SingleInstanceOutcome,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use ui::{
    MermaidSvgPayload, ResolvedLocalImagePayload, UiSessionSnapshot, build_session_snapshot,
    theme::{AccentPalette, PaletteCssTokens},
};

#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    pub file_path: Option<PathBuf>,
}

pub struct TauriAppState {
    pub session: Mutex<Session>,
    pub single_instance: Mutex<Option<SingleInstanceGuard>>,
    pub file_watcher: Mutex<Option<FocusedFileWatcher>>,
    pub last_watched_path: Mutex<Option<PathBuf>>,
}

impl TauriAppState {
    pub fn sync_file_watcher(&self, current_file_path: Option<PathBuf>, suppress_ms: Option<u64>) {
        if let Some(ms) = suppress_ms {
            if let Ok(guard) = self.file_watcher.lock() {
                if let Some(ref watcher) = *guard {
                    watcher.suppress_notifications_for(ms);
                }
            }
        }
        if let Ok(mut last_guard) = self.last_watched_path.lock() {
            if *last_guard != current_file_path {
                *last_guard = current_file_path.clone();
                if let Ok(w_guard) = self.file_watcher.lock() {
                    if let Some(ref watcher) = *w_guard {
                        watcher.set_watched_file(current_file_path);
                    }
                }
            }
        }
    }
}

#[tauri::command]
fn get_session_snapshot(state: State<'_, Arc<TauriAppState>>) -> Result<UiSessionSnapshot, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    Ok(build_session_snapshot(&session))
}

#[tauri::command]
fn dispatch_action(
    app: AppHandle,
    state: State<'_, Arc<TauriAppState>>,
    action: Action,
) -> Result<UiSessionSnapshot, String> {
    let is_save = matches!(
        action,
        Action::SaveDocument | Action::SaveAsFile(_) | Action::ConfirmRecoverySave
    );
    let is_fullscreen_action = matches!(
        action,
        Action::ToggleFullscreen | Action::SetFullscreen(_)
    );

    let (snapshot, current_path, fullscreen_state) = {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        if is_save {
            state.sync_file_watcher(session.current_file_path.clone(), Some(180));
        }
        session.apply(action);
        let snap = build_session_snapshot(&session);
        (snap, session.current_file_path.clone(), session.fullscreen)
    };

    state.sync_file_watcher(current_path, None);

    if is_fullscreen_action {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.set_fullscreen(fullscreen_state);
        }
    }

    Ok(snapshot)
}

#[tauri::command]
fn update_editor_buffer(
    state: State<'_, Arc<TauriAppState>>,
    tab_id: u64,
    text: String,
    selection_primary: usize,
    selection_secondary: usize,
) -> Result<UiSessionSnapshot, String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    let active_tab_id = session
        .tabs
        .get(session.active_tab_index)
        .map(|t| t.tab_id);
    if active_tab_id == Some(tab_id) {
        session.apply(Action::UpdateBufferText(text));
        session.apply(Action::SetSelection(selection_primary, selection_secondary));
    }
    Ok(build_session_snapshot(&session))
}

#[cfg(target_os = "linux")]
async fn pick_open_file_native(
    app: &AppHandle,
    dir: PathBuf,
    title: String,
    accept_label: String,
    cancel_label: String,
    md_filter_label: String,
    all_filter_label: String,
) -> Option<PathBuf> {
    use gtk::prelude::{FileChooserExt, NativeDialogExt};
    use std::cell::RefCell;
    use std::rc::Rc;

    let (tx, rx) = std::sync::mpsc::sync_channel::<Option<PathBuf>>(1);
    let app_for_ui = app.clone();
    if app
        .run_on_main_thread(move || {
            let gtk_win = app_for_ui
                .get_webview_window("main")
                .and_then(|w| w.gtk_window().ok());
            let dialog = gtk::FileChooserNative::new(
                Some(&title),
                gtk_win.as_ref(),
                gtk::FileChooserAction::Open,
                Some(&accept_label),
                Some(&cancel_label),
            );
            dialog.set_modal(true);
            dialog.set_local_only(true);
            if dir.is_dir() {
                let _ = dialog.set_current_folder(&dir);
            }

            let md_filter = gtk::FileFilter::new();
            md_filter.set_name(Some(&md_filter_label));
            for pat in ["*.md", "*.markdown", "*.mdown", "*.mkd", "*.txt"] {
                md_filter.add_pattern(pat);
            }
            dialog.add_filter(md_filter);

            let all_filter = gtk::FileFilter::new();
            all_filter.set_name(Some(&all_filter_label));
            all_filter.add_pattern("*");
            dialog.add_filter(all_filter);

            let holder: Rc<RefCell<Option<gtk::FileChooserNative>>> =
                Rc::new(RefCell::new(Some(dialog.clone())));
            let holder_cb = Rc::clone(&holder);
            let tx_cell = RefCell::new(Some(tx));

            dialog.connect_response(move |dlg, resp| {
                let picked = if resp == gtk::ResponseType::Accept {
                    dlg.filename()
                } else {
                    None
                };
                dlg.destroy();
                if let Some(sender) = tx_cell.borrow_mut().take() {
                    let _ = sender.send(picked);
                }
                let _ = holder_cb.borrow_mut().take();
            });

            dialog.show();
        })
        .is_err()
    {
        return None;
    }

    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
}

#[cfg(not(target_os = "linux"))]
async fn pick_open_file_native(
    app: &AppHandle,
    dir: PathBuf,
    title: String,
    _accept_label: String,
    _cancel_label: String,
    md_filter_label: String,
    all_filter_label: String,
) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = std::sync::mpsc::sync_channel::<Option<PathBuf>>(1);
    let mut builder = app
        .dialog()
        .file()
        .set_title(&title)
        .add_filter(&md_filter_label, &["md", "markdown", "mdown", "mkd", "txt"])
        .add_filter(&all_filter_label, &["*"]);
    if dir.is_dir() {
        builder = builder.set_directory(&dir);
    }
    if let Some(win) = app.get_webview_window("main") {
        builder = builder.set_parent(&win);
    }
    builder.pick_file(move |fp| {
        let _ = tx.send(fp.and_then(|p| p.into_path().ok()));
    });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
}

#[cfg(target_os = "linux")]
async fn pick_save_file_native(
    app: &AppHandle,
    dir: PathBuf,
    default_file_name: String,
    title: String,
    accept_label: String,
    cancel_label: String,
    filter_label: String,
    extensions: Vec<String>,
    all_filter_label: String,
) -> Option<PathBuf> {
    use gtk::prelude::{FileChooserExt, NativeDialogExt};
    use std::cell::RefCell;
    use std::rc::Rc;

    let primary_ext = extensions.first().cloned();
    let (tx, rx) = std::sync::mpsc::sync_channel::<Option<PathBuf>>(1);
    let app_for_ui = app.clone();
    if app
        .run_on_main_thread(move || {
            let gtk_win = app_for_ui
                .get_webview_window("main")
                .and_then(|w| w.gtk_window().ok());
            let dialog = gtk::FileChooserNative::new(
                Some(&title),
                gtk_win.as_ref(),
                gtk::FileChooserAction::Save,
                Some(&accept_label),
                Some(&cancel_label),
            );
            dialog.set_modal(true);
            dialog.set_local_only(true);
            dialog.set_create_folders(true);
            dialog.set_do_overwrite_confirmation(true);
            if dir.is_dir() {
                let _ = dialog.set_current_folder(&dir);
            }
            if !default_file_name.is_empty() {
                dialog.set_current_name(&default_file_name);
            }

            if !extensions.is_empty() {
                let fmt_filter = gtk::FileFilter::new();
                fmt_filter.set_name(Some(&filter_label));
                for ext in &extensions {
                    fmt_filter.add_pattern(&format!("*.{ext}"));
                }
                dialog.add_filter(fmt_filter);
            }

            let all_filter = gtk::FileFilter::new();
            all_filter.set_name(Some(&all_filter_label));
            all_filter.add_pattern("*");
            dialog.add_filter(all_filter);

            let holder: Rc<RefCell<Option<gtk::FileChooserNative>>> =
                Rc::new(RefCell::new(Some(dialog.clone())));
            let holder_cb = Rc::clone(&holder);
            let tx_cell = RefCell::new(Some(tx));

            dialog.connect_response(move |dlg, resp| {
                let picked = if resp == gtk::ResponseType::Accept {
                    dlg.filename()
                } else {
                    None
                };
                dlg.destroy();
                if let Some(sender) = tx_cell.borrow_mut().take() {
                    let _ = sender.send(picked);
                }
                let _ = holder_cb.borrow_mut().take();
            });

            dialog.show();
        })
        .is_err()
    {
        return None;
    }

    let picked = tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()?;
    if picked.extension().is_none() {
        if let Some(ext) = primary_ext {
            return Some(picked.with_extension(ext));
        }
    }
    Some(picked)
}

#[cfg(not(target_os = "linux"))]
async fn pick_save_file_native(
    app: &AppHandle,
    dir: PathBuf,
    default_file_name: String,
    title: String,
    _accept_label: String,
    _cancel_label: String,
    filter_label: String,
    extensions: Vec<String>,
    all_filter_label: String,
) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;

    let primary_ext = extensions.first().cloned();
    let (tx, rx) = std::sync::mpsc::sync_channel::<Option<PathBuf>>(1);
    let ext_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
    let mut builder = app.dialog().file().set_title(&title);
    if !ext_refs.is_empty() {
        builder = builder.add_filter(&filter_label, &ext_refs);
    }
    builder = builder.add_filter(&all_filter_label, &["*"]);
    if dir.is_dir() {
        builder = builder.set_directory(&dir);
    }
    if !default_file_name.is_empty() {
        builder = builder.set_file_name(&default_file_name);
    }
    if let Some(win) = app.get_webview_window("main") {
        builder = builder.set_parent(&win);
    }
    builder.save_file(move |fp| {
        let _ = tx.send(fp.and_then(|p| p.into_path().ok()));
    });
    let picked = tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()?;
    if picked.extension().is_none() {
        if let Some(ext) = primary_ext {
            return Some(picked.with_extension(ext));
        }
    }
    Some(picked)
}

#[tauri::command]
async fn open_native_file_dialog(
    app: AppHandle,
    state: State<'_, Arc<TauriAppState>>,
) -> Result<UiSessionSnapshot, String> {
    let (dir, title, accept_lbl, cancel_lbl, md_lbl, all_lbl) = {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        if session.overlay != accent_session::ActiveOverlay::None {
            session.apply(Action::SetOverlay(accent_session::ActiveOverlay::None));
        }
        let i18n = ui::build_i18n_map(session.locale);
        (
            session.file_dialog_dir.clone(),
            i18n.get("open_file")
                .cloned()
                .unwrap_or_else(|| "Open".to_string()),
            i18n.get("dialog_accept_open")
                .cloned()
                .unwrap_or_else(|| "Open".to_string()),
            i18n.get("cancel")
                .cloned()
                .unwrap_or_else(|| "Cancel".to_string()),
            i18n.get("dialog_md_files")
                .cloned()
                .unwrap_or_else(|| "Markdown".to_string()),
            i18n.get("dialog_all_files")
                .cloned()
                .unwrap_or_else(|| "All Files".to_string()),
        )
    };

    let picked = pick_open_file_native(
        &app,
        dir,
        title,
        accept_lbl,
        cancel_lbl,
        md_lbl,
        all_lbl,
    )
    .await;

    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    if let Some(path) = picked {
        session.apply(Action::OpenFile(path));
    }
    let cur_path = session.current_file_path.clone();
    let snap = build_session_snapshot(&session);
    drop(session);
    state.sync_file_watcher(cur_path, None);
    Ok(snap)
}

#[tauri::command]
async fn save_as_native_file_dialog(
    app: AppHandle,
    state: State<'_, Arc<TauriAppState>>,
) -> Result<UiSessionSnapshot, String> {
    let (dir, file_name, title, accept_lbl, cancel_lbl, md_lbl, all_lbl) = {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        if session.overlay != accent_session::ActiveOverlay::None {
            session.apply(Action::SetOverlay(accent_session::ActiveOverlay::None));
        }
        let initial_dir = session
            .current_file_path
            .as_ref()
            .and_then(|p| p.parent())
            .filter(|p| p.is_dir() && *p != std::env::temp_dir().as_path())
            .map(PathBuf::from)
            .unwrap_or_else(|| session.file_dialog_dir.clone());
        let i18n = ui::build_i18n_map(session.locale);
        (
            initial_dir,
            session.document_title.clone(),
            i18n.get("save_as")
                .cloned()
                .unwrap_or_else(|| "Save As".to_string()),
            i18n.get("dialog_accept_save")
                .cloned()
                .unwrap_or_else(|| "Save".to_string()),
            i18n.get("cancel")
                .cloned()
                .unwrap_or_else(|| "Cancel".to_string()),
            i18n.get("dialog_md_files")
                .cloned()
                .unwrap_or_else(|| "Markdown".to_string()),
            i18n.get("dialog_all_files")
                .cloned()
                .unwrap_or_else(|| "All Files".to_string()),
        )
    };

    let saved = pick_save_file_native(
        &app,
        dir,
        file_name,
        title,
        accept_lbl,
        cancel_lbl,
        md_lbl,
        vec!["md".to_string(), "markdown".to_string()],
        all_lbl,
    )
    .await;

    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    if let Some(path) = saved {
        state.sync_file_watcher(Some(path.clone()), Some(180));
        session.apply(Action::SaveAsFile(path));
    }
    let cur_path = session.current_file_path.clone();
    let snap = build_session_snapshot(&session);
    drop(session);
    state.sync_file_watcher(cur_path, None);
    Ok(snap)
}

#[tauri::command]
async fn export_with_native_dialog(
    app: AppHandle,
    state: State<'_, Arc<TauriAppState>>,
    format: Option<accent_session::ExportFormat>,
    execute_immediately: bool,
) -> Result<UiSessionSnapshot, String> {
    let (dir, file_name, title, accept_lbl, cancel_lbl, filter_lbl, ext_str, all_lbl) = {
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        if let Some(fmt) = format {
            session.apply(Action::SelectExportFormat(fmt));
        }
        if execute_immediately && session.overlay != accent_session::ActiveOverlay::None {
            session.apply(Action::SetOverlay(accent_session::ActiveOverlay::None));
        }
        let fmt = session.selected_export_format;
        let ext = fmt.extension().to_string();
        let stem = session
            .document_title
            .trim_end_matches(".md")
            .trim_end_matches(".markdown");
        let existing_out = PathBuf::from(session.export_output_path.trim());
        let initial_dir = session
            .current_file_path
            .as_ref()
            .and_then(|p| p.parent())
            .filter(|p| p.is_dir() && *p != std::env::temp_dir().as_path())
            .map(PathBuf::from)
            .or_else(|| {
                existing_out
                    .parent()
                    .filter(|p| p.is_dir() && *p != std::env::temp_dir().as_path())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| session.file_dialog_dir.clone());
        let default_name = format!("{stem}.{ext}");
        let i18n = ui::build_i18n_map(session.locale);
        (
            initial_dir,
            default_name,
            format!(
                "{} ({})",
                i18n.get("export_btn")
                    .cloned()
                    .unwrap_or_else(|| "Export".to_string()),
                fmt.sidebar_label()
            ),
            i18n.get("export_btn")
                .cloned()
                .unwrap_or_else(|| "Export".to_string()),
            i18n.get("cancel")
                .cloned()
                .unwrap_or_else(|| "Cancel".to_string()),
            fmt.sidebar_label().to_string(),
            ext,
            i18n.get("dialog_all_files")
                .cloned()
                .unwrap_or_else(|| "All Files".to_string()),
        )
    };

    let chosen = pick_save_file_native(
        &app,
        dir,
        file_name,
        title,
        accept_lbl,
        cancel_lbl,
        filter_lbl,
        vec![ext_str],
        all_lbl,
    )
    .await;

    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    if let Some(path) = chosen {
        if let Some(parent) = path.parent() {
            if parent.is_dir() {
                session.apply(Action::SetFileDialogDir(parent.to_path_buf()));
            }
        }
        session.apply(Action::SetExportOutputPath(
            path.to_string_lossy().to_string(),
        ));
        if execute_immediately {
            session.apply(Action::ExecuteExport);
        }
    }
    Ok(build_session_snapshot(&session))
}

pub fn validate_external_http_url(url: &str) -> Result<&str, String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("Unsupported URL scheme".to_string());
    }
    if trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("Invalid characters in URL".to_string());
    }
    Ok(trimmed)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let safe_url = validate_external_http_url(&url)?;
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(safe_url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", safe_url])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(safe_url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn preview_palette_tokens(
    preset: ColorPalettePreset,
    mode: ThemeMode,
) -> PaletteCssTokens {
    AccentPalette::for_preset(preset, mode).to_css_tokens()
}

#[tauri::command]
fn render_mermaid_diagram(
    state: State<'_, Arc<TauriAppState>>,
    source: String,
    theme_override: Option<ThemeMode>,
) -> Result<MermaidSvgPayload, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    let diagram = if let Some(mode) = theme_override {
        let m_theme = match mode {
            ThemeMode::Light => MermaidTheme::Light,
            ThemeMode::Sepia => MermaidTheme::Sepia,
            ThemeMode::SemiDark => MermaidTheme::SemiDark,
            ThemeMode::Dark => MermaidTheme::Dark,
        };
        session
            .mermaid_cache
            .borrow_mut()
            .get_or_render_screen_themed(&source, m_theme)
    } else {
        session.render_mermaid(&source)
    };
    Ok(MermaidSvgPayload {
        content_hash: diagram.content_hash,
        svg: diagram.svg.clone(),
        width: diagram.width,
        height: diagram.height,
    })
}

pub fn render_session_preview_html_with_scope<F>(
    session: &Session,
    on_allowed_asset_file: F,
) -> String
where
    F: FnMut(&std::path::Path),
{
    let base_dir = session.current_file_path.as_deref().and_then(|p| p.parent());
    accent_export::render_markdown_to_webview_preview_html(
        session.buffer.as_str(),
        session.markdown_config,
        base_dir,
        |code| session.render_mermaid(code).svg.clone(),
        on_allowed_asset_file,
    )
}

pub fn render_session_preview_html(session: &Session) -> String {
    render_session_preview_html_with_scope(session, |_| {})
}

#[tauri::command]
fn render_preview_html(
    app: AppHandle,
    state: State<'_, Arc<TauriAppState>>,
) -> Result<String, String> {
    let session = state.session.lock().map_err(|e| e.to_string())?;
    let scope = app.asset_protocol_scope();
    Ok(render_session_preview_html_with_scope(&session, |canon_path| {
        let _ = scope.allow_file(canon_path);
    }))
}

#[tauri::command]
fn resolve_local_image(
    app: AppHandle,
    state: State<'_, Arc<TauriAppState>>,
    base_file_path: Option<String>,
    raw_url: String,
) -> Result<ResolvedLocalImagePayload, String> {
    let effective_file_path = match base_file_path {
        Some(ref s) if !s.trim().is_empty() => Some(PathBuf::from(s)),
        _ => {
            let session = state.session.lock().map_err(|e| e.to_string())?;
            session.current_file_path.clone()
        }
    };
    let base_dir = effective_file_path.as_deref().and_then(|p| p.parent());
    let Some(resolved) = accent_export::resolve_local_image_file(&raw_url, base_dir) else {
        return Ok(ResolvedLocalImagePayload {
            ok: false,
            kind: "missing".to_string(),
            src: String::new(),
            canonical_path: None,
        });
    };

    if resolved.ext_lower == "svg" {
        if let Some(data_uri) = accent_export::load_local_image_as_data_uri(&resolved) {
            return Ok(ResolvedLocalImagePayload {
                ok: true,
                kind: "data".to_string(),
                src: data_uri,
                canonical_path: None,
            });
        }
        return Ok(ResolvedLocalImagePayload {
            ok: false,
            kind: "missing".to_string(),
            src: String::new(),
            canonical_path: None,
        });
    }

    let _ = app.asset_protocol_scope().allow_file(&resolved.canonical_path);
    let asset_url = accent_export::convert_canonical_path_to_tauri_asset_url(&resolved.canonical_path);
    let canon_str = resolved.canonical_path.to_string_lossy().to_string();
    Ok(ResolvedLocalImagePayload {
        ok: true,
        kind: "asset".to_string(),
        src: asset_url,
        canonical_path: Some(canon_str),
    })
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

/// Generates a multi-resolution Windows `.ico` file (`256x256`, `64x64`, `32x32`, `16x16` PNG frames)
/// directly from the official Accent SVG logo.
pub fn generate_windows_ico() -> Option<Vec<u8>> {
    let scales = [
        (0u8, 0.5_f32),      // 256x256 (0 in ICONDIRENTRY means 256px)
        (64u8, 0.125_f32),   // 64x64
        (32u8, 0.0625_f32),  // 32x32
        (16u8, 0.03125_f32), // 16x16
    ];
    let mut png_frames = Vec::with_capacity(scales.len());
    for (dim_byte, scale) in scales {
        let (_w, _h, _rgba, png_bytes) = accent_mermaid::rasterize_svg(OFFICIAL_ICON_SVG, scale)?;
        png_frames.push((dim_byte, png_bytes));
    }

    let count = png_frames.len() as u16;
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&count.to_le_bytes());

    let mut data_offset: u32 = 6 + (16 * png_frames.len() as u32);
    for (dim_byte, png_bytes) in &png_frames {
        let size = png_bytes.len() as u32;
        ico.push(*dim_byte);
        ico.push(*dim_byte);
        ico.push(0);
        ico.push(0);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&size.to_le_bytes());
        ico.extend_from_slice(&data_offset.to_le_bytes());
        data_offset += size;
    }

    for (_dim_byte, png_bytes) in png_frames {
        ico.extend_from_slice(&png_bytes);
    }
    Some(ico)
}

pub fn run_app(opts: LaunchOptions) -> Result<(), String> {
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

    // If DISPLAY=:99 is requested or running headless, force X11 on :99 with software compositing
    let want_xvfb_99 = std::env::var("DISPLAY").as_deref() == Ok(":99")
        || (std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err());
    if want_xvfb_99 && std::path::Path::new("/tmp/.X11-unix/X99").exists() {
        unsafe {
            std::env::set_var("DISPLAY", ":99");
            std::env::remove_var("WAYLAND_DISPLAY");
            std::env::remove_var("XAUTHORITY");
            std::env::set_var("GDK_BACKEND", "x11");
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        }
    }

    let mut session = Session::default();
    if let Some(fp) = opts.file_path {
        session.apply(Action::OpenFile(fp));
    }
    let initial_watched_path = session.current_file_path.clone();
    let file_watcher = FocusedFileWatcher::new().ok();
    if let Some(ref watcher) = file_watcher {
        watcher.set_watched_file(initial_watched_path.clone());
    }

    let app_state = Arc::new(TauriAppState {
        session: Mutex::new(session),
        single_instance: Mutex::new(single_instance),
        file_watcher: Mutex::new(file_watcher),
        last_watched_path: Mutex::new(initial_watched_path),
    });

    let state_for_setup = Arc::clone(&app_state);
    let state_for_exit = Arc::clone(&app_state);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_session_snapshot,
            dispatch_action,
            update_editor_buffer,
            open_native_file_dialog,
            save_as_native_file_dialog,
            export_with_native_dialog,
            open_external_url,
            preview_palette_tokens,
            render_mermaid_diagram,
            render_preview_html,
            resolve_local_image,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let bg_state = Arc::clone(&state_for_setup);
            std::thread::Builder::new()
                .name("accent-wal-and-ipc-worker".to_string())
                .spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_millis(60));
                        let mut emit_snapshot: Option<UiSessionSnapshot> = None;
                        let mut should_focus_window = false;

                        // 1. Check Single-Instance IPC requests
                        let mut forwarded_files = Vec::new();
                        if let Ok(si_guard) = bg_state.single_instance.lock() {
                            if let Some(ref guard) = *si_guard {
                                let reqs = guard.drain_requests();
                                if !reqs.is_empty() {
                                    should_focus_window = true;
                                    for req in reqs {
                                        if let IpcRequest::OpenFile(path) = req {
                                            forwarded_files.push(path);
                                        }
                                    }
                                }
                            }
                        }

                        // 2. Check OS-native FocusedFileWatcher notification
                        let file_changed = bg_state
                            .file_watcher
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().map(|w| w.take_change_notification()))
                            .unwrap_or(false);

                        // 3. Poll < 300ms WAL journal flush and debounced settings flush
                        let mut updated_watched_path: Option<Option<PathBuf>> = None;
                        if let Ok(mut session) = bg_state.session.lock() {
                            let now = Instant::now();
                            let _ = session.poll_journal_flush(now);
                            let _ = session.poll_settings_flush(now);

                            if !forwarded_files.is_empty() {
                                for path in forwarded_files {
                                    session.apply(Action::OpenFile(path));
                                }
                                updated_watched_path = Some(session.current_file_path.clone());
                                emit_snapshot = Some(build_session_snapshot(&session));
                            } else if file_changed && session.disk_watcher_enabled {
                                let prev_banner = session.disk_modified_banner_active;
                                session.apply(Action::CheckExternalFileModification);
                                if session.disk_modified_banner_active != prev_banner
                                    || session.disk_modified_banner_active
                                {
                                    emit_snapshot = Some(build_session_snapshot(&session));
                                }
                            }
                        }

                        if let Some(new_path) = updated_watched_path {
                            bg_state.sync_file_watcher(new_path, None);
                        }

                        if should_focus_window {
                            if let Some(win) = handle.get_webview_window("main") {
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                            }
                        }

                        if let Some(snap) = emit_snapshot {
                            let _ = handle.emit("accent://session-updated", snap);
                        }
                    }
                })
                .ok();
            Ok(())
        })
        .on_window_event(move |_window, event| {
            if matches!(
                event,
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
            ) {
                if let Ok(mut session) = state_for_exit.session.lock() {
                    session.on_clean_exit();
                }
            }
        })
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
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
        assert_eq!(light.bg_canvas, ui::theme::Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(light.fg_primary, ui::theme::Color32::from_rgb(0x2e, 0x34, 0x36));

        let sepia = ui::theme::AccentPalette::for_mode(ThemeMode::Sepia);
        assert_eq!(sepia.bg_canvas, ui::theme::Color32::from_rgb(0xf9, 0xf3, 0xe9));
        assert_eq!(sepia.fg_primary, ui::theme::Color32::from_rgb(0x4f, 0x39, 0x15));

        let semi_dark = ui::theme::AccentPalette::for_mode(ThemeMode::SemiDark);
        assert_eq!(semi_dark.bg_canvas, ui::theme::Color32::from_rgb(0x2e, 0x31, 0x36));
        assert_eq!(semi_dark.fg_primary, ui::theme::Color32::from_rgb(0xe6, 0xe8, 0xeb));

        let dark = ui::theme::AccentPalette::for_mode(ThemeMode::Dark);
        assert_eq!(dark.bg_canvas, ui::theme::Color32::from_rgb(0x2b, 0x2b, 0x2f));
        assert_eq!(dark.fg_primary, ui::theme::Color32::from_rgb(0xee, 0xee, 0xee));
    }

    #[test]
    fn test_ergonomic_palette_presets() {
        let amber_light = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::AmberCoral,
            ThemeMode::Light,
        );
        assert_eq!(amber_light.fg_heading, ui::theme::Color32::from_rgb(0x9a, 0x47, 0x08));
        assert_eq!(amber_light.fg_subheading, ui::theme::Color32::from_rgb(0x25, 0x58, 0x9e));
        assert_eq!(amber_light.markup_emphasis_fg, ui::theme::Color32::from_rgb(0x8a, 0x62, 0x00));
        assert_eq!(amber_light.markup_code_fg, ui::theme::Color32::from_rgb(0xc8, 0x28, 0x50));
        assert_eq!(amber_light.markup_codeblock_fg, ui::theme::Color32::from_rgb(0x1e, 0x82, 0x4c));
        assert_eq!(amber_light.markup_quote_bar, ui::theme::Color32::from_rgb(0xc8, 0x5a, 0x17));

        let amber_sepia = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::AmberCoral,
            ThemeMode::Sepia,
        );
        assert_eq!(amber_sepia.fg_heading, ui::theme::Color32::from_rgb(0x8f, 0x3f, 0x06));
        assert_eq!(amber_sepia.fg_subheading, ui::theme::Color32::from_rgb(0x20, 0x4e, 0x8e));
        assert_eq!(amber_sepia.markup_emphasis_fg, ui::theme::Color32::from_rgb(0x80, 0x5a, 0x00));
        assert_eq!(amber_sepia.markup_code_fg, ui::theme::Color32::from_rgb(0xb8, 0x22, 0x48));
        assert_eq!(amber_sepia.markup_codeblock_fg, ui::theme::Color32::from_rgb(0x1e, 0x82, 0x4c));
        assert_eq!(amber_sepia.markup_quote_bar, ui::theme::Color32::from_rgb(0xc8, 0x5a, 0x17));

        let amber_semi = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::AmberCoral,
            ThemeMode::SemiDark,
        );
        assert_eq!(amber_semi.bg_canvas, ui::theme::Color32::from_rgb(0x28, 0x2b, 0x32));
        assert_eq!(amber_semi.fg_primary, ui::theme::Color32::from_rgb(0xed, 0xef, 0xf2));
        assert_eq!(amber_semi.fg_heading, ui::theme::Color32::from_rgb(0xf2, 0xc1, 0x2e));
        assert_eq!(amber_semi.fg_subheading, ui::theme::Color32::from_rgb(0x85, 0xaa, 0xfa));
        assert_eq!(amber_semi.markup_bold_fg, ui::theme::Color32::from_rgb(0xf2, 0xc1, 0x2e));
        assert_eq!(amber_semi.markup_emphasis_fg, ui::theme::Color32::from_rgb(0xe0, 0xbe, 0x4a));
        assert_eq!(amber_semi.markup_code_fg, ui::theme::Color32::from_rgb(0xff, 0x75, 0x93));
        assert_eq!(amber_semi.markup_codeblock_fg, ui::theme::Color32::from_rgb(0x6b, 0xcb, 0x77));
        assert_eq!(amber_semi.markup_quote_bar, ui::theme::Color32::from_rgb(0xe0, 0x7a, 0x38));

        let amber_dark = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::AmberCoral,
            ThemeMode::Dark,
        );
        assert_eq!(amber_dark.bg_canvas, ui::theme::Color32::from_rgb(0x18, 0x1a, 0x1d));
        assert_eq!(amber_dark.bg_headerbar, ui::theme::Color32::from_rgb(0x12, 0x14, 0x17));
        assert_eq!(amber_dark.bg_code_block, ui::theme::Color32::from_rgb(0x21, 0x24, 0x2b));
        assert_eq!(amber_dark.fg_primary, ui::theme::Color32::from_rgb(0xe6, 0xe8, 0xeb));
        assert_eq!(amber_dark.fg_heading, ui::theme::Color32::from_rgb(0xea, 0xb7, 0x00));
        assert_eq!(amber_dark.fg_subheading, ui::theme::Color32::from_rgb(0x7a, 0xa2, 0xf7));
        assert_eq!(amber_dark.markup_bold_fg, ui::theme::Color32::from_rgb(0xea, 0xb7, 0x00));
        assert_eq!(amber_dark.markup_emphasis_fg, ui::theme::Color32::from_rgb(0xd9, 0xb6, 0x48));
        assert_eq!(amber_dark.markup_quote_bar, ui::theme::Color32::from_rgb(0xe0, 0x7a, 0x38));
        assert_eq!(amber_dark.markup_quote_fg, ui::theme::Color32::from_rgb(0xd8, 0xd2, 0xc8));
        assert_eq!(amber_dark.markup_code_fg, ui::theme::Color32::from_rgb(0xff, 0x6b, 0x8b));
        assert_eq!(amber_dark.markup_codeblock_fg, ui::theme::Color32::from_rgb(0x6b, 0xcb, 0x77));
        assert_eq!(amber_dark.bg_code_inline, ui::theme::Color32::from_rgb(0x28, 0x2c, 0x34));
        assert_eq!(amber_dark.accent_link, ui::theme::Color32::from_rgb(0x7a, 0xa2, 0xf7));

        let ed_light = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::Light,
        );
        assert_eq!(ed_light.bg_canvas, ui::theme::Color32::from_rgb(0xfa, 0xf9, 0xf5));
        assert_eq!(ed_light.accent_orange, ui::theme::Color32::from_rgb(0x2b, 0x6c, 0xb0));

        let ed_sepia = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::Sepia,
        );
        assert_eq!(ed_sepia.bg_canvas, ui::theme::Color32::from_rgb(0xf4, 0xef, 0xe6));
        assert_eq!(ed_sepia.accent_orange, ui::theme::Color32::from_rgb(0x2c, 0x6e, 0x63));

        let ed_semi = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::SemiDark,
        );
        assert_eq!(ed_semi.bg_canvas, ui::theme::Color32::from_rgb(0x2c, 0x30, 0x38));

        let ed_dark = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::EditorialSlate,
            ThemeMode::Dark,
        );
        assert_eq!(ed_dark.bg_canvas, ui::theme::Color32::from_rgb(0x19, 0x1c, 0x22));

        let nordic_sepia = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::NordicForest,
            ThemeMode::Sepia,
        );
        assert_eq!(nordic_sepia.bg_canvas, ui::theme::Color32::from_rgb(0xf3, 0xea, 0xd3));

        let nordic_semi = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::NordicForest,
            ThemeMode::SemiDark,
        );
        assert_eq!(nordic_semi.bg_canvas, ui::theme::Color32::from_rgb(0x2e, 0x34, 0x40));

        let velvet_semi = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::VelvetDawn,
            ThemeMode::SemiDark,
        );
        assert_eq!(velvet_semi.bg_canvas, ui::theme::Color32::from_rgb(0x2f, 0x2c, 0x3c));

        let velvet_dark = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::VelvetDawn,
            ThemeMode::Dark,
        );
        assert_eq!(velvet_dark.bg_canvas, ui::theme::Color32::from_rgb(0x1f, 0x1d, 0x29));

        let classic_semi = ui::theme::AccentPalette::for_preset(
            ColorPalettePreset::Classic,
            ThemeMode::SemiDark,
        );
        assert_eq!(classic_semi.bg_canvas, ui::theme::Color32::from_rgb(0x2e, 0x31, 0x36));
    }

    #[test]
    fn test_build_session_snapshot_completeness() {
        let test_dir = std::env::temp_dir().join(format!(
            "accent_ui_snap_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&test_dir);
        let mut session = Session::with_recovery_dir(test_dir.clone());
        session.apply(Action::SetPalettePreset(ColorPalettePreset::EditorialSlate));
        session.apply(Action::SetTheme(ThemeMode::SemiDark));
        let snap = build_session_snapshot(&session);
        assert_eq!(snap.theme, ThemeMode::SemiDark);
        assert!(!snap.tabs.is_empty());
        assert!(!snap.outline_headings.is_empty());
        assert_eq!(snap.palette_css.bg_canvas, "#2c3038");
        assert_eq!(snap.i18n.get("open").map(String::as_str), Some("Apri"));

        // Verify unified comrak Preview HTML (Mermaid SVG gating, §6.9 autolinks, §6.11 tagfilter, data-code-lang)
        session.apply(Action::UpdateBufferText(
            "# Titolo\n\nVisita https://accent-editor.org e <kbd>Ctrl+P</kbd>\n\n<script>evil()</script>\n\n```json\n{\"a\": 1}\n```\n\n```mermaid\nflowchart LR\n  A[Start] --> B[End]\n```\n".to_string(),
        ));
        let with_svg = render_session_preview_html(&session);
        assert!(with_svg.contains("data-code-lang=\"json\""));
        assert!(with_svg.contains("<a href=\"https://accent-editor.org\">https://accent-editor.org</a>"));
        assert!(with_svg.contains("<kbd>Ctrl+P</kbd>"));
        assert!(!with_svg.contains("<script>") && with_svg.contains("&lt;script>"));
        assert!(with_svg.contains("class=\"accent-mermaid-block\""));
        assert!(with_svg.contains("<svg"));
        assert!(with_svg.contains("#282B32"));

        // Disable Mermaid toggle -> mermaid block remains plain code block
        let mut no_mermaid_cfg = session.markdown_config;
        no_mermaid_cfg.extensions.mermaid = false;
        session.apply(Action::SetMarkdownConfig(no_mermaid_cfg));
        let without_svg = render_session_preview_html(&session);
        assert!(!without_svg.contains("accent-mermaid-block"));
        assert!(without_svg.contains("data-code-lang=\"mermaid\""));

        let _ = std::fs::remove_dir_all(&test_dir);
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
        assert_eq!(VERSION, "0.8.30");
        assert_eq!(accent_core::VERSION, VERSION);
        let version_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../VERSION");
        let file_ver = std::fs::read_to_string(version_file)
            .expect("Failed to read root VERSION file");
        assert_eq!(file_ver.trim(), VERSION);
    }

    #[test]
    fn test_external_url_validation_and_i18n_completeness() {
        assert_eq!(
            validate_external_http_url("https://doc.rust-lang.org"),
            Ok("https://doc.rust-lang.org")
        );
        assert_eq!(
            validate_external_http_url("http://example.com/path?a=1&b=2"),
            Ok("http://example.com/path?a=1&b=2")
        );
        assert!(validate_external_http_url("javascript:alert(1)").is_err());
        assert!(validate_external_http_url("file:///etc/passwd").is_err());
        assert!(validate_external_http_url("https://bad url/space").is_err());

        for loc in [
            accent_session::Locale::It,
            accent_session::Locale::En,
            accent_session::Locale::Es,
        ] {
            let map = ui::build_i18n_map(loc);
            for required_key in [
                "brightness_mode",
                "brightness_sub",
                "export_fmt_prefix",
                "browse",
                "wal_title",
                "watcher_title",
                "enabled",
                "disabled",
                "unsaved_tab_title",
                "unsaved_tab_msg",
                "discard_changes",
                "html_copied",
                "sc_new_tab",
                "sc_live_preview",
                "sc_split_preview",
                "dialog_accept_open",
                "dialog_accept_save",
                "base_gfm",
                "base_commonmark",
                "extensions_title",
                "extensions_sub",
                "extensions_disabled_cm",
                "ext_math",
                "ext_mermaid",
                "ext_alerts",
                "ext_footnotes",
                "ext_frontmatter",
            ] {
                let v = map.get(required_key).map(String::as_str).unwrap_or("");
                assert!(
                    !v.is_empty(),
                    "Missing or empty i18n key {required_key} for locale {loc:?}"
                );
            }
        }
    }
}
