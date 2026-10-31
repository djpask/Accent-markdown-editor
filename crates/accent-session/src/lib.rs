pub use accent_core::MarkdownDialect;
use accent_core::{DocumentBuffer, DocumentStats, DEFAULT_SAMPLE_MARKDOWN, TUTORIAL_MARKDOWN};
pub use accent_export::ExportFormat;
pub use accent_mermaid::{MermaidCache, MermaidTheme, RenderedMermaidDiagram};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Sepia,
    Dark,
}

impl ThemeMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "light" => Some(Self::Light),
            "sepia" => Some(Self::Sepia),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorPalettePreset {
    /// Default: Editorial Paper & Slate (Alabaster Paper, Warm Book Paper, Midnight Slate)
    #[default]
    EditorialSlate,
    /// Nordic Zen & Forest (Snow Mist, Everforest Warm, Nordic Night)
    NordicForest,
    /// Rosé Pine & Solarized Modern (Dawn Cream, Warm Sand, Velvet Dusk)
    VelvetDawn,
    /// Classic high-contrast palette
    Classic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    En,
    It,
    Es,
}

impl Locale {
    pub fn from_code(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Some(Self::En),
            "it" | "italian" | "italiano" => Some(Self::It),
            "es" | "spanish" | "espanol" | "español" => Some(Self::Es),
            _ => None,
        }
    }
}

pub struct I18n;

