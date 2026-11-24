<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import {
    EditorState,
    Compartment,
    type Range,
  } from '@codemirror/state';
  import {
    EditorView,
    keymap,
    drawSelection,
    highlightSpecialChars,
    Decoration,
    type DecorationSet,
    ViewPlugin,
    type ViewUpdate,
  } from '@codemirror/view';
  import { defaultKeymap } from '@codemirror/commands';
  import {
    syntaxHighlighting,
    HighlightStyle,
  } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import {
    buildMarkdownLanguageSupport,
    codeBlockTextTag,
    createLivePreviewExtensions,
    inlineCodeTag,
    listMarkerTag,
  } from './livePreviewAst';
  import { charToUtf16Offset, utf16ToCharOffset } from './textOffsets';
  import type { MarkdownConfig, ThemeMode } from './types';

  interface Props {
    activeTabId: number;
    currentFilePath?: string | null;
    bufferText: string;
    bufferRevision: number;
    externalSyncToken: number;
    cursorChar: number;
    selectionStartChar: number;
    selectionEndChar: number;
    viewWhileTyping: boolean;
    markdownConfig: MarkdownConfig;
    themeMode: ThemeMode;
    proseFont: string;
    codeFont: string;
    fontSize: number;
    lineHeight: number;
    columnMaxWidth: string;
    columnPaddingX: string;
    searchQuery?: string;
    searchBarOpen?: boolean;
    chromeHidden?: boolean;
    readOnly?: boolean;
    onEdit: (text: string, selPrimary: number, selSecondary: number, isDocChanged: boolean) => void;
  }

  let {
    activeTabId,
    currentFilePath = null,
    bufferText,
    bufferRevision: _bufferRevision,
    externalSyncToken,
    cursorChar,
    selectionStartChar,
    selectionEndChar,
    viewWhileTyping,
    markdownConfig,
    themeMode,
    proseFont,
    codeFont,
    fontSize,
    lineHeight,
    columnMaxWidth,
    columnPaddingX,
    searchQuery = '',
    searchBarOpen = false,
    chromeHidden = false,
    readOnly = false,
    onEdit,
  }: Props = $props();

  let hostEl: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let lastLoadedTabId = -1;
  let lastAppliedSyncToken = -1;
  let suppressOutgoing = false;

  const languageCompartment = new Compartment();
  const themeCompartment = new Compartment();
  const readOnlyCompartment = new Compartment();
  const livePreviewCompartment = new Compartment();
  const searchHighlightCompartment = new Compartment();

  const accentMarkdownHighlight = HighlightStyle.define([
    { tag: tags.quote, color: 'var(--markup-quote-fg)' },
    { tag: tags.contentSeparator, color: 'var(--fg-muted)', fontWeight: '600' },
    { tag: tags.heading1, color: 'var(--fg-heading)', fontWeight: '700' },
    {
      tag: [
        tags.heading2,
        tags.heading3,
        tags.heading4,
        tags.heading5,
        tags.heading6,
      ],
      color: 'var(--fg-subheading)',
      fontWeight: '700',
    },
    { tag: tags.heading, color: 'var(--fg-heading)', fontWeight: '700' },
    { tag: tags.strong, fontWeight: '700', color: 'var(--markup-bold-fg)' },
    { tag: tags.emphasis, fontStyle: 'italic', color: 'var(--markup-emphasis-fg)' },
    { tag: tags.strikethrough, textDecoration: 'line-through', color: 'var(--fg-muted)' },
    {
      tag: inlineCodeTag,
      fontFamily: 'var(--code-font-stack)',
      color: 'var(--markup-code-fg)',
      backgroundColor: 'var(--bg-code-inline)',
      borderRadius: '4px',
    },
    {
      tag: codeBlockTextTag,
      fontFamily: 'var(--code-font-stack)',
      color: 'var(--markup-codeblock-fg)',
    },
    { tag: tags.escape, color: 'var(--syntax-number)' },
    { tag: tags.character, color: 'var(--syntax-number)' },
    { tag: tags.link, color: 'var(--accent-link)', textDecoration: 'underline' },
    { tag: tags.url, color: 'var(--markup-link-url)' },
    { tag: listMarkerTag, color: 'var(--markup-list-marker)', fontWeight: '700' },
    { tag: tags.processingInstruction, color: 'var(--fg-muted)' },
    { tag: tags.meta, color: 'var(--fg-muted)' },
    // Code block syntax highlighting (JSON, Rust, Bash, Python, TOML, YAML, HTML, CSS, JS/TS)
    {
      tag: [tags.variableName, tags.definition(tags.variableName)],
      color: 'var(--fg-primary)',
    },
    {
      tag: [
        tags.propertyName,
        tags.attributeName,
        tags.definition(tags.propertyName),
        tags.labelName,
      ],
      color: 'var(--syntax-key)',
    },
    {
      tag: [tags.string, tags.special(tags.string), tags.attributeValue, tags.regexp],
      color: 'var(--syntax-string)',
    },
    {
      tag: [tags.number, tags.bool, tags.null, tags.atom],
      color: 'var(--syntax-number)',
    },
    {
      tag: [
        tags.keyword,
        tags.controlKeyword,
        tags.operatorKeyword,
        tags.definitionKeyword,
        tags.moduleKeyword,
        tags.modifier,
      ],
      color: 'var(--syntax-keyword)',
      fontWeight: '600',
    },
    {
      tag: [
        tags.function(tags.variableName),
        tags.function(tags.propertyName),
        tags.standard(tags.variableName),
        tags.macroName,
      ],
      color: 'var(--syntax-function)',
    },
    {
      tag: [tags.typeName, tags.className, tags.namespace, tags.tagName],
      color: 'var(--syntax-type)',
    },
    {
      tag: [tags.comment, tags.lineComment, tags.blockComment, tags.docComment],
      color: 'var(--syntax-comment)',
      fontStyle: 'italic',
    },
    {
      tag: [
        tags.punctuation,
        tags.separator,
        tags.brace,
        tags.squareBracket,
        tags.paren,
        tags.operator,
      ],
      color: 'var(--syntax-punctuation)',
    },
  ]);

  function buildEditorTheme() {
    return EditorView.theme({
      '&': {
        height: '100%',
        backgroundColor: 'transparent',
        color: 'var(--fg-primary)',
        fontFamily: 'var(--prose-font-stack)',
        fontSize: `${fontSize}px`,
        lineHeight: `${lineHeight}`,
      },
      '.cm-scroller': {
        overflow: 'auto',
        fontFamily: 'var(--prose-font-stack)',
        lineHeight: `${lineHeight}`,
        paddingTop: '76px',
        paddingBottom: '96px',
        transition: 'padding-top 0.18s cubic-bezier(0.22, 1, 0.36, 1)',
      },
      '.cm-content': {
        maxWidth: columnMaxWidth,
        margin: '0 auto',
        paddingTop: '0',
        paddingLeft: columnPaddingX,
        paddingRight: columnPaddingX,
        caretColor: 'var(--fg-primary)',
        fontFamily: 'var(--prose-font-stack)',
      },
      '.cm-line': {
        padding: '1px 0',
      },
      '.cm-line:first-child': {
        marginTop: '0 !important',
        paddingTop: '0 !important',
      },
      '&.cm-focused': {
        outline: 'none',
      },
      '.cm-cursor, .cm-dropCursor': {
        borderLeftColor: 'var(--fg-primary)',
        borderLeftWidth: '2px',
      },
      '.cm-selectionBackground, ::selection': {
        backgroundColor: 'color-mix(in srgb, var(--accent-orange) 26%, transparent) !important',
      },
    });
  }

  function buildSearchDecorations(view: EditorView, activeQuery: string): DecorationSet {
    if (!activeQuery) return Decoration.none;
    const needle = activeQuery.toLowerCase();
    if (!needle) return Decoration.none;
    const doc = view.state.doc;
    const sel = view.state.selection.main;
    const ranges: Range<Decoration>[] = [];
    for (let ln = 1; ln <= doc.lines; ln++) {
      const line = doc.line(ln);
      const lower = line.text.toLowerCase();
      let idx = 0;
      while (idx <= lower.length - needle.length) {
        const found = lower.indexOf(needle, idx);
        if (found === -1) break;
        const from = line.from + found;
        const to = from + needle.length;
        if (to <= line.to && to > from) {
          const isCurrent = sel.from === from && sel.to === to;
          ranges.push(
            Decoration.mark({
              class: isCurrent ? 'cm-search-match cm-search-match-current' : 'cm-search-match',
            }).range(from, to)
          );
        }
        idx = found + Math.max(1, needle.length);
      }
    }
    return Decoration.set(ranges, true);
  }

  function createSearchHighlightPlugin(activeQuery: string) {
    return ViewPlugin.fromClass(
      class {
        decorations: DecorationSet;
        constructor(view: EditorView) {
          this.decorations = buildSearchDecorations(view, activeQuery);
        }
        update(update: ViewUpdate) {
          if (update.docChanged || update.selectionSet || update.viewportChanged) {
            this.decorations = buildSearchDecorations(update.view, activeQuery);
          }
        }
      },
      {
        decorations: (v) => v.decorations,
      }
    );
  }

  export function focusEditor() {
    view?.focus();
  }

  export function scrollToCharOffset(charOffset: number) {
    if (!view) return;
    const txt = view.state.doc.toString();
    const u16 = charToUtf16Offset(txt, charOffset);
    const clamped = Math.max(0, Math.min(u16, view.state.doc.length));
    suppressOutgoing = true;
    view.dispatch({
      selection: { anchor: clamped, head: clamped },
      effects: EditorView.scrollIntoView(clamped, { y: 'start', yMargin: 84 }),
    });
    suppressOutgoing = false;
    view.focus();
  }

  function resolveUtf16Selection(
    txt: string,
    selStartChar: number,
    selEndChar: number,
    curChar: number
  ): { anchor: number; head: number } {
    const startU16 = Math.min(charToUtf16Offset(txt, selStartChar), txt.length);
    const endU16 = Math.min(charToUtf16Offset(txt, selEndChar), txt.length);
    const curU16 = Math.min(charToUtf16Offset(txt, curChar), txt.length);
    if (endU16 > startU16) {
      if (curChar === selStartChar) {
        return { anchor: endU16, head: startU16 };
      }
      return { anchor: startU16, head: endU16 };
    }
    return { anchor: curU16, head: curU16 };
  }

  onMount(() => {
    if (!hostEl) return;
    lastLoadedTabId = activeTabId;
    lastAppliedSyncToken = externalSyncToken;
    const initSel = resolveUtf16Selection(
      bufferText,
      selectionStartChar,
      selectionEndChar,
      cursorChar
    );

    const startState = EditorState.create({
      doc: bufferText,
      selection: initSel,
      extensions: [
        highlightSpecialChars(),
        drawSelection(),
        EditorView.lineWrapping,
        languageCompartment.of(buildMarkdownLanguageSupport(markdownConfig)),
        syntaxHighlighting(accentMarkdownHighlight),
        keymap.of([...defaultKeymap]),
        themeCompartment.of(buildEditorTheme()),
        readOnlyCompartment.of(EditorState.readOnly.of(readOnly)),
        livePreviewCompartment.of(
          createLivePreviewExtensions(
            viewWhileTyping,
            themeMode,
            markdownConfig,
            currentFilePath
          )
        ),
        searchHighlightCompartment.of(
          createSearchHighlightPlugin(searchBarOpen ? searchQuery : '')
        ),
        EditorView.updateListener.of((update) => {
          if (suppressOutgoing) return;
          if (update.docChanged || update.selectionSet) {
            const txt = update.state.doc.toString();
            const mainSel = update.state.selection.main;
            const headChar = utf16ToCharOffset(txt, mainSel.head);
            const anchorChar = utf16ToCharOffset(txt, mainSel.anchor);
            onEdit(txt, headChar, anchorChar, update.docChanged);
          }
        }),
      ],
    });

    view = new EditorView({
      state: startState,
      parent: hostEl,
    });
  });

  $effect(() => {
    // React only to tab switches or explicit backend-originated mutations (externalSyncToken)
    const tabChanged = activeTabId !== lastLoadedTabId;
    const syncRequested = externalSyncToken !== lastAppliedSyncToken;

    if (view && (tabChanged || syncRequested)) {
      lastLoadedTabId = activeTabId;
      lastAppliedSyncToken = externalSyncToken;
      suppressOutgoing = true;
      const curDoc = view.state.doc.toString();
      const nextSel = resolveUtf16Selection(
        bufferText,
        selectionStartChar,
        selectionEndChar,
        cursorChar
      );
      if (tabChanged || curDoc !== bufferText) {
        view.dispatch({
          changes: { from: 0, to: view.state.doc.length, insert: bufferText },
          selection: nextSel,
          ...(tabChanged ? {} : { effects: EditorView.scrollIntoView(nextSel.head, { y: 'nearest' }) }),
        });
      } else {
        view.dispatch({
          selection: nextSel,
          effects: EditorView.scrollIntoView(nextSel.head, { y: 'center' }),
        });
      }
      suppressOutgoing = false;
    }
  });

  $effect(() => {
    // Reconfigure language compartment, typography, column geometry, view-while-typing, search highlights, and theme dynamically
    const _pf = proseFont;
    const _cf = codeFont;
    const _fs = fontSize;
    const _lh = lineHeight;
    const _cw = columnMaxWidth;
    const _cp = columnPaddingX;
    const _ro = readOnly;
    const _vwt = viewWhileTyping;
    const _tm = themeMode;
    const _mcBase = markdownConfig.base;
    const _mcMath = markdownConfig.extensions.math;
    const _mcMermaid = markdownConfig.extensions.mermaid;
    const _mcAlerts = markdownConfig.extensions.alerts;
    const _mcFootnotes = markdownConfig.extensions.footnotes;
    const _mcFrontmatter = markdownConfig.extensions.frontmatter;
    const _cfp = currentFilePath;
    const activeSearch = searchBarOpen ? searchQuery : '';
    if (view) {
      view.dispatch({
        effects: [
          languageCompartment.reconfigure(buildMarkdownLanguageSupport(markdownConfig)),
          themeCompartment.reconfigure(buildEditorTheme()),
          readOnlyCompartment.reconfigure(EditorState.readOnly.of(readOnly)),
          livePreviewCompartment.reconfigure(
            createLivePreviewExtensions(
              viewWhileTyping,
              themeMode,
              markdownConfig,
              _cfp
            )
          ),
          searchHighlightCompartment.reconfigure(createSearchHighlightPlugin(activeSearch)),
        ],
      });
    }
  });

  onDestroy(() => {
    view?.destroy();
    view = null;
  });
