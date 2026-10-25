pub use accent_core::MarkdownDialect;
use accent_core::{DocumentBuffer, DocumentStats, DEFAULT_SAMPLE_MARKDOWN, TUTORIAL_MARKDOWN};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

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
    SetFileDialogDir(PathBuf),
    SetFileDialogInput(String),
    ExportDocument(String),
    InsertText(String),
    MoveCursor(usize),
}

#[derive(Debug, Clone)]
pub struct Session {
    pub document_title: String,
    pub current_file_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub recent_files: Vec<PathBuf>,
    pub file_dialog_dir: PathBuf,
    pub file_dialog_input: String,
    pub status_message: Option<String>,
    pub buffer: DocumentBuffer,
    pub theme: ThemeMode,
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
        Self {
            document_title: "esempio_accent.md".to_string(),
            current_file_path: Some(sample_path),
            is_dirty: false,
            recent_files,
            file_dialog_dir: cwd,
            file_dialog_input: "documento.md".to_string(),
            status_message: None,
            buffer: DocumentBuffer::default(),
            theme: ThemeMode::Sepia,
            locale: Locale::It,
            dialect: MarkdownDialect::Gfm,
            line_width: LineWidthMode::Compact,
            typography: TypographyConfig::default(),
            font_catalog: SystemFontCatalog::discover(),
            preview_active: false,
            preview_layout: PreviewLayout::HalfWidth,
            view_while_typing: false,
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
        }
    }
}

impl Session {
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::SetTheme(theme) => self.theme = theme,
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
                        self.display_word_count_override = None;
                        self.is_dirty = true;
                        self.buffer.set_text(&updated);
                    }
                }
            }
            Action::SetOverlay(overlay) => {
                self.chrome_autohidden = false;
                if self.overlay == overlay {
                    self.overlay = ActiveOverlay::None;
                } else {
                    self.overlay = overlay;
                }
            }
            Action::UpdateBufferText(new_text) => {
                if new_text != self.buffer.text() {
                    self.display_word_count_override = None;
                    self.is_dirty = true;
                    if self.overlay == ActiveOverlay::None && !self.search_bar_open {
                        self.chrome_autohidden = true;
                    }
                    self.buffer.set_text(&new_text);
                }
            }
            Action::SetSelection(primary, secondary) => {
                self.buffer.set_selection(primary, secondary);
            }
            Action::ApplyFormat(cmd) => {
                self.display_word_count_override = None;
                self.is_dirty = true;
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
                self.document_title = "untitled.md".to_string();
                self.current_file_path = None;
                self.is_dirty = false;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                self.buffer.set_text("# Nuovo Documento\n\nInizia a scrivere qui...\n");
                self.buffer.set_cursor(self.buffer.len_chars());
                self.overlay = ActiveOverlay::None;
            }
            Action::OpenFile(path) => {
                if let Ok(content) = std::fs::read_to_string(&path) {
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
                    self.is_dirty = false;
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
                self.document_title = "tutorial_accent.md".to_string();
                self.current_file_path = Some(tutorial_path);
                self.is_dirty = false;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                self.buffer.set_text(TUTORIAL_MARKDOWN);
                self.buffer.set_cursor(0);
                self.status_message = Some("Tutorial caricato".to_string());
                self.overlay = ActiveOverlay::None;
            }
            Action::SaveDocument => {
                let target = self
                    .current_file_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("/tmp/accent_autosave.md"));
                if accent_storage::atomic_save(&target, &self.buffer.text()).is_ok() {
                    self.is_dirty = false;
                    self.current_file_path = Some(target.clone());
                    self.status_message = Some(format!("Salvato in {}", target.display()));
                }
            }
            Action::SaveAsFile(path) => {
                if accent_storage::atomic_save(&path, &self.buffer.text()).is_ok() {
                    let title = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("documento.md")
                        .to_string();
                    self.document_title = title;
                    self.current_file_path = Some(path.clone());
                    self.is_dirty = false;
                    self.recent_files.retain(|p| p != &path);
                    self.recent_files.insert(0, path.clone());
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
            Action::ExportDocument(fmt) => {
                let html = self.buffer.render_html(self.dialect);
                let out_path = PathBuf::from(format!(
                    "/tmp/{}.{}",
                    self.document_title.trim_end_matches(".md"),
                    fmt.to_ascii_lowercase()
                ));
                let _ = accent_storage::atomic_save(&out_path, &html);
                self.status_message = Some(format!("Esportato in {}", out_path.display()));
                self.overlay = ActiveOverlay::None;
            }
            Action::InsertText(text) => {
                self.display_word_count_override = None;
                self.is_dirty = true;
                if self.overlay == ActiveOverlay::None && !self.search_bar_open {
                    self.chrome_autohidden = true;
                }
                self.buffer.insert_text(&text);
            }
            Action::MoveCursor(offset) => {
                self.buffer.set_cursor(offset);
            }
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
}
