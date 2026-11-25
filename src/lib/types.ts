export type ThemeMode = 'Light' | 'Sepia' | 'SemiDark' | 'Dark';

export type ColorPalettePreset =
  | 'AmberCoral'
  | 'EditorialSlate'
  | 'NordicForest'
  | 'VelvetDawn'
  | 'Classic';

export type Locale = 'En' | 'It' | 'Es';

export type MarkdownBase = 'Gfm' | 'CommonMark';

export interface MarkdownExtensions {
  math: boolean;
  mermaid: boolean;
  alerts: boolean;
  footnotes: boolean;
  frontmatter: boolean;
}

export interface MarkdownConfig {
  base: MarkdownBase;
  extensions: MarkdownExtensions;
}

export type LineWidthMode =
  | 'Compact'
  | 'Medium'
  | 'Wide'
  | { Fluid: { margin_pct: number } };

export type PreviewLayout = 'HalfWidth' | 'FullWidth' | 'HalfHeight' | 'Windowed';

export type StatMetric =
  | 'Characters'
  | 'Words'
  | 'Sentences'
  | 'Paragraphs'
  | 'ReadingTime';

export type ActiveOverlay =
  | 'None'
  | 'OpenMenu'
  | 'MainMenu'
  | 'SaveMenu'
  | 'PreviewMenu'
  | 'StatsMenu'
  | 'LineWidthMenu'
  | 'PreferencesDialog'
  | 'ExportDialog'
  | 'OpenFileDialog'
  | 'SaveFileDialog'
  | 'ShortcutsDialog'
  | 'AboutDialog';

export type ExportFormat = 'Pdf' | 'Docx' | 'Odt' | 'Latex' | 'Epub' | 'Html';

export type FormatCommand =
  | 'Bold'
  | 'Italic'
  | 'Strikethrough'
  | 'Heading'
  | 'BulletList'
  | 'OrderedList'
  | 'TaskList'
  | 'BlockQuote'
  | 'Code'
  | 'Link'
  | 'Image'
  | 'Table';

export interface PaletteCssTokens {
  bg_canvas: string;
  bg_headerbar: string;
  border_headerbar: string;
  fg_primary: string;
  fg_heading: string;
  fg_subheading: string;
  fg_muted: string;
  bg_code_block: string;
  bg_code_inline: string;
  bg_pill_button: string;
  bg_popover: string;
  bg_preview: string;
  divider_preview: string;
  accent_orange: string;
  accent_link: string;
  markup_bold_fg: string;
  markup_emphasis_fg: string;
  markup_code_fg: string;
  markup_codeblock_fg: string;
  markup_list_marker: string;
  markup_task_done: string;
  markup_quote_bar: string;
  markup_quote_fg: string;
  markup_link_url: string;
  bg_table_header: string;
  spellcheck_red: string;
  scrollbar_thumb: string;
}

export interface ThemeCircleColors {
  light: string;
  sepia: string;
  semi_dark: string;
  dark: string;
}

export interface TypographyConfig {
  prose_font_family: string;
  code_font_family: string;
  font_size: number;
  line_height: number;
}

export interface TabSnapshot {
  index: number;
  tab_id: number;
  document_title: string;
  current_file_path: string | null;
  is_dirty: boolean;
  recovered_from_crash: boolean;
  recovery_banner_active: boolean;
  disk_modified_banner_active: boolean;
}

export interface StatsSnapshot {
  characters: string;
  words: string;
  sentences: string;
  paragraphs: string;
  reading_time: string;
  raw_words: number;
  raw_characters: number;
  has_selection: boolean;
}

export interface OutlineHeadingSnapshot {
  level: number;
  text: string;
  line: number;
  char_offset: number;
}

export interface MermaidSvgPayload {
  content_hash: number;
  svg: string;
  width: number;
  height: number;
}

export interface ResolvedLocalImagePayload {
  ok: boolean;
  kind: 'asset' | 'data' | 'missing';
  src: string;
  canonical_path: string | null;
}