impl I18n {
    pub fn tr(locale: Locale, key: &str) -> &'static str {
        match (locale, key) {
            // Headerbar
            (Locale::It, "open") => "Apri",
            (Locale::En, "open") => "Open",
            (Locale::Es, "open") => "Abrir",

            (Locale::It, "open_file") => "Apri documento...",
            (Locale::En, "open_file") => "Open Document...",
            (Locale::Es, "open_file") => "Abrir documento...",

            (Locale::It, "recent_docs") => "Documenti recenti",
            (Locale::En, "recent_docs") => "Recent Documents",
            (Locale::Es, "recent_docs") => "Documentos recientes",

            (Locale::It, "save") => "Salva",
            (Locale::En, "save") => "Save",
            (Locale::Es, "save") => "Guardar",

            (Locale::It, "save_as") => "Salva come...",
            (Locale::En, "save_as") => "Save As...",
            (Locale::Es, "save_as") => "Guardar como...",

            (Locale::It, "advanced_export") => "Esportazione avanzata...",
            (Locale::En, "advanced_export") => "Advanced Export...",
            (Locale::Es, "advanced_export") => "Exportación avanzada...",

            (Locale::It, "copy_html") => "Copia HTML",
            (Locale::En, "copy_html") => "Copy HTML",
            (Locale::Es, "copy_html") => "Copiar HTML",

            // Main Menu Popover
            (Locale::It, "hemingway_mode") => "Modalità Hemingway",
            (Locale::En, "hemingway_mode") => "Hemingway Mode",
            (Locale::Es, "hemingway_mode") => "Modo Hemingway",

            (Locale::It, "focus_mode") => "Modalità concentrazione",
            (Locale::En, "focus_mode") => "Focus Mode",
            (Locale::Es, "focus_mode") => "Modo concentración",

            (Locale::It, "view_while_typing") => "Anteprima durante la digitazione",
            (Locale::En, "view_while_typing") => "View-while-typing (Live)",
            (Locale::Es, "view_while_typing") => "Vista en vivo al escribir",

            (Locale::It, "find_replace") => "Trova e sostituisci",
            (Locale::En, "find_replace") => "Find and Replace",
            (Locale::Es, "find_replace") => "Buscar y reemplazar",

            (Locale::It, "preferences") => "Preferenze",
            (Locale::En, "preferences") => "Preferences",
            (Locale::Es, "preferences") => "Preferencias",

            (Locale::It, "open_tutorial") => "Apri tutorial",
            (Locale::En, "open_tutorial") => "Open Tutorial",
            (Locale::Es, "open_tutorial") => "Abrir tutorial",

            (Locale::It, "keyboard_shortcuts") => "Scorciatoie da tastiera",
            (Locale::En, "keyboard_shortcuts") => "Keyboard Shortcuts",
            (Locale::Es, "keyboard_shortcuts") => "Atajos de teclado",

            (Locale::It, "about") => "Informazioni su Accent",
            (Locale::En, "about") => "About Accent",
            (Locale::Es, "about") => "Acerca de Accent",

            (Locale::It, "language") => "Lingua",
            (Locale::En, "language") => "Language",
            (Locale::Es, "language") => "Idioma",

            // Line Width & Preferences
            (Locale::It, "line_width") => "Larghezza riga",
            (Locale::En, "line_width") => "Line Width",
            (Locale::Es, "line_width") => "Ancho de línea",

            (Locale::It, "line_width_sub") => "80 car. · 110 car. · 140 car. · Fluida",
            (Locale::En, "line_width_sub") => "80ch · 110ch · 140ch · Fluid",
            (Locale::Es, "line_width_sub") => "80 car. · 110 car. · 140 car. · Fluido",

            (Locale::It, "compact") => "Compatta (~80 car.)",
            (Locale::En, "compact") => "Compact (~80 ch)",
            (Locale::Es, "compact") => "Compacto (~80 car.)",

            (Locale::It, "medium") => "Media (110 car.)",
            (Locale::En, "medium") => "Medium (110 ch)",
            (Locale::Es, "medium") => "Medio (110 car.)",

            (Locale::It, "wide") => "Ampia (140+ car.)",
            (Locale::En, "wide") => "Wide (140+ ch)",
            (Locale::Es, "wide") => "Amplio (140+ car.)",

            (Locale::It, "fluid") => "Fluida (Tutta larghezza)",
            (Locale::En, "fluid") => "Fluid (Full-Width)",
            (Locale::Es, "fluid") => "Fluido (Ancho completo)",

            (Locale::It, "fluid_margin") => "Margine laterale fluido",
            (Locale::En, "fluid_margin") => "Fluid Lateral Margin",
            (Locale::Es, "fluid_margin") => "Margen lateral fluido",

            (Locale::It, "fluid_margin_sub") => "Margine dinamico su ciascun lato della finestra",
            (Locale::En, "fluid_margin_sub") => "Dynamic lateral margin on each side of the window",
            (Locale::Es, "fluid_margin_sub") => "Margen dinámico a cada lado de la ventana",

            (Locale::It, "prose_font") => "Carattere testo (Prosa)",
            (Locale::En, "prose_font") => "Prose Font (Body)",
            (Locale::Es, "prose_font") => "Fuente de texto (Prosa)",

            (Locale::It, "prose_font_sub") => "Famiglie di caratteri di sistema rilevate da fontdb",
            (Locale::En, "prose_font_sub") => "System font families discovered via fontdb",
            (Locale::Es, "prose_font_sub") => "Familias de fuentes del sistema vía fontdb",

            (Locale::It, "code_font") => "Carattere codice (Monospace)",
            (Locale::En, "code_font") => "Code Font (Monospace)",
            (Locale::Es, "code_font") => "Fuente de código (Monospace)",

            (Locale::It, "code_font_sub") => "Carattere indipendente per codice inline e blocchi",
            (Locale::En, "code_font_sub") => "Independent inline & fenced code block font",
            (Locale::Es, "code_font_sub") => "Fuente independiente para código en línea y bloques",

            (Locale::It, "font_size") => "Dimensione carattere",
            (Locale::En, "font_size") => "Font Size",
            (Locale::Es, "font_size") => "Tamaño de fuente",

            (Locale::It, "line_height") => "Interlinea",
            (Locale::En, "line_height") => "Line Height",
            (Locale::Es, "line_height") => "Altura de línea",

            (Locale::It, "size_lh_sub") => "Controllo granulare dimensione (pt) e moltiplicatore interlinea",
            (Locale::En, "size_lh_sub") => "Granular TextFormat size (pt) and line-height multiplier",
            (Locale::Es, "size_lh_sub") => "Control granular de tamaño (pt) e interlineado",

            (Locale::It, "input_format") => "Formato di input",
            (Locale::En, "input_format") => "Input Format",
            (Locale::Es, "input_format") => "Formato de entrada",

            (Locale::It, "input_format_sub") => "Dialetto Markdown analizzato dall'AST nativo",
            (Locale::En, "input_format_sub") => "Markdown dialect parsed by the native AST",
            (Locale::Es, "input_format_sub") => "Dialecto Markdown analizado por el AST nativo",

            (Locale::It, "color_palette") => "Palette cromatica",
            (Locale::En, "color_palette") => "Color Palette",
            (Locale::Es, "color_palette") => "Paleta de colores",

            (Locale::It, "color_palette_sub") => "Tonalità ergonomiche per concentrazione e riposo visivo",
            (Locale::En, "color_palette_sub") => "Ergonomic tones engineered for focus and eye comfort",
            (Locale::Es, "color_palette_sub") => "Tonos ergonómicos para concentración y descanso visual",

            (Locale::It, "palette_editorial") => "Editorial",
            (Locale::En, "palette_editorial") => "Editorial",
            (Locale::Es, "palette_editorial") => "Editorial",

            (Locale::It, "palette_nordic") => "Nordic",
            (Locale::En, "palette_nordic") => "Nordic",
            (Locale::Es, "palette_nordic") => "Nordic",

            (Locale::It, "palette_velvet") => "Rosé Pine",
            (Locale::En, "palette_velvet") => "Rosé Pine",
            (Locale::Es, "palette_velvet") => "Rosé Pine",

            (Locale::It, "palette_classic") => "Classica",
            (Locale::En, "palette_classic") => "Classic",
            (Locale::Es, "palette_classic") => "Clásica",

            // Preview / Mode Popover (mutually exclusive & unambiguous)
            (Locale::It, "editor_modes_hdr") => "Modalità editor",
            (Locale::En, "editor_modes_hdr") => "Editor Modes",
            (Locale::Es, "editor_modes_hdr") => "Modos de editor",

            (Locale::It, "preview_panel_hdr") => "Pannello di anteprima",
            (Locale::En, "preview_panel_hdr") => "Preview Panel",
            (Locale::Es, "preview_panel_hdr") => "Panel de vista previa",

            (Locale::It, "editor_only_raw") => "Solo editor (Markdown sorgente)",
            (Locale::En, "editor_only_raw") => "Editor Only (Source Markdown)",
            (Locale::Es, "editor_only_raw") => "Solo editor (Markdown fuente)",

            (Locale::It, "no_preview") => "Solo editor (Markdown sorgente)",
            (Locale::En, "no_preview") => "Editor Only (Source Markdown)",
            (Locale::Es, "no_preview") => "Solo editor (Markdown fuente)",

            (Locale::It, "half_width") => "A metà larghezza",
            (Locale::En, "half_width") => "Half-Width",
            (Locale::Es, "half_width") => "Mitad de ancho",

            (Locale::It, "full_width") => "A tutta pagina (Solo lettura)",
            (Locale::En, "full_width") => "Full-Width (Reader Mode)",
            (Locale::Es, "full_width") => "Página completa (Solo lectura)",

            (Locale::It, "half_height") => "A metà altezza",
            (Locale::En, "half_height") => "Half-Height",
            (Locale::Es, "half_height") => "Mitad de alto",

            (Locale::It, "windowed") => "In una finestra separata",
            (Locale::En, "windowed") => "Windowed",
            (Locale::Es, "windowed") => "En ventana separada",

            (Locale::It, "back_to_editor") => "✎ Torna all'editor",
            (Locale::En, "back_to_editor") => "✎ Back to Editor",
            (Locale::Es, "back_to_editor") => "✎ Volver al editor",

            (Locale::It, "new_doc") => "Nuovo documento",
            (Locale::En, "new_doc") => "New Document",
            (Locale::Es, "new_doc") => "Nuevo documento",

            (Locale::It, "cancel") => "Annulla",
            (Locale::En, "cancel") => "Cancel",
            (Locale::Es, "cancel") => "Cancelar",

            // Find / Replace bar
            (Locale::It, "find_placeholder") => "Trova nel documento...",
            (Locale::En, "find_placeholder") => "Find in document...",
            (Locale::Es, "find_placeholder") => "Buscar en el documento...",

            (Locale::It, "replace_placeholder") => "Sostituisci con...",
            (Locale::En, "replace_placeholder") => "Replace with...",
            (Locale::Es, "replace_placeholder") => "Reemplazar con...",

            (Locale::It, "replace_btn") => "Sostituisci",
            (Locale::En, "replace_btn") => "Replace",
            (Locale::Es, "replace_btn") => "Reemplazar",

            // Statistics units
            (Locale::It, "characters") => "caratteri",
            (Locale::En, "characters") => "characters",
            (Locale::Es, "characters") => "caracteres",

            (Locale::It, "words") => "parole",
            (Locale::En, "words") => "words",
            (Locale::Es, "words") => "palabras",

            (Locale::It, "sentences") => "frasi",
            (Locale::En, "sentences") => "sentences",
            (Locale::Es, "sentences") => "oraciones",

            (Locale::It, "paragraphs") => "paragrafi",
            (Locale::En, "paragraphs") => "paragraphs",
            (Locale::Es, "paragraphs") => "párrafos",

            (Locale::It, "reading_time") => "di tempo di lettura",
            (Locale::En, "reading_time") => "reading time",
            (Locale::Es, "reading_time") => "de tiempo de lectura",

            // Hover Tooltips (Headerbar & Bottom Formatting Bar)
            (Locale::It, "tooltip_new_doc") => "Nuovo documento (Ctrl+N)",
            (Locale::En, "tooltip_new_doc") => "New Document (Ctrl+N)",
            (Locale::Es, "tooltip_new_doc") => "Nuevo documento (Ctrl+N)",

            (Locale::It, "tooltip_open") => "Apri documento recente o sfoglia i file (Ctrl+O)",
            (Locale::En, "tooltip_open") => "Open recent document or browse files (Ctrl+O)",
            (Locale::Es, "tooltip_open") => "Abrir documento reciente o explorar archivos (Ctrl+O)",

            (Locale::It, "tooltip_save") => "Salva il documento corrente (Ctrl+S)",
            (Locale::En, "tooltip_save") => "Save current document (Ctrl+S)",
            (Locale::Es, "tooltip_save") => "Guardar el documento actual (Ctrl+S)",

            (Locale::It, "tooltip_save_menu") => "Salva come... o esporta in altri formati (Ctrl+Shift+S)",
            (Locale::En, "tooltip_save_menu") => "Save As... or export to other formats (Ctrl+Shift+S)",
            (Locale::Es, "tooltip_save_menu") => "Guardar como... o exportar a otros formatos (Ctrl+Shift+S)",

            (Locale::It, "tooltip_line_width") => "Scegli larghezza colonna di testo (80ch, 110ch, 140ch, Fluida)",
            (Locale::En, "tooltip_line_width") => "Choose text column width (80ch, 110ch, 140ch, Fluid)",
            (Locale::Es, "tooltip_line_width") => "Elegir ancho de columna de texto (80ch, 110ch, 140ch, Fluido)",

            (Locale::It, "tooltip_search") => "Trova e sostituisci nel documento (Ctrl+F / Ctrl+H)",
            (Locale::En, "tooltip_search") => "Find and replace in document (Ctrl+F / Ctrl+H)",
            (Locale::Es, "tooltip_search") => "Buscar y reemplazar en el documento (Ctrl+F / Ctrl+H)",

            (Locale::It, "tooltip_preview") => "Modalità editor e pannello di anteprima (Ctrl+L / Ctrl+Shift+P)",
            (Locale::En, "tooltip_preview") => "Editor modes and preview panel (Ctrl+L / Ctrl+Shift+P)",
            (Locale::Es, "tooltip_preview") => "Modos de editor y panel de vista previa (Ctrl+L / Ctrl+Shift+P)",

            (Locale::It, "tooltip_main_menu") => "Menu principale, temi e preferenze (F10)",
            (Locale::En, "tooltip_main_menu") => "Main menu, themes and preferences (F10)",
            (Locale::Es, "tooltip_main_menu") => "Menú principal, temas y preferencias (F10)",

            (Locale::It, "tooltip_minimize") => "Riduci a icona la finestra",
            (Locale::En, "tooltip_minimize") => "Minimize window",
            (Locale::Es, "tooltip_minimize") => "Minimizar ventana",

            (Locale::It, "tooltip_maximize") => "Ingrandisci o ripristina la finestra",
            (Locale::En, "tooltip_maximize") => "Maximize or restore window",
            (Locale::Es, "tooltip_maximize") => "Maximizar o restaurar ventana",

            (Locale::It, "tooltip_close") => "Chiudi Accent",
            (Locale::En, "tooltip_close") => "Close Accent",
            (Locale::Es, "tooltip_close") => "Cerrar Accent",

            (Locale::It, "tooltip_bold") => "Grassetto (**testo**) (Ctrl+B)",
            (Locale::En, "tooltip_bold") => "Bold (**text**) (Ctrl+B)",
            (Locale::Es, "tooltip_bold") => "Negrita (**texto**) (Ctrl+B)",

            (Locale::It, "tooltip_italic") => "Corsivo (*testo*) (Ctrl+I)",
            (Locale::En, "tooltip_italic") => "Italic (*text*) (Ctrl+I)",
            (Locale::Es, "tooltip_italic") => "Cursiva (*texto*) (Ctrl+I)",

            (Locale::It, "tooltip_strikethrough") => "Barrato (~~testo~~) (Ctrl+Shift+X)",
            (Locale::En, "tooltip_strikethrough") => "Strikethrough (~~text~~) (Ctrl+Shift+X)",
            (Locale::Es, "tooltip_strikethrough") => "Tachado (~~texto~~) (Ctrl+Shift+X)",

            (Locale::It, "tooltip_heading") => "Inserisci titolo / intestazione (##)",
            (Locale::En, "tooltip_heading") => "Insert heading (##)",
            (Locale::Es, "tooltip_heading") => "Insertar encabezado (##)",

            (Locale::It, "tooltip_bullet_list") => "Inserisci elenco puntato (-)",
            (Locale::En, "tooltip_bullet_list") => "Insert bullet list (-)",
            (Locale::Es, "tooltip_bullet_list") => "Insertar lista con viñetas (-)",

            (Locale::It, "tooltip_ordered_list") => "Inserisci elenco numerato (1.)",
            (Locale::En, "tooltip_ordered_list") => "Insert numbered list (1.)",
            (Locale::Es, "tooltip_ordered_list") => "Insertar lista numerada (1.)",

            (Locale::It, "tooltip_task_list") => "Inserisci elenco di attività (- [ ])",
            (Locale::En, "tooltip_task_list") => "Insert task list (- [ ])",
            (Locale::Es, "tooltip_task_list") => "Insertar lista de tareas (- [ ])",

            (Locale::It, "tooltip_blockquote") => "Inserisci citazione (>)",
            (Locale::En, "tooltip_blockquote") => "Insert blockquote (>)",
            (Locale::Es, "tooltip_blockquote") => "Insertar cita (>)",

            (Locale::It, "tooltip_code") => "Codice inline (`codice`)",
            (Locale::En, "tooltip_code") => "Inline code (`code`)",
            (Locale::Es, "tooltip_code") => "Código en línea (`código`)",

            (Locale::It, "tooltip_link") => "Inserisci collegamento ([testo](url))",
            (Locale::En, "tooltip_link") => "Insert link ([text](url))",
            (Locale::Es, "tooltip_link") => "Insertar enlace ([texto](url))",

            (Locale::It, "tooltip_image") => "Inserisci immagine (![alt](url))",
            (Locale::En, "tooltip_image") => "Insert image (![alt](url))",
            (Locale::Es, "tooltip_image") => "Insertar imagen (![alt](url))",

            (Locale::It, "tooltip_table") => "Inserisci tabella Markdown",
            (Locale::En, "tooltip_table") => "Insert Markdown table",
            (Locale::Es, "tooltip_table") => "Insertar tabla Markdown",

            (Locale::It, "tooltip_toggle_toolbar") => "Mostra o nascondi barra degli strumenti di formattazione",
            (Locale::En, "tooltip_toggle_toolbar") => "Show or hide formatting toolbar",
            (Locale::Es, "tooltip_toggle_toolbar") => "Mostrar u ocultar barra de formato",

            (Locale::It, "tooltip_stats") => "Statistiche documento (parole, caratteri, frasi, paragrafi, tempo di lettura)",
            (Locale::En, "tooltip_stats") => "Document statistics (words, characters, sentences, paragraphs, reading time)",
            (Locale::Es, "tooltip_stats") => "Estadísticas del documento (palabras, caracteres, oraciones, párrafos, tiempo de lectura)",

            // Phase 4: Crash Recovery Banner (it, en, es)
            (Locale::It, "recovery_banner_msg") => "⚠ Documento ripristinato da una chiusura imprevista (modifiche recuperate dal journal)",
            (Locale::En, "recovery_banner_msg") => "⚠ Document restored from an unexpected exit (unsaved edits recovered from journal)",
            (Locale::Es, "recovery_banner_msg") => "⚠ Documento restaurado tras un cierre inesperado (cambios recuperados del journal)",

            (Locale::It, "recovery_confirm_save") => "Conferma e Salva",
            (Locale::En, "recovery_confirm_save") => "Confirm & Save",
            (Locale::Es, "recovery_confirm_save") => "Confirmar y Guardar",

            (Locale::It, "recovery_discard") => "Scarta ripristino",
            (Locale::En, "recovery_discard") => "Discard Recovery",
            (Locale::Es, "recovery_discard") => "Descartar recuperación",

            // Phase 5: Interactive Export Dialog (it, en, es)
            (Locale::It, "export_formats_hdr") => "Formati",
            (Locale::En, "export_formats_hdr") => "Formats",
            (Locale::Es, "export_formats_hdr") => "Formatos",

            (Locale::It, "export_dest_path") => "Percorso di destinazione",
            (Locale::En, "export_dest_path") => "Destination Path",
            (Locale::Es, "export_dest_path") => "Ruta de destino",

            (Locale::It, "export_page_size") => "Dimensione pagina",
            (Locale::En, "export_page_size") => "Page Size",
            (Locale::Es, "export_page_size") => "Tamaño de página",

            (Locale::It, "export_syntax_hl") => "Colorazione della sintassi e Mermaid",
            (Locale::En, "export_syntax_hl") => "Syntax Highlighting & Mermaid",
            (Locale::Es, "export_syntax_hl") => "Resaltado de sintaxis y Mermaid",

            (Locale::It, "export_syntax_hl_sub") => "Usa la colorazione della sintassi e incorpora i diagrammi",
            (Locale::En, "export_syntax_hl_sub") => "Use syntax highlighting and embed diagrams",
            (Locale::Es, "export_syntax_hl_sub") => "Usar resaltado de sintaxis e incrustar diagramas",

            (Locale::It, "export_btn") => "Esporta",
            (Locale::En, "export_btn") => "Export",
            (Locale::Es, "export_btn") => "Exportar",

            _ => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineWidthMode {
    /// ~80 characters per line (`712px` at default font size).
    Compact,
    /// 100–120 characters per line (`920px`).
    Medium,
    /// 140+ characters per line (`1060px`).
    Wide,
    /// Full-width fluid layout with configurable lateral margin percentage (`2..=25%`).
    Fluid { margin_pct: u8 },
}

impl LineWidthMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        let lower = s.trim().to_ascii_lowercase();
        match lower.as_str() {
            "compact" | "80ch" => Some(Self::Compact),
            "medium" | "110ch" => Some(Self::Medium),
            "wide" | "140ch" => Some(Self::Wide),
            "fluid" | "full" | "full-width" => Some(Self::Fluid { margin_pct: 6 }),
            _ => {
                if let Some(pct_str) = lower.strip_prefix("fluid:") {
                    if let Ok(pct) = pct_str.parse::<u8>() {
                        return Some(Self::Fluid {
                            margin_pct: pct.clamp(2, 25),
                        });
                    }
                }
                None
            }
        }
    }

