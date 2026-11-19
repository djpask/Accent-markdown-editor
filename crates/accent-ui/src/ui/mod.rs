pub mod theme;

use accent_core::{AstSpanKind, MarkdownDialect};
use accent_session::{
    ActiveOverlay, ColorPalettePreset, ExportFormat, I18n, LineWidthMode, Locale, PreviewLayout,
    Session, StatMetric, ThemeMode, TypographyConfig, format_thousands,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use theme::{AccentPalette, PaletteCssTokens};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub index: usize,
    pub tab_id: u64,
    pub document_title: String,
    pub current_file_path: Option<String>,
    pub is_dirty: bool,
    pub recovered_from_crash: bool,
    pub recovery_banner_active: bool,
    pub disk_modified_banner_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSnapshot {
    pub characters: String,
    pub words: String,
    pub sentences: String,
    pub paragraphs: String,
    pub reading_time: String,
    pub raw_words: usize,
    pub raw_characters: usize,
    pub has_selection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineHeadingSnapshot {
    pub level: u8,
    pub text: String,
    pub line: usize,
    pub char_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCircleColors {
    pub light: String,
    pub sepia: String,
    pub semi_dark: String,
    pub dark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MermaidSvgPayload {
    pub content_hash: u64,
    pub svg: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSessionSnapshot {
    pub tabs: Vec<TabSnapshot>,
    pub active_tab_index: usize,
    pub active_tab_id: u64,
    pub document_title: String,
    pub current_file_path: Option<String>,
    pub is_dirty: bool,
    pub buffer_text: String,
    pub buffer_revision: u64,
    pub cursor_char: usize,
    pub selection_start_char: usize,
    pub selection_end_char: usize,
    pub theme: ThemeMode,
    pub palette_preset: ColorPalettePreset,
    pub palette_css: PaletteCssTokens,
    pub theme_circle_colors: ThemeCircleColors,
    pub locale: Locale,
    pub dialect: MarkdownDialect,
    pub line_width: LineWidthMode,
    pub line_width_badge: String,
    pub fluid_margin_pct: u8,
    pub typography: TypographyConfig,
    pub zoom_percentage: u32,
    pub preview_active: bool,
    pub preview_layout: PreviewLayout,
    pub view_while_typing: bool,
    pub fullscreen: bool,
    pub toolbar_expanded: bool,
    pub search_bar_open: bool,
    pub chrome_autohidden: bool,
    pub search_query: String,
    pub replace_text: String,
    pub stat_metric: StatMetric,
    pub active_stat_label: String,
    pub stats: StatsSnapshot,
    pub overlay: ActiveOverlay,
    pub recovery_banner_active: bool,
    pub disk_modified_banner_active: bool,
    pub status_message: Option<String>,
    pub recent_files: Vec<String>,
    pub selected_export_format: ExportFormat,
    pub export_output_path: String,
    pub export_syntax_highlight: bool,
    pub font_families: Vec<String>,
    pub prose_font_presets: Vec<String>,
    pub code_font_presets: Vec<String>,
    pub outline_headings: Vec<OutlineHeadingSnapshot>,
    pub i18n: BTreeMap<String, String>,
}

pub fn build_i18n_map(locale: Locale) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let keys = [
        "open",
        "open_file",
        "recent_docs",
        "save",
        "save_as",
        "advanced_export",
        "copy_html",
        "fullscreen_mode",
        "view_while_typing",
        "find_replace",
        "preferences",
        "open_tutorial",
        "keyboard_shortcuts",
        "about",
        "language",
        "line_width",
        "line_width_sub",
        "compact",
        "medium",
        "wide",
        "fluid",
        "fluid_margin",
        "fluid_margin_sub",
        "prose_font",
        "prose_font_sub",
        "code_font",
        "code_font_sub",
        "font_size",
        "line_height",
        "size_lh_sub",
        "input_format",
        "input_format_sub",
        "color_palette",
        "color_palette_sub",
        "palette_editorial",
        "palette_nordic",
        "palette_velvet",
        "palette_classic",
        "editor_modes_hdr",
        "preview_panel_hdr",
        "editor_only_raw",
        "no_preview",
        "half_width",
        "full_width",
        "half_height",
        "windowed",
        "back_to_editor",
        "new_doc",
        "cancel",
        "find_placeholder",
        "replace_placeholder",
        "replace_btn",
        "characters",
        "words",
        "sentences",
        "paragraphs",
        "reading_time",
        "tooltip_new_doc",
        "tooltip_open",
        "tooltip_save",
        "tooltip_save_menu",
        "tooltip_line_width",
        "tooltip_search",
        "tooltip_preview",
        "tooltip_main_menu",
        "tooltip_new_tab",
        "tooltip_close_tab",
        "tooltip_minimize",
        "tooltip_maximize",
        "tooltip_close",
        "tooltip_bold",
        "tooltip_italic",
        "tooltip_strikethrough",
        "tooltip_heading",
        "tooltip_bullet_list",
        "tooltip_ordered_list",
        "tooltip_task_list",
        "tooltip_blockquote",
        "tooltip_code",
        "tooltip_link",
        "tooltip_image",
        "tooltip_table",
        "tooltip_toggle_toolbar",
        "tooltip_stats",
        "recovery_banner_msg",
        "recovery_confirm_save",
        "recovery_discard",
        "disk_mod_banner_msg",
        "disk_mod_banner_dirty_msg",
        "disk_mod_reload",
        "disk_mod_ignore",
        "export_formats_hdr",
        "export_dest_path",
        "export_page_size",
        "export_syntax_hl",
        "export_syntax_hl_sub",
        "export_btn",
    ];
    for k in keys {
        map.insert(k.to_string(), I18n::tr(locale, k).to_string());
    }

    // Additional UI labels for Outline, Themes & Palettes modal, and Tabbed Preferences
    let extras: &[(&str, &str, &str, &str)] = &[
        ("outline", "Indice", "Outline", "Índice"),
        (
            "tooltip_outline",
            "Mostra o nascondi indice del documento",
            "Show or hide document outline",
            "Mostrar u ocultar índice del documento",
        ),
        (
            "themes_palettes",
            "Temi e palette",
            "Themes & Palettes",
            "Temas y paletas",
        ),
        ("theme_light", "Chiaro", "Light", "Claro"),
        ("theme_sepia", "Sepia", "Sepia", "Sepia"),
        ("theme_semidark", "Semi-Dark", "Semi-Dark", "Semi-Oscuro"),
        ("theme_dark", "Scuro", "Dark", "Oscuro"),
        (
            "tab_typography",
            "Tipografia e Pagina",
            "Typography & Page",
            "Tipografía y Página",
        ),
        (
            "tab_editor_lang",
            "Editor e Lingua",
            "Editor & Language",
            "Editor e Idioma",
        ),
        (
            "restore_defaults",
            "Ripristina predefiniti",
            "Restore Defaults",
            "Restaurar predeterminados",
        ),
        (
            "live_preview",
            "Anteprima combinata dal vivo",
            "Live Combined Preview",
            "Vista previa combinada en vivo",
        ),
        (
            "view_edit_raw",
            "Modifica (Sorgente)",
            "Edit (Source)",
            "Editar (Fuente)",
        ),
        (
            "view_inline_live",
            "Anteprima in-line •",
            "In-line Live Preview •",
            "Vista previa en línea •",
        ),
        (
            "view_split_right",
            "Anteprima split a destra",
            "Split Preview Right",
            "Vista dividida a la derecha",
        ),
        (
            "view_split_bottom",
            "Anteprima split sotto",
            "Split Preview Bottom",
            "Vista dividida abajo",
        ),
        (
            "view_readonly",
            "Solo lettura",
            "Read Only",
            "Solo lectura",
        ),
        (
            "status_saved",
            "Salvato",
            "Saved",
            "Guardado",
        ),
        (
            "status_unsaved",
            "Modifiche locali (WAL attivo)",
            "Unsaved edits (WAL active)",
            "Cambios locales (WAL activo)",
        ),
        (
            "outline_empty",
            "Nessuna intestazione (#)",
            "No headings (#)",
            "Sin encabezados (#)",
        ),
    ];
    for &(k, it, en, es) in extras {
        let val = match locale {
            Locale::It => it,
            Locale::En => en,
            Locale::Es => es,
        };
        map.insert(k.to_string(), val.to_string());
    }
    map
}

pub fn extract_outline_headings(session: &Session) -> Vec<OutlineHeadingSnapshot> {
    let text = session.buffer.as_str();
    let spans = session
        .buffer
        .compute_live_spans(session.dialect, false, 0..0);
    let mut headings = Vec::new();
    for ls in spans.iter() {
        if let AstSpanKind::Heading { level } = ls.span.kind {
            let start_b = ls.span.byte_range.start.min(text.len());
            let end_b = ls.span.byte_range.end.min(text.len());
            if start_b < end_b {
                let raw_line = text[start_b..end_b].lines().next().unwrap_or("").trim();
                let cleaned = raw_line
                    .trim_start_matches('#')
                    .trim_end_matches('#')
                    .trim();
                if !cleaned.is_empty() {
                    headings.push(OutlineHeadingSnapshot {
                        level,
                        text: cleaned.to_string(),
                        line: ls.span.start_line,
                        char_offset: ls.span.char_range.start,
                    });
                }
            }
        }
    }
    headings
}

pub fn build_session_snapshot(session: &Session) -> UiSessionSnapshot {
    let active_pal = AccentPalette::for_preset(session.palette_preset, session.theme);
    let palette_css = active_pal.to_css_tokens();
    let theme_circle_colors = ThemeCircleColors {
        light: AccentPalette::for_preset(session.palette_preset, ThemeMode::Light)
            .bg_canvas
            .to_hex(),
        sepia: AccentPalette::for_preset(session.palette_preset, ThemeMode::Sepia)
            .bg_canvas
            .to_hex(),
        semi_dark: AccentPalette::for_preset(session.palette_preset, ThemeMode::SemiDark)
            .bg_canvas
            .to_hex(),
        dark: AccentPalette::for_preset(session.palette_preset, ThemeMode::Dark)
            .bg_canvas
            .to_hex(),
    };

    let tabs: Vec<TabSnapshot> = session
        .tabs
        .iter()
        .enumerate()
        .map(|(idx, t)| {
            let is_active = idx == session.active_tab_index;
            TabSnapshot {
                index: idx,
                tab_id: t.tab_id,
                document_title: if is_active {
                    session.document_title.clone()
                } else {
                    t.document_title.clone()
                },
                current_file_path: if is_active {
                    session
                        .current_file_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    t.current_file_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                },
                is_dirty: if is_active {
                    session.is_dirty
                } else {
                    t.is_dirty
                },
                recovered_from_crash: if is_active {
                    session.recovered_from_crash
                } else {
                    t.recovered_from_crash
                },
                recovery_banner_active: if is_active {
                    session.recovery_banner_active
                } else {
                    t.recovery_banner_active
                },
                disk_modified_banner_active: if is_active {
                    session.disk_modified_banner_active
                } else {
                    t.disk_modified_banner_active
                },
            }
        })
        .collect();

    let active_tab_id = session
        .tabs
        .get(session.active_tab_index)
        .map_or(1, |t| t.tab_id);
    let sel_chars = session.buffer.selection_char_range();
    let has_selection = session.buffer.selection_stats().is_some();
    let doc_stats = session.active_stats();

    let stats = StatsSnapshot {
        characters: format!(
            "{} {}",
            format_thousands(doc_stats.characters, session.locale),
            I18n::tr(session.locale, "characters")
        ),
        words: format!(
            "{} {}",
            format_thousands(doc_stats.words, session.locale),
            I18n::tr(session.locale, "words")
        ),
        sentences: format!(
            "{} {}",
            format_thousands(doc_stats.sentences, session.locale),
            I18n::tr(session.locale, "sentences")
        ),
        paragraphs: format!(
            "{} {}",
            format_thousands(doc_stats.paragraphs, session.locale),
            I18n::tr(session.locale, "paragraphs")
        ),
        reading_time: format!(
            "{} {}",
            doc_stats.format_reading_time(),
            I18n::tr(session.locale, "reading_time")
        ),
        raw_words: doc_stats.words,
        raw_characters: doc_stats.characters,
        has_selection,
    };

    let fluid_margin_pct = match session.line_width {
        LineWidthMode::Fluid { margin_pct } => margin_pct,
        _ => 6,
    };

    UiSessionSnapshot {
        tabs,
        active_tab_index: session.active_tab_index,
        active_tab_id,
        document_title: session.document_title.clone(),
        current_file_path: session
            .current_file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        is_dirty: session.is_dirty,
        buffer_text: session.buffer.text(),
        buffer_revision: session.buffer.revision(),
        cursor_char: session.buffer.cursor(),
        selection_start_char: sel_chars.start,
        selection_end_char: sel_chars.end,
        theme: session.theme,
        palette_preset: session.palette_preset,
        palette_css,
        theme_circle_colors,
        locale: session.locale,
        dialect: session.dialect,
        line_width: session.line_width,
        line_width_badge: session.line_width.short_badge().to_string(),
        fluid_margin_pct,
        typography: session.typography.clone(),
        zoom_percentage: session.zoom_percentage(),
        preview_active: session.preview_active,
        preview_layout: session.preview_layout,
        view_while_typing: session.view_while_typing,
        fullscreen: session.fullscreen,
        toolbar_expanded: session.toolbar_expanded,
        search_bar_open: session.search_bar_open,
        chrome_autohidden: session.chrome_autohidden,
        search_query: session.search_query.clone(),
        replace_text: session.replace_text.clone(),
        stat_metric: session.stat_metric,
        active_stat_label: session.format_active_stat(),
        stats,
        overlay: session.overlay,
        recovery_banner_active: session.recovery_banner_active,
        disk_modified_banner_active: session.disk_modified_banner_active,
        status_message: session.status_message.clone(),
        recent_files: session
            .recent_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        selected_export_format: session.selected_export_format,
        export_output_path: session.export_output_path.clone(),
        export_syntax_highlight: session.export_syntax_highlight,
        font_families: session.font_catalog.families.clone(),
        prose_font_presets: session.font_catalog.prose_presets.clone(),
        code_font_presets: session.font_catalog.code_presets.clone(),
        outline_headings: extract_outline_headings(session),
        i18n: build_i18n_map(session.locale),
    }
}
