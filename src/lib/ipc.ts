import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type {
  ColorPalettePreset,
  MermaidSvgPayload,
  PaletteCssTokens,
  SessionAction,
  ThemeMode,
  UiSessionSnapshot,
} from './types';

export async function getSessionSnapshot(): Promise<UiSessionSnapshot> {
  return invoke<UiSessionSnapshot>('get_session_snapshot');
}

export async function dispatchAction(action: SessionAction): Promise<UiSessionSnapshot> {
  return invoke<UiSessionSnapshot>('dispatch_action', { action });
}

export async function updateEditorBuffer(
  tabId: number,
  text: string,
  selectionPrimary: number,
  selectionSecondary: number
): Promise<UiSessionSnapshot> {
  return invoke<UiSessionSnapshot>('update_editor_buffer', {
    tabId,
    text,
    selectionPrimary,
    selectionSecondary,
  });
}

export async function openNativeFileDialog(): Promise<UiSessionSnapshot> {
  return invoke<UiSessionSnapshot>('open_native_file_dialog');
}

export async function saveAsNativeFileDialog(): Promise<UiSessionSnapshot> {
  return invoke<UiSessionSnapshot>('save_as_native_file_dialog');
}

export async function previewPaletteTokens(
  preset: ColorPalettePreset,
  mode: ThemeMode
): Promise<PaletteCssTokens> {
  return invoke<PaletteCssTokens>('preview_palette_tokens', { preset, mode });
}

export async function renderMermaidDiagram(
  source: string,
  themeOverride?: ThemeMode
): Promise<MermaidSvgPayload> {
  return invoke<MermaidSvgPayload>('render_mermaid_diagram', {
    source,
    themeOverride: themeOverride ?? null,
  });
}

export async function renderPreviewHtml(): Promise<string> {
  return invoke<string>('render_preview_html');
}

export async function onSessionUpdated(
  callback: (snapshot: UiSessionSnapshot) => void
): Promise<UnlistenFn> {
  return listen<UiSessionSnapshot>('accent://session-updated', (event) => {
    callback(event.payload);
  });
}

export async function windowMinimize(): Promise<void> {
  await getCurrentWindow().minimize();
}

export async function windowToggleMaximize(): Promise<boolean> {
  const win = getCurrentWindow();
  await win.toggleMaximize();
  return win.isMaximized();
}

export async function windowClose(): Promise<void> {
  await getCurrentWindow().close();
}

export async function windowStartDrag(): Promise<void> {
  await getCurrentWindow().startDragging();
}

export type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';

export async function windowStartResize(direction: ResizeDirection): Promise<void> {
  await getCurrentWindow().startResizeDragging(direction);
}

export function applyPaletteToRoot(tokens: PaletteCssTokens, theme: ThemeMode): void {
  const root = document.documentElement;
  const isDark = theme === 'Dark' || theme === 'SemiDark';
  root.style.colorScheme = isDark ? 'dark' : 'light';
  root.dataset.theme = theme.toLowerCase();

  root.style.setProperty('--bg-canvas', tokens.bg_canvas);
  root.style.setProperty('--bg-headerbar', tokens.bg_headerbar);
  root.style.setProperty('--border-headerbar', tokens.border_headerbar);
  root.style.setProperty('--fg-primary', tokens.fg_primary);
  root.style.setProperty('--fg-heading', tokens.fg_heading);
  root.style.setProperty('--fg-subheading', tokens.fg_subheading);
  root.style.setProperty('--fg-muted', tokens.fg_muted);
  root.style.setProperty('--bg-code-block', tokens.bg_code_block);
  root.style.setProperty('--bg-code-inline', tokens.bg_code_inline);
  root.style.setProperty('--bg-pill-button', tokens.bg_pill_button);
  root.style.setProperty('--bg-popover', tokens.bg_popover);
  root.style.setProperty('--bg-preview', tokens.bg_preview);
  root.style.setProperty('--divider-preview', tokens.divider_preview);
  root.style.setProperty('--accent-orange', tokens.accent_orange);
  root.style.setProperty('--accent-link', tokens.accent_link);
  root.style.setProperty('--markup-bold-fg', tokens.markup_bold_fg);
  root.style.setProperty('--markup-emphasis-fg', tokens.markup_emphasis_fg);
  root.style.setProperty('--markup-code-fg', tokens.markup_code_fg);
  root.style.setProperty('--markup-codeblock-fg', tokens.markup_codeblock_fg);
  root.style.setProperty('--markup-list-marker', tokens.markup_list_marker);
  root.style.setProperty('--markup-task-done', tokens.markup_task_done);
  root.style.setProperty('--markup-quote-bar', tokens.markup_quote_bar);
  root.style.setProperty('--markup-quote-fg', tokens.markup_quote_fg);
  root.style.setProperty('--markup-link-url', tokens.markup_link_url);
  root.style.setProperty('--bg-table-header', tokens.bg_table_header);
  root.style.setProperty('--spellcheck-red', tokens.spellcheck_red);
  root.style.setProperty('--scrollbar-thumb', tokens.scrollbar_thumb);
  root.style.setProperty('--scrollbar-track', 'transparent');

  // Code block syntax highlighting semantic tokens (keys in periwinkle/cobalt blue, strings in vibrant green)
  root.style.setProperty('--syntax-key', isDark ? '#7aa2f7' : '#25589e');
  root.style.setProperty('--syntax-string', isDark ? '#6bcb77' : '#1e824c');
  root.style.setProperty('--syntax-number', isDark ? '#e09f5a' : '#9a4708');
  root.style.setProperty('--syntax-keyword', isDark ? '#c792ea' : '#7c3aed');
  root.style.setProperty('--syntax-function', isDark ? '#7aa2f7' : '#25589e');
  root.style.setProperty('--syntax-type', isDark ? '#eab700' : '#8f3f06');
  root.style.setProperty('--syntax-comment', tokens.fg_muted);
  root.style.setProperty('--syntax-punctuation', tokens.fg_primary);
}
