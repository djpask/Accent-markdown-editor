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
    /// Default: Amber & Coral (mdfried-inspired triadic Gold + Coral + Periwinkle palette)
    #[default]
    AmberCoral,
    /// Editorial Paper & Slate (Alabaster Paper, Warm Book Paper, Midnight Slate)
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

            (Locale::It, "fullscreen_mode") => "Schermo intero",
            (Locale::En, "fullscreen_mode") => "Full Screen",
            (Locale::Es, "fullscreen_mode") => "Pantalla completa",

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

            (Locale::It, "tooltip_search") => "Trova e sostituisci nel documento (Ctrl+F)",
            (Locale::En, "tooltip_search") => "Find and replace in document (Ctrl+F)",
            (Locale::Es, "tooltip_search") => "Buscar y reemplazar en el documento (Ctrl+F)",

            (Locale::It, "tooltip_preview") => "Modalità editor e pannello di anteprima",
            (Locale::En, "tooltip_preview") => "Editor modes and preview panel",
            (Locale::Es, "tooltip_preview") => "Modos de editor y panel de vista previa",

            (Locale::It, "tooltip_main_menu") => "Menu principale, temi e preferenze",
            (Locale::En, "tooltip_main_menu") => "Main menu, themes and preferences",
            (Locale::Es, "tooltip_main_menu") => "Menú principal, temas y preferencias",

            (Locale::It, "tooltip_new_tab") => "Nuova scheda (Ctrl+T)",
            (Locale::En, "tooltip_new_tab") => "New tab (Ctrl+T)",
            (Locale::Es, "tooltip_new_tab") => "Nueva pestaña (Ctrl+T)",

            (Locale::It, "tooltip_close_tab") => "Chiudi scheda (Ctrl+W)",
            (Locale::En, "tooltip_close_tab") => "Close tab (Ctrl+W)",
            (Locale::Es, "tooltip_close_tab") => "Cerrar pestaña (Ctrl+W)",

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

            // External File Modification on Disk Banner (it, en, es)
            (Locale::It, "disk_mod_banner_msg") => "⟳ Il file sul disco è stato modificato da un'altra applicazione. Vuoi ricaricarlo?",
            (Locale::En, "disk_mod_banner_msg") => "⟳ The file on disk was modified by another application. Reload it?",
            (Locale::Es, "disk_mod_banner_msg") => "⟳ El archivo en disco fue modificado por otra aplicación. ¿Recargarlo?",

            (Locale::It, "disk_mod_banner_dirty_msg") => "⚠ Il file sul disco è stato modificato esternamente (hai modifiche non salvate). Ricaricare dal file?",
            (Locale::En, "disk_mod_banner_dirty_msg") => "⚠ The file on disk was modified externally (you have unsaved edits). Reload from file?",
            (Locale::Es, "disk_mod_banner_dirty_msg") => "⚠ El archivo en disco fue modificado externamente (tienes cambios sin guardar). ¿Recargar?",

            (Locale::It, "disk_mod_reload") => "Ricarica dal file",
            (Locale::En, "disk_mod_reload") => "Reload from File",
            (Locale::Es, "disk_mod_reload") => "Recargar archivo",

            (Locale::It, "disk_mod_ignore") => "Ignora",
            (Locale::En, "disk_mod_ignore") => "Ignore",
            (Locale::Es, "disk_mod_ignore") => "Ignorar",

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

pub const DEFAULT_FONT_SIZE: f32 = 17.35;
pub const MIN_FONT_SIZE: f32 = 10.0;
pub const MAX_FONT_SIZE: f32 = 36.0;

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
            font_size: DEFAULT_FONT_SIZE,
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
        static CATALOG_CACHE: std::sync::OnceLock<SystemFontCatalog> = std::sync::OnceLock::new();
        CATALOG_CACHE.get_or_init(Self::discover_uncached).clone()
    }

    fn discover_uncached() -> Self {
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
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomByStep(f32),
    SetPreviewLayout(PreviewLayout),
    SetPreviewMode { active: bool, layout: PreviewLayout },
    SetEditorOnlyRaw,
    SetEditorLivePreview,
    SetStatMetric(StatMetric),
    ToggleSplitPreview,
    ToggleViewWhileTyping,
    SetViewWhileTyping(bool),
    ToggleHemingway,
    NotifyHemingwayBlocked,
    ToggleFocusMode,
    ToggleFullscreen,
    SetFullscreen(bool),
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
    NewTab,
    SwitchTab(usize),
    CloseTab(usize),
    NextTab,
    PrevTab,
    Undo,
    Redo,
    OpenFile(PathBuf),
    OpenTutorial,
    SaveDocument,
    SaveAsFile(PathBuf),
    ConfirmRecoverySave,
    DiscardRecovery,
    CheckExternalFileModification,
    ReloadFromDisk,
    DismissExternalModification,
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
    pub open_tab_files: Vec<PathBuf>,
    pub active_tab_index: usize,
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
            open_tab_files: Vec::new(),
            active_tab_index: 0,
        }
    }
}

/// Debounce threshold for *Save-while-typing* WAL journaling (`< 300ms` per `SPECS.md` Pillar 8).
pub const JOURNAL_DEBOUNCE_MS: u64 = 200;