    pub fn short_badge(self) -> &'static str {
        match self {
            Self::Compact => "80ch",
            Self::Medium => "110ch",
            Self::Wide => "140ch",
            Self::Fluid { .. } => "Fluid",
        }
    }

    /// Computes `(x_body, col_width)` for a given `pane_width`, preserving space on the left for outdented `#` markers.
    pub fn compute_column_bounds(self, pane_width: f32) -> (f32, f32) {
        let min_left_margin = 68.0_f32;
        let max_avail = (pane_width - min_left_margin * 1.4).max(200.0);
        let col_width = match self {
            Self::Compact => 712.0_f32.min(max_avail),
            Self::Medium => 920.0_f32.min(max_avail),
            Self::Wide => 1060.0_f32.min(max_avail),
            Self::Fluid { margin_pct } => {
                let pct = f32::from(margin_pct.clamp(2, 25)) / 100.0;
                let margin_px = (pane_width * pct).max(min_left_margin);
                (pane_width - margin_px * 2.0).max(200.0)
            }
        };
        let x_body = ((pane_width - col_width) * 0.5)
            .max(min_left_margin)
            .round();
        (x_body, col_width)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyConfig {
    pub prose_font_family: String,
    pub code_font_family: String,
    pub font_size: f32,
    pub line_height: f32,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            prose_font_family: "Nimbus Mono PS".to_string(),
            code_font_family: "Nimbus Mono PS".to_string(),
            font_size: 17.35,
            line_height: 1.505,
        }
    }
}

/// Discovers system-installed fonts via `fontdb` (`SPECS.md` Pillar 4).
#[derive(Debug, Clone)]
pub struct SystemFontCatalog {
    pub families: Vec<String>,
    pub prose_presets: Vec<String>,
    pub code_presets: Vec<String>,
}

impl Default for SystemFontCatalog {
    fn default() -> Self {
        Self::discover()
    }
}