</script>

<div
  class="cm-editor-host"
  class:is-chrome-hidden={chromeHidden}
  bind:this={hostEl}
  data-testid="codemirror-surface"
></div>

<style>
  .cm-editor-host {
    width: 100%;
    height: 100%;
    overflow: clip;
  }

  .cm-editor-host.is-chrome-hidden :global(.cm-scroller) {
    padding-top: 36px !important;
  }

  :global(.cm-content > .cm-line:first-child),
  :global(.cm-line:first-child.cm-live-h1),
  :global(.cm-line:first-child.cm-live-h2),
  :global(.cm-line:first-child.cm-live-heading-line) {
    margin-top: 0 !important;
    padding-top: 0 !important;
  }

  :global(.cm-live-heading-line) {
    font-weight: 700;
  }
  :global(.cm-live-h1) {
    font-size: 1.44em;
    line-height: 1.32;
    color: var(--fg-heading);
    padding-top: 8px !important;
    padding-bottom: 4px !important;
  }
  :global(.cm-live-h2) {
    font-size: 1.26em;
    line-height: 1.35;
    color: var(--fg-subheading);
    padding-top: 6px !important;
    padding-bottom: 3px !important;
  }
  :global(.cm-live-h3) {
    font-size: 1.14em;
    color: var(--fg-subheading);
  }
  :global(.cm-live-h4),
  :global(.cm-live-h5),
  :global(.cm-live-h6) {
    font-size: 1.05em;
    color: var(--fg-subheading);
  }

  :global(.cm-live-setext-heading + .cm-live-setext-heading) {
    padding-top: 0 !important;
  }

  :global(.cm-live-setext-underline-line) {
    color: var(--fg-muted);
    opacity: 0.72;
  }

  :global(.cm-live-hr-line) {
    padding-top: 8px !important;
    padding-bottom: 8px !important;
  }

  :global(.cm-thematic-break-widget) {
    display: inline-block;
    width: 100%;
    height: 0;
    border-top: 1.5px solid var(--border-headerbar);
    vertical-align: middle;
    cursor: pointer;
  }

  :global(.cm-live-blockquote-line) {
    position: relative;
    border-left: 3px solid var(--markup-quote-bar);
    padding-left: 14px !important;
    color: var(--markup-quote-fg);
    font-style: normal;
  }

  :global(.cm-live-blockquote-d2) {
    padding-left: 28px !important;
  }
  :global(.cm-live-blockquote-d2::before) {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 11px;
    width: 3px;
    background-color: color-mix(in srgb, var(--markup-quote-bar) 72%, transparent);
    pointer-events: none;
  }

  :global(.cm-live-blockquote-d3),
  :global(.cm-live-blockquote-d4) {
    padding-left: 42px !important;
  }
  :global(.cm-live-blockquote-d3::before),
  :global(.cm-live-blockquote-d4::before) {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 11px;
    width: 3px;
    background-color: color-mix(in srgb, var(--markup-quote-bar) 72%, transparent);
    box-shadow: 14px 0 0 0 color-mix(in srgb, var(--markup-quote-bar) 52%, transparent);
    pointer-events: none;
  }

  :global(.cm-live-alert-line) {
    border-left-width: 4px;
    background-color: color-mix(in srgb, var(--bg-code-block) 45%, transparent);
    border-radius: 0;
    padding-right: 12px !important;
  }
  :global(.cm-live-alert-first) {
    border-top-right-radius: 7px;
    padding-top: 6px !important;
  }
  :global(.cm-live-alert-last) {
    border-bottom-right-radius: 7px;
    padding-bottom: 6px !important;
  }
  :global(.cm-live-alert-note) {
    border-left-color: var(--accent-link);
  }
  :global(.cm-live-alert-tip) {
    border-left-color: var(--markup-task-done);
  }
  :global(.cm-live-alert-important) {
    border-left-color: var(--syntax-keyword);
  }
  :global(.cm-live-alert-warning) {
    border-left-color: var(--accent-orange);
  }
  :global(.cm-live-alert-caution) {
    border-left-color: var(--spellcheck-red);
  }

  :global(.cm-alert-badge) {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 700;
    letter-spacing: 0.01em;
    padding: 1px 8px 1px 0;
    cursor: pointer;
    user-select: none;
  }
  :global(.cm-alert-badge-icon) {
    font-size: 0.96em;
    line-height: 1;
  }
  :global(.cm-alert-badge-note) {
    color: var(--accent-link);
  }
  :global(.cm-alert-badge-tip) {
    color: var(--markup-task-done);
  }
  :global(.cm-alert-badge-important) {
    color: var(--syntax-keyword);
  }
  :global(.cm-alert-badge-warning) {
    color: var(--accent-orange);
  }
  :global(.cm-alert-badge-caution) {
    color: var(--spellcheck-red);
  }

  :global(.cm-live-codeblock-line) {
    background-color: var(--bg-code-block);
    color: var(--markup-codeblock-fg);
    font-family: var(--code-font-stack) !important;
    font-size: 0.88em !important;
    line-height: 1.38 !important;
    padding-left: 14px !important;
    padding-right: 14px !important;
    border-left: 1px solid color-mix(in srgb, var(--border-headerbar) 75%, transparent);
    border-right: 1px solid color-mix(in srgb, var(--border-headerbar) 75%, transparent);
    border-radius: 0;
  }

  :global(.cm-live-codeblock-first) {
    border-top: 1px solid color-mix(in srgb, var(--border-headerbar) 75%, transparent);
    border-top-left-radius: 7px;
    border-top-right-radius: 7px;
    padding-top: 6px !important;
  }

  :global(.cm-live-codeblock-last) {
    border-bottom: 1px solid color-mix(in srgb, var(--border-headerbar) 75%, transparent);
    border-bottom-left-radius: 7px;
    border-bottom-right-radius: 7px;
    padding-bottom: 6px !important;
  }

  :global(.cm-live-codeblock-first[data-code-lang]) {
    position: relative;
    padding-right: 76px !important;
  }

  :global(.cm-live-codeblock-first[data-code-lang]::after) {
    content: attr(data-code-lang);
    position: absolute;
    top: 5px;
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

  :global(.cm-live-fence-muted) {
    opacity: 0.28;
  }

  :global(.cm-live-bold),
  :global(.cm-live-bold *) {
    font-weight: 700;
  }
  :global(.cm-live-bold) {
    color: var(--markup-bold-fg);
  }

  :global(.cm-live-italic),
  :global(.cm-live-italic *) {
    font-style: italic;
  }
  :global(.cm-live-italic) {
    color: var(--markup-emphasis-fg);
  }

  :global(.cm-live-bold.cm-live-italic),
  :global(.cm-live-italic .cm-live-bold),
  :global(.cm-live-bold .cm-live-italic) {
    color: var(--markup-bold-fg);
  }

  :global(.cm-live-strike) {
    text-decoration: line-through;
    color: var(--fg-muted);
  }

  :global(.cm-live-link),
  :global(.cm-live-link *) {
    color: var(--accent-link);
    text-decoration: underline;
    cursor: pointer;
  }

  :global(.cm-live-inline-code),
  :global(.cm-live-inline-code *) {
    font-family: var(--code-font-stack) !important;
    color: var(--markup-code-fg) !important;
    background-color: var(--bg-code-inline);
    padding: 1px 5px;
    border-radius: 4px;
  }

  :global(.cm-live-plain-brackets),
  :global(.cm-live-plain-brackets *) {
    color: var(--fg-primary) !important;
    text-decoration: none !important;
  }

  :global(.cm-live-list-marker),
  :global(.cm-live-list-marker *) {
    color: var(--markup-list-marker) !important;
    font-weight: 700;
  }

  :global(.cm-task-checkbox) {
    display: inline-block;
    color: var(--markup-list-marker);
    font-weight: 700;
    cursor: pointer;
    user-select: none;
    border-radius: 3px;
    padding: 0 1px;
    transition: transform 0.08s ease, color 0.12s ease;
  }

  :global(.cm-task-checkbox:hover) {
    background-color: color-mix(in srgb, var(--accent-orange) 16%, transparent);
  }

  :global(.cm-task-checkbox.is-checked) {
    color: var(--markup-task-done);
  }

  :global(.cm-entity-widget) {
    cursor: text;
  }

  :global(.cm-footnote-ref) {
    font-size: 0.78em;
    font-weight: 700;
    line-height: 1;
    color: var(--accent-link);
    cursor: pointer;
    padding: 0 1px;
  }

  :global(.cm-live-footnote-def-line) {
    font-size: 0.92em;
    color: var(--fg-muted);
    border-left: 2px solid var(--border-headerbar);
    padding-left: 10px !important;
  }

  :global(.cm-live-footnote-label),
  :global(.cm-live-footnote-label *) {
    color: var(--accent-link) !important;
    font-weight: 700;
    text-decoration: none !important;
  }

  :global(.cm-katex-inline-widget) {
    display: inline-block;
    padding: 0 2px;
    cursor: pointer;
  }

  :global(.cm-katex-display-widget) {
    display: block;
    margin: 10px 0;
    padding: 10px 14px;
    border-radius: 8px;
    background-color: color-mix(in srgb, var(--bg-code-block) 55%, transparent);
    text-align: center;
    cursor: pointer;
  }

  :global(.cm-katex-display-widget.is-in-blockquote) {
    border-left: 3px solid var(--markup-quote-bar);
  }

  :global(.cm-mermaid-widget) {
    display: flex;
    justify-content: center;
    margin: 12px 0;
    padding: 10px;
    border-radius: 10px;
    background-color: color-mix(in srgb, var(--bg-code-block) 45%, transparent);
    cursor: pointer;
    overflow-x: auto;
  }

  :global(.cm-mermaid-widget.is-in-blockquote) {
    border-left: 3px solid var(--markup-quote-bar);
  }

  :global(.cm-mermaid-widget svg) {
    display: block;
    max-width: 100%;
    height: auto;
  }

  /* CommonMark / GFM §6.7 Caret-Aware ImageWidget & Compact Fallback Badge */
  :global(.cm-image-widget.is-standalone.is-loaded) {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    max-width: 100%;
    margin: 6px 0;
    padding: 6px;
    border-radius: 9px;
    border: 1px solid color-mix(in srgb, var(--border-headerbar) 85%, transparent);
    background-color: color-mix(in srgb, var(--bg-code-block) 36%, transparent);
    box-shadow: 0 2px 10px color-mix(in srgb, #000000 8%, transparent);
    cursor: pointer;
    user-select: none;
  }

  :global(.cm-image-widget.is-inline.is-loaded) {
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
    max-width: 100%;
    margin: 2px 3px;
    padding: 3px;
    border-radius: 6px;
    border: 1px solid color-mix(in srgb, var(--border-headerbar) 80%, transparent);
    background-color: color-mix(in srgb, var(--bg-code-block) 32%, transparent);
    box-shadow: 0 1px 6px color-mix(in srgb, #000000 7%, transparent);
    cursor: pointer;
    user-select: none;
  }

  :global(.cm-image-widget-img) {
    display: block;
    max-width: 100%;
    max-height: 320px;
    width: auto;
    height: auto;
    object-fit: contain;
    border-radius: 5px;
  }

  :global(.cm-image-widget.is-inline .cm-image-widget-img) {
    max-height: 160px;
  }

  :global(.cm-image-fallback-badge) {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 100%;
    padding: 2px 10px;
    margin: 1px 2px;
    border-radius: 6px;
    border: 1px dashed var(--border-headerbar);
    background-color: color-mix(in srgb, var(--bg-code-block) 52%, transparent);
    color: var(--fg-muted);
    font-size: 0.9em;
    line-height: 1.35;
    cursor: pointer;
    user-select: none;
    vertical-align: baseline;
  }

  :global(.cm-image-fallback-badge:hover) {
    border-color: var(--accent-orange);
    color: var(--fg-primary);
  }

  :global(.cm-image-fallback-icon) {
    font-size: 0.95em;
    line-height: 1;
    flex-shrink: 0;
  }

  :global(.cm-image-fallback-alt) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 42ch;
  }

  /* GFM §4.10 Caret-Aware Live Table Widget & Aligned Monospace Source Lines */
  :global(.cm-table-widget) {
    display: block;
    width: 100%;
    max-width: 100%;
    padding: 8px 0;
    user-select: none;
  }

  :global(.cm-table-widget.is-in-blockquote) {
    border-left: 3px solid var(--markup-quote-bar);
    padding-left: 14px;
  }

  :global(.cm-table-scroll) {
    width: 100%;
    max-width: 100%;
    overflow-x: auto;
    border-radius: 8px;
    border: 1px solid var(--border-headerbar);
    background-color: color-mix(in srgb, var(--bg-canvas) 90%, var(--bg-code-block));
    scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
    scrollbar-width: thin;
  }

  :global(.cm-table-grid) {
    width: 100%;
    border-collapse: collapse;
    border-spacing: 0;
    font-family: var(--prose-font-stack);
    font-size: 0.96em;
    line-height: 1.45;
    color: var(--fg-primary);
    overflow-wrap: normal;
    word-break: normal;
  }

  :global(.cm-table-grid th),
  :global(.cm-table-grid td) {
    padding: 8px 12px;
    border-right: 1px solid var(--border-headerbar);
    border-bottom: 1px solid var(--border-headerbar);
    vertical-align: top;
    overflow-wrap: normal;
    word-break: normal;
    cursor: text;
    transition: background-color 0.12s ease;
  }

  :global(.cm-table-grid th:last-child),
  :global(.cm-table-grid td:last-child) {
    border-right: none;
  }

  :global(.cm-table-grid tbody tr:last-child td),
  :global(.cm-table-grid thead:last-child tr:last-child th) {
    border-bottom: none;
  }

  :global(.cm-table-grid :where(th, td):not([data-align])),
  :global(.cm-table-grid [data-align='left']) {
    text-align: left;
  }

  :global(.cm-table-grid [data-align='center']) {
    text-align: center;
  }

  :global(.cm-table-grid [data-align='right']) {
    text-align: right;
  }

  :global(.cm-table-grid th) {
    background-color: var(--bg-table-header);
    font-weight: 700;
    color: var(--fg-primary);
  }

  :global(.cm-table-grid tbody tr:nth-child(even) td) {
    background-color: color-mix(in srgb, var(--bg-code-block) 18%, transparent);
  }

  @media (hover: hover) {
    :global(.cm-table-grid tbody tr:hover td) {
      background-color: color-mix(in srgb, var(--bg-pill-button) 36%, transparent);
    }

    :global(.cm-table-grid th:hover) {
      background-color: color-mix(in srgb, var(--accent-orange) 12%, var(--bg-table-header));
    }

    :global(.cm-table-grid tbody tr:hover td:hover) {
      background-color: color-mix(in srgb, var(--accent-orange) 12%, var(--bg-pill-button));
    }
  }

  :global(.cm-table-grid strong) {
    color: var(--markup-bold-fg);
    font-weight: 700;
  }

  :global(.cm-table-grid em) {
    color: var(--markup-emphasis-fg);
    font-style: italic;
  }

  :global(.cm-table-grid del) {
    color: var(--fg-muted);
    text-decoration: line-through;
  }

  :global(.cm-table-grid code) {
    font-family: var(--code-font-stack);
    font-size: 0.9em;
    background-color: var(--bg-code-inline);
    color: var(--markup-code-fg);
    padding: 1px 5px;
    border-radius: 4px;
    white-space: nowrap;
  }

  :global(.cm-table-grid a.cm-table-link) {
    color: var(--accent-link);
    text-decoration: underline;
    cursor: pointer;
  }

  :global(.cm-table-grid kbd) {
    font-family: var(--code-font-stack);
    font-size: 0.86em;
    padding: 1px 6px;
    border-radius: 4px;
    border: 1px solid var(--border-headerbar);
    background-color: var(--bg-code-inline);
    color: var(--fg-primary);
  }

  :global(.cm-table-math) {
    display: inline-block;
    padding: 0 2px;
  }

  :global(.cm-live-table-source-line) {
    font-family: var(--code-font-stack) !important;
    font-size: 0.92em !important;
    line-height: 1.44 !important;
    font-variant-ligatures: none;
    background-color: color-mix(in srgb, var(--bg-code-block) 28%, transparent);
    padding-left: 10px !important;
    padding-right: 10px !important;
    border-left: 1px solid color-mix(in srgb, var(--border-headerbar) 80%, transparent);
    border-right: 1px solid color-mix(in srgb, var(--border-headerbar) 80%, transparent);
  }

  :global(.cm-live-table-source-header) {
    background-color: color-mix(in srgb, var(--bg-table-header) 62%, transparent);
  }

  :global(.cm-live-table-source-first) {
    border-top: 1px solid color-mix(in srgb, var(--border-headerbar) 80%, transparent);
    border-top-left-radius: 7px;
    border-top-right-radius: 7px;
    padding-top: 4px !important;
  }

  :global(.cm-live-table-source-last) {
    border-bottom: 1px solid color-mix(in srgb, var(--border-headerbar) 80%, transparent);
    border-bottom-left-radius: 7px;
    border-bottom-right-radius: 7px;
    padding-bottom: 4px !important;
  }

  :global(.cm-search-match) {
    background-color: color-mix(in srgb, var(--accent-orange) 22%, transparent);
    border-radius: 3px;
  }

  :global(.cm-search-match-current) {
    background-color: color-mix(in srgb, var(--accent-orange) 48%, transparent);
    outline: 1.5px solid var(--accent-orange);
    border-radius: 3px;
  }
</style>