/// Represents an open document tab with its own isolated buffer, undo/redo stack, cursor, dirty state, and WAL metadata.
#[derive(Debug, Clone)]
pub struct DocumentTab {
    pub tab_id: u64,
    pub document_title: String,
    pub current_file_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub original_file_hash: Option<String>,
    pub last_edit_instant: Option<std::time::Instant>,
    pub last_edit_utc_ms: u64,
    pub journal_pending_flush: bool,
    pub recovered_from_crash: bool,
    pub recovery_banner_active: bool,
    pub disk_modified_banner_active: bool,
    pub dismissed_disk_hash: Option<String>,
    pub buffer: DocumentBuffer,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub tabs: Vec<DocumentTab>,
    pub active_tab_index: usize,
    pub next_tab_id: u64,
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
    pub disk_modified_banner_active: bool,
    pub dismissed_disk_hash: Option<String>,
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
    pub fullscreen: bool,
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
            static TEST_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let seq = TEST_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let test_dir = std::env::temp_dir().join(format!(
                "accent_unit_test_default_{}_{seq}",
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
        let initial_tab = DocumentTab {
            tab_id: 1,
            document_title: "esempio_accent.md".to_string(),
            current_file_path: Some(sample_path.clone()),
            is_dirty: false,
            original_file_hash: initial_hash.clone(),
            last_edit_instant: None,
            last_edit_utc_ms: 0,
            journal_pending_flush: false,
            recovered_from_crash: false,
            recovery_banner_active: false,
            disk_modified_banner_active: false,
            dismissed_disk_hash: None,
            buffer: default_buf.clone(),
        };
        let mut session = Self {
            tabs: vec![initial_tab],
            active_tab_index: 0,
            next_tab_id: 2,
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
            disk_modified_banner_active: false,
            dismissed_disk_hash: None,
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
            fullscreen: false,
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
        session.sync_active_tab_to_slot();
        session
    }

    /// Synchronizes the top-level active document fields of `Session` into `self.tabs[self.active_tab_index]`.
    pub fn sync_active_tab_to_slot(&mut self) {
        if self.tabs.is_empty() {
            let id = self.next_tab_id;
            self.next_tab_id += 1;
            self.tabs.push(DocumentTab {
                tab_id: id,
                document_title: self.document_title.clone(),
                current_file_path: self.current_file_path.clone(),
                is_dirty: self.is_dirty,
                original_file_hash: self.original_file_hash.clone(),
                last_edit_instant: self.last_edit_instant,
                last_edit_utc_ms: self.last_edit_utc_ms,
                journal_pending_flush: self.journal_pending_flush,
                recovered_from_crash: self.recovered_from_crash,
                recovery_banner_active: self.recovery_banner_active,
                disk_modified_banner_active: self.disk_modified_banner_active,
                dismissed_disk_hash: self.dismissed_disk_hash.clone(),
                buffer: self.buffer.clone(),
            });
            self.active_tab_index = 0;
            return;
        }
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        let slot = &mut self.tabs[self.active_tab_index];
        if slot.document_title != self.document_title {
            slot.document_title = self.document_title.clone();
        }
        if slot.current_file_path != self.current_file_path {
            slot.current_file_path = self.current_file_path.clone();
        }
        slot.is_dirty = self.is_dirty;
        if slot.original_file_hash != self.original_file_hash {
            slot.original_file_hash = self.original_file_hash.clone();
        }
        slot.last_edit_instant = self.last_edit_instant;
        slot.last_edit_utc_ms = self.last_edit_utc_ms;
        slot.journal_pending_flush = self.journal_pending_flush;
        slot.recovered_from_crash = self.recovered_from_crash;
        slot.recovery_banner_active = self.recovery_banner_active;
        slot.disk_modified_banner_active = self.disk_modified_banner_active;
        if slot.dismissed_disk_hash != self.dismissed_disk_hash {
            slot.dismissed_disk_hash = self.dismissed_disk_hash.clone();
        }
        if !std::ptr::eq(slot.buffer.as_str(), self.buffer.as_str())
            || slot.buffer.revision() != self.buffer.revision()
        {
            slot.buffer = self.buffer.clone();
        } else {
            let (p, s) = self.buffer.raw_selection();
            slot.buffer.set_selection(p, s);
        }
    }

    /// Loads `self.tabs[self.active_tab_index]` into the top-level active document fields of `Session`.
    pub fn load_active_tab_from_slot(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        let slot = &self.tabs[self.active_tab_index];
        self.document_title = slot.document_title.clone();
        self.current_file_path = slot.current_file_path.clone();
        self.is_dirty = slot.is_dirty;
        self.original_file_hash = slot.original_file_hash.clone();
        self.last_edit_instant = slot.last_edit_instant;
        self.last_edit_utc_ms = slot.last_edit_utc_ms;
        self.journal_pending_flush = slot.journal_pending_flush;
        self.recovered_from_crash = slot.recovered_from_crash;
        self.recovery_banner_active = slot.recovery_banner_active;
        self.disk_modified_banner_active = slot.disk_modified_banner_active;
        self.dismissed_disk_hash = slot.dismissed_disk_hash.clone();
        self.buffer = slot.buffer.clone();
        self.file_dialog_input = self.document_title.clone();
        self.display_word_count_override = None;
        self.update_default_export_path();
    }

    /// Checks whether the file on disk for the currently active tab has been modified externally
    /// compared to `self.original_file_hash` (and `self.dismissed_disk_hash`).
    pub fn check_active_tab_disk_modification(&mut self) -> bool {
        let Some(ref path) = self.current_file_path else {
            self.disk_modified_banner_active = false;
            return false;
        };
        let Ok(disk_content) = std::fs::read_to_string(path) else {
            return false;
        };
        let disk_hash = accent_storage::compute_content_hash(&disk_content);
        let buf_hash = accent_storage::compute_content_hash(&self.buffer.text());

        if self.original_file_hash.as_deref() == Some(disk_hash.as_str()) || disk_hash == buf_hash {
            if self.disk_modified_banner_active {
                self.disk_modified_banner_active = false;
            }
            if !self.is_dirty {
                self.original_file_hash = Some(disk_hash);
            }
            return false;
        }

        if self.dismissed_disk_hash.as_deref() == Some(disk_hash.as_str()) {
            return false;
        }

        self.disk_modified_banner_active = true;
        self.chrome_autohidden = false;
        let msg_key = if self.is_dirty {
            "disk_mod_banner_dirty_msg"
        } else {
            "disk_mod_banner_msg"
        };
        self.status_message = Some(I18n::tr(self.locale, msg_key).to_string());
        true
    }

    /// Returns the current typography zoom scale factor relative to `DEFAULT_FONT_SIZE` (`1.0` = 100%).
    pub fn zoom_scale(&self) -> f32 {
        (self.typography.font_size / DEFAULT_FONT_SIZE).clamp(0.5, 2.5)
    }

    /// Returns the current typography zoom percentage (e.g. `100`, `115`, `85`).
    pub fn zoom_percentage(&self) -> u32 {
        ((self.typography.font_size / DEFAULT_FONT_SIZE) * 100.0).round() as u32
    }

    fn next_untitled_name(&self) -> String {
        if !self.tabs.iter().any(|t| t.document_title == "untitled.md") {
            return "untitled.md".to_string();
        }
        let mut n = 2usize;
        loop {
            let candidate = format!("untitled-{n}.md");
            if !self.tabs.iter().any(|t| t.document_title == candidate) {
                return candidate;
            }
            n += 1;
        }
    }

    fn paths_match(a: &std::path::Path, b: &std::path::Path) -> bool {
        accent_storage::normalize_path(a) == accent_storage::normalize_path(b)
    }

    fn is_single_untouched_placeholder(&self) -> bool {
        self.tabs.len() == 1
            && !self.is_dirty
            && !self.tabs[0].is_dirty
            && (self.current_file_path.is_none()
                || self.current_file_path.as_deref()
                    == Some(std::path::Path::new("/tmp/esempio_accent.md")))
    }

    fn apply_journal_to_tab(tab: &mut DocumentTab, entry: accent_storage::RecoveryJournalEntry) {
        let title = entry
            .target_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| entry.document_title.clone());
        let mut restored_buf = DocumentBuffer::from_str_content(&entry.content);
        restored_buf.set_cursor(entry.cursor_char_offset.min(restored_buf.len_chars()));
        tab.document_title = title;
        tab.current_file_path = entry.target_path;
        tab.original_file_hash = entry.original_file_hash;
        tab.is_dirty = true;
        tab.journal_pending_flush = false;
        tab.recovered_from_crash = true;
        tab.recovery_banner_active = true;
        tab.buffer = restored_buf;
    }

    /// Snapshots all user preferences, recent files, and open tabs into `PersistedSettings`.
    pub fn to_persisted_settings(&self) -> PersistedSettings {
        let mut open_tab_files = Vec::new();
        for (idx, tab) in self.tabs.iter().enumerate() {
            let path_opt = if idx == self.active_tab_index {
                self.current_file_path.as_ref()
            } else {
                tab.current_file_path.as_ref()
            };
            if let Some(p) = path_opt {
                if !open_tab_files
                    .iter()
                    .any(|existing: &PathBuf| Self::paths_match(existing, p))
                {
                    open_tab_files.push(p.clone());
                }
            }
        }
        let active_tab_index = self
            .current_file_path
            .as_ref()
            .and_then(|cur| open_tab_files.iter().position(|p| Self::paths_match(p, cur)))
            .unwrap_or(0);

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
            hemingway_mode: false,
            focus_mode: self.focus_mode,
            toolbar_expanded: self.toolbar_expanded,
            stat_metric: self.stat_metric,
            selected_export_format: self.selected_export_format,
            export_syntax_highlight: self.export_syntax_highlight,
            last_opened_dir: Some(self.file_dialog_dir.clone()),
            last_opened_file: self.current_file_path.clone(),
            recent_files: self.recent_files.clone(),
            open_tab_files,
            active_tab_index,
        }
    }

    /// Atomically persists `PersistedSettings` to `self.config_path` (`settings.json`).
    pub fn save_settings(&self) -> std::io::Result<()> {
        let settings = self.to_persisted_settings();
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        accent_storage::atomic_save(&self.config_path, &json)
    }

    /// Loads persisted settings, open tabs, and last opened directory/file from `self.config_path` if present.
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
        if !self.typography.font_size.is_finite() {
            self.typography.font_size = DEFAULT_FONT_SIZE;
        }
        self.typography.font_size = self
            .typography
            .font_size
            .clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
        self.typography.line_height = self.typography.line_height.clamp(1.1, 2.2);
        self.preview_active = settings.preview_active;
        self.preview_layout = settings.preview_layout;
        self.view_while_typing = settings.view_while_typing;
        // Hemingway mode is a per-session writing mode and must never start enabled across restarts
        // so the user is never unexpectedly locked out of deleting text.
        self.hemingway_mode = false;
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