impl SystemFontCatalog {
    pub fn discover() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let mut set = BTreeSet::new();
        for face in db.faces() {
            for (name, _) in &face.families {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    set.insert(trimmed.to_string());
                }
            }
        }

        let prose_curated = [
            "Nimbus Mono PS",
            "Adwaita Sans",
            "Cantarell",
            "Inter",
            "Literata",
            "Charter",
            "FreeSerif",
            "FreeSans",
        ];
        let code_curated = [
            "Nimbus Mono PS",
            "JetBrains Mono",
            "Adwaita Mono",
            "FreeMono",
            "DejaVu Sans Mono",
        ];

        let mut prose_presets = Vec::new();
        for &c in &prose_curated {
            if set.contains(c) || c == "Nimbus Mono PS" || c == "Inter" || c == "Literata" {
                prose_presets.push(c.to_string());
            }
        }
        for fam in set.iter().take(40) {
            if !prose_presets.contains(fam) {
                prose_presets.push(fam.clone());
            }
        }

        let mut code_presets = Vec::new();
        for &c in &code_curated {
            if set.contains(c) || c == "Nimbus Mono PS" || c == "JetBrains Mono" {
                code_presets.push(c.to_string());
            }
        }
        for fam in set.iter().filter(|f| {
            let l = f.to_ascii_lowercase();
            l.contains("mono") || l.contains("code") || l.contains("courier")
        }) {
            if !code_presets.contains(fam) {
                code_presets.push(fam.clone());
            }
        }

        let families: Vec<String> = if set.is_empty() {
            vec![
                "Nimbus Mono PS".to_string(),
                "Adwaita Sans".to_string(),
                "FreeSans".to_string(),
                "FreeMono".to_string(),
            ]
        } else {
            set.into_iter().collect()
        };

        Self {
            families,
            prose_presets,
            code_presets,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewLayout {
    HalfWidth,
    FullWidth,
    HalfHeight,
    Windowed,
}

impl PreviewLayout {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "halfwidth" | "half-width" | "split" | "horizontal" => Some(Self::HalfWidth),
            "fullwidth" | "full-width" | "full" => Some(Self::FullWidth),
            "halfheight" | "half-height" | "vertical" => Some(Self::HalfHeight),
            "windowed" | "window" => Some(Self::Windowed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatMetric {
    Characters,
    Words,
    Sentences,
    Paragraphs,
    ReadingTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveOverlay {
    None,
    OpenMenu,
    MainMenu,
    SaveMenu,
    PreviewMenu,
    StatsMenu,
    LineWidthMenu,
    PreferencesDialog,
    ExportDialog,
    OpenFileDialog,
    SaveFileDialog,
    ShortcutsDialog,
    AboutDialog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatCommand {
    Bold,
    Italic,
    Strikethrough,
    Heading,
    BulletList,
    OrderedList,
    TaskList,
    BlockQuote,
    Code,
    Link,
    Image,
    Table,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    SetTheme(ThemeMode),
    SetPalettePreset(ColorPalettePreset),
    SetLocale(Locale),
    SetDialect(MarkdownDialect),
    SetLineWidth(LineWidthMode),
    SetProseFont(String),
    SetCodeFont(String),
    SetFontSize(f32),
    SetLineHeight(f32),
    SetPreviewLayout(PreviewLayout),
    SetPreviewMode { active: bool, layout: PreviewLayout },
    SetEditorOnlyRaw,
    SetEditorLivePreview,
    SetStatMetric(StatMetric),
    ToggleSplitPreview,
    ToggleViewWhileTyping,
    SetViewWhileTyping(bool),
    ToggleHemingway,
    ToggleFocusMode,
    ToggleToolbar,
    ToggleSearchBar,
    SetChromeAutohidden(bool),
    SetSearchQuery(String),
    SetReplaceText(String),
    FindNext,
    FindPrev,
    ReplaceNext,
    SetOverlay(ActiveOverlay),
    UpdateBufferText(String),
    SetSelection(usize, usize),
    ApplyFormat(FormatCommand),
    NewDocument,
    OpenFile(PathBuf),
    OpenTutorial,
    SaveDocument,
    SaveAsFile(PathBuf),
    ConfirmRecoverySave,
    DiscardRecovery,
    SetFileDialogDir(PathBuf),
    SetFileDialogInput(String),
    SelectExportFormat(ExportFormat),
    SetExportOutputPath(String),
    ToggleExportSyntaxHighlight,
    ExecuteExport,
    ExportDocument(String),
    InsertText(String),
    MoveCursor(usize),
}

/// User preferences and session state persisted atomically across restarts in `settings.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedSettings {
    pub theme: ThemeMode,
    pub palette_preset: ColorPalettePreset,
    pub locale: Locale,
    pub dialect: MarkdownDialect,
    pub line_width: LineWidthMode,
    pub typography: TypographyConfig,
    pub preview_active: bool,
    pub preview_layout: PreviewLayout,
    pub view_while_typing: bool,
    pub hemingway_mode: bool,
    pub focus_mode: bool,
    pub toolbar_expanded: bool,
    pub stat_metric: StatMetric,
    pub selected_export_format: ExportFormat,
    pub export_syntax_highlight: bool,
    pub last_opened_dir: Option<PathBuf>,
    pub last_opened_file: Option<PathBuf>,
    pub recent_files: Vec<PathBuf>,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Sepia,
            palette_preset: ColorPalettePreset::EditorialSlate,
            locale: Locale::It,
            dialect: MarkdownDialect::Gfm,
            line_width: LineWidthMode::Compact,
            typography: TypographyConfig::default(),
            preview_active: false,
            preview_layout: PreviewLayout::HalfWidth,
            view_while_typing: true,
            hemingway_mode: false,
            focus_mode: false,
            toolbar_expanded: true,
            stat_metric: StatMetric::Words,
            selected_export_format: ExportFormat::Pdf,
            export_syntax_highlight: true,
            last_opened_dir: None,
            last_opened_file: None,
            recent_files: Vec::new(),
        }
    }
}

/// Debounce threshold for *Save-while-typing* WAL journaling (`< 300ms` per `SPECS.md` Pillar 8).
pub const JOURNAL_DEBOUNCE_MS: u64 = 200;

#[derive(Debug, Clone)]
pub struct Session {
    pub document_title: String,
    pub current_file_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub config_path: PathBuf,
    pub recovery_dir: PathBuf,
    pub original_file_hash: Option<String>,
    pub last_edit_instant: Option<std::time::Instant>,
    pub last_edit_utc_ms: u64,
    pub journal_pending_flush: bool,
    pub recovered_from_crash: bool,
    pub recovery_banner_active: bool,
    pub recent_files: Vec<PathBuf>,
    pub file_dialog_dir: PathBuf,
    pub file_dialog_input: String,
    pub selected_export_format: ExportFormat,
    pub export_output_path: String,
    pub export_syntax_highlight: bool,
    pub mermaid_cache: RefCell<MermaidCache>,
    pub status_message: Option<String>,
    pub buffer: DocumentBuffer,
    pub theme: ThemeMode,
    pub palette_preset: ColorPalettePreset,
    pub locale: Locale,
    pub dialect: MarkdownDialect,
    pub line_width: LineWidthMode,
    pub typography: TypographyConfig,
    pub font_catalog: SystemFontCatalog,
    pub preview_active: bool,
    pub preview_layout: PreviewLayout,
    pub view_while_typing: bool,
    pub hemingway_mode: bool,
    pub focus_mode: bool,
    pub toolbar_expanded: bool,
    pub search_bar_open: bool,
    pub chrome_autohidden: bool,
    pub search_query: String,
    pub replace_text: String,
    pub stat_metric: StatMetric,
    pub overlay: ActiveOverlay,
    /// Optional word count override for legacy snapshot harness only (`None` in interactive runtime)
    pub display_word_count_override: Option<usize>,
}

impl Default for Session {
    fn default() -> Self {
        #[cfg(test)]
        {
            let test_dir = std::env::temp_dir().join(format!(
                "accent_unit_test_default_{}",
                std::process::id()
            ));
            let _ = std::fs::remove_file(test_dir.join("settings.json"));
            Self::with_recovery_dir(test_dir)
        }
        #[cfg(not(test))]
        {
            Self::with_config_and_recovery(
                accent_storage::default_config_path(),
                accent_storage::default_recovery_dir(),
            )
        }
    }
}

impl Session {
    pub fn with_recovery_dir(recovery_dir: PathBuf) -> Self {
        let config_path = recovery_dir.join("settings.json");
        Self::with_config_and_recovery(config_path, recovery_dir)
    }

    pub fn with_config_and_recovery(config_path: PathBuf, recovery_dir: PathBuf) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let sample_path = PathBuf::from("/tmp/esempio_accent.md");
        if !sample_path.exists() {
            let _ = std::fs::write(&sample_path, DEFAULT_SAMPLE_MARKDOWN);
        }
        let mut recent_files = Vec::new();
        for candidate in [
            cwd.join("SPECS.md"),
            cwd.join("AGENTS.md"),
            sample_path.clone(),
        ] {
            if candidate.exists() {
                recent_files.push(candidate);
            }
        }
        let default_buf = DocumentBuffer::default();
        let initial_hash = Some(accent_storage::compute_content_hash(&default_buf.text()));
        let mut session = Self {
            document_title: "esempio_accent.md".to_string(),
            current_file_path: Some(sample_path),
            is_dirty: false,
            config_path,
            recovery_dir,
            original_file_hash: initial_hash,
            last_edit_instant: None,
            last_edit_utc_ms: 0,
            journal_pending_flush: false,
            recovered_from_crash: false,
            recovery_banner_active: false,
            recent_files,
            file_dialog_dir: cwd,
            file_dialog_input: "documento.md".to_string(),
            selected_export_format: ExportFormat::Pdf,
            export_output_path: "/tmp/esempio_accent.pdf".to_string(),
            export_syntax_highlight: true,
            mermaid_cache: RefCell::new(MermaidCache::new()),
            status_message: None,
            buffer: default_buf,
            theme: ThemeMode::Sepia,
            palette_preset: ColorPalettePreset::EditorialSlate,
            locale: Locale::It,
            dialect: MarkdownDialect::Gfm,
            line_width: LineWidthMode::Compact,
            typography: TypographyConfig::default(),
            font_catalog: SystemFontCatalog::discover(),
            preview_active: false,
            preview_layout: PreviewLayout::HalfWidth,
            view_while_typing: true,
            hemingway_mode: false,
            focus_mode: false,
            toolbar_expanded: true,
            search_bar_open: false,
            chrome_autohidden: false,
            search_query: String::new(),
            replace_text: String::new(),
            stat_metric: StatMetric::Words,
            overlay: ActiveOverlay::None,
            display_word_count_override: None,
        };
        session.load_settings_from_disk();
        session.check_startup_crash_recovery();
        session
    }

    /// Snapshots all user preferences, recent files, and last opened folder/file into `PersistedSettings`.
    pub fn to_persisted_settings(&self) -> PersistedSettings {
        PersistedSettings {
            theme: self.theme,
            palette_preset: self.palette_preset,
            locale: self.locale,
            dialect: self.dialect,
            line_width: self.line_width,
            typography: self.typography.clone(),
            preview_active: self.preview_active,
            preview_layout: self.preview_layout,
            view_while_typing: self.view_while_typing,
            hemingway_mode: self.hemingway_mode,
            focus_mode: self.focus_mode,
            toolbar_expanded: self.toolbar_expanded,
            stat_metric: self.stat_metric,
            selected_export_format: self.selected_export_format,
            export_syntax_highlight: self.export_syntax_highlight,
            last_opened_dir: Some(self.file_dialog_dir.clone()),
            last_opened_file: self.current_file_path.clone(),
            recent_files: self.recent_files.clone(),
        }
    }

    /// Atomically persists `PersistedSettings` to `self.config_path` (`settings.json`).
    pub fn save_settings(&self) -> std::io::Result<()> {
        let settings = self.to_persisted_settings();
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        accent_storage::atomic_save(&self.config_path, &json)
    }

    /// Loads persisted settings and last opened directory/file from `self.config_path` if present.
    pub fn load_settings_from_disk(&mut self) -> bool {
        let Ok(raw) = std::fs::read_to_string(&self.config_path) else {
            return false;
        };
        let Ok(settings) = serde_json::from_str::<PersistedSettings>(&raw) else {
            return false;
        };

        self.theme = settings.theme;
        self.palette_preset = settings.palette_preset;
        self.locale = settings.locale;
        self.dialect = settings.dialect;
        self.line_width = settings.line_width;
        self.typography = settings.typography;
        self.typography.font_size = self.typography.font_size.clamp(12.0, 28.0);
        self.typography.line_height = self.typography.line_height.clamp(1.1, 2.2);
        self.preview_active = settings.preview_active;
        self.preview_layout = settings.preview_layout;
        self.view_while_typing = settings.view_while_typing;
        self.hemingway_mode = settings.hemingway_mode;
        self.focus_mode = settings.focus_mode;
        self.toolbar_expanded = settings.toolbar_expanded;
        self.stat_metric = settings.stat_metric;
        self.selected_export_format = settings.selected_export_format;
        self.export_syntax_highlight = settings.export_syntax_highlight;

        if let Some(dir) = settings.last_opened_dir {
            if dir.is_dir() {
                self.file_dialog_dir = dir;
            }
        }

        let existing_recents: Vec<PathBuf> = settings
            .recent_files
            .into_iter()
            .filter(|p| p.exists())
            .take(8)
            .collect();
        if !existing_recents.is_empty() {
            self.recent_files = existing_recents;
        }

        if let Some(last_file) = settings.last_opened_file {
            if last_file.is_file() {
                if let Ok(content) = std::fs::read_to_string(&last_file) {
                    let title = last_file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("documento.md")
                        .to_string();
                    self.document_title = title.clone();
                    self.file_dialog_input = title;
                    if let Some(parent) = last_file.parent() {
                        if parent.is_dir() && self.file_dialog_dir == std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) {
                            self.file_dialog_dir = parent.to_path_buf();
                        }
                    }
                    self.current_file_path = Some(last_file);
                    self.original_file_hash = Some(accent_storage::compute_content_hash(&content));
                    self.buffer.set_text(&content);
                    self.buffer.set_cursor(0);
                }
            }
        }
        self.update_default_export_path();
        true
    }

    pub fn render_mermaid(&self, source: &str) -> Arc<RenderedMermaidDiagram> {
        let m_theme = match self.theme {
            ThemeMode::Light => MermaidTheme::Light,
            ThemeMode::Sepia => MermaidTheme::Sepia,
            ThemeMode::Dark => MermaidTheme::Dark,
        };
        self.mermaid_cache
            .borrow_mut()
            .get_or_render_themed(source, m_theme)
    }

    fn update_default_export_path(&mut self) {
        let stem = self
            .document_title
            .trim_end_matches(".md")
            .trim_end_matches(".markdown");
        let base_dir = self
            .current_file_path
            .as_ref()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| std::path::Path::new("/tmp"));
        let candidate = base_dir.join(format!(
            "{stem}.{}",
            self.selected_export_format.extension()
        ));
        self.export_output_path = candidate.to_string_lossy().to_string();
    }

    fn mark_dirty_edit(&mut self) {
        self.display_word_count_override = None;
        self.is_dirty = true;
        self.journal_pending_flush = true;
        self.last_edit_instant = Some(std::time::Instant::now());
        self.last_edit_utc_ms = accent_storage::current_utc_ms();
    }

    /// Immediately persists the current buffer state into the recovery WAL journal.
    pub fn flush_journal_now(&mut self) -> std::io::Result<PathBuf> {
        let cur_char = self.buffer.cursor();
        let cur_byte = self.buffer.char_to_byte(cur_char);
        let entry = accent_storage::RecoveryJournalEntry::new(
            self.current_file_path.clone(),
            self.document_title.clone(),
            self.buffer.text(),
            cur_char,
            cur_byte,
            self.original_file_hash.clone(),
            self.is_dirty,
        );
        let written = accent_storage::write_journal_entry(&self.recovery_dir, &entry)?;
        self.journal_pending_flush = false;
        Ok(written)
    }

    /// Checks whether the `< 300ms` debounce window has elapsed and flushes the journal if needed.
    pub fn poll_journal_flush(&mut self, now: std::time::Instant) -> bool {
        if !self.is_dirty || !self.journal_pending_flush {
            return false;
        }
        let debounce = std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS);
        let ready = match self.last_edit_instant {
            Some(last) => now.saturating_duration_since(last) >= debounce,
            None => true,
        };
        if ready {
            let _ = self.flush_journal_now();
            true
        } else {
            false
        }
    }

    /// Returns the remaining duration before the pending journal flush should be triggered.
    pub fn time_until_next_journal_flush(
        &self,
        now: std::time::Instant,
    ) -> Option<std::time::Duration> {
        if !self.is_dirty || !self.journal_pending_flush {
            return None;
        }
        let debounce = std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS);
        match self.last_edit_instant {
            Some(last) => Some(debounce.saturating_sub(now.saturating_duration_since(last))),
            None => Some(std::time::Duration::ZERO),
        }
    }

    /// Restores the editor buffer and cursor from a `RecoveryJournalEntry` discovered on startup.
    pub fn restore_from_journal_entry(&mut self, entry: accent_storage::RecoveryJournalEntry) {
        self.document_title = entry.document_title.clone();
        self.file_dialog_input = entry.document_title.clone();
        if let Some(ref target) = entry.target_path {
            if let Some(parent) = target.parent() {
                self.file_dialog_dir = parent.to_path_buf();
            }
        }
        self.current_file_path = entry.target_path;
        self.original_file_hash = entry.original_file_hash;
        self.display_word_count_override = None;
        self.buffer.set_text(&entry.content);
        self.buffer
            .set_cursor(entry.cursor_char_offset.min(self.buffer.len_chars()));
        self.is_dirty = true;
        self.journal_pending_flush = false;
        self.recovered_from_crash = true;
        self.recovery_banner_active = true;
        self.chrome_autohidden = false;
        self.status_message = Some(I18n::tr(self.locale, "recovery_banner_msg").to_string());
    }

    /// Scans `self.recovery_dir` on startup to automatically restore any unclosed dirty session.
    pub fn check_startup_crash_recovery(&mut self) -> bool {
        if let Some(entry) = accent_storage::find_recovery_journal_for_path(
            &self.recovery_dir,
            self.current_file_path.as_deref(),
        ) {
            self.restore_from_journal_entry(entry);
            return true;
        }
        if let Some(entry) = accent_storage::scan_recovery_journals(&self.recovery_dir)
            .into_iter()
            .next()
        {
            self.restore_from_journal_entry(entry);
            return true;
        }
        false
    }

    /// Cleans up the recovery journal and persists user settings when the application closes normally.
    pub fn on_clean_exit(&mut self) {
        let _ = self.save_settings();
        if self.is_dirty {
            if self.journal_pending_flush {
                let _ = self.flush_journal_now();
            }
        } else {
            let _ = accent_storage::remove_journal_for_target(
                &self.recovery_dir,
                self.current_file_path.as_deref(),
            );
        }
    }

    pub fn apply(&mut self, action: Action) {
        let prev_overlay = self.overlay;
        let should_persist_settings = matches!(
            action,
            Action::SetTheme(_)
                | Action::SetPalettePreset(_)
                | Action::SetLocale(_)
                | Action::SetDialect(_)
                | Action::SetLineWidth(_)
                | Action::SetProseFont(_)
                | Action::SetCodeFont(_)
                | Action::SetFontSize(_)
                | Action::SetLineHeight(_)
                | Action::SetPreviewLayout(_)
                | Action::SetPreviewMode { .. }
                | Action::SetEditorOnlyRaw
                | Action::SetEditorLivePreview
                | Action::SetStatMetric(_)
                | Action::ToggleSplitPreview
                | Action::ToggleViewWhileTyping
                | Action::SetViewWhileTyping(_)
                | Action::ToggleHemingway
                | Action::ToggleFocusMode
                | Action::ToggleToolbar
                | Action::OpenFile(_)
                | Action::SaveDocument
                | Action::ConfirmRecoverySave
                | Action::SaveAsFile(_)
                | Action::SetFileDialogDir(_)
                | Action::SelectExportFormat(_)
                | Action::ToggleExportSyntaxHighlight
                | Action::ExecuteExport
        );
        match action {
            Action::SetTheme(theme) => self.theme = theme,
            Action::SetPalettePreset(preset) => self.palette_preset = preset,
            Action::SetLocale(locale) => self.locale = locale,
            Action::SetDialect(dialect) => self.dialect = dialect,
            Action::SetLineWidth(lw) => self.line_width = lw,
            Action::SetProseFont(family) => self.typography.prose_font_family = family,
            Action::SetCodeFont(family) => self.typography.code_font_family = family,
            Action::SetFontSize(size) => self.typography.font_size = size.clamp(12.0, 28.0),
            Action::SetLineHeight(lh) => self.typography.line_height = lh.clamp(1.1, 2.2),
            Action::SetPreviewLayout(layout) => {
                self.preview_layout = layout;
                self.preview_active = true;
            }
            Action::SetPreviewMode { active, layout } => {
                self.preview_active = active;
                self.preview_layout = layout;
            }
            Action::SetEditorOnlyRaw => {
                self.preview_active = false;
                self.view_while_typing = false;
            }
            Action::SetEditorLivePreview => {
                self.preview_active = false;
                self.view_while_typing = true;
            }
            Action::SetStatMetric(metric) => self.stat_metric = metric,
            Action::ToggleSplitPreview => self.preview_active = !self.preview_active,
            Action::ToggleViewWhileTyping => {
                self.view_while_typing = !self.view_while_typing;
                if self.view_while_typing {
                    self.preview_active = false;
                }
            }
            Action::SetViewWhileTyping(v) => {
                self.view_while_typing = v;
                if v {
                    self.preview_active = false;
                }
            }
            Action::ToggleHemingway => self.hemingway_mode = !self.hemingway_mode,
            Action::ToggleFocusMode => self.focus_mode = !self.focus_mode,
            Action::ToggleToolbar => self.toolbar_expanded = !self.toolbar_expanded,
            Action::ToggleSearchBar => {
                self.search_bar_open = !self.search_bar_open;
                if self.search_bar_open {
                    self.overlay = ActiveOverlay::None;
                }
                self.chrome_autohidden = false;
            }
            Action::SetChromeAutohidden(hidden) => {
                self.chrome_autohidden = hidden;
            }
            Action::SetSearchQuery(q) => self.search_query = q,
            Action::SetReplaceText(r) => self.replace_text = r,
            Action::FindNext => {
                if !self.search_query.is_empty() {
                    let current = self.buffer.text();
                    let cur_char = self.buffer.cursor();
                    let cur_byte = self.buffer.char_to_byte(cur_char).min(current.len());
                    let match_byte = current[cur_byte..]
                        .find(&self.search_query)
                        .map(|off| cur_byte + off)
                        .or_else(|| current.find(&self.search_query));
                    if let Some(byte_start) = match_byte {
                        let byte_end = byte_start + self.search_query.len();
                        let char_start = self.buffer.byte_to_char(byte_start);
                        let char_end = self.buffer.byte_to_char(byte_end);
                        self.buffer.set_selection(char_end, char_start);
                    }
                }
            }
            Action::FindPrev => {
                if !self.search_query.is_empty() {
                    let current = self.buffer.text();
                    let sel_min = self.buffer.selection_char_range().start;
                    let cur_byte = self.buffer.char_to_byte(sel_min).min(current.len());
                    let match_byte = current[..cur_byte]
                        .rfind(&self.search_query)
                        .or_else(|| current.rfind(&self.search_query));
                    if let Some(byte_start) = match_byte {
                        let byte_end = byte_start + self.search_query.len();
                        let char_start = self.buffer.byte_to_char(byte_start);
                        let char_end = self.buffer.byte_to_char(byte_end);
                        self.buffer.set_selection(char_end, char_start);
                    }
                }
            }
            Action::ReplaceNext => {
                if !self.search_query.is_empty() {
                    let current = self.buffer.text();
                    if let Some(pos) = current.find(&self.search_query) {
                        let mut updated = String::with_capacity(current.len());
                        updated.push_str(&current[..pos]);
                        updated.push_str(&self.replace_text);
                        updated.push_str(&current[pos + self.search_query.len()..]);
                        self.mark_dirty_edit();
                        self.buffer.set_text(&updated);
                    }
                }
            }
            Action::SetOverlay(overlay) => {
                self.chrome_autohidden = false;
                if self.overlay == overlay {
                    self.overlay = ActiveOverlay::None;
                } else {
                    if overlay == ActiveOverlay::ExportDialog {
                        self.update_default_export_path();
                    }
                    self.overlay = overlay;
                }
            }
            Action::UpdateBufferText(new_text) => {
                if new_text != self.buffer.text() {
                    self.mark_dirty_edit();
                    if self.overlay == ActiveOverlay::None
                        && !self.search_bar_open
                        && !self.recovery_banner_active
                    {
                        self.chrome_autohidden = true;
                    }
                    self.buffer.set_text(&new_text);
                }
            }
            Action::SetSelection(primary, secondary) => {
                self.buffer.set_selection(primary, secondary);
                if self.is_dirty {
                    self.journal_pending_flush = true;
                    self.last_edit_instant = Some(std::time::Instant::now());
                }
            }
            Action::ApplyFormat(cmd) => {
                self.mark_dirty_edit();
                self.chrome_autohidden = false;
                match cmd {
                    FormatCommand::Bold => self.buffer.wrap_selection("**", "**", "bold"),
                    FormatCommand::Italic => self.buffer.wrap_selection("*", "*", "italic"),
                    FormatCommand::Strikethrough => {
                        self.buffer.wrap_selection("~~", "~~", "strikethrough")
                    }
                    FormatCommand::Heading => self.buffer.insert_text("\n## Heading\n"),
                    FormatCommand::BulletList => self.buffer.insert_text("\n- Item\n"),
                    FormatCommand::OrderedList => self.buffer.insert_text("\n1. Item\n"),
                    FormatCommand::TaskList => self.buffer.insert_text("\n- [ ] Task\n"),
                    FormatCommand::BlockQuote => self.buffer.insert_text("\n> Quote\n"),
                    FormatCommand::Code => self.buffer.wrap_selection("`", "`", "code"),
                    FormatCommand::Link => {
                        self.buffer.wrap_selection("[", "](https://example.com)", "link")
                    }
                    FormatCommand::Image => {
                        self.buffer.wrap_selection("![", "](image.png)", "alt text")
                    }
                    FormatCommand::Table => {
                        self.buffer.insert_text(
                            "\n| Column 1 | Column 2 |\n|:---|:---|\n| Cell A | Cell B |\n",
                        );
                    }
                }
            }
            Action::NewDocument => {
                let _ = accent_storage::remove_journal_for_target(&self.recovery_dir, None);
                self.document_title = "untitled.md".to_string();
                self.current_file_path = None;
                self.is_dirty = false;
                self.journal_pending_flush = false;
                self.recovered_from_crash = false;
                self.recovery_banner_active = false;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                let init_txt = "# Nuovo Documento\n\nInizia a scrivere qui...\n";
                self.original_file_hash = Some(accent_storage::compute_content_hash(init_txt));
                self.buffer.set_text(init_txt);
                self.buffer.set_cursor(self.buffer.len_chars());
                self.overlay = ActiveOverlay::None;
            }
            Action::OpenFile(path) => {
                let disk_content = std::fs::read_to_string(&path).ok();
                let disk_hash = disk_content
                    .as_deref()
                    .map(accent_storage::compute_content_hash);

                if let Some(mut entry) = accent_storage::find_recovery_journal_for_path(
                    &self.recovery_dir,
                    Some(&path),
                ) {
                    if entry.original_file_hash.is_none() {
                        entry.original_file_hash = disk_hash;
                    }
                    self.restore_from_journal_entry(entry);
                    self.recent_files.retain(|p| p != &path);
                    self.recent_files.insert(0, path);
                    if self.recent_files.len() > 8 {
                        self.recent_files.truncate(8);
                    }
                    self.overlay = ActiveOverlay::None;
                } else if let Some(content) = disk_content {
                    let title = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("documento.md")
                        .to_string();
                    self.document_title = title.clone();
                    self.file_dialog_input = title;
                    if let Some(parent) = path.parent() {
                        self.file_dialog_dir = parent.to_path_buf();
                    }
                    self.current_file_path = Some(path.clone());
                    self.update_default_export_path();
                    self.original_file_hash = disk_hash;
                    self.is_dirty = false;
                    self.journal_pending_flush = false;
                    self.recovered_from_crash = false;
                    self.recovery_banner_active = false;
                    self.chrome_autohidden = false;
                    self.display_word_count_override = None;
                    self.buffer.set_text(&content);
                    self.buffer.set_cursor(0);
                    self.recent_files.retain(|p| p != &path);
                    self.recent_files.insert(0, path);
                    if self.recent_files.len() > 8 {
                        self.recent_files.truncate(8);
                    }
                    self.status_message = Some(format!("Aperto: {}", self.document_title));
                    self.overlay = ActiveOverlay::None;
                }
            }
            Action::OpenTutorial => {
                let tutorial_path = PathBuf::from("/tmp/accent_tutorial.md");
                let _ = std::fs::write(&tutorial_path, TUTORIAL_MARKDOWN);
                let _ = accent_storage::remove_journal_for_target(
                    &self.recovery_dir,
                    Some(&tutorial_path),
                );
                self.document_title = "tutorial_accent.md".to_string();
                self.current_file_path = Some(tutorial_path);
                self.original_file_hash =
                    Some(accent_storage::compute_content_hash(TUTORIAL_MARKDOWN));
                self.is_dirty = false;
                self.journal_pending_flush = false;
                self.recovered_from_crash = false;
                self.recovery_banner_active = false;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                self.buffer.set_text(TUTORIAL_MARKDOWN);
                self.buffer.set_cursor(0);
                self.status_message = Some("Tutorial caricato".to_string());
                self.overlay = ActiveOverlay::None;
            }
            Action::SaveDocument | Action::ConfirmRecoverySave => {
                let target = self
                    .current_file_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("/tmp/accent_autosave.md"));
                let text = self.buffer.text();
                if accent_storage::atomic_save(&target, &text).is_ok() {
                    let _ = accent_storage::remove_journal_for_target(
                        &self.recovery_dir,
                        self.current_file_path.as_deref(),
                    );
                    let _ = accent_storage::remove_journal_for_target(
                        &self.recovery_dir,
                        Some(&target),
                    );
                    self.is_dirty = false;
                    self.journal_pending_flush = false;
                    self.recovered_from_crash = false;
                    self.recovery_banner_active = false;
                    self.chrome_autohidden = false;
                    self.original_file_hash = Some(accent_storage::compute_content_hash(&text));
                    self.current_file_path = Some(target.clone());
                    self.status_message = Some(format!("Salvato in {}", target.display()));
                }
            }
            Action::DiscardRecovery => {
                let _ = accent_storage::remove_journal_for_target(
                    &self.recovery_dir,
                    self.current_file_path.as_deref(),
                );
                self.is_dirty = false;
                self.journal_pending_flush = false;
                self.recovered_from_crash = false;
                self.recovery_banner_active = false;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                if let Some(path) = self.current_file_path.clone() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.original_file_hash =
                            Some(accent_storage::compute_content_hash(&content));
                        self.buffer.set_text(&content);
                        self.buffer.set_cursor(0);
                    }
                } else {
                    let init_txt = "# Nuovo Documento\n\nInizia a scrivere qui...\n";
                    self.original_file_hash = Some(accent_storage::compute_content_hash(init_txt));
                    self.buffer.set_text(init_txt);
                    self.buffer.set_cursor(0);
                }
                self.status_message = Some("Ripristino scartato".to_string());
            }
            Action::SaveAsFile(path) => {
                let text = self.buffer.text();
                if accent_storage::atomic_save(&path, &text).is_ok() {
                    let _ = accent_storage::remove_journal_for_target(
                        &self.recovery_dir,
                        self.current_file_path.as_deref(),
                    );
                    let _ =
                        accent_storage::remove_journal_for_target(&self.recovery_dir, Some(&path));
                    let title = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("documento.md")
                        .to_string();
                    self.document_title = title.clone();
                    self.file_dialog_input = title;
                    if let Some(parent) = path.parent() {
                        if parent.is_dir() {
                            self.file_dialog_dir = parent.to_path_buf();
                        }
                    }
                    self.current_file_path = Some(path.clone());
                    self.update_default_export_path();
                    self.original_file_hash = Some(accent_storage::compute_content_hash(&text));
                    self.is_dirty = false;
                    self.journal_pending_flush = false;
                    self.recovered_from_crash = false;
                    self.recovery_banner_active = false;
                    self.chrome_autohidden = false;
                    self.recent_files.retain(|p| p != &path);
                    self.recent_files.insert(0, path.clone());
                    if self.recent_files.len() > 8 {
                        self.recent_files.truncate(8);
                    }
                    self.status_message = Some(format!("Salvato come {}", path.display()));
                    self.overlay = ActiveOverlay::None;
                }
            }
            Action::SetFileDialogDir(dir) => {
                if dir.is_dir() {
                    self.file_dialog_dir = dir;
                }
            }
            Action::SetFileDialogInput(input) => {
                self.file_dialog_input = input;
            }
            Action::SelectExportFormat(fmt) => {
                self.selected_export_format = fmt;
                let cur_path = PathBuf::from(&self.export_output_path);
                let parent = cur_path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or_else(|| std::path::Path::new("/tmp"));
                let stem = cur_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| self.document_title.trim_end_matches(".md"));
                self.export_output_path = parent
                    .join(format!("{stem}.{}", fmt.extension()))
                    .to_string_lossy()
                    .to_string();
            }
            Action::SetExportOutputPath(path_str) => {
                self.export_output_path = path_str;
            }
            Action::ToggleExportSyntaxHighlight => {
                self.export_syntax_highlight = !self.export_syntax_highlight;
            }
            Action::ExecuteExport => {
                let fmt = self.selected_export_format;
                let out_path = PathBuf::from(self.export_output_path.trim());
                match accent_export::export_document_bytes(
                    &self.buffer,
                    self.dialect,
                    &self.document_title,
                    fmt,
                ) {
                    Ok(bytes) => {
                        if accent_storage::atomic_save_bytes(&out_path, &bytes).is_ok() {
                            self.status_message =
                                Some(format!("Esportato in {}", out_path.display()));
                            self.overlay = ActiveOverlay::None;
                        } else {
                            self.status_message = Some(format!(
                                "Errore scrittura in {}",
                                out_path.display()
                            ));
                        }
                    }
                    Err(err) => {
                        self.status_message = Some(format!("Errore esportazione: {err}"));
                    }
                }
            }
            Action::ExportDocument(fmt_str) => {
                let fmt = ExportFormat::from_code(&fmt_str);
                self.selected_export_format = fmt;
                self.update_default_export_path();
                let out_path = PathBuf::from(&self.export_output_path);
                if let Ok(bytes) = accent_export::export_document_bytes(
                    &self.buffer,
                    self.dialect,
                    &self.document_title,
                    fmt,
                ) {
                    let _ = accent_storage::atomic_save_bytes(&out_path, &bytes);
                    self.status_message = Some(format!("Esportato in {}", out_path.display()));
                }
                self.overlay = ActiveOverlay::None;
            }
            Action::InsertText(text) => {
                self.mark_dirty_edit();
                if self.overlay == ActiveOverlay::None
                    && !self.search_bar_open
                    && !self.recovery_banner_active
                {
                    self.chrome_autohidden = true;
                }
                self.buffer.insert_text(&text);
            }
            Action::MoveCursor(offset) => {
                self.buffer.set_cursor(offset);
                if self.is_dirty {
                    self.journal_pending_flush = true;
                    self.last_edit_instant = Some(std::time::Instant::now());
                }
            }
        }

        // Automatically persist user settings whenever a preference/directory/file changes
        // OR when closing the Preferences dialog / settings overlays.
        let closed_settings_overlay =
            prev_overlay != ActiveOverlay::None && self.overlay != prev_overlay;
        if should_persist_settings || closed_settings_overlay {
            let _ = self.save_settings();
        }
    }

    pub fn format_active_stat(&self) -> String {
        let stats: DocumentStats = self.buffer.stats();
        match self.stat_metric {
            StatMetric::Characters => {
                format!(
                    "{} {}",
                    format_thousands(stats.characters, self.locale),
                    I18n::tr(self.locale, "characters")
                )
            }
            StatMetric::Words => {
                let w = self.display_word_count_override.unwrap_or(stats.words);
                format!(
                    "{} {}",
                    format_thousands(w, self.locale),
                    I18n::tr(self.locale, "words")
                )
            }
            StatMetric::Sentences => {
                format!(
                    "{} {}",
                    format_thousands(stats.sentences, self.locale),
                    I18n::tr(self.locale, "sentences")
                )
            }
            StatMetric::Paragraphs => {
                format!(
                    "{} {}",
                    format_thousands(stats.paragraphs, self.locale),
                    I18n::tr(self.locale, "paragraphs")
                )
            }
            StatMetric::ReadingTime => {
                format!(
                    "{} {}",
                    stats.format_reading_time(),
                    I18n::tr(self.locale, "reading_time")
                )
            }
        }
    }

    pub fn inspect_json(&self) -> String {
        let stats = self.buffer.stats();
        let live_spans = self.buffer.compute_live_spans(
            self.dialect,
            self.view_while_typing,
            self.buffer.selection_byte_range(),
        );
        serde_json::json!({
            "document_title": self.document_title,
            "theme": self.theme,
            "locale": self.locale,
            "dialect": self.dialect,
            "line_width": self.line_width,
            "typography": self.typography,
            "discovered_system_fonts": self.font_catalog.families.len(),
            "preview_active": self.preview_active,
            "preview_layout": self.preview_layout,
            "view_while_typing": self.view_while_typing,
            "hemingway_mode": self.hemingway_mode,
            "focus_mode": self.focus_mode,
            "cursor": self.buffer.cursor(),
            "live_span_count": live_spans.len(),
            "stats": stats,
        })
        .to_string()
    }
}