export interface UiSessionSnapshot {
  tabs: TabSnapshot[];
  active_tab_index: number;
  active_tab_id: number;
  document_title: string;
  current_file_path: string | null;
  is_dirty: boolean;
  buffer_text: string;
  buffer_revision: number;
  cursor_char: number;
  selection_start_char: number;
  selection_end_char: number;
  theme: ThemeMode;
  palette_preset: ColorPalettePreset;
  palette_css: PaletteCssTokens;
  theme_circle_colors: ThemeCircleColors;
  locale: Locale;
  markdown_config: MarkdownConfig;
  line_width: LineWidthMode;
  line_width_badge: string;
  fluid_margin_pct: number;
  typography: TypographyConfig;
  zoom_percentage: number;
  preview_active: boolean;
  preview_layout: PreviewLayout;
  view_while_typing: boolean;
  fullscreen: boolean;
  toolbar_expanded: boolean;
  search_bar_open: boolean;
  chrome_autohidden: boolean;
  search_query: string;
  replace_text: string;
  stat_metric: StatMetric;
  active_stat_label: string;
  stats: StatsSnapshot;
  overlay: ActiveOverlay;
  recovery_banner_active: boolean;
  disk_modified_banner_active: boolean;
  status_message: string | null;
  recent_files: string[];
  selected_export_format: ExportFormat;
  export_output_path: string;
  export_syntax_highlight: boolean;
  wal_autosave_enabled: boolean;
  disk_watcher_enabled: boolean;
  font_families: string[];
  prose_font_presets: string[];
  code_font_presets: string[];
  outline_headings: OutlineHeadingSnapshot[];
  i18n: Record<string, string>;
}

export type SessionAction =
  | { SetTheme: ThemeMode }
  | { SetPalettePreset: ColorPalettePreset }
  | { SetLocale: Locale }
  | { SetMarkdownConfig: MarkdownConfig }
  | { SetLineWidth: LineWidthMode }
  | { SetProseFont: string }
  | { SetCodeFont: string }
  | { SetFontSize: number }
  | { SetLineHeight: number }
  | 'ZoomIn'
  | 'ZoomOut'
  | 'ZoomReset'
  | { ZoomByStep: number }
  | { SetPreviewLayout: PreviewLayout }
  | { SetPreviewMode: { active: boolean; layout: PreviewLayout } }
  | 'SetEditorOnlyRaw'
  | 'SetEditorLivePreview'
  | { SetStatMetric: StatMetric }
  | 'ToggleSplitPreview'
  | 'ToggleViewWhileTyping'
  | { SetViewWhileTyping: boolean }
  | 'ToggleFullscreen'
  | { SetFullscreen: boolean }
  | 'ToggleToolbar'
  | 'ToggleSearchBar'
  | { SetChromeAutohidden: boolean }
  | { SetSearchQuery: string }
  | { SetReplaceText: string }
  | 'FindNext'
  | 'FindPrev'
  | 'ReplaceNext'
  | { SetOverlay: ActiveOverlay }
  | { UpdateBufferText: string }
  | { SetSelection: [number, number] }
  | { ApplyFormat: FormatCommand }
  | 'NewDocument'
  | 'NewTab'
  | { SwitchTab: number }
  | { CloseTab: number }
  | 'NextTab'
  | 'PrevTab'
  | 'Undo'
  | 'Redo'
  | { OpenFile: string }
  | 'OpenTutorial'
  | 'SaveDocument'
  | { SaveAsFile: string }
  | 'ConfirmRecoverySave'
  | 'DiscardRecovery'
  | 'CheckExternalFileModification'
  | 'ReloadFromDisk'
  | 'DismissExternalModification'
  | { SelectExportFormat: ExportFormat }
  | { SetExportOutputPath: string }
  | 'ToggleExportSyntaxHighlight'
  | 'ExecuteExport'
  | { ExportDocument: string }
  | { SetWalAutosave: boolean }
  | { SetDiskWatcher: boolean }
  | 'DiscardAllUnsavedForExit'
  | { SetStatusMessage: string }
  | { InsertText: string }
  | { MoveCursor: number };