        // Restore open tabs if present, otherwise fall back to `last_opened_file`
        let mut restored_tabs: Vec<DocumentTab> = Vec::new();
        for tab_path in &settings.open_tab_files {
            if restored_tabs.iter().any(|t| {
                t.current_file_path
                    .as_deref()
                    .is_some_and(|p| Self::paths_match(p, tab_path))
            }) {
                continue;
            }
            if tab_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(tab_path) {
                    let title = tab_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("documento.md")
                        .to_string();
                    let mut buf = DocumentBuffer::from_str_content(&content);
                    buf.set_cursor(0);
                    let id = self.next_tab_id;
                    self.next_tab_id += 1;
                    restored_tabs.push(DocumentTab {
                        tab_id: id,
                        document_title: title,
                        current_file_path: Some(tab_path.clone()),
                        is_dirty: false,
                        original_file_hash: Some(accent_storage::compute_content_hash(&content)),
                        last_edit_instant: None,
                        last_edit_utc_ms: 0,
                        journal_pending_flush: false,
                        recovered_from_crash: false,
                        recovery_banner_active: false,
                        disk_modified_banner_active: false,
                        dismissed_disk_hash: None,
                        buffer: buf,
                    });
                }
            }
        }

        if !restored_tabs.is_empty() {
            self.tabs = restored_tabs;
            self.active_tab_index = settings
                .active_tab_index
                .min(self.tabs.len().saturating_sub(1));
            self.load_active_tab_from_slot();
        } else if let Some(last_file) = settings.last_opened_file {
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
                        if parent.is_dir()
                            && self.file_dialog_dir
                                == std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                        {
                            self.file_dialog_dir = parent.to_path_buf();
                        }
                    }
                    self.current_file_path = Some(last_file);
                    self.original_file_hash = Some(accent_storage::compute_content_hash(&content));
                    let mut fresh_buf = DocumentBuffer::from_str_content(&content);
                    fresh_buf.set_cursor(0);
                    self.buffer = fresh_buf;
                    self.sync_active_tab_to_slot();
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

    /// Checks whether the `< 300ms` debounce window has elapsed and flushes the journal for any dirty tab.
    pub fn poll_journal_flush(&mut self, now: std::time::Instant) -> bool {
        let has_pending = (self.is_dirty && self.journal_pending_flush)
            || self
                .tabs
                .iter()
                .any(|t| t.is_dirty && t.journal_pending_flush);
        if !has_pending {
            return false;
        }
        self.sync_active_tab_to_slot();
        let debounce = std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS);
        let mut any_flushed = false;
        for tab in &mut self.tabs {
            if !tab.is_dirty || !tab.journal_pending_flush {
                continue;
            }
            let ready = match tab.last_edit_instant {
                Some(last) => now.saturating_duration_since(last) >= debounce,
                None => true,
            };
            if ready {
                let cur_char = tab.buffer.cursor();
                let cur_byte = tab.buffer.char_to_byte(cur_char);
                let entry = accent_storage::RecoveryJournalEntry::new(
                    tab.current_file_path.clone(),
                    tab.document_title.clone(),
                    tab.buffer.text(),
                    cur_char,
                    cur_byte,
                    tab.original_file_hash.clone(),
                    tab.is_dirty,
                );
                if accent_storage::write_journal_entry(&self.recovery_dir, &entry).is_ok() {
                    tab.journal_pending_flush = false;
                    any_flushed = true;
                }
            }
        }
        if self.active_tab_index < self.tabs.len() {
            self.journal_pending_flush = self.tabs[self.active_tab_index].journal_pending_flush;
        }
        any_flushed
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
        let title = entry
            .target_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| entry.document_title.clone());
        self.document_title = title.clone();
        self.file_dialog_input = title;
        if let Some(ref target) = entry.target_path {
            if let Some(parent) = target.parent() {
                if parent.is_dir() {
                    self.file_dialog_dir = parent.to_path_buf();
                }
            }
        }
        self.current_file_path = entry.target_path;
        self.original_file_hash = entry.original_file_hash;
        self.display_word_count_override = None;
        let mut restored_buf = DocumentBuffer::from_str_content(&entry.content);
        restored_buf.set_cursor(entry.cursor_char_offset.min(restored_buf.len_chars()));
        self.buffer = restored_buf;
        self.is_dirty = true;
        self.journal_pending_flush = false;
        self.recovered_from_crash = true;
        self.recovery_banner_active = true;
        self.chrome_autohidden = false;
        self.update_default_export_path();
        self.status_message = Some(I18n::tr(self.locale, "recovery_banner_msg").to_string());
    }

    /// Scans `self.recovery_dir` on startup to automatically restore any unclosed dirty session
    /// into its matching tab without ever overwriting a different document's tab.
    pub fn check_startup_crash_recovery(&mut self) -> bool {
        self.sync_active_tab_to_slot();
        let entries = accent_storage::scan_recovery_journals(&self.recovery_dir);
        if entries.is_empty() {
            return false;
        }

        let mut any_recovered = false;
        for mut entry in entries {
            if let Some(ref target_path) = entry.target_path {
                if target_path.is_file() {
                    let disk_content = std::fs::read_to_string(target_path).ok();
                    let disk_hash = disk_content
                        .as_deref()
                        .map(accent_storage::compute_content_hash);
                    if entry.original_file_hash.is_none() {
                        entry.original_file_hash = disk_hash.clone();
                    }
                    let entry_hash = accent_storage::compute_content_hash(&entry.content);
                    let matches_disk = disk_hash
                        .as_deref()
                        .is_some_and(|dh| dh == entry_hash.as_str())
                        || disk_content
                            .as_deref()
                            .is_some_and(|dc| dc == entry.content.as_str());
                    if matches_disk {
                        let _ = accent_storage::remove_journal_for_target(
                            &self.recovery_dir,
                            Some(target_path),
                        );
                        continue;
                    }
                } else {
                    let in_open_tabs = self.tabs.iter().any(|t| {
                        t.current_file_path
                            .as_deref()
                            .is_some_and(|p| Self::paths_match(p, target_path))
                    });
                    if !in_open_tabs {
                        let _ = accent_storage::remove_journal_for_target(
                            &self.recovery_dir,
                            Some(target_path),
                        );
                        continue;
                    }
                }
            }

            let matching_idx = match entry.target_path.as_deref() {
                Some(target) => self.tabs.iter().position(|t| {
                    t.current_file_path
                        .as_deref()
                        .is_some_and(|p| Self::paths_match(p, target))
                }),
                None => self
                    .tabs
                    .iter()
                    .position(|t| t.current_file_path.is_none() && !t.is_dirty),
            };

            if let Some(idx) = matching_idx {
                Self::apply_journal_to_tab(&mut self.tabs[idx], entry);
                any_recovered = true;
            } else if self.is_single_untouched_placeholder() {
                Self::apply_journal_to_tab(&mut self.tabs[0], entry);
                self.active_tab_index = 0;
                any_recovered = true;
            }
        }

        if any_recovered {
            self.load_active_tab_from_slot();
            if self.recovery_banner_active {
                self.status_message =
                    Some(I18n::tr(self.locale, "recovery_banner_msg").to_string());
            }
        }
        any_recovered
    }

    /// Cleans up the recovery journal and persists user settings when the application closes normally.
    pub fn on_clean_exit(&mut self) {
        self.sync_active_tab_to_slot();
        let _ = self.save_settings();
        for tab in &mut self.tabs {
            let current_hash = accent_storage::compute_content_hash(&tab.buffer.text());
            if !tab.recovered_from_crash
                && tab.original_file_hash.as_deref() == Some(current_hash.as_str())
            {
                tab.is_dirty = false;
                tab.journal_pending_flush = false;
            }
            if tab.is_dirty {
                if tab.journal_pending_flush {
                    let cur_char = tab.buffer.cursor();
                    let cur_byte = tab.buffer.char_to_byte(cur_char);
                    let entry = accent_storage::RecoveryJournalEntry::new(
                        tab.current_file_path.clone(),
                        tab.document_title.clone(),
                        tab.buffer.text(),
                        cur_char,
                        cur_byte,
                        tab.original_file_hash.clone(),
                        tab.is_dirty,
                    );
                    let _ = accent_storage::write_journal_entry(&self.recovery_dir, &entry);
                    tab.journal_pending_flush = false;
                }
            } else {
                let _ = accent_storage::remove_journal_for_target(
                    &self.recovery_dir,
                    tab.current_file_path.as_deref(),
                );
            }
        }
        // Remove any orphan journal in `self.recovery_dir` not belonging to an open dirty tab
        for entry in accent_storage::scan_recovery_journals(&self.recovery_dir) {
            let belongs_to_dirty_tab = self.tabs.iter().any(|t| {
                t.is_dirty
                    && match (&t.current_file_path, &entry.target_path) {
                        (Some(a), Some(b)) => Self::paths_match(a, b),
                        (None, None) => true,
                        _ => false,
                    }
            });
            if !belongs_to_dirty_tab {
                let _ = accent_storage::remove_journal_for_target(
                    &self.recovery_dir,
                    entry.target_path.as_deref(),
                );
            }
        }
        if self.active_tab_index < self.tabs.len() {
            self.journal_pending_flush = self.tabs[self.active_tab_index].journal_pending_flush;
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
                | Action::ZoomIn
                | Action::ZoomOut
                | Action::ZoomReset
                | Action::ZoomByStep(_)
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
                | Action::NewDocument
                | Action::NewTab
                | Action::SwitchTab(_)
                | Action::CloseTab(_)
                | Action::NextTab
                | Action::PrevTab
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
            Action::SetFontSize(size) => {
                if size.is_finite() {
                    self.typography.font_size = size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                }
            }
            Action::SetLineHeight(lh) => {
                if lh.is_finite() {
                    self.typography.line_height = lh.clamp(1.1, 2.2);
                }
            }
            Action::ZoomIn => {
                self.typography.font_size =
                    (self.typography.font_size + 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                self.status_message = Some(format!(
                    "Zoom: {}% ({:.1} pt)",
                    self.zoom_percentage(),
                    self.typography.font_size
                ));
            }
            Action::ZoomOut => {
                self.typography.font_size =
                    (self.typography.font_size - 1.0).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                self.status_message = Some(format!(
                    "Zoom: {}% ({:.1} pt)",
                    self.zoom_percentage(),
                    self.typography.font_size
                ));
            }
            Action::ZoomReset => {
                self.typography.font_size = DEFAULT_FONT_SIZE;
                self.status_message = Some(format!(
                    "Zoom: 100% ({:.1} pt)",
                    self.typography.font_size
                ));
            }
            Action::ZoomByStep(delta) => {
                if delta.is_finite() && delta.abs() > 1e-4 {
                    self.typography.font_size =
                        (self.typography.font_size + delta).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
                    self.status_message = Some(format!(
                        "Zoom: {}% ({:.1} pt)",
                        self.zoom_percentage(),
                        self.typography.font_size
                    ));
                }
            }
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
            Action::ToggleHemingway => {
                self.hemingway_mode = !self.hemingway_mode;
                self.chrome_autohidden = false;
                self.status_message = Some(if self.hemingway_mode {
                    "Modalità Hemingway attiva (cancellazione disabilitata)".to_string()
                } else {
                    "Modalità Hemingway disattivata".to_string()
                });
            }
            Action::NotifyHemingwayBlocked => {
                self.chrome_autohidden = false;
                self.status_message = Some(
                    "Modalità Hemingway attiva: cancellazione bloccata".to_string(),
                );
            }
            Action::ToggleFocusMode => self.focus_mode = !self.focus_mode,
            Action::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
                self.chrome_autohidden = false;
                self.overlay = ActiveOverlay::None;
            }
            Action::SetFullscreen(fs) => {
                self.fullscreen = fs;
                self.chrome_autohidden = false;
            }
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
                    if self.overlay == ActiveOverlay::None
                        && !self.search_bar_open
                        && !self.recovery_banner_active
                        && !self.disk_modified_banner_active
                    {
                        self.chrome_autohidden = true;
                    }
                    self.buffer.set_text(&new_text);
                    let new_hash = accent_storage::compute_content_hash(&new_text);
                    if !self.recovered_from_crash
                        && self.original_file_hash.as_deref() == Some(new_hash.as_str())
                    {
                        self.display_word_count_override = None;
                        self.is_dirty = false;
                        self.journal_pending_flush = false;
                        let _ = accent_storage::remove_journal_for_target(
                            &self.recovery_dir,
                            self.current_file_path.as_deref(),
                        );
                    } else {
                        self.mark_dirty_edit();
                    }
                }
            }
            Action::SetSelection(primary, secondary) => {
                self.buffer.set_selection(primary, secondary);
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
            Action::NewDocument | Action::NewTab => {
                self.sync_active_tab_to_slot();
                let _ = accent_storage::remove_journal_for_target(&self.recovery_dir, None);
                let title = self.next_untitled_name();
                let init_txt = "# Nuovo Documento\n\nInizia a scrivere qui...\n";
                let mut new_buf = DocumentBuffer::from_str_content(init_txt);
                new_buf.set_cursor(new_buf.len_chars());
                let id = self.next_tab_id;
                self.next_tab_id += 1;
                let new_tab = DocumentTab {
                    tab_id: id,
                    document_title: title,
                    current_file_path: None,
                    is_dirty: false,
                    original_file_hash: Some(accent_storage::compute_content_hash(init_txt)),
                    last_edit_instant: None,
                    last_edit_utc_ms: 0,
                    journal_pending_flush: false,
                    recovered_from_crash: false,
                    recovery_banner_active: false,
                    disk_modified_banner_active: false,
                    dismissed_disk_hash: None,
                    buffer: new_buf,
                };
                self.tabs.push(new_tab);
                self.active_tab_index = self.tabs.len() - 1;
                self.load_active_tab_from_slot();
                self.status_message = None;
                self.chrome_autohidden = false;
                self.search_bar_open = false;
                self.overlay = ActiveOverlay::None;
            }
            Action::SwitchTab(idx) => {
                if idx < self.tabs.len() && idx != self.active_tab_index {
                    self.sync_active_tab_to_slot();
                    self.active_tab_index = idx;
                    self.load_active_tab_from_slot();
                    self.check_active_tab_disk_modification();
                    self.status_message = if self.recovery_banner_active {
                        Some(I18n::tr(self.locale, "recovery_banner_msg").to_string())
                    } else if self.disk_modified_banner_active {
                        let key = if self.is_dirty {
                            "disk_mod_banner_dirty_msg"
                        } else {
                            "disk_mod_banner_msg"
                        };
                        Some(I18n::tr(self.locale, key).to_string())
                    } else {
                        None
                    };
                    self.chrome_autohidden = false;
                    self.overlay = ActiveOverlay::None;
                }
            }
            Action::NextTab => {
                if self.tabs.len() > 1 {
                    self.sync_active_tab_to_slot();
                    self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
                    self.load_active_tab_from_slot();
                    self.check_active_tab_disk_modification();
                    self.status_message = if self.recovery_banner_active {
                        Some(I18n::tr(self.locale, "recovery_banner_msg").to_string())
                    } else if self.disk_modified_banner_active {
                        let key = if self.is_dirty {
                            "disk_mod_banner_dirty_msg"
                        } else {
                            "disk_mod_banner_msg"
                        };
                        Some(I18n::tr(self.locale, key).to_string())
                    } else {
                        None
                    };
                    self.chrome_autohidden = false;
                    self.overlay = ActiveOverlay::None;
                }
            }
            Action::PrevTab => {
                if self.tabs.len() > 1 {
                    self.sync_active_tab_to_slot();
                    self.active_tab_index =
                        (self.active_tab_index + self.tabs.len() - 1) % self.tabs.len();
                    self.load_active_tab_from_slot();
                    self.check_active_tab_disk_modification();
                    self.status_message = if self.recovery_banner_active {
                        Some(I18n::tr(self.locale, "recovery_banner_msg").to_string())
                    } else if self.disk_modified_banner_active {
                        let key = if self.is_dirty {
                            "disk_mod_banner_dirty_msg"
                        } else {
                            "disk_mod_banner_msg"
                        };
                        Some(I18n::tr(self.locale, key).to_string())
                    } else {
                        None
                    };
                    self.chrome_autohidden = false;
                    self.overlay = ActiveOverlay::None;
                }
            }
            Action::CloseTab(idx) => {
                if idx < self.tabs.len() {
                    self.sync_active_tab_to_slot();
                    let _ = accent_storage::remove_journal_for_target(
                        &self.recovery_dir,
                        self.tabs[idx].current_file_path.as_deref(),
                    );
                    if self.tabs.len() <= 1 {
                        let init_txt = "# Nuovo Documento\n\nInizia a scrivere qui...\n";
                        let mut new_buf = DocumentBuffer::from_str_content(init_txt);
                        new_buf.set_cursor(new_buf.len_chars());
                        let id = self.next_tab_id;
                        self.next_tab_id += 1;
                        self.tabs[0] = DocumentTab {
                            tab_id: id,
                            document_title: "untitled.md".to_string(),
                            current_file_path: None,
                            is_dirty: false,
                            original_file_hash: Some(accent_storage::compute_content_hash(
                                init_txt,
                            )),
                            last_edit_instant: None,
                            last_edit_utc_ms: 0,
                            journal_pending_flush: false,
                            recovered_from_crash: false,
                            recovery_banner_active: false,
                            disk_modified_banner_active: false,
                            dismissed_disk_hash: None,
                            buffer: new_buf,
                        };
                        self.active_tab_index = 0;
                    } else {
                        self.tabs.remove(idx);
                        if idx < self.active_tab_index {
                            self.active_tab_index -= 1;
                        } else if self.active_tab_index >= self.tabs.len() {
                            self.active_tab_index = self.tabs.len() - 1;
                        }
                    }
                    self.load_active_tab_from_slot();
                    self.check_active_tab_disk_modification();
                    self.status_message = if self.recovery_banner_active {
                        Some(I18n::tr(self.locale, "recovery_banner_msg").to_string())
                    } else if self.disk_modified_banner_active {
                        let key = if self.is_dirty {
                            "disk_mod_banner_dirty_msg"
                        } else {
                            "disk_mod_banner_msg"
                        };
                        Some(I18n::tr(self.locale, key).to_string())
                    } else {
                        None
                    };
                    self.chrome_autohidden = false;
                }
            }
            Action::Undo => {
                if self.buffer.undo() {
                    let current_hash = accent_storage::compute_content_hash(&self.buffer.text());
                    self.is_dirty =
                        self.original_file_hash.as_deref() != Some(current_hash.as_str());
                    self.journal_pending_flush = self.is_dirty;
                    if !self.is_dirty {
                        let _ = accent_storage::remove_journal_for_target(
                            &self.recovery_dir,
                            self.current_file_path.as_deref(),
                        );
                    }
                    self.last_edit_instant = Some(std::time::Instant::now());
                }
            }
            Action::Redo => {
                if self.buffer.redo() {
                    let current_hash = accent_storage::compute_content_hash(&self.buffer.text());
                    self.is_dirty =
                        self.original_file_hash.as_deref() != Some(current_hash.as_str());
                    self.journal_pending_flush = self.is_dirty;
                    if !self.is_dirty {
                        let _ = accent_storage::remove_journal_for_target(
                            &self.recovery_dir,
                            self.current_file_path.as_deref(),
                        );
                    }
                    self.last_edit_instant = Some(std::time::Instant::now());
                }
            }
            Action::OpenFile(path) => {
                self.sync_active_tab_to_slot();
                let disk_content = std::fs::read_to_string(&path).ok();
                let disk_hash = disk_content
                    .as_deref()
                    .map(accent_storage::compute_content_hash);

                // 1. Check if `path` is already open in an existing tab (normalized deduplication).
                // Never re-run crash recovery on an already-open tab, as its live state is already in memory.
                if let Some(existing_idx) = self.tabs.iter().position(|t| {
                    t.current_file_path
                        .as_deref()
                        .is_some_and(|p| Self::paths_match(p, &path))
                }) {
                    self.active_tab_index = existing_idx;
                    self.load_active_tab_from_slot();
                    if let Some(parent) = path.parent() {
                        if parent.is_dir() {
                            self.file_dialog_dir = parent.to_path_buf();
                        }
                    }
                    self.recent_files.retain(|p| !Self::paths_match(p, &path));
                    self.recent_files.insert(0, path);
                    if self.recent_files.len() > 8 {
                        self.recent_files.truncate(8);
                    }
                    if !self.recovery_banner_active {
                        self.status_message = Some(format!("Aperto: {}", self.document_title));
                    }
                    self.check_active_tab_disk_modification();
                    self.overlay = ActiveOverlay::None;
                } else {
                    // Decide whether to reuse the current tab (only if it is an untouched untitled buffer
                    // or the sole untouched initial sample `/tmp/esempio_accent.md`) or open a new tab.
                    let is_untouched_placeholder = self.is_single_untouched_placeholder()
                        || (!self.is_dirty && self.current_file_path.is_none());

                    let valid_recovery_entry = accent_storage::find_recovery_journal_for_path(
                        &self.recovery_dir,
                        Some(&path),
                    )
                    .and_then(|mut entry| {
                        entry.target_path = Some(path.clone());
                        if entry.original_file_hash.is_none() {
                            entry.original_file_hash = disk_hash.clone();
                        }
                        let entry_hash = accent_storage::compute_content_hash(&entry.content);
                        let matches_disk = disk_hash
                            .as_deref()
                            .is_some_and(|dh| dh == entry_hash.as_str())
                            || disk_content
                                .as_deref()
                                .is_some_and(|dc| dc == entry.content.as_str());
                        if matches_disk {
                            let _ = accent_storage::remove_journal_for_target(
                                &self.recovery_dir,
                                Some(&path),
                            );
                            None
                        } else {
                            Some(entry)
                        }
                    });

                    if let Some(entry) = valid_recovery_entry {
                        let title = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("documento.md")
                            .to_string();
                        if !is_untouched_placeholder {
                            let id = self.next_tab_id;
                            self.next_tab_id += 1;
                            self.tabs.push(DocumentTab {
                                tab_id: id,
                                document_title: title,
                                current_file_path: Some(path.clone()),
                                is_dirty: true,
                                original_file_hash: entry.original_file_hash.clone(),
                                last_edit_instant: None,
                                last_edit_utc_ms: 0,
                                journal_pending_flush: false,
                                recovered_from_crash: true,
                                recovery_banner_active: true,
                                disk_modified_banner_active: false,
                                dismissed_disk_hash: None,
                                buffer: DocumentBuffer::default(),
                            });
                            self.active_tab_index = self.tabs.len() - 1;
                        }
                        self.disk_modified_banner_active = false;
                        self.dismissed_disk_hash = None;
                        self.restore_from_journal_entry(entry);
                        self.recent_files.retain(|p| !Self::paths_match(p, &path));
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
                        let mut fresh_buf = DocumentBuffer::from_str_content(&content);
                        fresh_buf.set_cursor(0);
                        if !is_untouched_placeholder {
                            let id = self.next_tab_id;
                            self.next_tab_id += 1;
                            self.tabs.push(DocumentTab {
                                tab_id: id,
                                document_title: title.clone(),
                                current_file_path: Some(path.clone()),
                                is_dirty: false,
                                original_file_hash: disk_hash.clone(),
                                last_edit_instant: None,
                                last_edit_utc_ms: 0,
                                journal_pending_flush: false,
                                recovered_from_crash: false,
                                recovery_banner_active: false,
                                disk_modified_banner_active: false,
                                dismissed_disk_hash: None,
                                buffer: fresh_buf.clone(),
                            });
                            self.active_tab_index = self.tabs.len() - 1;
                        }
                        self.document_title = title.clone();
                        self.file_dialog_input = title;
                        if let Some(parent) = path.parent() {
                            if parent.is_dir() {
                                self.file_dialog_dir = parent.to_path_buf();
                            }
                        }
                        self.current_file_path = Some(path.clone());
                        self.update_default_export_path();
                        self.original_file_hash = disk_hash;
                        self.is_dirty = false;
                        self.journal_pending_flush = false;
                        self.recovered_from_crash = false;
                        self.recovery_banner_active = false;
                        self.disk_modified_banner_active = false;
                        self.dismissed_disk_hash = None;
                        self.chrome_autohidden = false;
                        self.display_word_count_override = None;
                        self.buffer = fresh_buf;
                        self.recent_files.retain(|p| !Self::paths_match(p, &path));
                        self.recent_files.insert(0, path);
                        if self.recent_files.len() > 8 {
                            self.recent_files.truncate(8);
                        }
                        self.status_message = Some(format!("Aperto: {}", self.document_title));
                        self.overlay = ActiveOverlay::None;
                    }
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
                self.disk_modified_banner_active = false;
                self.dismissed_disk_hash = None;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                let mut tut_buf = DocumentBuffer::from_str_content(TUTORIAL_MARKDOWN);
                tut_buf.set_cursor(0);
                self.buffer = tut_buf;
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
                    self.disk_modified_banner_active = false;
                    self.dismissed_disk_hash = None;
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
                self.disk_modified_banner_active = false;
                self.dismissed_disk_hash = None;
                self.chrome_autohidden = false;
                self.display_word_count_override = None;
                if let Some(path) = self.current_file_path.clone() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.original_file_hash =
                            Some(accent_storage::compute_content_hash(&content));
                        let mut clean_buf = DocumentBuffer::from_str_content(&content);
                        clean_buf.set_cursor(0);
                        self.buffer = clean_buf;
                    } else {
                        let init_txt = "# Nuovo Documento\n\nInizia a scrivere qui...\n";
                        self.document_title = self.next_untitled_name();
                        self.current_file_path = None;
                        self.original_file_hash =
                            Some(accent_storage::compute_content_hash(init_txt));
                        let mut clean_buf = DocumentBuffer::from_str_content(init_txt);
                        clean_buf.set_cursor(0);
                        self.buffer = clean_buf;
                    }
                } else {
                    let init_txt = "# Nuovo Documento\n\nInizia a scrivere qui...\n";
                    self.original_file_hash = Some(accent_storage::compute_content_hash(init_txt));
                    let mut clean_buf = DocumentBuffer::from_str_content(init_txt);
                    clean_buf.set_cursor(0);
                    self.buffer = clean_buf;
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
                    self.disk_modified_banner_active = false;
                    self.dismissed_disk_hash = None;
                    self.chrome_autohidden = false;
                    self.recent_files.retain(|p| !Self::paths_match(p, &path));
                    self.recent_files.insert(0, path.clone());
                    if self.recent_files.len() > 8 {
                        self.recent_files.truncate(8);
                    }
                    self.status_message = Some(format!("Salvato come {}", path.display()));
                    self.overlay = ActiveOverlay::None;
                }
            }
            Action::CheckExternalFileModification => {
                self.check_active_tab_disk_modification();
            }
            Action::ReloadFromDisk => {
                if let Some(path) = self.current_file_path.clone() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let _ = accent_storage::remove_journal_for_target(
                            &self.recovery_dir,
                            Some(&path),
                        );
                        let prev_cursor = self.buffer.cursor();
                        let mut fresh_buf = DocumentBuffer::from_str_content(&content);
                        fresh_buf.set_cursor(prev_cursor.min(fresh_buf.len_chars()));
                        self.buffer = fresh_buf;
                        self.original_file_hash =
                            Some(accent_storage::compute_content_hash(&content));
                        self.is_dirty = false;
                        self.journal_pending_flush = false;
                        self.recovered_from_crash = false;
                        self.recovery_banner_active = false;
                        self.disk_modified_banner_active = false;
                        self.dismissed_disk_hash = None;
                        self.chrome_autohidden = false;
                        self.display_word_count_override = None;
                        self.status_message =
                            Some(format!("Ricaricato da disco: {}", self.document_title));
                    }
                }
            }
            Action::DismissExternalModification => {
                if let Some(ref path) = self.current_file_path {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        self.dismissed_disk_hash =
                            Some(accent_storage::compute_content_hash(&content));
                    }
                }
                self.disk_modified_banner_active = false;
                self.chrome_autohidden = false;
                self.is_dirty = true;
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
                    && !self.disk_modified_banner_active
                {
                    self.chrome_autohidden = true;
                }
                self.buffer.insert_text(&text);
            }
            Action::MoveCursor(offset) => {
                self.buffer.set_cursor(offset);
            }
        }

        // Keep `self.tabs[self.active_tab_index]` in sync with top-level active document fields
        self.sync_active_tab_to_slot();

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
        assert!(!restored.hemingway_mode);
        assert!(restored.focus_mode);
        assert_eq!(restored.stat_metric, StatMetric::ReadingTime);
        assert_eq!(restored.file_dialog_dir, docs_folder);
        assert_eq!(restored.current_file_path, Some(note_path.clone()));
        assert_eq!(restored.document_title, "appunti.md");
        assert!(restored.buffer.text().contains("Appunti Persistenti"));
        assert!(restored.recent_files.contains(&note_path));

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_text_zoom_in_out_reset_and_clamping() {
        let mut session = Session::default();
        assert!((session.typography.font_size - DEFAULT_FONT_SIZE).abs() < 1e-4);
        assert_eq!(session.zoom_percentage(), 100);

        // Zoom In (+1.0 pt per step)
        session.apply(Action::ZoomIn);
        assert!((session.typography.font_size - (DEFAULT_FONT_SIZE + 1.0)).abs() < 1e-4);
        assert!(session.zoom_percentage() > 100);

        // Zoom Out (-1.0 pt per step)
        session.apply(Action::ZoomOut);
        assert!((session.typography.font_size - DEFAULT_FONT_SIZE).abs() < 1e-4);
        assert_eq!(session.zoom_percentage(), 100);

        // ZoomByStep beyond MAX_FONT_SIZE -> clamped to MAX_FONT_SIZE (36.0)
        session.apply(Action::ZoomByStep(100.0));
        assert!((session.typography.font_size - MAX_FONT_SIZE).abs() < 1e-4);

        // ZoomByStep below MIN_FONT_SIZE -> clamped to MIN_FONT_SIZE (10.0)
        session.apply(Action::ZoomByStep(-200.0));
        assert!((session.typography.font_size - MIN_FONT_SIZE).abs() < 1e-4);

        // ZoomReset -> returns to DEFAULT_FONT_SIZE (17.35, 100%)
        session.apply(Action::ZoomReset);
        assert!((session.typography.font_size - DEFAULT_FONT_SIZE).abs() < 1e-4);
        assert_eq!(session.zoom_percentage(), 100);
    }

    #[test]
    fn test_text_zoom_persistence_roundtrip() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_zoom_persist_test_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        let config_path = base_dir.join("settings.json");

        {
            let mut session =
                Session::with_config_and_recovery(config_path.clone(), recovery_dir.clone());
            session.apply(Action::ZoomByStep(4.65)); // 17.35 + 4.65 = 22.0 pt
            assert!((session.typography.font_size - 22.0).abs() < 1e-3);
        }

        let restored = Session::with_config_and_recovery(config_path, recovery_dir);
        assert!((restored.typography.font_size - 22.0).abs() < 1e-3);
        assert_eq!(restored.zoom_percentage(), 127);

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn stress_test_text_zoom_rapid_wheel_oscillations_10000_steps() {
        let mut session = Session::default();
        for i in 0..10_000 {
            let delta = match i % 7 {
                0 => 0.75,
                1 => -1.25,
                2 => 3.5,
                3 => -4.1,
                4 => f32::NAN,
                5 => f32::INFINITY,
                _ => 0.33,
            };
            session.apply(Action::ZoomByStep(delta));
            assert!(session.typography.font_size.is_finite());
            assert!(
                (MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&session.typography.font_size),
                "Font size {} out of bounds at step {i}",
                session.typography.font_size
            );
            if i % 500 == 0 {
                session.apply(Action::ZoomReset);
                assert!((session.typography.font_size - DEFAULT_FONT_SIZE).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn test_multitab_lifecycle_new_switch_close_and_deduplication() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_multitab_lifecycle_test_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        std::fs::create_dir_all(&base_dir).unwrap();
        let file_a = base_dir.join("alpha.md");
        let file_b = base_dir.join("beta.md");
        let file_c = base_dir.join("gamma.md");
        accent_storage::atomic_save(&file_a, "# Alpha 🚀\n").unwrap();
        accent_storage::atomic_save(&file_b, "# Beta 🦀\n").unwrap();
        accent_storage::atomic_save(&file_c, "# Gamma ✨\n").unwrap();

        let mut session = Session::with_recovery_dir(base_dir.join("recovery"));
        assert_eq!(session.tabs.len(), 1);

        // First OpenFile reuses the initial placeholder tab (`esempio_accent.md`)
        session.apply(Action::OpenFile(file_a.clone()));
        assert_eq!(session.tabs.len(), 1);
        assert_eq!(session.document_title, "alpha.md");
        assert_eq!(session.active_tab_index, 0);

        // Second OpenFile opens in a NEW tab (index 1)
        session.apply(Action::OpenFile(file_b.clone()));
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.document_title, "beta.md");
        assert_eq!(session.active_tab_index, 1);

        // Third OpenFile opens in a NEW tab (index 2)
        session.apply(Action::OpenFile(file_c.clone()));
        assert_eq!(session.tabs.len(), 3);
        assert_eq!(session.document_title, "gamma.md");
        assert_eq!(session.active_tab_index, 2);

        // Re-opening `alpha.md` deduplicates and switches back to Tab 0!
        session.apply(Action::OpenFile(file_a.clone()));
        assert_eq!(session.tabs.len(), 3);
        assert_eq!(session.active_tab_index, 0);
        assert_eq!(session.document_title, "alpha.md");
        assert_eq!(session.buffer.text(), "# Alpha 🚀\n");

        // NextTab / PrevTab wrap around
        session.apply(Action::PrevTab);
        assert_eq!(session.active_tab_index, 2);
        assert_eq!(session.document_title, "gamma.md");
        session.apply(Action::NextTab);
        assert_eq!(session.active_tab_index, 0);
        assert_eq!(session.document_title, "alpha.md");

        // Close middle tab (index 1, `beta.md`)
        session.apply(Action::CloseTab(1));
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.tabs[0].document_title, "alpha.md");
        assert_eq!(session.tabs[1].document_title, "gamma.md");

        // Close remaining 2 tabs -> closing the last tab leaves a clean `untitled.md` tab
        session.apply(Action::CloseTab(1));
        session.apply(Action::CloseTab(0));
        assert_eq!(session.tabs.len(), 1);
        assert_eq!(session.document_title, "untitled.md");
        assert!(session.current_file_path.is_none());

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_multitab_isolated_undo_redo_dirty_state_and_cursors() {
        let mut session = Session::default();
        session.apply(Action::UpdateBufferText("Testo Tab Zero".to_string()));
        session.apply(Action::MoveCursor(5));
        assert!(session.is_dirty);

        // Open a second tab (`untitled.md`)
        session.apply(Action::NewTab);
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.active_tab_index, 1);
        assert!(!session.is_dirty);
        session.apply(Action::InsertText(" Aggiunta Tab Uno 🚀".to_string()));
        assert!(session.is_dirty);
        let tab1_edited = session.buffer.text();

        // Switch back to Tab 0 and verify its buffer, cursor, and dirty state are intact
        session.apply(Action::SwitchTab(0));
        assert_eq!(session.active_tab_index, 0);
        assert_eq!(session.buffer.text(), "Testo Tab Zero");
        assert_eq!(session.buffer.cursor(), 5);
        assert!(session.is_dirty);

        // Undo in Tab 0 must NOT affect Tab 1
        session.apply(Action::Undo);
        assert_ne!(session.buffer.text(), "Testo Tab Zero");
        session.apply(Action::SwitchTab(1));
        assert_eq!(session.buffer.text(), tab1_edited);

        // Undo and Redo in Tab 1
        session.apply(Action::Undo);
        assert!(!session.buffer.text().contains("Aggiunta Tab Uno 🚀"));
        session.apply(Action::Redo);
        assert_eq!(session.buffer.text(), tab1_edited);
    }

    #[test]
    fn test_multitab_persistence_and_wal_recovery() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_multitab_persist_wal_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        let config_path = base_dir.join("settings.json");
        std::fs::create_dir_all(&recovery_dir).unwrap();

        let doc1 = base_dir.join("doc1.md");
        let doc2 = base_dir.join("doc2.md");
        accent_storage::atomic_save(&doc1, "# Doc 1\n").unwrap();
        accent_storage::atomic_save(&doc2, "# Doc 2\n").unwrap();

        {
            let mut session =
                Session::with_config_and_recovery(config_path.clone(), recovery_dir.clone());
            session.apply(Action::OpenFile(doc1.clone()));
            session.apply(Action::OpenFile(doc2.clone()));
            assert_eq!(session.tabs.len(), 2);
            assert_eq!(session.active_tab_index, 1);
            // Edit Tab 0 (`doc1.md`) and flush WAL
            session.apply(Action::SwitchTab(0));
            session.apply(Action::UpdateBufferText("# Doc 1 Modificato non salvato\n".to_string()));
            let future = std::time::Instant::now()
                + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 50);
            assert!(session.poll_journal_flush(future));
        }

        // Restart session: both tabs must be restored and active_tab_index == 0 with WAL recovery!
        let restored = Session::with_config_and_recovery(config_path, recovery_dir);
        assert_eq!(restored.tabs.len(), 2);
        assert_eq!(restored.active_tab_index, 0);
        assert_eq!(restored.document_title, "doc1.md");
        assert!(restored.recovered_from_crash);
        assert_eq!(restored.buffer.text(), "# Doc 1 Modificato non salvato\n");
        assert_eq!(restored.tabs[1].document_title, "doc2.md");
        assert_eq!(restored.tabs[1].buffer.text(), "# Doc 2\n");

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn stress_test_multitab_100_concurrent_tabs_heavy_switching_and_editing() {
        let mut session = Session::default();
        // Open 99 additional tabs -> 100 concurrent tabs total
        for i in 1..100 {
            session.apply(Action::NewTab);
            session.apply(Action::UpdateBufferText(format!(
                "# Tab Numero {i} 🚀\n\nContenuto iniziale del tab {i}.\n"
            )));
        }
        assert_eq!(session.tabs.len(), 100);

        // Perform 2,500 cross-tab switches, edits, format commands, and cursor movements
        for step in 0..2_500 {
            let target_tab = (step * 37) % session.tabs.len();
            session.apply(Action::SwitchTab(target_tab));
            assert_eq!(session.active_tab_index, target_tab);

            match step % 5 {
                0 => session.apply(Action::InsertText(" ✨🦀".to_string())),
                1 => session.apply(Action::ApplyFormat(FormatCommand::Bold)),
                2 => session.apply(Action::NextTab),
                3 => session.apply(Action::PrevTab),
                _ => {
                    session.apply(Action::Undo);
                    session.apply(Action::Redo);
                }
            }
        }

        // Close 50 tabs from various positions and verify remaining 50 tabs are valid and consistent
        for close_step in 0..50 {
            let idx = (close_step * 7) % session.tabs.len();
            session.apply(Action::CloseTab(idx));
        }
        assert_eq!(session.tabs.len(), 50);
        assert!(session.active_tab_index < 50);
        assert_eq!(
            session.document_title,
            session.tabs[session.active_tab_index].document_title
        );
        assert_eq!(
            session.buffer.text(),
            session.tabs[session.active_tab_index].buffer.text()
        );
    }

    #[test]
    fn test_open_file_never_taints_new_tab_undo_stack_with_previous_tab_content() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_undo_isolation_test_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        std::fs::create_dir_all(&base_dir).unwrap();
        let file_a = base_dir.join("file_a.md");
        let file_b = base_dir.join("file_b.md");
        accent_storage::atomic_save(&file_a, "# Contenuto Esclusivo di File A\n").unwrap();
        accent_storage::atomic_save(&file_b, "# Contenuto Esclusivo di File B\n").unwrap();

        let mut session = Session::with_recovery_dir(base_dir.join("recovery"));
        session.apply(Action::OpenFile(file_a.clone()));
        assert_eq!(session.document_title, "file_a.md");
        assert_eq!(session.buffer.text(), "# Contenuto Esclusivo di File A\n");

        // Open `file_b.md` in a second tab and immediately trigger Undo:
        // Tab B must NEVER revert to `file_a.md`'s content or the default sample buffer!
        session.apply(Action::OpenFile(file_b.clone()));
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.active_tab_index, 1);
        assert_eq!(session.document_title, "file_b.md");
        assert_eq!(session.buffer.text(), "# Contenuto Esclusivo di File B\n");

        session.apply(Action::Undo);
        assert_eq!(session.document_title, "file_b.md");
        assert_eq!(session.buffer.text(), "# Contenuto Esclusivo di File B\n");
        assert!(!session.is_dirty);

        // Also verify Tab 0 (`file_a.md`) did not retain the initial sample markdown on its undo stack
        session.apply(Action::SwitchTab(0));
        session.apply(Action::Undo);
        assert_eq!(session.document_title, "file_a.md");
        assert_eq!(session.buffer.text(), "# Contenuto Esclusivo di File A\n");
        assert!(!session.is_dirty);

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_reopening_dirty_open_tab_does_not_trigger_false_crash_recovery() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_no_false_recovery_on_open_tab_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        std::fs::create_dir_all(&recovery_dir).unwrap();
        let file_a = base_dir.join("primo.md");
        let file_b = base_dir.join("secondo.md");
        accent_storage::atomic_save(&file_a, "# Primo\n").unwrap();
        accent_storage::atomic_save(&file_b, "# Secondo\n").unwrap();

        let mut session = Session::with_recovery_dir(recovery_dir);
        session.apply(Action::OpenFile(file_a.clone()));
        session.apply(Action::OpenFile(file_b.clone()));

        // Edit Tab 0 (`primo.md`) and flush live WAL journal during normal editing
        session.apply(Action::SwitchTab(0));
        session.apply(Action::UpdateBufferText("# Primo modificato live\n".to_string()));
        let future =
            std::time::Instant::now() + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 30);
        assert!(session.poll_journal_flush(future));
        assert!(!session.recovered_from_crash);
        assert!(!session.recovery_banner_active);

        // Switch to Tab 1 (`secondo.md`), then invoke `OpenFile(primo.md)`:
        // Must switch back to Tab 0 WITHOUT falsely claiming a crash recovery!
        session.apply(Action::SwitchTab(1));
        session.apply(Action::OpenFile(file_a.clone()));
        assert_eq!(session.active_tab_index, 0);
        assert_eq!(session.document_title, "primo.md");
        assert_eq!(session.buffer.text(), "# Primo modificato live\n");
        assert!(session.is_dirty);
        assert!(!session.recovered_from_crash);
        assert!(!session.recovery_banner_active);

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_closing_dirty_or_recovered_tab_removes_wal_journal_no_zombie_recovery() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_close_dirty_tab_wal_cleanup_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        std::fs::create_dir_all(&recovery_dir).unwrap();
        let file_a = base_dir.join("a.md");
        let file_b = base_dir.join("b.md");
        accent_storage::atomic_save(&file_a, "# A originale\n").unwrap();
        accent_storage::atomic_save(&file_b, "# B originale\n").unwrap();

        let mut session = Session::with_recovery_dir(recovery_dir.clone());
        session.apply(Action::OpenFile(file_a.clone()));
        session.apply(Action::OpenFile(file_b.clone()));

        // Edit `b.md` and flush its WAL journal
        session.apply(Action::UpdateBufferText("# B sporco non salvato\n".to_string()));
        let future =
            std::time::Instant::now() + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 30);
        assert!(session.poll_journal_flush(future));
        assert!(
            accent_storage::find_recovery_journal_for_path(&recovery_dir, Some(&file_b)).is_some()
        );

        // Explicitly close Tab 1 (`b.md`) -> its WAL journal MUST be removed
        session.apply(Action::CloseTab(1));
        assert_eq!(session.tabs.len(), 1);
        assert!(
            accent_storage::find_recovery_journal_for_path(&recovery_dir, Some(&file_b)).is_none()
        );

        // Re-opening `b.md` must load the clean disk file without any false crash recovery
        session.apply(Action::OpenFile(file_b.clone()));
        assert_eq!(session.document_title, "b.md");
        assert_eq!(session.buffer.text(), "# B originale\n");
        assert!(!session.is_dirty);
        assert!(!session.recovered_from_crash);
        assert!(!session.recovery_banner_active);

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_startup_recovery_with_multiple_tabs_restores_exact_matching_tab_only() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_multitab_exact_slot_recovery_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        let recovery_dir = base_dir.join("recovery");
        let config_path = base_dir.join("settings.json");
        std::fs::create_dir_all(&recovery_dir).unwrap();

        let doc1 = base_dir.join("specs1.md");
        let doc2 = base_dir.join("readme.md");
        let doc3 = base_dir.join("specs2.md");
        accent_storage::atomic_save(&doc1, "# Specs 1 Disk\n").unwrap();
        accent_storage::atomic_save(&doc2, "# Readme Disk\n").unwrap();
        accent_storage::atomic_save(&doc3, "# Specs 2 Disk\n").unwrap();

        {
            let mut session =
                Session::with_config_and_recovery(config_path.clone(), recovery_dir.clone());
            session.apply(Action::OpenFile(doc1.clone()));
            session.apply(Action::OpenFile(doc2.clone()));
            session.apply(Action::OpenFile(doc3.clone()));
            assert_eq!(session.tabs.len(), 3);
            assert_eq!(session.active_tab_index, 2);

            // Edit Tab 0 (`specs1.md`), flush WAL, then switch back to Tab 2 (`specs2.md`) before crash
            session.apply(Action::SwitchTab(0));
            session.apply(Action::UpdateBufferText(
                "# Specs 1 Modificato Prima del Crash\n".to_string(),
            ));
            let future = std::time::Instant::now()
                + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 40);
            assert!(session.poll_journal_flush(future));
            session.apply(Action::SwitchTab(2));
            assert_eq!(session.active_tab_index, 2);
        }

        // Also inject a stale orphan journal for a deleted temp file and a redundant journal for `doc2`
        let orphan_path = base_dir.join("deleted_temp_file.md");
        let orphan_entry = accent_storage::RecoveryJournalEntry::new(
            Some(orphan_path.clone()),
            "deleted_temp_file.md".to_string(),
            "# Orphan Content\n".to_string(),
            0,
            0,
            None,
            true,
        );
        accent_storage::write_journal_entry(&recovery_dir, &orphan_entry).unwrap();

        let redundant_entry = accent_storage::RecoveryJournalEntry::new(
            Some(doc2.clone()),
            "readme.md".to_string(),
            "# Readme Disk\n".to_string(),
            0,
            0,
            Some(accent_storage::compute_content_hash("# Readme Disk\n")),
            true,
        );
        accent_storage::write_journal_entry(&recovery_dir, &redundant_entry).unwrap();

        // Restart session:
        // - Active tab is still Tab 2 (`specs2.md`) with its exact disk content (`# Specs 2 Disk\n`) and NO recovery banner!
        // - Tab 0 (`specs1.md`) has the recovered content (`# Specs 1 Modificato Prima del Crash\n`)!
        // - Tab 1 (`readme.md`) is clean (`!is_dirty`, `!recovered_from_crash`) and its redundant journal was cleaned up!
        // - The orphan journal for `deleted_temp_file.md` was deleted and did not overwrite any tab!
        let restored = Session::with_config_and_recovery(config_path, recovery_dir.clone());
        assert_eq!(restored.tabs.len(), 3);
        assert_eq!(restored.active_tab_index, 2);
        assert_eq!(restored.document_title, "specs2.md");
        assert_eq!(restored.buffer.text(), "# Specs 2 Disk\n");
        assert!(!restored.is_dirty);
        assert!(!restored.recovered_from_crash);
        assert!(!restored.recovery_banner_active);

        assert_eq!(restored.tabs[0].document_title, "specs1.md");
        assert_eq!(
            restored.tabs[0].buffer.text(),
            "# Specs 1 Modificato Prima del Crash\n"
        );
        assert!(restored.tabs[0].is_dirty);
        assert!(restored.tabs[0].recovered_from_crash);
        assert!(restored.tabs[0].recovery_banner_active);

        assert_eq!(restored.tabs[1].document_title, "readme.md");
        assert_eq!(restored.tabs[1].buffer.text(), "# Readme Disk\n");
        assert!(!restored.tabs[1].is_dirty);
        assert!(!restored.tabs[1].recovered_from_crash);

        assert!(
            accent_storage::find_recovery_journal_for_path(&recovery_dir, Some(&orphan_path))
                .is_none()
        );
        assert!(
            accent_storage::find_recovery_journal_for_path(&recovery_dir, Some(&doc2)).is_none()
        );

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_hemingway_mode_never_persists_across_restarts_and_cursor_moves_do_not_trigger_wal_writes() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_hemingway_and_wal_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base_dir);
        std::fs::create_dir_all(&base_dir).unwrap();
        let config_path = base_dir.join("settings.json");
        let recovery_dir = base_dir.join("recovery");
        let doc_path = base_dir.join("atdb_sample.md");

        let sample = "Paragrafo valido.\n\n____testo vecchio da cancellare_ start___ \n* **`B_Tipo_Notifica__c`** (Constant String)\n#### Stato della notifica (campo `Status__c`)\n____testo vecchio da cancellare_end___\n\n### 3.5 Stato del processo di notifica\n";
        accent_storage::atomic_save(&doc_path, sample).unwrap();

        // Simulate a legacy settings.json with `"hemingway_mode": true`
        std::fs::write(
            &config_path,
            format!(
                r#"{{"hemingway_mode": true, "open_tab_files": ["{}"], "active_tab_index": 0}}"#,
                doc_path.display()
            ),
        )
        .unwrap();

        let mut session =
            Session::with_config_and_recovery(config_path.clone(), recovery_dir.clone());
        // Hemingway mode MUST be reset to false on startup so deletions are never blocked across sessions
        assert!(!session.hemingway_mode);
        assert!(session
            .buffer
            .text()
            .contains("____testo vecchio da cancellare_ start___"));

        // Delete the marked block and verify the buffer updates and flushes once
        let cleaned = "Paragrafo valido.\n\n### 3.5 Stato del processo di notifica\n";
        session.apply(Action::UpdateBufferText(cleaned.to_string()));
        assert_eq!(session.buffer.text(), cleaned);
        assert!(session.is_dirty);
        assert!(session.journal_pending_flush);

        let future =
            std::time::Instant::now() + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 30);
        assert!(session.poll_journal_flush(future));
        assert!(!session.journal_pending_flush);

        // Moving the cursor or changing selection in a dirty buffer must NOT schedule another disk WAL write
        session.apply(Action::MoveCursor(5));
        assert!(!session.journal_pending_flush);
        session.apply(Action::SetSelection(2, 10));
        assert!(!session.journal_pending_flush);
        assert!(!session.poll_journal_flush(
            future + std::time::Duration::from_millis(JOURNAL_DEBOUNCE_MS + 50)
        ));

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_external_file_modification_detection_reload_and_dismiss() {
        let base_dir = std::env::temp_dir().join(format!(
            "accent_ext_file_mod_test_{}_{}",
            std::process::id(),
            accent_storage::current_utc_ms()
        ));
        std::fs::create_dir_all(&base_dir).unwrap();
        let config_path = base_dir.join("settings.json");
        let recovery_dir = base_dir.join("recovery");
        let doc_a = base_dir.join("doc_a.md");
        let doc_b = base_dir.join("doc_b.md");

        accent_storage::atomic_save(&doc_a, "# Doc A v1\n").unwrap();
        accent_storage::atomic_save(&doc_b, "# Doc B v1\n").unwrap();

        let mut session =
            Session::with_config_and_recovery(config_path.clone(), recovery_dir.clone());
        session.apply(Action::OpenFile(doc_a.clone()));
        session.apply(Action::OpenFile(doc_b.clone()));
        assert_eq!(session.active_tab_index, 1);
        assert!(!session.disk_modified_banner_active);

        // External write to doc_b (focused tab)
        accent_storage::atomic_save(&doc_b, "# Doc B v2 (modificato esternamente)\n").unwrap();
        session.apply(Action::CheckExternalFileModification);
        assert!(session.disk_modified_banner_active);

        // Dismiss external modification -> banner hides, buffer stays v1, marked dirty
        session.apply(Action::DismissExternalModification);
        assert!(!session.disk_modified_banner_active);
        assert!(session.is_dirty);
        assert_eq!(session.buffer.text(), "# Doc B v1\n");

        // Re-checking with the same disk content does NOT re-trigger the dismissed banner
        session.apply(Action::CheckExternalFileModification);
        assert!(!session.disk_modified_banner_active);

        // A newer external write to doc_b DOES trigger the banner again
        accent_storage::atomic_save(&doc_b, "# Doc B v3 (nuova modifica esterna)\n").unwrap();
        session.apply(Action::CheckExternalFileModification);
        assert!(session.disk_modified_banner_active);

        // Reload from disk -> buffer updates to v3, banner closes, dirty flag cleared
        session.apply(Action::ReloadFromDisk);
        assert!(!session.disk_modified_banner_active);
        assert!(!session.is_dirty);
        assert_eq!(session.buffer.text(), "# Doc B v3 (nuova modifica esterna)\n");

        // Modify doc_a on disk while doc_b is focused, then switch to doc_a via PrevTab -> banner activates immediately
        accent_storage::atomic_save(&doc_a, "# Doc A v2 (modificato mentre in background)\n")
            .unwrap();
        session.apply(Action::PrevTab);
        assert_eq!(session.active_tab_index, 0);
        assert!(session.disk_modified_banner_active);
        session.apply(Action::ReloadFromDisk);
        assert_eq!(
            session.buffer.text(),
            "# Doc A v2 (modificato mentre in background)\n"
        );
        assert!(!session.disk_modified_banner_active);

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_fullscreen_toggle_and_set() {
        let mut session = Session::default();
        assert!(!session.fullscreen);
        session.apply(Action::ToggleFullscreen);
        assert!(session.fullscreen);
        session.apply(Action::ToggleFullscreen);
        assert!(!session.fullscreen);
        session.apply(Action::SetFullscreen(true));
        assert!(session.fullscreen);
    }
}