pub fn format_thousands(n: usize, locale: Locale) -> String {
    let s = n.to_string();
    if s.len() <= 3 {
        return s;
    }
    let sep = match locale {
        Locale::En => ',',
        Locale::It | Locale::Es => '.',
    };
    let mut out = String::new();
    for (idx, ch) in s.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(sep);
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_translations_en_it_es() {
        assert_eq!(I18n::tr(Locale::It, "open"), "Apri");
        assert_eq!(I18n::tr(Locale::En, "open"), "Open");
        assert_eq!(I18n::tr(Locale::Es, "open"), "Abrir");

        assert_eq!(I18n::tr(Locale::It, "words"), "parole");
        assert_eq!(I18n::tr(Locale::En, "words"), "words");
        assert_eq!(I18n::tr(Locale::Es, "words"), "palabras");
    }

    #[test]
    fn test_session_actions_and_stat_formatting() {
        let mut session = Session::default();
        let actual_words = session.buffer.stats().words;
        assert!(actual_words > 150);
        assert_eq!(
            session.format_active_stat(),
            format!("{} parole", format_thousands(actual_words, Locale::It))
        );

        session.apply(Action::SetTheme(ThemeMode::Dark));
        assert_eq!(session.theme, ThemeMode::Dark);

        session.apply(Action::SetLocale(Locale::En));
        assert_eq!(
            session.format_active_stat(),
            format!("{} words", format_thousands(actual_words, Locale::En))
        );

        session.apply(Action::SetLocale(Locale::Es));
        assert_eq!(
            session.format_active_stat(),
            format!("{} palabras", format_thousands(actual_words, Locale::Es))
        );

        // Verify mutually-exclusive editor & preview modes
        session.apply(Action::SetEditorLivePreview);
        assert!(!session.preview_active && session.view_while_typing);

        session.apply(Action::SetPreviewMode {
            active: true,
            layout: PreviewLayout::HalfWidth,
        });
        assert!(session.preview_active && session.preview_layout == PreviewLayout::HalfWidth);

        session.apply(Action::SetEditorOnlyRaw);
        assert!(!session.preview_active && !session.view_while_typing);

        // Verify opening a real file from disk
        session.apply(Action::OpenTutorial);
        assert_eq!(session.document_title, "tutorial_accent.md");
        assert!(session.buffer.text().contains("Benvenuto in Accent"));
    }

    #[test]
    fn test_line_width_modes_and_fontdb_discovery() {
        let mut session = Session::default();
        assert!(!session.font_catalog.families.is_empty());

        let (x_compact, w_compact) = session.line_width.compute_column_bounds(1200.0);
        assert_eq!(w_compact, 712.0);
        assert_eq!(x_compact, 244.0);

        session.apply(Action::SetLineWidth(LineWidthMode::Medium));
        let (_, w_medium) = session.line_width.compute_column_bounds(1200.0);
        assert_eq!(w_medium, 920.0);

        session.apply(Action::SetLineWidth(LineWidthMode::Wide));
        let (_, w_wide) = session.line_width.compute_column_bounds(1200.0);
        assert_eq!(w_wide, 1060.0);

        session.apply(Action::SetLineWidth(LineWidthMode::Fluid { margin_pct: 8 }));
        let (x_fluid, w_fluid) = session.line_width.compute_column_bounds(1200.0);
        assert!(w_fluid > 980.0);
        assert!(x_fluid >= 68.0);
    }

    #[test]
    fn test_crash_recovery_debounced_wal_and_session_restoration() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_session_wal_test_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        std::fs::create_dir_all(&recovery_dir).expect("create recovery_dir");

        let doc_path = base_dir.join("session_crash_test.md");
        let initial_content = "# Documento Iniziale\n\nParagrafo originale.\n";
        accent_storage::atomic_save(&doc_path, initial_content).expect("save initial file");

        // 1. Start session, open file, edit text without saving, and trigger debounced WAL flush
        {
            let mut session = Session::with_recovery_dir(recovery_dir.clone());
            session.apply(Action::OpenFile(doc_path.clone()));
            assert!(!session.is_dirty);
            assert!(!session.recovered_from_crash);

            let edited = "# Documento Iniziale\n\nParagrafo originale.\nFrase scritta prima del crash improvviso";
            session.apply(Action::UpdateBufferText(edited.to_string()));
            session.apply(Action::MoveCursor(edited.chars().count()));
            assert!(session.is_dirty);
            assert!(session.journal_pending_flush);

            // Simulate elapsed time >= JOURNAL_DEBOUNCE_MS (< 300ms)
            let future = std::time::Instant::now()
                + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 20);
            assert!(session.poll_journal_flush(future));
            assert!(!session.journal_pending_flush);
            // Drop session without saving (simulating SIGKILL / power loss)
        }

        // Verify original file on disk was NOT touched before explicit save
        assert_eq!(
            std::fs::read_to_string(&doc_path).expect("read disk file"),
            initial_content
        );

        // 2. Restart session and open the same file -> must automatically restore from WAL
        let mut restarted = Session::with_recovery_dir(recovery_dir.clone());
        restarted.apply(Action::OpenFile(doc_path.clone()));
        assert!(restarted.is_dirty);
        assert!(restarted.recovered_from_crash);
        assert!(restarted.recovery_banner_active);
        assert!(restarted
            .buffer
            .text()
            .contains("Frase scritta prima del crash improvviso"));

        // 3. Confirm & Save -> atomically updates target file and removes recovery journal
        restarted.apply(Action::ConfirmRecoverySave);
        assert!(!restarted.is_dirty);
        assert!(!restarted.recovered_from_crash);
        assert!(!restarted.recovery_banner_active);
        assert!(
            accent_storage::find_recovery_journal_for_path(&recovery_dir, Some(&doc_path))
                .is_none()
        );
        assert!(std::fs::read_to_string(&doc_path)
            .expect("read saved file")
            .contains("Frase scritta prima del crash improvviso"));

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_persistent_settings_and_last_folder_across_restarts() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_settings_persist_test_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        let config_path = base_dir.join("config").join("settings.json");
        let docs_folder = base_dir.join("my_notes");
        std::fs::create_dir_all(&docs_folder).expect("create docs_folder");
        let note_path = docs_folder.join("appunti.md");
        accent_storage::atomic_save(&note_path, "# Appunti Persistenti\n\nTesto salvato.\n")
            .expect("write note_path");

        {
            let mut session =
                Session::with_config_and_recovery(config_path.clone(), recovery_dir.clone());
            // Open Preferences dialog, customize settings, open a file in docs_folder, and close Preferences
            session.apply(Action::SetOverlay(ActiveOverlay::PreferencesDialog));
            session.apply(Action::SetTheme(ThemeMode::Dark));
            session.apply(Action::SetPalettePreset(ColorPalettePreset::NordicForest));
            session.apply(Action::SetLocale(Locale::En));
            session.apply(Action::SetDialect(MarkdownDialect::Glfm));
            session.apply(Action::SetLineWidth(LineWidthMode::Wide));
            session.apply(Action::SetProseFont("Literata".to_string()));
            session.apply(Action::SetCodeFont("JetBrains Mono".to_string()));
            session.apply(Action::SetFontSize(19.5));
            session.apply(Action::SetLineHeight(1.65));
            session.apply(Action::ToggleHemingway);
            session.apply(Action::ToggleFocusMode);
            session.apply(Action::SetStatMetric(StatMetric::ReadingTime));
            // Close Preferences dialog -> triggers automatic save_settings()
            session.apply(Action::SetOverlay(ActiveOverlay::None));
            assert!(config_path.is_file());

            // Open file inside docs_folder -> updates file_dialog_dir, recent_files, current_file_path
            session.apply(Action::OpenFile(note_path.clone()));
            assert_eq!(session.file_dialog_dir, docs_folder);
        }

        // Restart a fresh Session pointing to the same config_path and verify everything persisted
        let restored = Session::with_config_and_recovery(config_path, recovery_dir);
        assert_eq!(restored.theme, ThemeMode::Dark);
        assert_eq!(restored.palette_preset, ColorPalettePreset::NordicForest);
        assert_eq!(restored.locale, Locale::En);
        assert_eq!(restored.dialect, MarkdownDialect::Glfm);
        assert_eq!(restored.line_width, LineWidthMode::Wide);
        assert_eq!(restored.typography.prose_font_family, "Literata");
        assert_eq!(restored.typography.code_font_family, "JetBrains Mono");
        assert!((restored.typography.font_size - 19.5).abs() < 1e-4);
        assert!((restored.typography.line_height - 1.65).abs() < 1e-4);
        assert!(restored.hemingway_mode);
        assert!(restored.focus_mode);
        assert_eq!(restored.stat_metric, StatMetric::ReadingTime);
        assert_eq!(restored.file_dialog_dir, docs_folder);
        assert_eq!(restored.current_file_path, Some(note_path.clone()));
        assert_eq!(restored.document_title, "appunti.md");
        assert!(restored.buffer.text().contains("Appunti Persistenti"));
        assert!(restored.recent_files.contains(&note_path));

        let _ = std::fs::remove_dir_all(&base_dir);
    }
}
