<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import CodeMirrorEditor from './lib/CodeMirrorEditor.svelte';
  import { renderMathInPreviewHtml } from './lib/mathAndMermaid';
  import { charToUtf16Offset } from './lib/textOffsets';
  import {
    applyPaletteToRoot,
    dispatchAction,
    exportWithNativeDialog,
    getSessionSnapshot,
    onSessionUpdated,
    openExternalUrl,
    openNativeFileDialog,
    previewPaletteTokens,
    renderPreviewHtml,
    saveAsNativeFileDialog,
    updateEditorBuffer,
    windowClose,
    windowMinimize,
    windowStartDrag,
    windowStartResize,
    windowToggleMaximize,
    type ResizeDirection,
  } from './lib/ipc';
  import type {
    ColorPalettePreset,
    ExportFormat,
    LineWidthMode,
    Locale,
    MarkdownBase,
    MarkdownConfig,
    MarkdownExtensions,
    PaletteCssTokens,
    SessionAction,
    StatMetric,
    ThemeMode,
    TypographyConfig,
    UiSessionSnapshot,
  } from './lib/types';

  interface PreferencesRollbackState {
    typography: TypographyConfig;
    line_width: LineWidthMode;
    markdown_config: MarkdownConfig;
    locale: Locale;
    wal_autosave_enabled: boolean;
    disk_watcher_enabled: boolean;
  }

  const DEFAULT_MARKDOWN_CONFIG: MarkdownConfig = {
    base: 'Gfm',
    extensions: {
      math: true,
      mermaid: true,
      alerts: true,
      footnotes: false,
      frontmatter: false,
    },
  };

  const PALETTE_SWATCHES: Record<ColorPalettePreset, [string, string, string]> = {
    AmberCoral: ['#181a1d', '#eab700', '#ff6b8b'],
    EditorialSlate: ['#faf9f5', '#2b6cb0', '#2c6e63'],
    NordicForest: ['#f7f9fc', '#3a9485', '#5e81ac'],
    VelvetDawn: ['#faf4ed', '#9b5b4a', '#c4a7e7'],
    Classic: ['#ffffff', '#db632c', '#1c71d8'],
  };

  let snapshot: UiSessionSnapshot | null = $state(null);
  let externalSyncToken = $state(0);
  let latestEditSeq = 0;
  let isMaximized = $state(false);
  let outlineOpen = $state(false);
  let themesModalOpen = $state(false);
  let pendingCloseTabIndex: number | null = $state(null);
  let draftTheme: ThemeMode = $state('Sepia');
  let draftPreset: ColorPalettePreset = $state('AmberCoral');
  let draftPreviewTokens: PaletteCssTokens | null = $state(null);
  let prefsTab: 'typography' | 'editor' = $state('typography');
  let prefsRollback: PreferencesRollbackState | null = $state(null);
  let previewHtml = $state('');
  let tabStripEl: HTMLDivElement | undefined = $state();
  let searchInputEl: HTMLInputElement | undefined = $state();
  let previewPaneEl: HTMLElement | undefined = $state();
  let tabStripCanScrollLeft = $state(false);
  let tabStripCanScrollRight = $state(false);
  let cmRef: CodeMirrorEditor | undefined = $state();
  let lastMousePos = { x: -1, y: -1 };
  let wasSearchBarOpen = false;
  let lastActiveTabIdForScroll = -1;
  let unlistenSession: (() => void) | null = null;
  let unlistenResize: (() => void) | null = null;

  function tr(key: string, fallback = ''): string {
    return snapshot?.i18n?.[key] ?? (fallback || key);
  }

  function actionMutatesEditorOrSelection(action: SessionAction): boolean {
    if (typeof action === 'string') {
      return (
        action === 'Undo' ||
        action === 'Redo' ||
        action === 'NewDocument' ||
        action === 'NewTab' ||
        action === 'NextTab' ||
        action === 'PrevTab' ||
        action === 'OpenTutorial' ||
        action === 'FindNext' ||
        action === 'FindPrev' ||
        action === 'ReplaceNext' ||
        action === 'DiscardRecovery' ||
        action === 'ReloadFromDisk'
      );
    }
    return (
      'SwitchTab' in action ||
      'CloseTab' in action ||
      'OpenFile' in action ||
      'ApplyFormat' in action ||
      'InsertText' in action
    );
  }

  function syncSnapshot(next: UiSessionSnapshot, syncEditor = false) {
    const tabChanged = snapshot?.active_tab_id !== next.active_tab_id;
    snapshot = next;
    if (syncEditor || tabChanged) {
      externalSyncToken += 1;
    }
    applyPaletteToRoot(next.palette_css, next.theme);
    const root = document.documentElement;
    root.style.setProperty(
      '--prose-font-stack',
      `"${next.typography.prose_font_family}", "FreeMono", "Cascadia Mono", "DejaVu Sans Mono", monospace`
    );
    root.style.setProperty(
      '--code-font-stack',
      `"${next.typography.code_font_family}", "FreeMono", "Cascadia Mono", "DejaVu Sans Mono", monospace`
    );
    if (next.preview_active) {
      void refreshPreviewHtml();
    }
    queueMicrotask(() => {
      updateTabOverflowState();
      if (next.active_tab_id !== lastActiveTabIdForScroll) {
        lastActiveTabIdForScroll = next.active_tab_id;
        const activeCard = tabStripEl?.querySelector('.tab-card.is-active');
        activeCard?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
        updateTabOverflowState();
      }
    });
  }

  $effect(() => {
    const openNow = Boolean(snapshot?.search_bar_open);
    if (openNow && !wasSearchBarOpen) {
      wasSearchBarOpen = true;
      queueMicrotask(() => {
        searchInputEl?.focus();
        searchInputEl?.select();
      });
    } else if (!openNow && wasSearchBarOpen) {
      wasSearchBarOpen = false;
      queueMicrotask(() => {
        cmRef?.focusEditor();
      });
    }
  });

  function isMathEnabledInConfig(snap: UiSessionSnapshot | null): boolean {
    return Boolean(
      snap &&
        snap.markdown_config.base === 'Gfm' &&
        snap.markdown_config.extensions.math
    );
  }

  async function refreshPreviewHtml() {
    try {
      const rawHtml = await renderPreviewHtml();
      previewHtml = renderMathInPreviewHtml(rawHtml, isMathEnabledInConfig(snapshot));
    } catch {
      // Ignore transient preview errors
    }
  }

  async function send(action: SessionAction) {
    const shouldSyncEditor = actionMutatesEditorOrSelection(action);
    if (shouldSyncEditor) {
      latestEditSeq += 1;
    }
    try {
      const next = await dispatchAction(action);
      syncSnapshot(next, shouldSyncEditor);
    } catch (err) {
      console.error('dispatchAction failed:', err);
    }
  }

  async function handleEditorEdit(
    text: string,
    selPrimary: number,
    selSecondary: number,
    isDocChanged: boolean
  ) {
    if (!snapshot) return;
    const mySeq = ++latestEditSeq;
    try {
      if (isDocChanged) {
        const next = await updateEditorBuffer(
          snapshot.active_tab_id,
          text,
          selPrimary,
          selSecondary
        );
        if (mySeq === latestEditSeq) {
          syncSnapshot(next, false);
        }
      } else if (
        selPrimary !== snapshot.cursor_char ||
        Math.min(selPrimary, selSecondary) !== snapshot.selection_start_char ||
        Math.max(selPrimary, selSecondary) !== snapshot.selection_end_char
      ) {
        const next = await dispatchAction({ SetSelection: [selPrimary, selSecondary] });
        if (mySeq === latestEditSeq) {
          syncSnapshot(next, false);
        }
      }
    } catch (err) {
      console.error('updateEditorBuffer failed:', err);
    }
  }

  function updateTabOverflowState() {
    if (!tabStripEl) return;
    const maxScroll = tabStripEl.scrollWidth - tabStripEl.clientWidth;
    tabStripCanScrollLeft = tabStripEl.scrollLeft > 2;
    tabStripCanScrollRight = maxScroll > 2 && tabStripEl.scrollLeft < maxScroll - 2;
  }

  function scrollTabStrip(dir: -1 | 1) {
    if (!tabStripEl) return;
    tabStripEl.scrollBy({ left: dir * 180, behavior: 'smooth' });
    setTimeout(updateTabOverflowState, 180);
  }

  function handleTabStripWheel(e: WheelEvent) {
    if (!tabStripEl) return;
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (Math.abs(delta) > 0.5) {
      e.preventDefault();
      tabStripEl.scrollLeft += delta;
      updateTabOverflowState();
    }
  }

  function handleGlobalMouseMove(e: MouseEvent) {
    if (!snapshot) return;
    const dx = e.clientX - lastMousePos.x;
    const dy = e.clientY - lastMousePos.y;
    const distSq = dx * dx + dy * dy;
    lastMousePos = { x: e.clientX, y: e.clientY };
    if (snapshot.chrome_autohidden && distSq > 9) {
      void send({ SetChromeAutohidden: false });
    }
  }

  function isModalOverlayActive(): boolean {
    if (!snapshot) return false;
    return (
      themesModalOpen ||
      pendingCloseTabIndex !== null ||
      snapshot.overlay === 'PreferencesDialog' ||
      snapshot.overlay === 'ExportDialog' ||
      snapshot.overlay === 'ShortcutsDialog' ||
      snapshot.overlay === 'AboutDialog'
    );
  }

  async function requestCloseTab(idx: number) {
    if (!snapshot) return;
    const targetTab = snapshot.tabs[idx];
    if (!targetTab) return;
    if (!targetTab.is_dirty) {
      await send({ CloseTab: idx });
      return;
    }
    if (snapshot.overlay !== 'None') {
      await send({ SetOverlay: 'None' });
    }
    pendingCloseTabIndex = idx;
  }

  async function confirmDiscardAndCloseTab() {
    if (pendingCloseTabIndex === null) return;
    const idx = pendingCloseTabIndex;
    pendingCloseTabIndex = null;
    await send({ CloseTab: idx });
  }

  async function confirmSaveAndCloseTab() {
    if (!snapshot || pendingCloseTabIndex === null) return;
    const idx = pendingCloseTabIndex;
    pendingCloseTabIndex = null;
    if (idx !== snapshot.active_tab_index) {
      await send({ SwitchTab: idx });
    }
    if (snapshot?.current_file_path) {
      await send('SaveDocument');
      if (!snapshot?.is_dirty) {
        await send({ CloseTab: snapshot!.active_tab_index });
      }
    } else {
      latestEditSeq += 1;
      const next = await saveAsNativeFileDialog();
      syncSnapshot(next, true);
      if (!next.is_dirty && next.current_file_path) {
        await send({ CloseTab: next.active_tab_index });
      }
    }
  }

  function handleGlobalKeyDown(e: KeyboardEvent) {
    if (!snapshot) return;
    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;
    const alt = e.altKey;
    const target = e.target as HTMLElement | null;
    const isFormField = Boolean(target && target.closest('input, textarea, select'));

    if (
      snapshot.chrome_autohidden &&
      (e.key === 'Escape' ||
        e.key === 'F11' ||
        ctrl ||
        alt ||
        e.key === 'Home' ||
        e.key === 'End' ||
        e.key === 'PageUp' ||
        e.key === 'PageDown')
    ) {
      void send({ SetChromeAutohidden: false });
    }

    if (e.key === 'Escape') {
      if (pendingCloseTabIndex !== null) {
        pendingCloseTabIndex = null;
        e.preventDefault();
        return;
      }
      if (themesModalOpen) {
        themesModalOpen = false;
        e.preventDefault();
        return;
      }
      if (snapshot.overlay === 'PreferencesDialog') {
        void cancelPreferencesModal();
        e.preventDefault();
        return;
      }
      if (snapshot.overlay !== 'None') {
        void send({ SetOverlay: 'None' });
        e.preventDefault();
        return;
      }
      if (snapshot.search_bar_open) {
        void send('ToggleSearchBar');
        e.preventDefault();
        return;
      }
    }

    if (e.key === 'F11') {
      e.preventDefault();
      void send('ToggleFullscreen');
      return;
    }

    if (ctrl && !alt) {
      if (isModalOverlayActive()) {
        return;
      }
      const k = e.key.toLowerCase();
      if ((k === 'f' || k === 'h') && !shift) {
        e.preventDefault();
        e.stopPropagation();
        if (k === 'h' && snapshot.search_bar_open) {
          searchInputEl?.focus();
          searchInputEl?.select();
        } else {
          void send('ToggleSearchBar');
        }
      } else if (k === 's' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        if (snapshot.current_file_path) {
          void send('SaveDocument');
        } else {
          void handleSaveAsNativeDialog();
        }
      } else if (k === 's' && shift) {
        e.preventDefault();
        e.stopPropagation();
        void handleSaveAsNativeDialog();
      } else if (k === 'o' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        void handleOpenNativeDialog();
      } else if (k === 't' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        void send('NewTab');
      } else if (k === 'w' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        void requestCloseTab(snapshot.active_tab_index);
      } else if (e.key === 'Tab' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        void send('NextTab');
      } else if (e.key === 'Tab' && shift) {
        e.preventDefault();
        e.stopPropagation();
        void send('PrevTab');
      } else if (k === 'n' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        void send('NewDocument');
      } else if (k === 'l' && !shift) {
        e.preventDefault();
        e.stopPropagation();
        void send('ToggleViewWhileTyping');
      } else if (k === 'p' && shift) {
        e.preventDefault();
        e.stopPropagation();
        void send('ToggleSplitPreview');
      } else if (e.key === '+' || e.key === '=') {
        e.preventDefault();
        e.stopPropagation();
        void send('ZoomIn');
      } else if (e.key === '-') {
        e.preventDefault();
        e.stopPropagation();
        void send('ZoomOut');
      } else if (e.key === '0') {
        e.preventDefault();
        e.stopPropagation();
        void send('ZoomReset');
      } else if (!isFormField) {
        if (k === 'z' && !shift) {
          e.preventDefault();
          e.stopPropagation();
          void send('Undo');
        } else if ((k === 'z' && shift) || (k === 'y' && !shift)) {
          e.preventDefault();
          e.stopPropagation();
          void send('Redo');
        } else if (k === 'b' && !shift) {
          e.preventDefault();
          e.stopPropagation();
          void send({ ApplyFormat: 'Bold' });
        } else if (k === 'i' && !shift) {
          e.preventDefault();
          e.stopPropagation();
          void send({ ApplyFormat: 'Italic' });
        } else if (k === 'x' && shift) {
          e.preventDefault();
          e.stopPropagation();
          void send({ ApplyFormat: 'Strikethrough' });
        } else if (k === 'k' && !shift) {
          e.preventDefault();
          e.stopPropagation();
          void send({ ApplyFormat: 'Link' });
        }
      }
    }
  }

  function handleGlobalWheel(e: WheelEvent) {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const step = e.deltaY < 0 ? 0.5 : -0.5;
      void send({ ZoomByStep: step });
    }
  }

  async function handleOpenNativeDialog() {
    try {
      latestEditSeq += 1;
      const next = await openNativeFileDialog();
      syncSnapshot(next, true);
    } catch (err) {
      console.error('openNativeFileDialog failed:', err);
    }
  }

  async function handleSaveAsNativeDialog() {
    try {
      const next = await saveAsNativeFileDialog();
      syncSnapshot(next, false);
    } catch (err) {
      console.error('saveAsNativeFileDialog failed:', err);
    }
  }

  async function handleExportWithDialog(
    format: ExportFormat | null,
    executeImmediately: boolean
  ) {
    try {
      const next = await exportWithNativeDialog(format, executeImmediately);
      syncSnapshot(next, false);
    } catch (err) {
      console.error('exportWithNativeDialog failed:', err);
    }
  }

  async function openThemesModal() {
    if (!snapshot) return;
    draftTheme = snapshot.theme;
    draftPreset = snapshot.palette_preset;
    draftPreviewTokens = await previewPaletteTokens(draftPreset, draftTheme);
    await send({ SetOverlay: 'None' });
    themesModalOpen = true;
  }

  async function updateThemesDraft(mode: ThemeMode, preset: ColorPalettePreset) {
    draftTheme = mode;
    draftPreset = preset;
    draftPreviewTokens = await previewPaletteTokens(preset, mode);
  }

  async function saveThemesModal() {
    themesModalOpen = false;
    await send({ SetTheme: draftTheme });
    await send({ SetPalettePreset: draftPreset });
  }

  async function openPreferencesModal() {
    if (!snapshot) return;
    prefsRollback = {
      typography: { ...snapshot.typography },
      line_width: snapshot.line_width,
      markdown_config: {
        base: snapshot.markdown_config.base,
        extensions: { ...snapshot.markdown_config.extensions },
      },
      locale: snapshot.locale,
      wal_autosave_enabled: snapshot.wal_autosave_enabled,
      disk_watcher_enabled: snapshot.disk_watcher_enabled,
    };
    await send({ SetOverlay: 'PreferencesDialog' });
  }

  async function cancelPreferencesModal() {
    if (prefsRollback) {
      await send({ SetProseFont: prefsRollback.typography.prose_font_family });
      await send({ SetCodeFont: prefsRollback.typography.code_font_family });
      await send({ SetFontSize: prefsRollback.typography.font_size });
      await send({ SetLineHeight: prefsRollback.typography.line_height });
      await send({ SetLineWidth: prefsRollback.line_width });
      await send({ SetMarkdownConfig: prefsRollback.markdown_config });
      await send({ SetLocale: prefsRollback.locale });
      await send({ SetWalAutosave: prefsRollback.wal_autosave_enabled });
      await send({ SetDiskWatcher: prefsRollback.disk_watcher_enabled });
      prefsRollback = null;
    }
    await send({ SetOverlay: 'None' });
  }

  async function savePreferencesModal() {
    prefsRollback = null;
    await send({ SetOverlay: 'None' });
  }

  async function restoreTypographyDefaults() {
    await send({ SetProseFont: 'FreeMono' });
    await send({ SetCodeFont: 'FreeMono' });
    await send({ SetFontSize: 17.35 });
    await send({ SetLineHeight: 1.505 });
    await send({ SetLineWidth: 'Compact' });
  }

  async function restoreEditorLangDefaults() {
    await send({ SetWalAutosave: true });
    await send({ SetDiskWatcher: true });
    await send({
      SetMarkdownConfig: {
        base: DEFAULT_MARKDOWN_CONFIG.base,
        extensions: { ...DEFAULT_MARKDOWN_CONFIG.extensions },
      },
    });
    await send({ SetLocale: 'It' });
  }

  function setMarkdownBase(base: MarkdownBase) {
    if (!snapshot) return;
    void send({
      SetMarkdownConfig: {
        base,
        extensions: { ...snapshot.markdown_config.extensions },
      },
    });
  }

  function toggleMarkdownExtension(key: keyof MarkdownExtensions) {
    if (!snapshot || snapshot.markdown_config.base === 'CommonMark') return;
    void send({
      SetMarkdownConfig: {
        base: snapshot.markdown_config.base,
        extensions: {
          ...snapshot.markdown_config.extensions,
          [key]: !snapshot.markdown_config.extensions[key],
        },
      },
    });
  }

  function fallbackCopyText(text: string): boolean {
    try {
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.setAttribute('readonly', '');
      ta.style.position = 'fixed';
      ta.style.top = '-9999px';
      ta.style.opacity = '0';
      document.body.appendChild(ta);
      ta.select();
      const ok = document.execCommand('copy');
      document.body.removeChild(ta);
      return ok;
    } catch {
      return false;
    }
  }

  async function handleCopyHtml() {
    try {
      const rawHtml = await renderPreviewHtml();
      const html = renderMathInPreviewHtml(rawHtml, isMathEnabledInConfig(snapshot));
      let copied = false;
      try {
        await navigator.clipboard.writeText(html);
        copied = true;
      } catch {
        copied = fallbackCopyText(html);
      }
      await send({ SetOverlay: 'None' });
      if (copied) {
        await send({ SetStatusMessage: tr('html_copied', 'HTML copied') });
      }
    } catch {
      await send({ SetOverlay: 'None' });
    }
  }

  function scrollPreviewToHeadingIndex(idx: number) {
    if (!previewPaneEl) return;
    const headings = previewPaneEl.querySelectorAll(
      '.preview-prose h1, .preview-prose h2, .preview-prose h3, .preview-prose h4, .preview-prose h5, .preview-prose h6'
    );
    const target = headings[idx] as HTMLElement | undefined;
    target?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  function slugifyHeadingText(raw: string): string {
    return raw
      .trim()
      .toLowerCase()
      .replace(/[^\p{L}\p{N}\s-]/gu, '')
      .replace(/\s+/g, '-');
  }

  function handlePreviewClick(e: MouseEvent) {
    const anchor = (e.target as HTMLElement | null)?.closest('a');
    if (!anchor) return;
    e.preventDefault();
    e.stopPropagation();
    const href = anchor.getAttribute('href')?.trim() ?? '';
    if (!href) return;

    if (href.startsWith('#')) {
      const targetSlug = decodeURIComponent(href.slice(1)).trim().toLowerCase();
      if (!targetSlug || !previewPaneEl) return;
      const byId = previewPaneEl.querySelector(`[id="${CSS.escape(targetSlug)}"]`);
      if (byId) {
        byId.scrollIntoView({ behavior: 'smooth', block: 'start' });
        return;
      }
      const headings = previewPaneEl.querySelectorAll(
        '.preview-prose h1, .preview-prose h2, .preview-prose h3, .preview-prose h4, .preview-prose h5, .preview-prose h6'
      );
      for (const h of headings) {
        if (slugifyHeadingText(h.textContent ?? '') === targetSlug) {
          h.scrollIntoView({ behavior: 'smooth', block: 'start' });
          break;
        }
      }
      return;
    }

    if (href.startsWith('https://') || href.startsWith('http://')) {
      void openExternalUrl(href).catch((err) => {
        console.error('openExternalUrl failed:', err);
      });
      return;
    }

    if (
      !href.includes('://') &&
      !href.startsWith('javascript:') &&
      !href.startsWith('data:') &&
      /\.(md|markdown|txt)$/i.test(href)
    ) {
      const basePath = snapshot?.current_file_path;
      if (basePath) {
        const sep = basePath.includes('\\') ? '\\' : '/';
        const lastSep = Math.max(basePath.lastIndexOf('/'), basePath.lastIndexOf('\\'));
        const dir = lastSep >= 0 ? basePath.slice(0, lastSep) : '.';
        void send({ OpenFile: `${dir}${sep}${href}` });
      } else {
        void send({ OpenFile: href });
      }
    }
  }

  function computeColumnMaxWidth(lw: LineWidthMode): string {
    if (lw === 'Compact') return '712px';
    if (lw === 'Medium') return '920px';
    if (lw === 'Wide') return '1060px';
    return '100%';
  }

  function computeColumnPaddingX(lw: LineWidthMode, pct: number): string {
    if (typeof lw === 'object' && 'Fluid' in lw) {
      return `${Math.max(2, Math.min(25, pct))}vw`;
    }
    return '48px';
  }

  let lastHeaderClickAt = 0;
  let lastHeaderClickX = 0;
  let lastHeaderClickY = 0;

  function handleHeaderMouseDown(e: MouseEvent) {
    if (e.button !== 0 || snapshot?.fullscreen) return;
    const target = e.target as HTMLElement | null;
    if (
      target?.closest(
        'button, a, input, select, textarea, [role="button"], [role="menuitem"]'
      )
    ) {
      return;
    }

    // Stop bubbling to Tauri's global drag.js listener so stationary clicks don't trigger
    // an immediate X11 pointer grab (which breaks double-click on Linux) or double-toggle on Windows.
    e.preventDefault();
    e.stopPropagation();

    const now = performance.now();
    const isDoubleClick =
      now - lastHeaderClickAt <= 340 &&
      Math.hypot(e.screenX - lastHeaderClickX, e.screenY - lastHeaderClickY) <= 8;

    if (isDoubleClick) {
      lastHeaderClickAt = 0;
      void windowToggleMaximize().then((m) => {
        isMaximized = m;
      });
      return;
    }

    lastHeaderClickAt = now;
    lastHeaderClickX = e.screenX;
    lastHeaderClickY = e.screenY;

    const startX = e.screenX;
    const startY = e.screenY;
    const cleanup = () => {
      window.removeEventListener('mousemove', onMove, true);
      window.removeEventListener('mouseup', onUp, true);
    };
    const onMove = (ev: MouseEvent) => {
      if ((ev.buttons & 1) === 0) {
        cleanup();
        return;
      }
      if (Math.hypot(ev.screenX - startX, ev.screenY - startY) >= 4) {
        cleanup();
        lastHeaderClickAt = 0;
        void windowStartDrag();
      }
    };
    const onUp = () => {
      cleanup();
    };
    window.addEventListener('mousemove', onMove, true);
    window.addEventListener('mouseup', onUp, true);
  }

  function handleEdgeResize(dir: ResizeDirection, e: MouseEvent) {
    if (e.button !== 0 || isMaximized || snapshot?.fullscreen) return;
    e.preventDefault();
    e.stopPropagation();
    void windowStartResize(dir);
  }

  onMount(async () => {
    window.addEventListener('keydown', handleGlobalKeyDown, { capture: true });
    const win = getCurrentWindow();
    try {
      isMaximized = await win.isMaximized();
    } catch {
      // Ignore outside Tauri runtime
    }
    try {
      unlistenResize = await win.onResized(async () => {
        try {
          isMaximized = await win.isMaximized();
        } catch {
          // Ignore transient resize query errors
        }
        updateTabOverflowState();
      });
    } catch {
      // Ignore outside Tauri runtime
    }
    try {
      const initSnap = await getSessionSnapshot();
      syncSnapshot(initSnap, true);
    } catch (err) {
      console.error('Initial getSessionSnapshot failed:', err);
    }
    unlistenSession = await onSessionUpdated((updated) => {
      latestEditSeq += 1;
      syncSnapshot(updated, true);
    });
    window.addEventListener('resize', updateTabOverflowState);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleGlobalKeyDown, { capture: true });
    unlistenSession?.();
    unlistenResize?.();
    window.removeEventListener('resize', updateTabOverflowState);
  });

  let activeOutlineIndex = $derived.by(() => {
    if (!snapshot || snapshot.outline_headings.length === 0) return -1;
    let activeIdx = 0;
    for (let i = 0; i < snapshot.outline_headings.length; i++) {
      if (snapshot.outline_headings[i].char_offset <= snapshot.cursor_char) {
        activeIdx = i;
      } else {
        break;
      }
    }
    return activeIdx;
  });

  let searchMatchSummary = $derived.by(() => {
    if (!snapshot || !snapshot.search_query) return '0 / 0';
    const hay = snapshot.buffer_text.toLowerCase();
    const needle = snapshot.search_query.toLowerCase();
    if (!needle) return '0 / 0';
    const positions: number[] = [];
    let idx = 0;
    while (idx <= hay.length - needle.length) {
      const found = hay.indexOf(needle, idx);
      if (found === -1) break;
      positions.push(found);
      idx = found + Math.max(1, needle.length);
    }
    if (positions.length === 0) return '0 / 0';
    const cursorU16 = charToUtf16Offset(snapshot.buffer_text, snapshot.cursor_char);
    let cur = 1;
    for (let i = 0; i < positions.length; i++) {
      if (positions[i] < cursorU16) cur = i + 1;
    }
    return `${cur} / ${positions.length}`;
  });

  let chromeHidden = $derived(
    Boolean(
      snapshot &&
        snapshot.chrome_autohidden &&
        snapshot.overlay === 'None' &&
        !themesModalOpen &&
        pendingCloseTabIndex === null &&
        !snapshot.search_bar_open &&
        !snapshot.recovery_banner_active &&
        !snapshot.disk_modified_banner_active
    )
  );
</script>

<svelte:window onmousemove={handleGlobalMouseMove} onwheel={handleGlobalWheel} />

{#if snapshot}
  <div
    class="window-shell"
    class:is-maximized={isMaximized || snapshot.fullscreen}
    class:is-chrome-hidden={chromeHidden}
    class:has-banner={snapshot.recovery_banner_active || snapshot.disk_modified_banner_active}
    data-testid="accent-window-shell"
  >
    <!-- Edge & Corner Resize Handles for Frameless Window -->
    {#if !isMaximized && !snapshot.fullscreen}
      <div class="resize-edge resize-n" onmousedown={(e) => handleEdgeResize('North', e)} role="presentation"></div>
      <div class="resize-edge resize-s" onmousedown={(e) => handleEdgeResize('South', e)} role="presentation"></div>
      <div class="resize-edge resize-e" onmousedown={(e) => handleEdgeResize('East', e)} role="presentation"></div>
      <div class="resize-edge resize-w" onmousedown={(e) => handleEdgeResize('West', e)} role="presentation"></div>
      <div class="resize-corner resize-nw" onmousedown={(e) => handleEdgeResize('NorthWest', e)} role="presentation"></div>
      <div class="resize-corner resize-ne" onmousedown={(e) => handleEdgeResize('NorthEast', e)} role="presentation"></div>
      <div class="resize-corner resize-sw" onmousedown={(e) => handleEdgeResize('SouthWest', e)} role="presentation"></div>
      <div class="resize-corner resize-se" onmousedown={(e) => handleEdgeResize('SouthEast', e)} role="presentation"></div>
    {/if}

    <!-- ====================================================================
         1. INTEGRATED CSD TOP BAR (38px)
         ==================================================================== -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header
      class="csd-topbar chrome-autohide"
      class:is-hidden={chromeHidden}
      data-tauri-drag-region="deep"
      data-testid="csd-topbar"
      onmousedown={handleHeaderMouseDown}
    >
      <div class="csd-left" role="toolbar" aria-label="Document actions">
        <button
          id="btn-toggle-outline"
          type="button"
          class="csd-icon-btn"
          class:is-active={outlineOpen}
          aria-pressed={outlineOpen}
          aria-label={tr('tooltip_outline', 'Indice')}
          title={snapshot.overlay !== 'None' ? undefined : tr('tooltip_outline', 'Indice')}
          onclick={() => {
            if (snapshot?.overlay !== 'None') void send({ SetOverlay: 'None' });
            outlineOpen = !outlineOpen;
          }}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <rect x="2" y="2.5" width="12" height="11" rx="2" stroke="currentColor" stroke-width="1.4" />
            <line x1="6" y1="2.5" x2="6" y2="13.5" stroke="currentColor" stroke-width="1.4" />
          </svg>
        </button>

        <button
          id="btn-new-doc"
          type="button"
          class="csd-icon-btn"
          aria-label={tr('tooltip_new_doc')}
          title={snapshot.overlay !== 'None' ? undefined : tr('tooltip_new_doc')}
          onclick={() => send('NewDocument')}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M3.5 2.5H9.5L12.5 5.5V13.5H3.5V2.5Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
            <line x1="8" y1="7" x2="8" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            <line x1="6" y1="9" x2="10" y2="9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
          </svg>
        </button>

        <button
          id="btn-open-menu"
          type="button"
          class="csd-pill-btn"
          class:is-active={snapshot.overlay === 'OpenMenu'}
          aria-haspopup="menu"
          aria-expanded={snapshot.overlay === 'OpenMenu'}
          title={snapshot.overlay === 'OpenMenu' ? undefined : tr('tooltip_open')}
          onclick={() => send({ SetOverlay: 'OpenMenu' })}
        >
          <span>{tr('open')}</span>
          <span class="chevron" aria-hidden="true">▾</span>
        </button>

        <div class="csd-split-btn" role="group" aria-label={tr('save')}>
          <button
            id="btn-save-doc"
            type="button"
            class="csd-split-main"
            title={snapshot.overlay !== 'None' ? undefined : tr('tooltip_save')}
            onclick={() => {
              if (snapshot?.overlay !== 'None') void send({ SetOverlay: 'None' });
              if (snapshot?.current_file_path) {
                void send('SaveDocument');
              } else {
                void handleSaveAsNativeDialog();
              }
            }}
          >
            {tr('save')}
          </button>
          <button
            id="btn-save-menu"
            type="button"
            class="csd-split-arrow"
            class:is-active={snapshot.overlay === 'SaveMenu'}
            aria-label={tr('tooltip_save_menu')}
            aria-haspopup="menu"
            aria-expanded={snapshot.overlay === 'SaveMenu'}
            title={snapshot.overlay === 'SaveMenu' ? undefined : tr('tooltip_save_menu')}
            onclick={() => send({ SetOverlay: 'SaveMenu' })}
          >
            ▾
          </button>
        </div>
      </div>

      <!-- Center: Active Document Title (with dirty dot) -->
      <div class="csd-center" role="presentation">
        {#if snapshot.is_dirty}
          <span class="dirty-dot" title={tr('status_unsaved')}></span>
        {/if}
        <span class="csd-title">{snapshot.document_title}</span>
      </div>

      <!-- Right Controls: LineWidth, Search, View Mode, Main Menu ☰, Minimize ─, Maximize □, Close × -->
      <div class="csd-right" role="toolbar" aria-label="View and window controls">
        <button
          id="btn-linewidth-menu"
          type="button"
          class="csd-badge-btn"
          class:is-active={snapshot.overlay === 'LineWidthMenu'}
          title={snapshot.overlay === 'LineWidthMenu' ? undefined : tr('tooltip_line_width')}
          onclick={() => send({ SetOverlay: 'LineWidthMenu' })}
        >
          {snapshot.line_width_badge}
        </button>

        <button
          id="btn-search-toggle"
          type="button"
          class="csd-icon-btn"
          class:is-active={snapshot.search_bar_open}
          aria-pressed={snapshot.search_bar_open}
          aria-label={tr('tooltip_search')}
          title={snapshot.overlay !== 'None' ? undefined : tr('tooltip_search')}
          onclick={() => send('ToggleSearchBar')}
        >
          <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <circle cx="7" cy="7" r="4.5" stroke="currentColor" stroke-width="1.5" />
            <line x1="10.4" y1="10.4" x2="13.8" y2="13.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </button>

        <button
          id="btn-viewmode-menu"
          type="button"
          class="csd-pill-btn viewmode-pill"
          class:is-active={snapshot.overlay === 'PreviewMenu' || snapshot.preview_active}
          aria-label={tr('tooltip_preview')}
          aria-haspopup="menu"
          aria-expanded={snapshot.overlay === 'PreviewMenu'}
          title={snapshot.overlay === 'PreviewMenu' ? undefined : tr('tooltip_preview')}
          onclick={() => send({ SetOverlay: 'PreviewMenu' })}
        >
          <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <rect x="2" y="2.5" width="12" height="11" rx="2" stroke="currentColor" stroke-width="1.4" />
            {#if snapshot.preview_active}
              <line x1="8" y1="2.5" x2="8" y2="13.5" stroke="currentColor" stroke-width="1.4" />
            {:else if snapshot.view_while_typing}
              <circle cx="8" cy="8" r="2.2" fill="currentColor" />
            {/if}
          </svg>
          <span class="chevron" aria-hidden="true">▾</span>
        </button>

        <button
          id="btn-main-menu"
          type="button"
          class="csd-icon-btn"
          class:is-active={snapshot.overlay === 'MainMenu'}
          aria-label={tr('tooltip_main_menu')}
          aria-haspopup="menu"
          aria-expanded={snapshot.overlay === 'MainMenu'}
          title={snapshot.overlay === 'MainMenu' ? undefined : tr('tooltip_main_menu')}
          onclick={() => send({ SetOverlay: 'MainMenu' })}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <line x1="2.5" y1="4.2" x2="13.5" y2="4.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
            <line x1="2.5" y1="8" x2="13.5" y2="8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
            <line x1="2.5" y1="11.8" x2="13.5" y2="11.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </button>

        <span class="csd-win-sep" role="separator" aria-orientation="vertical"></span>

        <div class="csd-window-controls" role="group" aria-label="Window controls">
          <button
            id="csd-minimize-btn"
            type="button"
            class="csd-win-btn"
            aria-label={tr('tooltip_minimize')}
            title={tr('tooltip_minimize')}
            onclick={() => windowMinimize()}
          >
            ─
          </button>
          <button
            id="csd-maximize-btn"
            type="button"
            class="csd-win-btn"
            aria-label={tr('tooltip_maximize')}
            title={tr('tooltip_maximize')}
            onclick={async () => {
              isMaximized = await windowToggleMaximize();
            }}
          >
            □
          </button>
          <button
            id="csd-close-btn"
            type="button"
            class="csd-win-btn csd-win-close"
            aria-label={tr('tooltip_close')}
            title={tr('tooltip_close')}
            onclick={() => windowClose()}
          >
            ×
          </button>
        </div>
      </div>
    </header>

    <!-- ====================================================================
         2. ALWAYS-VISIBLE DEDICATED MULTI-TAB BAR (32px, directly below CSD)
         ==================================================================== -->
    <nav
      class="multitab-bar chrome-autohide"
      class:is-hidden={chromeHidden}
      aria-label="Open document tabs"
      data-testid="multitab-bar"
    >
      <div
        class="tab-strip scroller"
        data-tauri-drag-region
        bind:this={tabStripEl}
        onscroll={updateTabOverflowState}
        onwheel={handleTabStripWheel}
      >
        {#each snapshot.tabs as tab (tab.tab_id)}
          {@const isActive = tab.index === snapshot.active_tab_index}
          <div
            class="tab-card"
            class:is-active={isActive}
            data-testid={`tab-card-${tab.index}`}
            role="tab"
            tabindex="0"
            aria-selected={isActive}
            title={tab.current_file_path ?? tab.document_title}
            onclick={() => send({ SwitchTab: tab.index })}
            onauxclick={(e) => {
              if (e.button === 1) void requestCloseTab(tab.index);
            }}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                void send({ SwitchTab: tab.index });
              }
            }}
          >
            {#if isActive}
              <div class="tab-accent-top"></div>
            {/if}
            {#if tab.is_dirty}
              <span class="tab-dirty-dot" class:is-active={isActive}></span>
            {/if}
            <span class="tab-title">{tab.document_title}</span>
            <button
              type="button"
              class="tab-close-btn"
              aria-label={tr('tooltip_close_tab')}
              title={tr('tooltip_close_tab')}
              onclick={(e) => {
                e.stopPropagation();
                void requestCloseTab(tab.index);
              }}
            >
              ×
            </button>
          </div>
        {/each}
      </div>

      <div class="tab-bar-actions">
        {#if tabStripCanScrollLeft || tabStripCanScrollRight}
          <button
            id="tabbar-scroll-left"
            type="button"
            class="tab-ctrl-btn"
            aria-disabled={!tabStripCanScrollLeft}
            aria-label="Scroll tabs left"
            onclick={() => tabStripCanScrollLeft && scrollTabStrip(-1)}
          >
            ‹
          </button>
          <button
            id="tabbar-scroll-right"
            type="button"
            class="tab-ctrl-btn"
            aria-disabled={!tabStripCanScrollRight}
            aria-label="Scroll tabs right"
            onclick={() => tabStripCanScrollRight && scrollTabStrip(1)}
          >
            ›
          </button>
        {/if}

        <button
          id="tabbar-new-tab-btn"
          type="button"
          class="tab-ctrl-btn tab-add-btn"
          aria-label={tr('tooltip_new_tab')}
          title={tr('tooltip_new_tab')}
          onclick={() => send('NewTab')}
        >
          +
        </button>
      </div>
    </nav>

    <!-- ====================================================================
         3. WAL CRASH RECOVERY & EXTERNAL DISK MODIFICATION BANNERS
         ==================================================================== -->
    {#if snapshot.recovery_banner_active}
      <div class="notification-banner" role="alert" data-testid="recovery-banner">
        <span class="banner-text">{tr('recovery_banner_msg')}</span>
        <div class="banner-actions">
          <button
            type="button"
            class="banner-btn banner-btn-confirm"
            onclick={() =>
              snapshot?.current_file_path
                ? send('ConfirmRecoverySave')
                : handleSaveAsNativeDialog()}
          >
            {tr('recovery_confirm_save')}
          </button>
          <button
            type="button"
            class="banner-btn"
            onclick={() => send('DiscardRecovery')}
          >
            {tr('recovery_discard')}
          </button>
        </div>
      </div>
    {:else if snapshot.disk_modified_banner_active}
      <div class="notification-banner" role="alert" data-testid="disk-mod-banner">
        <span class="banner-text">
          {snapshot.is_dirty ? tr('disk_mod_banner_dirty_msg') : tr('disk_mod_banner_msg')}
        </span>
        <div class="banner-actions">
          <button
            type="button"
            class="banner-btn banner-btn-confirm"
            onclick={() => send('ReloadFromDisk')}
          >
            {tr('disk_mod_reload')}
          </button>
          <button
            type="button"
            class="banner-btn"
            onclick={() => send('DismissExternalModification')}
          >
            {tr('disk_mod_ignore')}
          </button>
        </div>
      </div>
    {/if}

    <!-- ====================================================================
         4. MAIN WORKSPACE: DOCUMENT OUTLINE DRAWER + CODEMIRROR 6 + PREVIEW
         ==================================================================== -->
    <div class="workspace-body">
      {#if outlineOpen}
        <aside
          class="outline-drawer chrome-autohide scroller"
          class:is-hidden={chromeHidden}
          aria-label={tr('outline', 'Indice')}
          data-testid="outline-drawer"
        >
          <div class="outline-header">
            <span>{tr('outline', 'Indice')}</span>
            <button
              type="button"
              class="outline-close"
              aria-label="Close outline"
              onclick={() => (outlineOpen = false)}
            >
              ×
            </button>
          </div>
          {#if snapshot.outline_headings.length === 0}
            <p class="outline-empty">{tr('outline_empty')}</p>
          {:else}
            <ul class="outline-list">
              {#each snapshot.outline_headings as h, idx (idx)}
                {@const isCurrentHeading = idx === activeOutlineIndex}
                <li>
                  <button
                    type="button"
                    class="outline-item"
                    class:is-active={isCurrentHeading}
                    aria-current={isCurrentHeading ? 'location' : undefined}
                    style={`padding-left: ${10 + (h.level - 1) * 12}px`}
                    onclick={() => {
                      cmRef?.scrollToCharOffset(h.char_offset);
                      scrollPreviewToHeadingIndex(idx);
                      void send({ MoveCursor: h.char_offset });
                    }}
                  >
                    <span class="outline-lvl">H{h.level}</span>
                    <span class="outline-txt">{h.text}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </aside>
      {/if}

      <main
        class="editor-workspace"
        class:split-right={snapshot.preview_active && snapshot.preview_layout === 'HalfWidth'}
        class:split-bottom={snapshot.preview_active && snapshot.preview_layout === 'HalfHeight'}
        class:read-only-full={snapshot.preview_active && snapshot.preview_layout === 'FullWidth'}
      >
        {#if !(snapshot.preview_active && snapshot.preview_layout === 'FullWidth')}
          <div class="editor-pane">
            <CodeMirrorEditor
              bind:this={cmRef}
              activeTabId={snapshot.active_tab_id}
              bufferText={snapshot.buffer_text}
              bufferRevision={snapshot.buffer_revision}
              externalSyncToken={externalSyncToken}
              cursorChar={snapshot.cursor_char}
              selectionStartChar={snapshot.selection_start_char}
              selectionEndChar={snapshot.selection_end_char}
              viewWhileTyping={snapshot.view_while_typing}
              markdownConfig={snapshot.markdown_config}
              themeMode={snapshot.theme}
              proseFont={snapshot.typography.prose_font_family}
              codeFont={snapshot.typography.code_font_family}
              fontSize={snapshot.typography.font_size}
              lineHeight={snapshot.typography.line_height}
              columnMaxWidth={computeColumnMaxWidth(snapshot.line_width)}
              columnPaddingX={computeColumnPaddingX(
                snapshot.line_width,
                snapshot.fluid_margin_pct
              )}
              searchQuery={snapshot.search_query}
              searchBarOpen={snapshot.search_bar_open}
              chromeHidden={chromeHidden}
              onEdit={handleEditorEdit}
            />
          </div>
        {/if}

        {#if snapshot.preview_active}
          <section
            class="preview-pane scroller"
            bind:this={previewPaneEl}
            data-testid="preview-pane"
          >
            {#if snapshot.preview_layout === 'FullWidth'}
              <div class="preview-reader-bar chrome-autohide" class:is-hidden={chromeHidden}>
                <button
                  type="button"
                  class="csd-pill-btn"
                  onclick={() => send('SetEditorLivePreview')}
                >
                  {tr('back_to_editor')}
                </button>
              </div>
            {/if}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <article
              class="preview-prose"
              onclick={handlePreviewClick}
              style={`max-width: ${computeColumnMaxWidth(snapshot.line_width)}; font-size: ${snapshot.typography.font_size}px; line-height: ${snapshot.typography.line_height};`}
            >
              {@html previewHtml}
            </article>
          </section>
        {/if}
      </main>
    </div>

    <!-- ====================================================================
         5. FLOATING FIND & REPLACE PANEL (Ctrl+F)
         ==================================================================== -->
    {#if snapshot.search_bar_open}
      <div class="search-panel" role="dialog" aria-label={tr('find_replace')} data-testid="search-panel">
        <div class="search-row">
          <div class="search-input-wrap">
            <input
              bind:this={searchInputEl}
              type="text"
              class="search-input search-query-input"
              placeholder={tr('find_placeholder')}
              aria-label={tr('find_placeholder')}
              value={snapshot.search_query}
              oninput={(e) => send({ SetSearchQuery: (e.currentTarget as HTMLInputElement).value })}
              onkeydown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  void send(e.shiftKey ? 'FindPrev' : 'FindNext');
                }
              }}
            />
            <span class="search-counter">{searchMatchSummary}</span>
          </div>
          <button type="button" class="search-mini-btn" onclick={() => send('FindPrev')} aria-label="Previous match" title="Previous (Shift+Enter)">↑</button>
          <button type="button" class="search-mini-btn" onclick={() => send('FindNext')} aria-label="Next match" title="Next (Enter)">↓</button>
          <button type="button" class="search-mini-btn" onclick={() => send('ToggleSearchBar')} aria-label="Close search" title="Close (Esc)">×</button>
        </div>
        <div class="search-row">
          <input
            type="text"
            class="search-input"
            placeholder={tr('replace_placeholder')}
            aria-label={tr('replace_placeholder')}
            value={snapshot.replace_text}
            oninput={(e) => send({ SetReplaceText: (e.currentTarget as HTMLInputElement).value })}
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                void send('ReplaceNext');
              }
            }}
          />
          <button type="button" class="search-action-btn" onclick={() => send('ReplaceNext')}>
            {tr('replace_btn')}
          </button>
        </div>
      </div>
    {/if}

    <!-- ====================================================================
         6. DUAL FLOATING BOTTOM ISLANDS (Left: 12 Vector Formatting Icons; Right: Status & Stats)
         ==================================================================== -->
    <footer
      class="bottom-islands-layer chrome-autohide"
      class:is-hidden={chromeHidden}
      data-testid="bottom-islands"
    >
      <!-- Left Floating Island: 12 Vector Markdown Formatting Controls + Collapse ‹ / Micro-Circle › -->
      {#if !(snapshot.preview_active && snapshot.preview_layout === 'FullWidth')}
        <div
          class="floating-island island-left"
          class:is-collapsed={!snapshot.toolbar_expanded}
          role="toolbar"
          aria-label="Markdown formatting"
          data-testid="island-formatting"
        >
          {#if snapshot.toolbar_expanded}
            <div class="fmt-group" role="group" aria-label="Inline typography">
              <button id="fmt-bold" type="button" class="fmt-btn" title={tr('tooltip_bold')} aria-label={tr('tooltip_bold')} onclick={() => send({ ApplyFormat: 'Bold' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path d="M4.5 3H8.8C10.3 3 11.4 3.9 11.4 5.3C11.4 6.3 10.7 7.1 9.8 7.4C11 7.7 11.9 8.6 11.9 9.9C11.9 11.5 10.6 12.5 8.9 12.5H4.5V3Z" stroke="currentColor" stroke-width="1.65" stroke-linejoin="round" />
                  <line x1="4.5" y1="7.5" x2="9.2" y2="7.5" stroke="currentColor" stroke-width="1.5" />
                </svg>
              </button>
              <button id="fmt-italic" type="button" class="fmt-btn" title={tr('tooltip_italic')} aria-label={tr('tooltip_italic')} onclick={() => send({ ApplyFormat: 'Italic' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <line x1="6.5" y1="3" x2="11.5" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                  <line x1="4.5" y1="12.5" x2="9.5" y2="12.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                  <line x1="9.2" y1="3" x2="6.8" y2="12.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                </svg>
              </button>
              <button id="fmt-strike" type="button" class="fmt-btn" title={tr('tooltip_strikethrough')} aria-label={tr('tooltip_strikethrough')} onclick={() => send({ ApplyFormat: 'Strikethrough' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path d="M10.8 4.8C10.3 3.8 9.2 3.2 7.8 3.2C6 3.2 4.8 4.1 4.8 5.4C4.8 8.2 11.2 7.4 11.2 10.4C11.2 11.8 9.8 12.8 7.9 12.8C6.2 12.8 5 12 4.4 10.8" stroke="currentColor" stroke-width="1.45" stroke-linecap="round" />
                  <line x1="2.5" y1="8" x2="13.5" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                </svg>
              </button>
              <button id="fmt-heading" type="button" class="fmt-btn" title={tr('tooltip_heading')} aria-label={tr('tooltip_heading')} onclick={() => send({ ApplyFormat: 'Heading' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <line x1="4.2" y1="3" x2="4.2" y2="12.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
                  <line x1="11.8" y1="3" x2="11.8" y2="12.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
                  <line x1="4.2" y1="7.8" x2="11.8" y2="7.8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
                </svg>
              </button>
            </div>

            <span class="fmt-sep" role="separator" aria-orientation="vertical"></span>

            <div class="fmt-group" role="group" aria-label="Lists and quotes">
              <button id="fmt-bullet" type="button" class="fmt-btn" title={tr('tooltip_bullet_list')} aria-label={tr('tooltip_bullet_list')} onclick={() => send({ ApplyFormat: 'BulletList' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <circle cx="3.5" cy="4.5" r="1.2" fill="currentColor" />
                  <circle cx="3.5" cy="8" r="1.2" fill="currentColor" />
                  <circle cx="3.5" cy="11.5" r="1.2" fill="currentColor" />
                  <line x1="6.5" y1="4.5" x2="13" y2="4.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                  <line x1="6.5" y1="8" x2="13" y2="8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                  <line x1="6.5" y1="11.5" x2="13" y2="11.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                </svg>
              </button>
              <button id="fmt-ordered" type="button" class="fmt-btn" title={tr('tooltip_ordered_list')} aria-label={tr('tooltip_ordered_list')} onclick={() => send({ ApplyFormat: 'OrderedList' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <text x="2" y="6.2" font-size="5.5" font-weight="700" fill="currentColor">1.</text>
                  <text x="2" y="12.2" font-size="5.5" font-weight="700" fill="currentColor">2.</text>
                  <line x1="7" y1="4.5" x2="13" y2="4.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                  <line x1="7" y1="10.5" x2="13" y2="10.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                </svg>
              </button>
              <button id="fmt-task" type="button" class="fmt-btn" title={tr('tooltip_task_list')} aria-label={tr('tooltip_task_list')} onclick={() => send({ ApplyFormat: 'TaskList' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <rect x="2.5" y="3" width="11" height="10" rx="2" stroke="currentColor" stroke-width="1.4" />
                  <path d="M5.2 8.2L7.1 10.1L10.8 5.9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
              <button id="fmt-quote" type="button" class="fmt-btn" title={tr('tooltip_blockquote')} aria-label={tr('tooltip_blockquote')} onclick={() => send({ ApplyFormat: 'BlockQuote' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path d="M6.5 4.5H3.8V8.2H6.2L5 11.5M12.2 4.5H9.5V8.2H11.9L10.7 11.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
            </div>

            <span class="fmt-sep" role="separator" aria-orientation="vertical"></span>

            <div class="fmt-group" role="group" aria-label="Code, links, media, and tables">
              <button id="fmt-code" type="button" class="fmt-btn" title={tr('tooltip_code')} aria-label={tr('tooltip_code')} onclick={() => send({ ApplyFormat: 'Code' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path d="M5.8 4.5L2.5 8L5.8 11.5M10.2 4.5L13.5 8L10.2 11.5" stroke="currentColor" stroke-width="1.45" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
              <button id="fmt-link" type="button" class="fmt-btn" title={tr('tooltip_link')} aria-label={tr('tooltip_link')} onclick={() => send({ ApplyFormat: 'Link' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path d="M6.8 9.2L9.2 6.8M5.5 11.2L4.2 12.5C3.1 13.6 1.8 12.3 2.9 11.2L4.5 9.6M10.5 4.8L11.8 3.5C12.9 2.4 14.2 3.7 13.1 4.8L11.5 6.4" stroke="currentColor" stroke-width="1.45" stroke-linecap="round" />
                </svg>
              </button>
              <button id="fmt-image" type="button" class="fmt-btn" title={tr('tooltip_image')} aria-label={tr('tooltip_image')} onclick={() => send({ ApplyFormat: 'Image' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <rect x="2.2" y="3" width="11.6" height="10" rx="1.8" stroke="currentColor" stroke-width="1.4" />
                  <circle cx="5.5" cy="6" r="1.1" fill="currentColor" />
                  <path d="M3 11.8L6.8 8.4L9.5 10.6L11.2 9.2L13 11.8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
              </button>
              <button id="fmt-table" type="button" class="fmt-btn" title={tr('tooltip_table')} aria-label={tr('tooltip_table')} onclick={() => send({ ApplyFormat: 'Table' })}>
                <svg width="15" height="15" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <rect x="2.2" y="3" width="11.6" height="10" rx="1.8" stroke="currentColor" stroke-width="1.4" />
                  <line x1="2.2" y1="6.8" x2="13.8" y2="6.8" stroke="currentColor" stroke-width="1.3" />
                  <line x1="8" y1="3" x2="8" y2="13" stroke="currentColor" stroke-width="1.3" />
                </svg>
              </button>
            </div>

            <span class="fmt-sep" role="separator" aria-orientation="vertical"></span>
          {/if}

          <button
            id="btn-toggle-toolbar"
            type="button"
            class="fmt-btn fmt-collapse-btn"
            aria-label={tr('tooltip_toggle_toolbar')}
            title={tr('tooltip_toggle_toolbar')}
            onclick={() => send('ToggleToolbar')}
          >
            {snapshot.toolbar_expanded ? '‹' : '›'}
          </button>
        </div>
      {/if}

      <!-- Right Floating Island: Status & Document Statistics Pill -->
      <div
        class="floating-island island-right"
        role="group"
        aria-label="Document status and statistics"
        data-testid="island-stats"
      >
        <span class="status-pill" role="status">
          <span class="status-dot" class:is-dirty={snapshot.is_dirty}></span>
          <span class="status-text">
            {snapshot.status_message ??
              (snapshot.is_dirty ? tr('status_unsaved') : tr('status_saved'))}
          </span>
        </span>
        <span class="fmt-sep" role="separator" aria-orientation="vertical"></span>
        <button
          id="btn-stats-popover"
          type="button"
          class="stats-pill-btn"
          class:is-active={snapshot.overlay === 'StatsMenu'}
          aria-haspopup="menu"
          aria-expanded={snapshot.overlay === 'StatsMenu'}
          title={snapshot.overlay === 'StatsMenu' ? undefined : tr('tooltip_stats')}
          onclick={() => send({ SetOverlay: 'StatsMenu' })}
        >
          <span>{snapshot.active_stat_label}</span>
          <span class="chevron" aria-hidden="true">▴</span>
        </button>
      </div>
    </footer>

    <!-- ====================================================================
         7. POPOVERS & MODAL DIALOGS
         ==================================================================== -->
    {#if snapshot.overlay !== 'None'}
      <div
        class="overlay-backdrop"
        class:is-modal={snapshot.overlay === 'PreferencesDialog' ||
          snapshot.overlay === 'ExportDialog' ||
          snapshot.overlay === 'ShortcutsDialog' ||
          snapshot.overlay === 'AboutDialog'}
        role="presentation"
        onclick={() =>
          snapshot?.overlay === 'PreferencesDialog'
            ? cancelPreferencesModal()
            : send({ SetOverlay: 'None' })}
      ></div>
    {/if}

    {#if snapshot.overlay === 'OpenMenu'}
      <div class="popover open-popover" role="menu" data-testid="popover-open">
        <div class="popover-section-title">{tr('recent_docs')}</div>
        {#each snapshot.recent_files as rf (rf)}
          {@const fname = rf.split(/[\\/]/).pop() ?? rf}
          <button
            type="button"
            class="popover-item"
            role="menuitem"
            title={rf}
            onclick={() => send({ OpenFile: rf })}
          >
            <span class="popover-item-main">{fname}</span>
            <span class="popover-item-sub">{rf}</span>
          </button>
        {/each}
        <div class="popover-divider"></div>
        <button
          id="menu-item-open-native"
          type="button"
          class="popover-item popover-item-accent"
          role="menuitem"
          onclick={handleOpenNativeDialog}
        >
          {tr('open_file')}
        </button>
      </div>
    {:else if snapshot.overlay === 'SaveMenu'}
      <div class="popover save-popover" role="menu" data-testid="popover-save">
        <button id="menu-item-save-as" type="button" class="popover-item" role="menuitem" onclick={handleSaveAsNativeDialog}>
          <span>{tr('save_as')}</span>
          <span class="shortcut-hint" aria-hidden="true">Ctrl+Shift+S</span>
        </button>
        <button
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => send({ SetOverlay: 'ExportDialog' })}
        >
          <span>{tr('advanced_export')}</span>
        </button>
        <button type="button" class="popover-item" role="menuitem" onclick={handleCopyHtml}>
          <span>{tr('copy_html')}</span>
        </button>
        <div class="popover-divider"></div>
        {#each ['Pdf', 'Docx', 'Odt', 'Latex', 'Epub', 'Html'] as fmt (fmt)}
          <button
            type="button"
            class="popover-item"
            role="menuitem"
            onclick={() => handleExportWithDialog(fmt as ExportFormat, true)}
          >
            <span>{tr('export_fmt_prefix', 'Esporta')} {fmt.toUpperCase()}</span>
          </button>
        {/each}
      </div>
    {:else if snapshot.overlay === 'LineWidthMenu'}
      <div class="popover linewidth-popover" role="menu" data-testid="popover-linewidth">
        <button
          type="button"
          class="popover-item"
          class:is-selected={snapshot.line_width === 'Compact'}
          onclick={() => {
            send({ SetLineWidth: 'Compact' });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('compact')}</span>
          {#if snapshot.line_width === 'Compact'}<span>✓</span>{/if}
        </button>
        <button
          type="button"
          class="popover-item"
          class:is-selected={snapshot.line_width === 'Medium'}
          onclick={() => {
            send({ SetLineWidth: 'Medium' });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('medium')}</span>
          {#if snapshot.line_width === 'Medium'}<span>✓</span>{/if}
        </button>
        <button
          type="button"
          class="popover-item"
          class:is-selected={snapshot.line_width === 'Wide'}
          onclick={() => {
            send({ SetLineWidth: 'Wide' });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('wide')}</span>
          {#if snapshot.line_width === 'Wide'}<span>✓</span>{/if}
        </button>
        <button
          type="button"
          class="popover-item"
          class:is-selected={typeof snapshot.line_width === 'object'}
          onclick={() => {
            send({ SetLineWidth: { Fluid: { margin_pct: snapshot?.fluid_margin_pct ?? 6 } } });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('fluid')}</span>
          {#if typeof snapshot.line_width === 'object'}<span>✓</span>{/if}
        </button>
      </div>
    {:else if snapshot.overlay === 'PreviewMenu'}
      <div class="popover preview-popover" role="menu" data-testid="popover-preview">
        <button
          type="button"
          class="popover-item"
          class:is-selected={!snapshot.preview_active && !snapshot.view_while_typing}
          onclick={() => {
            send('SetEditorOnlyRaw');
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('view_edit_raw')}</span>
          {#if !snapshot.preview_active && !snapshot.view_while_typing}<span>✓</span>{/if}
        </button>
        <button
          type="button"
          class="popover-item"
          class:is-selected={!snapshot.preview_active && snapshot.view_while_typing}
          onclick={() => {
            send('SetEditorLivePreview');
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('view_inline_live')}</span>
          {#if !snapshot.preview_active && snapshot.view_while_typing}<span>✓</span>{/if}
        </button>
        <div class="popover-divider"></div>
        <button
          type="button"
          class="popover-item"
          class:is-selected={snapshot.preview_active && snapshot.preview_layout === 'HalfWidth'}
          onclick={() => {
            send({ SetPreviewMode: { active: true, layout: 'HalfWidth' } });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('view_split_right')}</span>
          {#if snapshot.preview_active && snapshot.preview_layout === 'HalfWidth'}<span>✓</span>{/if}
        </button>
        <button
          type="button"
          class="popover-item"
          class:is-selected={snapshot.preview_active && snapshot.preview_layout === 'HalfHeight'}
          onclick={() => {
            send({ SetPreviewMode: { active: true, layout: 'HalfHeight' } });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('view_split_bottom')}</span>
          {#if snapshot.preview_active && snapshot.preview_layout === 'HalfHeight'}<span>✓</span>{/if}
        </button>
        <button
          type="button"
          class="popover-item"
          class:is-selected={snapshot.preview_active && snapshot.preview_layout === 'FullWidth'}
          onclick={() => {
            send({ SetPreviewMode: { active: true, layout: 'FullWidth' } });
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('view_readonly')}</span>
          {#if snapshot.preview_active && snapshot.preview_layout === 'FullWidth'}<span>✓</span>{/if}
        </button>
      </div>
    {:else if snapshot.overlay === 'MainMenu'}
      <div class="popover main-menu-popover" role="menu" data-testid="popover-main-menu">
        <!-- 4 Brightness Theme Circles (Light, Sepia, SemiDark, Dark) -->
        <div class="theme-circles-row" role="group" aria-label="Brightness theme">
          {#each [
            { mode: 'Light' as ThemeMode, color: snapshot.theme_circle_colors.light, label: tr('theme_light') },
            { mode: 'Sepia' as ThemeMode, color: snapshot.theme_circle_colors.sepia, label: tr('theme_sepia') },
            { mode: 'SemiDark' as ThemeMode, color: snapshot.theme_circle_colors.semi_dark, label: tr('theme_semidark') },
            { mode: 'Dark' as ThemeMode, color: snapshot.theme_circle_colors.dark, label: tr('theme_dark') },
          ] as item (item.mode)}
            <button
              type="button"
              class="theme-circle-btn"
              class:is-selected={snapshot.theme === item.mode}
              style={`background-color: ${item.color};`}
              aria-label={item.label}
              title={item.label}
              data-testid={`theme-circle-${item.mode.toLowerCase()}`}
              onclick={() => send({ SetTheme: item.mode })}
            >
              {#if snapshot.theme === item.mode}
                <span class="theme-circle-check">✓</span>
              {/if}
            </button>
          {/each}
        </div>

        <div class="popover-divider"></div>

        <button
          id="menu-item-themes-palettes"
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={openThemesModal}
        >
          <span>{tr('themes_palettes')}</span>
        </button>

        <button
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => {
            send('ToggleFullscreen');
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('fullscreen_mode')}</span>
          <span class="shortcut-hint">F11</span>
        </button>

        <button
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => {
            send('ToggleSearchBar');
            send({ SetOverlay: 'None' });
          }}
        >
          <span>{tr('find_replace')}</span>
          <span class="shortcut-hint">Ctrl+F</span>
        </button>

        <button
          id="menu-item-preferences"
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={openPreferencesModal}
        >
          <span>{tr('preferences')}</span>
        </button>

        <button
          id="menu-item-export"
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => send({ SetOverlay: 'ExportDialog' })}
        >
          <span>{tr('advanced_export')}</span>
        </button>

        <button
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => send('OpenTutorial')}
        >
          <span>{tr('open_tutorial')}</span>
        </button>

        <button
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => send({ SetOverlay: 'ShortcutsDialog' })}
        >
          <span>{tr('keyboard_shortcuts')}</span>
        </button>

        <button
          type="button"
          class="popover-item"
          role="menuitem"
          onclick={() => send({ SetOverlay: 'AboutDialog' })}
        >
          <span>{tr('about')}</span>
        </button>

        <div class="popover-divider"></div>

        <div class="lang-row">
          <span class="lang-label">{tr('language')}</span>
          <div class="lang-pills">
            {#each ['It', 'En', 'Es'] as loc (loc)}
              <button
                type="button"
                class="lang-pill"
                class:is-active={snapshot.locale === loc}
                onclick={() => send({ SetLocale: loc as Locale })}
              >
                {loc.toUpperCase()}
              </button>
            {/each}
          </div>
        </div>
      </div>
    {:else if snapshot.overlay === 'StatsMenu'}
      <div class="popover stats-popover" role="menu" data-testid="popover-stats">
        {#each [
          { metric: 'Characters' as StatMetric, label: snapshot.stats.characters },
          { metric: 'Words' as StatMetric, label: snapshot.stats.words },
          { metric: 'Sentences' as StatMetric, label: snapshot.stats.sentences },
          { metric: 'Paragraphs' as StatMetric, label: snapshot.stats.paragraphs },
          { metric: 'ReadingTime' as StatMetric, label: snapshot.stats.reading_time },
        ] as row (row.metric)}
          <button
            type="button"
            class="popover-item"
            class:is-selected={snapshot.stat_metric === row.metric}
            onclick={() => {
              send({ SetStatMetric: row.metric });
              send({ SetOverlay: 'None' });
            }}
          >
            <span>{row.label}</span>
            {#if snapshot.stat_metric === row.metric}<span>✓</span>{/if}
          </button>
        {/each}
      </div>
    {/if}

    <!-- ====================================================================
         8. DEDICATED THEMES & PALETTES MODAL (Live Combined Preview + Annulla/Salva)
         ==================================================================== -->
    {#if themesModalOpen}
      <div
        class="overlay-backdrop is-modal"
        role="presentation"
        onclick={() => (themesModalOpen = false)}
      ></div>
      <div
        class="modal-card themes-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="themes-modal-title"
        data-testid="modal-themes-palettes"
      >
        <div class="modal-header">
          <h2 id="themes-modal-title">{tr('themes_palettes')}</h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label="Close"
            onclick={() => (themesModalOpen = false)}
          >
            ×
          </button>
        </div>

        <div class="modal-body scroller">
          <div class="form-row">
            <div>
              <div class="form-label">{tr('brightness_mode', 'Modalità luminosità')}</div>
              <div class="form-sub">{tr('brightness_sub', 'Chiaro · Sepia · Semi-Dark · Scuro')}</div>
            </div>
            <div class="pill-group">
              {#each [
                { mode: 'Light' as ThemeMode, label: tr('theme_light'), dot: snapshot.theme_circle_colors.light },
                { mode: 'Sepia' as ThemeMode, label: tr('theme_sepia'), dot: snapshot.theme_circle_colors.sepia },
                { mode: 'SemiDark' as ThemeMode, label: tr('theme_semidark'), dot: snapshot.theme_circle_colors.semi_dark },
                { mode: 'Dark' as ThemeMode, label: tr('theme_dark'), dot: snapshot.theme_circle_colors.dark },
              ] as m (m.mode)}
                <button
                  type="button"
                  class="choice-pill pill-with-swatch"
                  class:is-active={draftTheme === m.mode}
                  onclick={() => updateThemesDraft(m.mode, draftPreset)}
                >
                  <span class="swatch-dot" style={`background-color: ${m.dot};`}></span>
                  <span>{m.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <div class="form-row">
            <div>
              <div class="form-label">{tr('color_palette')}</div>
              <div class="form-sub">{tr('color_palette_sub')}</div>
            </div>
            <div class="pill-group">
              {#each [
                { preset: 'AmberCoral' as ColorPalettePreset, label: 'Amber & Coral' },
                { preset: 'EditorialSlate' as ColorPalettePreset, label: tr('palette_editorial') },
                { preset: 'NordicForest' as ColorPalettePreset, label: tr('palette_nordic') },
                { preset: 'VelvetDawn' as ColorPalettePreset, label: tr('palette_velvet') },
                { preset: 'Classic' as ColorPalettePreset, label: tr('palette_classic') },
              ] as p (p.preset)}
                {@const sw = PALETTE_SWATCHES[p.preset]}
                <button
                  type="button"
                  class="choice-pill pill-with-swatch"
                  class:is-active={draftPreset === p.preset}
                  onclick={() => updateThemesDraft(draftTheme, p.preset)}
                >
                  <span class="swatch-triad">
                    <span class="swatch-dot" style={`background-color: ${sw[0]};`}></span>
                    <span class="swatch-dot" style={`background-color: ${sw[1]};`}></span>
                    <span class="swatch-dot" style={`background-color: ${sw[2]};`}></span>
                  </span>
                  <span>{p.label}</span>
                </button>
              {/each}
            </div>
          </div>

          {#if draftPreviewTokens}
            <div
              class="live-theme-preview"
              style={`background-color: ${draftPreviewTokens.bg_canvas}; color: ${draftPreviewTokens.fg_primary}; border-color: ${draftPreviewTokens.border_headerbar};`}
            >
              <div
                class="live-preview-bar"
                style={`background-color: ${draftPreviewTokens.bg_headerbar}; border-bottom-color: ${draftPreviewTokens.border_headerbar};`}
              >
                <span style={`color: ${draftPreviewTokens.fg_muted};`}>{tr('live_preview')}</span>
                <span
                  class="live-preview-badge"
                  style={`background-color: ${draftPreviewTokens.accent_orange}; color: #fff;`}
                >
                  {draftPreset} · {draftTheme}
                </span>
              </div>
              <div class="live-preview-content">
                <h3 style={`color: ${draftPreviewTokens.fg_heading}; margin: 0 0 4px 0;`}>
                  # Titolo H1 ({draftTheme})
                </h3>
                <h4 style={`color: ${draftPreviewTokens.fg_subheading}; margin: 0 0 8px 0; font-size: 13.2px;`}>
                  ## Sezione H2–H6
                </h4>
                <p style="margin: 0 0 10px 0;">
                  Scrittura calma con <strong style={`color: ${draftPreviewTokens.markup_bold_fg};`}>grassetto</strong>,
                  <em style={`color: ${draftPreviewTokens.markup_emphasis_fg}; font-style: italic;`}>corsivo</em>,
                  <code
                    style={`background-color: ${draftPreviewTokens.bg_code_inline}; color: ${draftPreviewTokens.markup_code_fg}; padding: 2px 6px; border-radius: 4px;`}
                  >
                    codice inline
                  </code>
                  e <span style={`color: ${draftPreviewTokens.accent_link}; text-decoration: underline;`}>collegamento ipertestuale</span>.
                </p>
                <blockquote
                  style={`margin: 0; padding: 4px 0 4px 12px; border-left: 3px solid ${draftPreviewTokens.markup_quote_bar}; color: ${draftPreviewTokens.markup_quote_fg}; font-style: normal;`}
                >
                  «La semplicità visiva protegge l'attenzione dell'autore.»
                </blockquote>
              </div>
            </div>
          {/if}
        </div>

        <div class="modal-footer">
          <button
            id="themes-modal-cancel"
            type="button"
            class="footer-btn"
            onclick={() => (themesModalOpen = false)}
          >
            {tr('cancel')}
          </button>
          <button
            id="themes-modal-save"
            type="button"
            class="footer-btn footer-btn-primary"
            onclick={saveThemesModal}
          >
            {tr('save')}
          </button>
        </div>
      </div>
    {/if}

    <!-- ====================================================================
         9. 2-TAB PREFERENCES MODAL (Tipografia e Pagina / Editor e Lingua)
         ==================================================================== -->
    {#if snapshot.overlay === 'PreferencesDialog'}
      <div
        class="modal-card prefs-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="prefs-modal-title"
        data-testid="modal-preferences"
      >
        <div class="modal-header">
          <h2 id="prefs-modal-title">{tr('preferences')}</h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label="Close"
            onclick={cancelPreferencesModal}
          >
            ×
          </button>
        </div>

        <div class="prefs-tabs" role="tablist">
          <button
            id="prefs-tab-typography"
            type="button"
            role="tab"
            aria-selected={prefsTab === 'typography'}
            class="prefs-tab-btn"
            class:is-active={prefsTab === 'typography'}
            onclick={() => (prefsTab = 'typography')}
          >
            {tr('tab_typography')}
          </button>
          <button
            id="prefs-tab-editor"
            type="button"
            role="tab"
            aria-selected={prefsTab === 'editor'}
            class="prefs-tab-btn"
            class:is-active={prefsTab === 'editor'}
            onclick={() => (prefsTab = 'editor')}
          >
            {tr('tab_editor_lang')}
          </button>
        </div>

        <div class="modal-body scroller">
          {#if prefsTab === 'typography'}
            <div class="form-row">
              <div>
                <div class="form-label">{tr('prose_font')}</div>
                <div class="form-sub">
                  {tr('prose_font_sub')} ({snapshot.font_families.length})
                </div>
              </div>
              <select
                class="form-select"
                aria-label={tr('prose_font')}
                value={snapshot.typography.prose_font_family}
                onchange={(e) =>
                  send({ SetProseFont: (e.currentTarget as HTMLSelectElement).value })}
              >
                {#each snapshot.font_families as fam (fam)}
                  <option value={fam}>{fam}</option>
                {/each}
              </select>
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('code_font')}</div>
                <div class="form-sub">{tr('code_font_sub')}</div>
              </div>
              <select
                class="form-select"
                aria-label={tr('code_font')}
                value={snapshot.typography.code_font_family}
                onchange={(e) =>
                  send({ SetCodeFont: (e.currentTarget as HTMLSelectElement).value })}
              >
                {#each snapshot.font_families as fam (fam)}
                  <option value={fam}>{fam}</option>
                {/each}
              </select>
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('font_size')} & {tr('line_height')}</div>
                <div class="form-sub">{tr('size_lh_sub')}</div>
              </div>
              <div class="stepper-group">
                <button
                  type="button"
                  class="stepper-btn"
                  aria-label="Decrease font size"
                  onclick={() => send({ SetFontSize: snapshot!.typography.font_size - 0.5 })}
                >
                  −
                </button>
                <span class="stepper-val">{snapshot.typography.font_size.toFixed(1)} pt</span>
                <button
                  type="button"
                  class="stepper-btn"
                  aria-label="Increase font size"
                  onclick={() => send({ SetFontSize: snapshot!.typography.font_size + 0.5 })}
                >
                  +
                </button>

                <button
                  type="button"
                  class="stepper-btn"
                  aria-label="Decrease line height"
                  onclick={() => send({ SetLineHeight: snapshot!.typography.line_height - 0.05 })}
                >
                  −
                </button>
                <span class="stepper-val">{snapshot.typography.line_height.toFixed(2)}×</span>
                <button
                  type="button"
                  class="stepper-btn"
                  aria-label="Increase line height"
                  onclick={() => send({ SetLineHeight: snapshot!.typography.line_height + 0.05 })}
                >
                  +
                </button>
              </div>
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('line_width')}</div>
                <div class="form-sub">{tr('line_width_sub')}</div>
              </div>
              <div class="pill-group">
                <button
                  type="button"
                  class="choice-pill"
                  class:is-active={snapshot.line_width === 'Compact'}
                  onclick={() => send({ SetLineWidth: 'Compact' })}
                >
                  80ch
                </button>
                <button
                  type="button"
                  class="choice-pill"
                  class:is-active={snapshot.line_width === 'Medium'}
                  onclick={() => send({ SetLineWidth: 'Medium' })}
                >
                  110ch
                </button>
                <button
                  type="button"
                  class="choice-pill"
                  class:is-active={snapshot.line_width === 'Wide'}
                  onclick={() => send({ SetLineWidth: 'Wide' })}
                >
                  140ch
                </button>
                <button
                  type="button"
                  class="choice-pill"
                  class:is-active={typeof snapshot.line_width === 'object'}
                  onclick={() =>
                    send({
                      SetLineWidth: { Fluid: { margin_pct: snapshot?.fluid_margin_pct ?? 6 } },
                    })}
                >
                  Fluid
                </button>
              </div>
            </div>

            <div class="form-row">
              <div>
                <label class="form-label" for="fluid-margin-slider">
                  {tr('fluid_margin')} ({snapshot.fluid_margin_pct}%)
                </label>
                <div class="form-sub">{tr('fluid_margin_sub')}</div>
              </div>
              <input
                id="fluid-margin-slider"
                class="accent-range-slider"
                type="range"
                min="2"
                max="25"
                step="1"
                value={snapshot.fluid_margin_pct}
                aria-valuetext={`${snapshot.fluid_margin_pct}%`}
                oninput={(e) =>
                  send({
                    SetLineWidth: {
                      Fluid: { margin_pct: Number((e.currentTarget as HTMLInputElement).value) },
                    },
                  })}
              />
            </div>

            <div class="tab-reset-row">
              <button
                id="btn-restore-typography-defaults"
                type="button"
                class="reset-btn"
                onclick={restoreTypographyDefaults}
              >
                {tr('restore_defaults')}
              </button>
            </div>
          {:else}
            {@const isCommonMark = snapshot.markdown_config.base === 'CommonMark'}
            <div class="form-row">
              <div>
                <div class="form-label">{tr('input_format')}</div>
                <div class="form-sub">{tr('input_format_sub')}</div>
              </div>
              <div class="pill-group" role="group" aria-label={tr('input_format')}>
                <button
                  id="btn-base-gfm"
                  type="button"
                  class="choice-pill"
                  class:is-active={snapshot.markdown_config.base === 'Gfm'}
                  aria-pressed={snapshot.markdown_config.base === 'Gfm'}
                  data-testid="btn-base-gfm"
                  onclick={() => setMarkdownBase('Gfm')}
                >
                  {tr('base_gfm', 'GFM Standard')}
                </button>
                <button
                  id="btn-base-commonmark"
                  type="button"
                  class="choice-pill"
                  class:is-active={isCommonMark}
                  aria-pressed={isCommonMark}
                  data-testid="btn-base-commonmark"
                  onclick={() => setMarkdownBase('CommonMark')}
                >
                  {tr('base_commonmark', 'CommonMark')}
                </button>
              </div>
            </div>

            <div
              class="extensions-card"
              class:is-disabled={isCommonMark}
              data-testid="extensions-card"
            >
              <div class="extensions-card-hdr">
                <div class="form-label">{tr('extensions_title', 'Estensioni modulari')}</div>
                <div class="form-sub">
                  {isCommonMark
                    ? tr('extensions_disabled_cm', 'Disattivate in modalità CommonMark rigorosa')
                    : tr('extensions_sub', 'Attivabili singolarmente su GFM Standard')}
                </div>
              </div>

              {#each [
                {
                  key: 'math' as keyof MarkdownExtensions,
                  id: 'btn-toggle-ext-math',
                  label: tr('ext_math', 'Matematica KaTeX'),
                  sub: tr('ext_math_sub', 'Formule inline ($...$) e blocchi matematici ($$...$$)'),
                },
                {
                  key: 'mermaid' as keyof MarkdownExtensions,
                  id: 'btn-toggle-ext-mermaid',
                  label: tr('ext_mermaid', 'Diagrammi Mermaid'),
                  sub: tr('ext_mermaid_sub', 'Rendering SVG nativo dei blocchi di codice mermaid'),
                },
                {
                  key: 'alerts' as keyof MarkdownExtensions,
                  id: 'btn-toggle-ext-alerts',
                  label: tr('ext_alerts', 'Avvisi / Callout'),
                  sub: tr('ext_alerts_sub', 'Blocchi evidenziati [!NOTE], [!TIP], [!IMPORTANT], [!WARNING], [!CAUTION]'),
                },
                {
                  key: 'footnotes' as keyof MarkdownExtensions,
                  id: 'btn-toggle-ext-footnotes',
                  label: tr('ext_footnotes', 'Note a piè di pagina'),
                  sub: tr('ext_footnotes_sub', 'Riferimenti nel testo [^1] e definizioni [^1]: ...'),
                },
                {
                  key: 'frontmatter' as keyof MarkdownExtensions,
                  id: 'btn-toggle-ext-frontmatter',
                  label: tr('ext_frontmatter', 'Frontmatter YAML'),
                  sub: tr('ext_frontmatter_sub', 'Intestazione metadati iniziale delimitata da ---'),
                },
              ] as ext (ext.key)}
                {@const isExtActive = !isCommonMark && snapshot.markdown_config.extensions[ext.key]}
                <div class="ext-toggle-row">
                  <div class="ext-toggle-meta">
                    <div class="ext-toggle-label">{ext.label}</div>
                    <div class="form-sub">{ext.sub}</div>
                  </div>
                  <button
                    id={ext.id}
                    type="button"
                    role="switch"
                    aria-checked={isExtActive}
                    aria-label={ext.label}
                    disabled={isCommonMark}
                    class="choice-pill ext-toggle-pill"
                    class:is-active={isExtActive}
                    data-testid={ext.id}
                    onclick={() => toggleMarkdownExtension(ext.key)}
                  >
                    {isExtActive ? tr('enabled', 'Attivo') : tr('disabled', 'Disattivato')}
                  </button>
                </div>
              {/each}
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('wal_title', 'Journal di ripristino WAL (< 300ms)')}</div>
                <div class="form-sub">
                  {tr('wal_sub')}
                </div>
              </div>
              <button
                id="btn-toggle-wal-autosave"
                type="button"
                class="choice-pill"
                class:is-active={snapshot.wal_autosave_enabled}
                aria-pressed={snapshot.wal_autosave_enabled}
                onclick={() => send({ SetWalAutosave: !snapshot!.wal_autosave_enabled })}
              >
                {snapshot.wal_autosave_enabled ? tr('enabled', 'Attivo') : tr('disabled', 'Disattivato')}
              </button>
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('watcher_title', 'Monitoraggio modifiche esterne sul disco')}</div>
                <div class="form-sub">
                  {tr('watcher_sub')}
                </div>
              </div>
              <button
                id="btn-toggle-disk-watcher"
                type="button"
                class="choice-pill"
                class:is-active={snapshot.disk_watcher_enabled}
                aria-pressed={snapshot.disk_watcher_enabled}
                onclick={() => send({ SetDiskWatcher: !snapshot!.disk_watcher_enabled })}
              >
                {snapshot.disk_watcher_enabled ? tr('enabled', 'Attivo') : tr('disabled', 'Disattivato')}
              </button>
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('language')}</div>
              </div>
              <div class="pill-group">
                {#each [
                  { code: 'It' as Locale, label: 'Italiano' },
                  { code: 'En' as Locale, label: 'English' },
                  { code: 'Es' as Locale, label: 'Español' },
                ] as l (l.code)}
                  <button
                    type="button"
                    class="choice-pill"
                    class:is-active={snapshot.locale === l.code}
                    onclick={() => send({ SetLocale: l.code })}
                  >
                    {l.label}
                  </button>
                {/each}
              </div>
            </div>

            <div class="tab-reset-row">
              <button
                id="btn-restore-editor-defaults"
                type="button"
                class="reset-btn"
                onclick={restoreEditorLangDefaults}
              >
                {tr('restore_defaults')}
              </button>
            </div>
          {/if}
        </div>

        <div class="modal-footer">
          <button
            id="prefs-modal-cancel"
            type="button"
            class="footer-btn"
            onclick={cancelPreferencesModal}
          >
            {tr('cancel')}
          </button>
          <button
            id="prefs-modal-save"
            type="button"
            class="footer-btn footer-btn-primary"
            onclick={savePreferencesModal}
          >
            {tr('save')}
          </button>
        </div>
      </div>
    {:else if snapshot.overlay === 'ExportDialog'}
      <div
        class="modal-card export-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="export-modal-title"
        data-testid="modal-export"
      >
        <div class="modal-header">
          <h2 id="export-modal-title">{tr('advanced_export')}</h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label="Close"
            onclick={() => send({ SetOverlay: 'None' })}
          >
            ×
          </button>
        </div>
        <div class="export-split-body">
          <aside class="export-sidebar">
            <div class="export-sidebar-hdr">{tr('export_formats_hdr')}</div>
            {#each [
              { fmt: 'Pdf' as ExportFormat, label: 'PDF (.pdf)' },
              { fmt: 'Docx' as ExportFormat, label: 'Word (.docx)' },
              { fmt: 'Odt' as ExportFormat, label: 'OpenDocument (.odt)' },
              { fmt: 'Latex' as ExportFormat, label: 'LaTeX (.tex)' },
              { fmt: 'Epub' as ExportFormat, label: 'EPUB (.epub)' },
              { fmt: 'Html' as ExportFormat, label: 'HTML5 (.html)' },
            ] as item (item.fmt)}
              <button
                type="button"
                class="export-fmt-btn"
                class:is-active={snapshot.selected_export_format === item.fmt}
                onclick={() => send({ SelectExportFormat: item.fmt })}
              >
                {item.label}
              </button>
            {/each}
          </aside>
          <div class="export-options-pane">
            <div class="form-col">
              <label class="form-label" for="export-dest-input">{tr('export_dest_path')}</label>
              <div class="export-dest-row">
                <input
                  id="export-dest-input"
                  type="text"
                  class="search-input"
                  value={snapshot.export_output_path}
                  oninput={(e) =>
                    send({ SetExportOutputPath: (e.currentTarget as HTMLInputElement).value })}
                />
                <button
                  id="btn-export-browse"
                  type="button"
                  class="choice-pill"
                  onclick={() => handleExportWithDialog(null, false)}
                >
                  {tr('browse', 'Sfoglia...')}
                </button>
              </div>
            </div>

            <div class="form-row">
              <span class="form-label">{tr('export_page_size')}</span>
              <span class="choice-pill is-active">A4</span>
            </div>

            <div class="form-row">
              <div>
                <div class="form-label">{tr('export_syntax_hl')}</div>
                <div class="form-sub">{tr('export_syntax_hl_sub')}</div>
              </div>
              <button
                type="button"
                class="choice-pill"
                class:is-active={snapshot.export_syntax_highlight}
                aria-pressed={snapshot.export_syntax_highlight}
                onclick={() => send('ToggleExportSyntaxHighlight')}
              >
                {snapshot.export_syntax_highlight ? 'ON' : 'OFF'}
              </button>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button
            type="button"
            class="footer-btn"
            onclick={() => send({ SetOverlay: 'None' })}
          >
            {tr('cancel')}
          </button>
          <button
            id="btn-execute-export"
            type="button"
            class="footer-btn footer-btn-primary"
            onclick={() => send('ExecuteExport')}
          >
            {tr('export_btn')}
          </button>
        </div>
      </div>
    {:else if snapshot.overlay === 'ShortcutsDialog' || snapshot.overlay === 'AboutDialog'}
      <div class="modal-card info-modal" role="dialog" aria-modal="true">
        <div class="modal-header">
          <h2>
            {snapshot.overlay === 'ShortcutsDialog' ? tr('keyboard_shortcuts') : tr('about')}
          </h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label="Close"
            onclick={() => send({ SetOverlay: 'None' })}
          >
            ×
          </button>
        </div>
        <div class="modal-body">
          {#if snapshot.overlay === 'ShortcutsDialog'}
            <div class="shortcuts-grid">
              <div><span>{tr('sc_new_tab')}</span><b>Ctrl+T</b></div>
              <div><span>{tr('sc_close_tab')}</span><b>Ctrl+W</b></div>
              <div><span>{tr('sc_cycle_tabs')}</span><b>Ctrl+Tab / Ctrl+Shift+Tab</b></div>
              <div><span>{tr('new_doc')}</span><b>Ctrl+N</b></div>
              <div><span>{tr('open_file')}</span><b>Ctrl+O</b></div>
              <div><span>{tr('save')} / {tr('save_as')}</span><b>Ctrl+S / Ctrl+Shift+S</b></div>
              <div><span>{tr('sc_undo_redo')}</span><b>Ctrl+Z / Ctrl+Shift+Z</b></div>
              <div><span>{tr('sc_inline_fmt')}</span><b>Ctrl+B / Ctrl+I / Ctrl+Shift+X</b></div>
              <div><span>{tr('sc_insert_link')}</span><b>Ctrl+K</b></div>
              <div><span>{tr('sc_live_preview')}</span><b>Ctrl+L</b></div>
              <div><span>{tr('sc_split_preview')}</span><b>Ctrl+Shift+P</b></div>
              <div><span>{tr('sc_zoom')}</span><b>Ctrl++ / Ctrl+- / Ctrl+0</b></div>
              <div><span>{tr('find_replace')}</span><b>Ctrl+F / Ctrl+H</b></div>
              <div><span>{tr('fullscreen_mode')}</span><b>F11</b></div>
            </div>
          {:else}
            <p style="margin: 0 0 8px 0; font-weight: 700;">Accent Markdown Editor v0.8.24</p>
            <p style="margin: 0; color: var(--fg-muted);">
              Rust Backend (L1–L4) • Tauri v2 • Svelte 5 • CodeMirror 6 • Offline KaTeX &amp; Mermaid
            </p>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ====================================================================
         10. UNSAVED TAB CLOSE CONFIRMATION MODAL
         ==================================================================== -->
    {#if pendingCloseTabIndex !== null}
      {@const closingTab = snapshot.tabs[pendingCloseTabIndex]}
      <div
        class="overlay-backdrop is-modal"
        role="presentation"
        onclick={() => (pendingCloseTabIndex = null)}
      ></div>
      <div
        class="modal-card info-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="unsaved-tab-modal-title"
        data-testid="modal-unsaved-tab"
      >
        <div class="modal-header">
          <h2 id="unsaved-tab-modal-title">
            {tr('unsaved_tab_title')} — {closingTab?.document_title ?? ''}
          </h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label={tr('cancel')}
            onclick={() => (pendingCloseTabIndex = null)}
          >
            ×
          </button>
        </div>
        <div class="modal-body">
          <p style="margin: 0; line-height: 1.5;">
            {tr('unsaved_tab_msg')}
          </p>
        </div>
        <div class="modal-footer">
          <!-- svelte-ignore a11y_autofocus -->
          <button
            id="unsaved-tab-cancel"
            type="button"
            class="footer-btn"
            autofocus
            onclick={() => (pendingCloseTabIndex = null)}
          >
            {tr('cancel')}
          </button>
          <button
            id="unsaved-tab-discard"
            type="button"
            class="footer-btn footer-btn-danger"
            onclick={confirmDiscardAndCloseTab}
          >
            {tr('discard_changes')}
          </button>
          <button
            id="unsaved-tab-save"
            type="button"
            class="footer-btn footer-btn-primary"
            onclick={confirmSaveAndCloseTab}
          >
            {tr('save')}
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .window-shell {
    position: relative;
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-canvas);
    color: var(--fg-primary);
    overflow: clip;
  }

  .window-shell.is-maximized {
    border: none;
  }

  /* Invisible frameless resize zones */
  .resize-edge {
    position: absolute;
    z-index: 90;
  }
  .resize-n { top: 0; left: 10px; right: 10px; height: 4px; cursor: ns-resize; }
  .resize-s { bottom: 0; left: 10px; right: 10px; height: 4px; cursor: ns-resize; }
  .resize-w { left: 0; top: 10px; bottom: 10px; width: 4px; cursor: ew-resize; }
  .resize-e { right: 0; top: 10px; bottom: 10px; width: 4px; cursor: ew-resize; }
  .resize-corner {
    position: absolute;
    width: 10px;
    height: 10px;
    z-index: 91;
  }
  .resize-nw { top: 0; left: 0; cursor: nwse-resize; }
  .resize-ne { top: 0; right: 0; cursor: nesw-resize; }
  .resize-sw { bottom: 0; left: 0; cursor: nesw-resize; }
  .resize-se { bottom: 0; right: 0; cursor: nwse-resize; }

  /* 1. Integrated CSD Top Bar */
  .csd-topbar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 38px;
    min-height: 38px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 10px;
    background-color: var(--bg-headerbar);
    border-bottom: 1px solid color-mix(in srgb, var(--border-headerbar) 65%, transparent);
    z-index: 30;
  }

  .csd-left,
  .csd-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .csd-center {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-width: 80px;
    overflow: hidden;
  }

  .csd-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--fg-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dirty-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background-color: var(--accent-orange);
    flex-shrink: 0;
  }

  .csd-icon-btn,
  .csd-pill-btn,
  .csd-badge-btn {
    min-inline-size: 28px;
    min-block-size: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    border-radius: 6px;
    padding: 0 7px;
    font-size: 12.8px;
    font-weight: 500;
    color: var(--fg-primary);
  }

  .csd-badge-btn {
    font-size: 11.8px;
    font-weight: 700;
    background-color: color-mix(in srgb, var(--bg-pill-button) 65%, transparent);
  }

  @media (hover: hover) {
    .csd-icon-btn:hover,
    .csd-pill-btn:hover,
    .csd-badge-btn:hover,
    .csd-split-main:hover,
    .csd-split-arrow:hover {
      background-color: var(--bg-pill-button);
    }
  }

  .csd-icon-btn.is-active,
  .csd-pill-btn.is-active,
  .csd-badge-btn.is-active,
  .csd-split-arrow.is-active {
    background-color: var(--bg-pill-button);
    color: var(--accent-orange);
  }

  .csd-split-btn {
    display: inline-flex;
    align-items: center;
    border-radius: 6px;
    background-color: color-mix(in srgb, var(--bg-pill-button) 55%, transparent);
    overflow: clip;
  }

  .csd-split-main {
    min-block-size: 28px;
    padding: 0 10px;
    font-size: 12.8px;
    font-weight: 600;
  }

  .csd-split-arrow {
    min-inline-size: 24px;
    min-block-size: 28px;
    border-left: 1px solid color-mix(in srgb, var(--border-headerbar) 80%, transparent);
    font-size: 11px;
  }

  .csd-win-sep {
    width: 1px;
    height: 16px;
    border-left: 1px solid var(--border-headerbar);
    margin: 0 2px;
  }

  .csd-window-controls {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  .csd-win-btn {
    min-inline-size: 28px;
    min-block-size: 26px;
    border-radius: 6px;
    font-size: 13.5px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  @media (hover: hover) {
    .csd-win-btn:hover {
      background-color: var(--bg-pill-button);
    }
    .csd-win-close:hover {
      background-color: #d94646;
      color: #ffffff;
    }
  }

  /* 2. Always-Visible Multi-Tab Bar */
  .multitab-bar {
    position: absolute;
    top: 38px;
    left: 0;
    right: 0;
    height: 32px;
    min-height: 32px;
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    padding: 3px 8px 0 10px;
    background-color: color-mix(in srgb, var(--bg-headerbar) 60%, var(--bg-code-block) 40%);
    border-bottom: 1px solid var(--border-headerbar);
    z-index: 29;
  }

  .tab-strip {
    flex: 1;
    display: flex;
    align-items: stretch;
    gap: 4px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .tab-strip::-webkit-scrollbar {
    display: none;
  }

  .tab-card {
    position: relative;
    flex: 1 1 155px;
    min-width: 68px;
    max-width: 204px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px 0 10px;
    border-radius: 7px 7px 0 0;
    font-size: 12.2px;
    color: var(--fg-muted);
    cursor: pointer;
    overflow: clip;
  }

  @media (hover: hover) {
    .tab-card:not(.is-active):hover {
      background-color: color-mix(in srgb, var(--bg-canvas) 55%, transparent);
      color: var(--fg-primary);
    }
  }

  .tab-card.is-active {
    background-color: var(--bg-canvas);
    color: var(--fg-primary);
    font-weight: 600;
    border-left: 1px solid var(--border-headerbar);
    border-right: 1px solid var(--border-headerbar);
    margin-bottom: -1px;
  }

  .tab-accent-top {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background-color: var(--accent-orange);
    border-radius: 6px 6px 0 0;
  }

  .tab-dirty-dot {
    width: 6.5px;
    height: 6.5px;
    border-radius: 50%;
    background-color: var(--fg-muted);
    flex-shrink: 0;
  }

  .tab-dirty-dot.is-active {
    background-color: var(--accent-orange);
  }

  .tab-title {
    flex: 1;
    min-inline-size: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-close-btn {
    min-inline-size: 18px;
    min-block-size: 18px;
    border-radius: 50%;
    font-size: 13px;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-muted);
    flex-shrink: 0;
  }

  @media (hover: hover) {
    .tab-close-btn:hover {
      background-color: var(--bg-pill-button);
      color: var(--fg-primary);
    }
  }

  .tab-bar-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    padding-left: 6px;
    padding-bottom: 2px;
  }

  .tab-ctrl-btn {
    min-inline-size: 24px;
    min-block-size: 24px;
    border-radius: 5px;
    font-size: 15px;
    font-weight: 700;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-primary);
  }

  .tab-ctrl-btn[aria-disabled='true'] {
    opacity: 0.35;
    cursor: default;
  }

  @media (hover: hover) {
    .tab-ctrl-btn:not([aria-disabled='true']):hover {
      background-color: var(--bg-pill-button);
    }
  }

  /* 3. Notification Banners */
  .notification-banner {
    position: absolute;
    top: 70px;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 20px;
    background-color: color-mix(in srgb, var(--accent-orange) 16%, var(--bg-headerbar));
    border-bottom: 1px solid var(--accent-orange);
    font-size: 13px;
    font-weight: 600;
    z-index: 28;
  }

  .window-shell.has-banner :global(.cm-scroller) {
    padding-top: 114px !important;
  }

  .window-shell.has-banner .preview-pane {
    padding-top: 114px;
  }

  .banner-actions {
    display: flex;
    gap: 8px;
  }

  .banner-btn {
    padding: 5px 12px;
    border-radius: 6px;
    background-color: var(--bg-pill-button);
    font-size: 12.5px;
    font-weight: 600;
  }

  .banner-btn-confirm {
    background-color: #2e8b57;
    color: #ffffff;
  }

  /* 4. Workspace Body (Outline Drawer + Editor + Preview) */
  .workspace-body {
    flex: 1;
    display: flex;
    min-height: 0;
    position: relative;
  }

  .outline-drawer {
    width: 248px;
    min-width: 248px;
    background-color: color-mix(in srgb, var(--bg-headerbar) 65%, var(--bg-canvas));
    border-right: 1px solid var(--border-headerbar);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    overscroll-behavior: contain;
    padding: 78px 8px 72px 8px;
    z-index: 20;
  }

  .outline-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 8px 10px 8px;
    font-size: 12.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--fg-muted);
  }

  .outline-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .outline-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px;
    border-radius: 6px;
    border-left: 2.5px solid transparent;
    text-align: left;
    font-size: 12.8px;
  }

  .outline-item.is-active {
    background-color: color-mix(in srgb, var(--accent-orange) 14%, transparent);
    border-left-color: var(--accent-orange);
    font-weight: 700;
    color: var(--fg-primary);
  }

  @media (hover: hover) {
    .outline-item:hover {
      background-color: var(--bg-pill-button);
    }
  }

  .outline-lvl {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--accent-orange);
    flex-shrink: 0;
  }

  .outline-txt {
    min-inline-size: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .outline-empty {
    padding: 12px 8px;
    font-size: 12.5px;
    color: var(--fg-muted);
  }

  .editor-workspace {
    flex: 1;
    display: flex;
    min-width: 0;
    min-height: 0;
  }

  .editor-workspace.split-right {
    flex-direction: row;
  }

  .editor-workspace.split-bottom {
    flex-direction: column;
  }

  .editor-pane {
    flex: 1;
    min-width: 0;
    min-height: 0;
    height: 100%;
  }

  .preview-pane {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    background-color: var(--bg-preview);
    border-left: 1px solid var(--divider-preview);
    padding: 76px 32px 96px 32px;
    transition: padding-top 0.18s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .window-shell.is-chrome-hidden .preview-pane {
    padding-top: 36px;
  }

  .editor-workspace.split-bottom .preview-pane {
    border-left: none;
    border-top: 1px solid var(--divider-preview);
    padding-top: 24px;
  }

  .preview-prose {
    position: relative;
    isolation: isolate;
    contain: layout style paint;
    overflow-wrap: break-word;
    margin: 0 auto;
    font-family: var(--prose-font-stack);
  }

  :global(.preview-prose > :first-child) {
    margin-top: 0 !important;
    padding-top: 0 !important;
  }

  :global(.preview-prose strong) {
    color: var(--markup-bold-fg);
    font-weight: 700;
  }

  :global(.preview-prose em) {
    color: var(--markup-emphasis-fg);
    font-style: italic;
  }

  :global(.preview-prose h1) {
    color: var(--fg-heading);
  }

  :global(.preview-prose h2),
  :global(.preview-prose h3),
  :global(.preview-prose h4),
  :global(.preview-prose h5),
  :global(.preview-prose h6) {
    color: var(--fg-subheading);
  }

  :global(.preview-prose a) {
    color: var(--accent-link);
  }

  :global(.preview-prose code) {
    font-family: var(--code-font-stack);
    background-color: var(--bg-code-inline);
    color: var(--markup-code-fg);
    padding: 2px 5px;
    border-radius: 4px;
  }

  :global(.preview-prose pre) {
    position: relative;
    font-family: var(--code-font-stack);
    font-size: 0.88em;
    line-height: 1.38;
    background-color: var(--bg-code-block);
    border: 1px solid color-mix(in srgb, var(--border-headerbar) 75%, transparent);
    padding: 14px 16px;
    border-radius: 7px;
    overflow-x: auto;
  }

  :global(.preview-prose pre[data-code-lang]::after) {
    content: attr(data-code-lang);
    position: absolute;
    top: 6px;
    right: 12px;
    padding: 1px 8px;
    border-radius: 4px;
    font-family: var(--code-font-stack);
    font-size: 0.82em;
    font-weight: 600;
    line-height: 1.32;
    letter-spacing: 0.03em;
    color: var(--fg-muted);
    background-color: color-mix(in srgb, var(--bg-canvas) 62%, var(--bg-code-block));
    border: 1px solid color-mix(in srgb, var(--border-headerbar) 85%, transparent);
    pointer-events: none;
    user-select: none;
  }

  :global(.preview-prose pre code) {
    background: none;
    color: var(--markup-codeblock-fg);
    padding: 0;
  }

  :global(.preview-prose blockquote) {
    margin: 12px 0;
    padding: 4px 0 4px 16px;
    border-left: 3px solid var(--markup-quote-bar);
    color: var(--markup-quote-fg);
    font-style: normal;
  }

  :global(.preview-prose table) {
    width: 100%;
    border-collapse: collapse;
    margin: 14px 0;
  }

  :global(.preview-prose th),
  :global(.preview-prose td) {
    border: 1px solid var(--border-headerbar);
    padding: 8px 12px;
  }

  :global(.preview-prose :where(th, td):not([align])) {
    text-align: left;
  }

  :global(.preview-prose [align='center']) {
    text-align: center;
  }

  :global(.preview-prose [align='right']) {
    text-align: right;
  }

  :global(.preview-prose th) {
    background-color: var(--bg-table-header);
    font-weight: 700;
  }

  :global(.preview-prose hr) {
    border: 0;
    border-top: 1px solid var(--border-headerbar);
    margin: 18px 0;
  }

  :global(.preview-prose kbd) {
    font-family: var(--code-font-stack);
    font-size: 0.86em;
    padding: 1px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-headerbar);
    background-color: var(--bg-code-inline);
    color: var(--fg-primary);
  }

  :global(.preview-prose details) {
    margin: 12px 0;
    padding: 8px 12px;
    border-radius: 7px;
    border: 1px solid var(--border-headerbar);
    background-color: color-mix(in srgb, var(--bg-code-block) 35%, transparent);
  }

  :global(.preview-prose summary) {
    font-weight: 600;
    cursor: pointer;
  }

  :global(.preview-prose .markdown-alert) {
    margin: 14px 0;
    padding: 10px 14px;
    border-left: 4px solid var(--accent-link);
    border-radius: 7px;
    background-color: color-mix(in srgb, var(--bg-code-block) 45%, transparent);
  }

  :global(.preview-prose .markdown-alert-title) {
    margin: 0 0 6px 0;
    font-weight: 700;
    color: var(--accent-link);
  }

  :global(.preview-prose .markdown-alert > :last-child) {
    margin-bottom: 0;
  }

  :global(.preview-prose .markdown-alert-tip) {
    border-left-color: var(--markup-task-done);
  }
  :global(.preview-prose .markdown-alert-tip .markdown-alert-title) {
    color: var(--markup-task-done);
  }

  :global(.preview-prose .markdown-alert-important) {
    border-left-color: var(--syntax-keyword);
  }
  :global(.preview-prose .markdown-alert-important .markdown-alert-title) {
    color: var(--syntax-keyword);
  }

  :global(.preview-prose .markdown-alert-warning) {
    border-left-color: var(--accent-orange);
  }
  :global(.preview-prose .markdown-alert-warning .markdown-alert-title) {
    color: var(--accent-orange);
  }

  :global(.preview-prose .markdown-alert-caution) {
    border-left-color: var(--spellcheck-red);
  }
  :global(.preview-prose .markdown-alert-caution .markdown-alert-title) {
    color: var(--spellcheck-red);
  }

  :global(.preview-prose section.footnotes) {
    margin-top: 24px;
    padding-top: 12px;
    border-top: 1px solid var(--border-headerbar);
    font-size: 0.9em;
    color: var(--fg-muted);
  }

  :global(.accent-mermaid-block) {
    display: flex;
    justify-content: center;
    margin: 16px 0;
    padding: 12px;
    border-radius: 10px;
    background-color: color-mix(in srgb, var(--bg-code-block) 45%, transparent);
    overflow-x: auto;
  }

  :global(.accent-mermaid-block svg) {
    display: block;
    max-width: 100%;
    height: auto;
  }

  :global(.katex-display-block) {
    margin: 14px 0;
    padding: 10px 14px;
    border-radius: 8px;
    background-color: color-mix(in srgb, var(--bg-code-block) 45%, transparent);
    overflow-x: auto;
  }

  /* 5. Floating Find & Replace Panel */
  .search-panel {
    position: absolute;
    top: 76px;
    right: 16px;
    width: 396px;
    padding: 10px;
    border-radius: 12px;
    background-color: var(--bg-popover);
    border: 1px solid var(--border-headerbar);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.16);
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 45;
  }

  .search-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .search-input-wrap {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input {
    width: 100%;
    height: 30px;
    padding: 0 10px;
    border-radius: 6px;
    border: 1px solid var(--border-headerbar);
    background-color: var(--bg-canvas);
    color: var(--fg-primary);
    font-size: 13px;
  }

  .search-query-input {
    padding-right: 58px;
  }

  .export-dest-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .export-dest-row .search-input {
    flex: 1;
    min-width: 0;
  }

  .search-counter {
    position: absolute;
    right: 8px;
    font-size: 11px;
    font-weight: 700;
    color: var(--accent-orange);
    pointer-events: none;
  }

  .search-mini-btn,
  .search-action-btn {
    min-inline-size: 28px;
    min-block-size: 28px;
    padding: 0 8px;
    border-radius: 6px;
    background-color: var(--bg-pill-button);
    font-size: 12.5px;
    font-weight: 600;
  }

  /* 6. Dual Floating Bottom Islands */
  .bottom-islands-layer {
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: 14px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    pointer-events: none;
    z-index: 30;
  }

  .floating-island {
    pointer-events: auto;
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 4px 8px;
    border-radius: 999px;
    background-color: color-mix(in srgb, var(--bg-popover) 92%, transparent);
    border: 1px solid var(--border-headerbar);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.12);
    backdrop-filter: blur(8px);
  }

  .island-left.is-collapsed {
    width: 34px;
    height: 34px;
    padding: 0;
    justify-content: center;
    border-radius: 50%;
  }

  .fmt-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .fmt-btn {
    min-inline-size: 28px;
    min-block-size: 26px;
    padding: 0 6px;
    border-radius: 999px;
    font-size: 12.5px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .fmt-btn svg {
    display: block;
  }

  @media (hover: hover) {
    .fmt-btn:hover,
    .stats-pill-btn:hover {
      background-color: var(--bg-pill-button);
    }
  }

  .fmt-sep {
    width: 1px;
    height: 15px;
    border-left: 1px solid var(--border-headerbar);
    margin: 0 3px;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    font-size: 12px;
    color: var(--fg-muted);
  }

  .status-dot {
    width: 6.5px;
    height: 6.5px;
    border-radius: 50%;
    background-color: var(--markup-task-done);
  }

  .status-dot.is-dirty {
    background-color: var(--accent-orange);
  }

  .stats-pill-btn {
    min-block-size: 26px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12.4px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .island-right {
    margin-inline-start: auto;
  }

  /* 7. Popovers & Modals */
  .overlay-backdrop {
    position: absolute;
    inset: 0;
    z-index: 25;
  }

  .overlay-backdrop.is-modal {
    z-index: 55;
    overflow: hidden;
    overscroll-behavior: contain;
    background-color: rgba(0, 0, 0, 0.36);
  }

  .popover {
    position: absolute;
    padding: 8px;
    border-radius: 12px;
    background-color: var(--bg-popover);
    border: 1px solid var(--border-headerbar);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.18);
    display: flex;
    flex-direction: column;
    gap: 2px;
    z-index: 50;
  }

  .open-popover { top: 42px; left: 72px; width: 310px; }
  .save-popover { top: 42px; left: 130px; width: 250px; }
  .linewidth-popover { top: 42px; right: 175px; width: 220px; }
  .preview-popover { top: 42px; right: 110px; width: 245px; }
  .main-menu-popover { top: 42px; right: 38px; width: 286px; }
  .stats-popover { bottom: 54px; right: 18px; width: 230px; }

  .popover-section-title {
    padding: 4px 10px;
    font-size: 11.5px;
    font-weight: 700;
    color: var(--fg-muted);
    text-transform: uppercase;
  }

  .popover-item {
    width: 100%;
    min-block-size: 30px;
    padding: 5px 10px;
    border-radius: 7px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    text-align: left;
    font-size: 13.4px;
  }

  .open-popover .popover-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
  }

  .popover-item-main { font-weight: 600; }
  .popover-item-sub {
    font-size: 11.2px;
    color: var(--fg-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  @media (hover: hover) {
    .popover-item:hover {
      background-color: var(--bg-pill-button);
    }
  }

  .popover-item.is-selected,
  .popover-item-accent {
    color: var(--accent-orange);
    font-weight: 600;
  }

  .popover-divider {
    height: 1px;
    background-color: var(--border-headerbar);
    margin: 4px 6px;
  }

  .shortcut-hint {
    font-size: 12px;
    color: var(--fg-muted);
  }

  .theme-circles-row {
    display: flex;
    align-items: center;
    justify-content: space-around;
    padding: 8px 6px;
  }

  .theme-circle-btn {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 1.5px solid var(--border-headerbar);
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .theme-circle-btn.is-selected {
    border: 2.5px solid var(--accent-orange);
  }

  .theme-circle-check {
    position: absolute;
    bottom: -2px;
    right: -2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background-color: var(--accent-orange);
    color: #fff;
    font-size: 10px;
    font-weight: 700;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .lang-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 10px;
  }

  .lang-label {
    font-size: 12.8px;
    color: var(--fg-muted);
  }

  .lang-pills {
    display: flex;
    gap: 4px;
  }

  .lang-pill {
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 11.8px;
    font-weight: 700;
    background-color: var(--bg-pill-button);
  }

  .lang-pill.is-active {
    background-color: var(--accent-orange);
    color: #ffffff;
  }

  /* Modal Dialogs */
  .modal-card {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 720px;
    max-width: 92vw;
    max-height: 86vh;
    border-radius: 14px;
    background-color: var(--bg-popover);
    border: 1px solid var(--border-headerbar);
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.28);
    display: flex;
    flex-direction: column;
    overflow: clip;
    z-index: 60;
  }

  .info-modal {
    width: 500px;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border-headerbar);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 15.5px;
    font-weight: 700;
  }

  .modal-close-btn {
    min-inline-size: 28px;
    min-block-size: 28px;
    border-radius: 50%;
    background-color: var(--bg-pill-button);
    font-size: 15px;
    font-weight: 700;
  }

  .prefs-tabs {
    display: flex;
    gap: 8px;
    padding: 10px 20px 0 20px;
    border-bottom: 1px solid var(--border-headerbar);
  }

  .prefs-tab-btn {
    padding: 8px 14px;
    border-bottom: 2.5px solid transparent;
    font-size: 13.5px;
    font-weight: 600;
    color: var(--fg-muted);
  }

  .prefs-tab-btn.is-active {
    color: var(--accent-orange);
    border-bottom-color: var(--accent-orange);
  }

  .modal-body {
    padding: 18px 22px;
    overflow-y: auto;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .form-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-label {
    font-size: 13.8px;
    font-weight: 600;
  }

  .form-sub {
    font-size: 11.8px;
    color: var(--fg-muted);
    margin-top: 2px;
  }

  .form-select {
    height: 32px;
    min-width: 220px;
    padding: 0 10px;
    border-radius: 7px;
    border: 1px solid var(--border-headerbar);
    background-color: var(--bg-canvas);
    color: var(--fg-primary);
    font-size: 13px;
  }

  .pill-group {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .choice-pill {
    padding: 6px 12px;
    border-radius: 7px;
    background-color: var(--bg-pill-button);
    font-size: 12.5px;
    font-weight: 600;
  }

  @media (hover: hover) {
    .choice-pill:hover:not(:disabled):not(.is-active) {
      background-color: color-mix(in srgb, var(--accent-orange) 14%, var(--bg-pill-button));
    }
  }

  .choice-pill:disabled {
    cursor: not-allowed;
    opacity: 0.65;
  }

  .extensions-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 10px;
    border: 1px solid var(--border-headerbar);
    background-color: color-mix(in srgb, var(--bg-canvas) 62%, var(--bg-popover));
    transition: opacity 0.15s ease;
  }

  .extensions-card.is-disabled {
    opacity: 0.48;
  }

  .extensions-card-hdr {
    padding-bottom: 4px;
    border-bottom: 1px solid color-mix(in srgb, var(--border-headerbar) 70%, transparent);
  }

  .ext-toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
  }

  .ext-toggle-meta {
    min-width: 0;
  }

  .ext-toggle-label {
    font-size: 13.2px;
    font-weight: 600;
  }

  .ext-toggle-pill {
    min-inline-size: 92px;
    text-align: center;
    flex-shrink: 0;
  }

  .pill-with-swatch {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .swatch-triad {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .swatch-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.22);
    display: inline-block;
  }

  .choice-pill.is-active {
    background-color: var(--accent-orange);
    color: #ffffff;
  }

  .stepper-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .stepper-btn {
    min-inline-size: 28px;
    min-block-size: 28px;
    border-radius: 6px;
    background-color: var(--bg-pill-button);
    font-weight: 700;
  }

  .stepper-val {
    min-width: 64px;
    text-align: center;
    font-size: 12.8px;
    font-weight: 700;
  }

  /* Brand-consistent range slider (separate WebKit and Firefox pseudo-rules) */
  .accent-range-slider {
    --slider-track-size: 6px;
    --slider-thumb-size: 18px;
    --slider-track-color: var(--bg-pill-button);
    --slider-fill-color: var(--accent-orange);
    accent-color: var(--accent-orange);
    appearance: none;
    inline-size: 180px;
    block-size: 28px;
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
  }

  .accent-range-slider::-webkit-slider-runnable-track {
    block-size: var(--slider-track-size);
    border-radius: 999px;
    background: var(--slider-track-color);
  }

  .accent-range-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    inline-size: var(--slider-thumb-size);
    block-size: var(--slider-thumb-size);
    margin-block-start: calc((var(--slider-track-size) - var(--slider-thumb-size)) / 2);
    border: 0;
    border-radius: 50%;
    background: var(--slider-fill-color);
  }

  .accent-range-slider::-moz-range-track {
    block-size: var(--slider-track-size);
    border-radius: 999px;
    background: var(--slider-track-color);
  }

  .accent-range-slider::-moz-range-thumb {
    inline-size: var(--slider-thumb-size);
    block-size: var(--slider-thumb-size);
    border: 0;
    border-radius: 50%;
    background: var(--slider-fill-color);
  }

  .tab-reset-row {
    display: flex;
    justify-content: flex-end;
    padding-top: 6px;
  }

  .reset-btn {
    padding: 6px 12px;
    border-radius: 7px;
    background-color: var(--bg-pill-button);
    font-size: 12.4px;
    font-weight: 600;
  }

  .live-theme-preview {
    border-radius: 10px;
    border: 1px solid;
    overflow: clip;
  }

  .live-preview-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    border-bottom: 1px solid;
    font-size: 12px;
    font-weight: 600;
  }

  .live-preview-badge {
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
  }

  .live-preview-content {
    padding: 14px 18px;
    font-size: 14px;
  }

  /* Export Dialog 2-column layout */
  .export-split-body {
    display: flex;
    min-height: 280px;
  }

  .export-sidebar {
    width: 196px;
    border-right: 1px solid var(--border-headerbar);
    padding: 12px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .export-sidebar-hdr {
    padding: 4px 10px 8px 10px;
    font-size: 11.5px;
    font-weight: 700;
    color: var(--fg-muted);
    text-transform: uppercase;
  }

  .export-fmt-btn {
    padding: 8px 12px;
    border-radius: 8px;
    text-align: left;
    font-size: 13.2px;
    font-weight: 500;
  }

  .export-fmt-btn.is-active {
    background-color: color-mix(in srgb, var(--accent-orange) 16%, transparent);
    border: 1px solid var(--accent-orange);
    color: var(--accent-orange);
    font-weight: 700;
  }

  .export-options-pane {
    flex: 1;
    padding: 18px 22px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    padding: 12px 20px;
    border-top: 1px solid var(--border-headerbar);
    background-color: color-mix(in srgb, var(--bg-headerbar) 50%, var(--bg-popover));
  }

  .footer-btn {
    padding: 7px 18px;
    border-radius: 8px;
    background-color: var(--bg-pill-button);
    font-size: 13.2px;
    font-weight: 600;
  }

  .footer-btn-primary {
    background-color: var(--accent-orange);
    color: #ffffff;
  }

  .footer-btn-danger {
    background-color: #c53030;
    color: #ffffff;
  }

  .shortcuts-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .shortcuts-grid > div {
    display: flex;
    justify-content: space-between;
    font-size: 13.5px;
  }
</style>
