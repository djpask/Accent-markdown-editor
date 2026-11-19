<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { EditorState, Compartment, StateEffect, type Range } from '@codemirror/state';
  import {
    EditorView,
    keymap,
    drawSelection,
    highlightSpecialChars,
    Decoration,
    type DecorationSet,
    ViewPlugin,
    type ViewUpdate,
    WidgetType,
  } from '@codemirror/view';
  import { defaultKeymap } from '@codemirror/commands';
  import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
  import {
    syntaxHighlighting,
    HighlightStyle,
    StreamLanguage,
    type Language,
  } from '@codemirror/language';
  import { jsonLanguage } from '@codemirror/lang-json';
  import { rustLanguage } from '@codemirror/lang-rust';
  import { pythonLanguage } from '@codemirror/lang-python';
  import { yamlLanguage } from '@codemirror/lang-yaml';
  import { htmlLanguage } from '@codemirror/lang-html';
  import { cssLanguage } from '@codemirror/lang-css';
  import {
    javascriptLanguage,
    typescriptLanguage,
    jsxLanguage,
    tsxLanguage,
  } from '@codemirror/lang-javascript';
  import { shell } from '@codemirror/legacy-modes/mode/shell';
  import { toml } from '@codemirror/legacy-modes/mode/toml';
  import { styleTags, tags, Tag } from '@lezer/highlight';
  import { getOrFetchMermaidSvg, renderLatexToHtml } from './mathAndMermaid';
  import type { ThemeMode } from './types';

  const shellLanguage = StreamLanguage.define(shell);
  const tomlLanguage = StreamLanguage.define(toml);
  const listMarkerTag = Tag.define();

  function resolveCodeLanguage(info: string): Language | null {
    const lang = info.trim().toLowerCase().split(/\s+/)[0];
    switch (lang) {
      case 'json':
      case 'jsonc':
      case 'json5':
        return jsonLanguage;
      case 'rust':
      case 'rs':
        return rustLanguage;
      case 'bash':
      case 'sh':
      case 'shell':
      case 'zsh':
      case 'console':
        return shellLanguage;
      case 'python':
      case 'py':
        return pythonLanguage;
      case 'toml':
      case 'ini':
        return tomlLanguage;
      case 'yaml':
      case 'yml':
        return yamlLanguage;
      case 'html':
      case 'htm':
      case 'xml':
      case 'svg':
        return htmlLanguage;
      case 'css':
      case 'scss':
      case 'less':
        return cssLanguage;
      case 'js':
      case 'javascript':
      case 'mjs':
      case 'cjs':
        return javascriptLanguage;
      case 'ts':
      case 'typescript':
      case 'mts':
        return typescriptLanguage;
      case 'jsx':
        return jsxLanguage;
      case 'tsx':
        return tsxLanguage;
      default:
        return null;
    }
  }

  interface Props {
    activeTabId: number;
    bufferText: string;
    bufferRevision: number;
    cursorChar: number;
    selectionStartChar: number;
    selectionEndChar: number;
    viewWhileTyping: boolean;
    themeMode: ThemeMode;
    proseFont: string;
    codeFont: string;
    fontSize: number;
    lineHeight: number;
    columnMaxWidth: string;
    columnPaddingX: string;
    chromeHidden?: boolean;
    readOnly?: boolean;
    onEdit: (text: string, selPrimary: number, selSecondary: number, isDocChanged: boolean) => void;
  }

  let {
    activeTabId,
    bufferText,
    bufferRevision,
    cursorChar,
    selectionStartChar,
    selectionEndChar,
    viewWhileTyping,
    themeMode,
    proseFont,
    codeFont,
    fontSize,
    lineHeight,
    columnMaxWidth,
    columnPaddingX,
    chromeHidden = false,
    readOnly = false,
    onEdit,
  }: Props = $props();

  let hostEl: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let lastLoadedTabId = -1;
  let lastKnownText = '';
  let lastAppliedSelStart = 0;
  let lastAppliedSelEnd = 0;
  let suppressOutgoing = false;

  const themeCompartment = new Compartment();
  const readOnlyCompartment = new Compartment();
  const livePreviewCompartment = new Compartment();
  const refreshLivePreviewEffect = StateEffect.define<boolean>();

  class KatexMathWidget extends WidgetType {
    latex: string;
    displayMode: boolean;
    targetPos: number;

    constructor(latex: string, displayMode: boolean, targetPos: number) {
      super();
      this.latex = latex;
      this.displayMode = displayMode;
      this.targetPos = targetPos;
    }

    eq(other: KatexMathWidget): boolean {
      return (
        this.latex === other.latex &&
        this.displayMode === other.displayMode &&
        this.targetPos === other.targetPos
      );
    }

    toDOM(view: EditorView): HTMLElement {
      const el = document.createElement(this.displayMode ? 'div' : 'span');
      el.className = this.displayMode ? 'cm-katex-display-widget' : 'cm-katex-inline-widget';
      el.dataset.testid = this.displayMode ? 'cm-katex-display' : 'cm-katex-inline';
      el.innerHTML = renderLatexToHtml(this.latex, this.displayMode);
      el.addEventListener('mousedown', (e) => {
        e.preventDefault();
        view.dispatch({
          selection: { anchor: this.targetPos, head: this.targetPos },
        });
        view.focus();
      });
      return el;
    }

    ignoreEvent(): boolean {
      return false;
    }
  }

  class MermaidSvgWidget extends WidgetType {
    code: string;
    theme: ThemeMode;
    targetPos: number;

    constructor(code: string, theme: ThemeMode, targetPos: number) {
      super();
      this.code = code;
      this.theme = theme;
      this.targetPos = targetPos;
    }

    eq(other: MermaidSvgWidget): boolean {
      return (
        this.code === other.code &&
        this.theme === other.theme &&
        this.targetPos === other.targetPos
      );
    }

    toDOM(view: EditorView): HTMLElement {
      const wrap = document.createElement('div');
      wrap.className = 'cm-mermaid-widget';
      wrap.dataset.testid = 'cm-mermaid-widget';
      wrap.title = 'Clicca per modificare il diagramma Mermaid';

      const cachedSvg = getOrFetchMermaidSvg(this.code, this.theme, () => {
        view.dispatch({ effects: refreshLivePreviewEffect.of(true) });
      });

      if (cachedSvg) {
        wrap.innerHTML = cachedSvg;
      } else {
        wrap.textContent = 'Rendering Mermaid diagram...';
      }

      wrap.addEventListener('mousedown', (e) => {
        e.preventDefault();
        view.dispatch({
          selection: { anchor: this.targetPos, head: this.targetPos },
        });
        view.focus();
      });
      return wrap;
    }

    ignoreEvent(): boolean {
      return false;
    }
  }

  class TaskCheckboxWidget extends WidgetType {
    checked: boolean;
    targetPos: number;

    constructor(checked: boolean, targetPos: number) {
      super();
      this.checked = checked;
      this.targetPos = targetPos;
    }

    eq(other: TaskCheckboxWidget): boolean {
      return this.checked === other.checked && this.targetPos === other.targetPos;
    }

    toDOM(view: EditorView): HTMLElement {
      const span = document.createElement('span');
      span.className = this.checked ? 'cm-task-checkbox is-checked' : 'cm-task-checkbox';
      span.textContent = this.checked ? '☑ ' : '☐ ';
      span.addEventListener('mousedown', (e) => {
        e.preventDefault();
        view.dispatch({
          selection: { anchor: this.targetPos, head: this.targetPos },
        });
        view.focus();
      });
      return span;
    }
  }

  function buildCaretAwareDecorations(
    view: EditorView,
    enabled: boolean,
    currentTheme: ThemeMode
  ): DecorationSet {
    const state = view.state;
    const doc = state.doc;
    const sel = state.selection.main;
    const selFrom = sel.from;
    const selTo = sel.to;
    const selStartLine = doc.lineAt(selFrom).number;
    const selEndLine = doc.lineAt(selTo).number;

    const isCaretInSpan = (from: number, to: number): boolean => {
      return selFrom <= to && selTo >= from;
    };

    const isCaretOnLines = (l1: number, l2: number): boolean => {
      return selStartLine <= l2 && selEndLine >= l1;
    };

    const ranges: Range<Decoration>[] = [];
    const totalLines = doc.lines;
    let lineNum = 1;

    while (lineNum <= totalLines) {
      const line = doc.line(lineNum);
      const text = line.text;
      const trimmed = text.trimStart();

      // 1. Check for Fenced Code Block (``` or ~~~)
      const fenceMatch = trimmed.match(/^(`{3,}|~{3,})\s*([^\s`~]*)/);
      if (fenceMatch) {
        const fenceChars = fenceMatch[1];
        const fenceLang = (fenceMatch[2] || '').toLowerCase();
        const startLineNum = lineNum;
        let endLineNum = lineNum;
        const bodyLines: string[] = [];

        let scan = lineNum + 1;
        while (scan <= totalLines) {
          const sLine = doc.line(scan);
          if (sLine.text.trimStart().startsWith(fenceChars)) {
            endLineNum = scan;
            break;
          }
          bodyLines.push(sLine.text);
          scan++;
        }

        if (endLineNum > startLineNum) {
          const caretInsideBlock = isCaretOnLines(startLineNum, endLineNum);

          if (enabled && fenceLang === 'mermaid' && !caretInsideBlock) {
            const mermaidCode = bodyLines.join('\n');
            const firstLine = doc.line(startLineNum);
            ranges.push(
              Decoration.widget({
                widget: new MermaidSvgWidget(mermaidCode, currentTheme, firstLine.from),
                side: -1,
              }).range(firstLine.from)
            );
            for (let ln = startLineNum; ln <= endLineNum; ln++) {
              const l = doc.line(ln);
              ranges.push(
                Decoration.line({ class: 'cm-live-mermaid-hidden-line' }).range(l.from)
              );
            }
          } else {
            for (let ln = startLineNum; ln <= endLineNum; ln++) {
              const l = doc.line(ln);
              const isFenceBoundary = ln === startLineNum || ln === endLineNum;
              if (enabled && isFenceBoundary && !caretInsideBlock) {
                ranges.push(
                  Decoration.line({
                    class: 'cm-live-codeblock-line cm-live-fence-muted',
                  }).range(l.from)
                );
              } else {
                ranges.push(
                  Decoration.line({ class: 'cm-live-codeblock-line' }).range(l.from)
                );
              }
            }
          }

          lineNum = endLineNum + 1;
          continue;
        }
      }

      if (!enabled) {
        lineNum++;
        continue;
      }

      // 2. Check for multi-line Display Math Block ($$ ... $$)
      if (trimmed.startsWith('$$') && (trimmed === '$$' || !trimmed.slice(2).includes('$$'))) {
        const startLineNum = lineNum;
        let endLineNum = lineNum;
        const mathLines: string[] = [];
        const firstRest = trimmed.slice(2).trim();
        if (firstRest) mathLines.push(firstRest);

        let scan = lineNum + 1;
        while (scan <= totalLines) {
          const sLine = doc.line(scan);
          const sTrim = sLine.text.trim();
          if (sTrim.endsWith('$$')) {
            const beforeEnd = sTrim.slice(0, -2).trim();
            if (beforeEnd) mathLines.push(beforeEnd);
            endLineNum = scan;
            break;
          }
          mathLines.push(sLine.text);
          scan++;
        }

        if (endLineNum > startLineNum) {
          const caretInsideMath = isCaretOnLines(startLineNum, endLineNum);
          if (!caretInsideMath) {
            const firstLine = doc.line(startLineNum);
            ranges.push(
              Decoration.widget({
                widget: new KatexMathWidget(mathLines.join('\n'), true, firstLine.from),
                side: -1,
              }).range(firstLine.from)
            );
            for (let ln = startLineNum; ln <= endLineNum; ln++) {
              const l = doc.line(ln);
              ranges.push(
                Decoration.line({ class: 'cm-live-mermaid-hidden-line' }).range(l.from)
              );
            }
          }
          lineNum = endLineNum + 1;
          continue;
        }
      }

      // 3. Single-line Display Math ($$...$$)
      const singleDisplayMath = trimmed.match(/^\$\$(.+)\$\$$/);
      if (singleDisplayMath && !isCaretOnLines(lineNum, lineNum)) {
        ranges.push(
          Decoration.replace({
            widget: new KatexMathWidget(singleDisplayMath[1], true, line.from),
          }).range(line.from, line.to)
        );
        lineNum++;
        continue;
      }

      // 4. ATX Headings (# .. ######)
      const headingMatch = text.match(/^(\s*#{1,6}\s+)(.+)$/);
      if (headingMatch) {
        const prefix = headingMatch[1];
        const level = prefix.trim().length;
        ranges.push(
          Decoration.line({
            class: `cm-live-heading-line cm-live-h${level}`,
          }).range(line.from)
        );
        if (!isCaretOnLines(lineNum, lineNum)) {
          ranges.push(
            Decoration.replace({}).range(line.from, line.from + prefix.length)
          );
        }
      }

      // 5. Blockquotes (> )
      const quoteMatch = text.match(/^(\s*>\s?)/);
      if (quoteMatch) {
        ranges.push(
          Decoration.line({ class: 'cm-live-blockquote-line' }).range(line.from)
        );
        if (!isCaretOnLines(lineNum, lineNum)) {
          ranges.push(
            Decoration.replace({}).range(line.from, line.from + quoteMatch[1].length)
          );
        }
      }

      // 6. Task Lists (- [ ] / - [x]) & Bullet/Ordered List Markers
      const taskMatch = text.match(/^(\s*[-*+]\s+\[([ xX])\]\s+)/);
      if (taskMatch && !isCaretOnLines(lineNum, lineNum)) {
        const prefixLen = taskMatch[1].length;
        const checked = taskMatch[2].toLowerCase() === 'x';
        ranges.push(
          Decoration.replace({
            widget: new TaskCheckboxWidget(checked, line.from),
          }).range(line.from, line.from + prefixLen)
        );
      } else {
        const listMarkerMatch = text.match(/^(\s*(?:[-*+]|\d+[.)]))(\s+)/);
        if (listMarkerMatch) {
          ranges.push(
            Decoration.mark({ class: 'cm-live-list-marker' }).range(
              line.from,
              line.from + listMarkerMatch[1].length
            )
          );
        }
      }

      // 7. Inline constructs (non-overlapping intervals on this line)
      const occupied: Array<[number, number]> = [];
      const isOccupied = (a: number, b: number) =>
        occupied.some(([x, y]) => a < y && b > x);
      const markOccupied = (a: number, b: number) => {
        occupied.push([a, b]);
      };

      // 7a. Inline Code (`code`)
      for (const m of text.matchAll(/`([^`\n]+)`/g)) {
        if (m.index === undefined) continue;
        const from = line.from + m.index;
        const to = from + m[0].length;
        if (isOccupied(from, to)) continue;
        markOccupied(from, to);
        if (!isCaretInSpan(from, to) && to - from > 2) {
          ranges.push(Decoration.replace({}).range(from, from + 1));
          ranges.push(
            Decoration.mark({ class: 'cm-live-inline-code' }).range(from + 1, to - 1)
          );
          ranges.push(Decoration.replace({}).range(to - 1, to));
        }
      }

      // 7b. Inline KaTeX Math ($...$)
      for (const m of text.matchAll(/(?<!\\|\$)\$([^\s$][^$\n]*?[^\s$]|[^\s$])\$/g)) {
        if (m.index === undefined) continue;
        const from = line.from + m.index;
        const to = from + m[0].length;
        if (isOccupied(from, to)) continue;
        markOccupied(from, to);
        if (!isCaretInSpan(from, to)) {
          ranges.push(
            Decoration.replace({
              widget: new KatexMathWidget(m[1], false, from),
            }).range(from, to)
          );
        }
      }

      // 7c. Inline Links ([label](url))
      for (const m of text.matchAll(/(?<!!)\[([^\]]+)\]\(([^)\n]+)\)/g)) {
        if (m.index === undefined) continue;
        const from = line.from + m.index;
        const to = from + m[0].length;
        const labelLen = m[1].length;
        if (isOccupied(from, to)) continue;
        markOccupied(from, to);
        if (!isCaretInSpan(from, to)) {
          ranges.push(Decoration.replace({}).range(from, from + 1));
          ranges.push(
            Decoration.mark({ class: 'cm-live-link' }).range(
              from + 1,
              from + 1 + labelLen
            )
          );
          ranges.push(Decoration.replace({}).range(from + 1 + labelLen, to));
        }
      }

      // 7d. Bold (**text** or __text__)
      for (const m of text.matchAll(/(\*\*|__)(?=\S)([^\n]*?\S)\1/g)) {
        if (m.index === undefined) continue;
        const from = line.from + m.index;
        const to = from + m[0].length;
        if (isOccupied(from, to)) continue;
        markOccupied(from, to);
        if (!isCaretInSpan(from, to) && to - from > 4) {
          ranges.push(Decoration.replace({}).range(from, from + 2));
          ranges.push(Decoration.mark({ class: 'cm-live-bold' }).range(from + 2, to - 2));
          ranges.push(Decoration.replace({}).range(to - 2, to));
        }
      }

      // 7e. Strikethrough (~~text~~)
      for (const m of text.matchAll(/~~(?=\S)([^\n]*?\S)~~/g)) {
        if (m.index === undefined) continue;
        const from = line.from + m.index;
        const to = from + m[0].length;
        if (isOccupied(from, to)) continue;
        markOccupied(from, to);
        if (!isCaretInSpan(from, to) && to - from > 4) {
          ranges.push(Decoration.replace({}).range(from, from + 2));
          ranges.push(
            Decoration.mark({ class: 'cm-live-strike' }).range(from + 2, to - 2)
          );
          ranges.push(Decoration.replace({}).range(to - 2, to));
        }
      }

      // 7f. Italic (*text*)
      for (const m of text.matchAll(/(?<!\*)\*(?=\S)([^*\n]*?\S)\*(?!\*)/g)) {
        if (m.index === undefined) continue;
        const from = line.from + m.index;
        const to = from + m[0].length;
        if (isOccupied(from, to)) continue;
        markOccupied(from, to);
        if (!isCaretInSpan(from, to) && to - from > 2) {
          ranges.push(Decoration.replace({}).range(from, from + 1));
          ranges.push(
            Decoration.mark({ class: 'cm-live-italic' }).range(from + 1, to - 1)
          );
          ranges.push(Decoration.replace({}).range(to - 1, to));
        }
      }

      lineNum++;
    }

    return Decoration.set(ranges, true);
  }

  function createLivePreviewPlugin(enabled: boolean, currentTheme: ThemeMode) {
    return ViewPlugin.fromClass(
      class {
        decorations: DecorationSet;
        constructor(view: EditorView) {
          this.decorations = buildCaretAwareDecorations(view, enabled, currentTheme);
        }
        update(update: ViewUpdate) {
          const hasRefreshEffect = update.transactions.some((tr) =>
            tr.effects.some((e) => e.is(refreshLivePreviewEffect))
          );
          if (
            update.docChanged ||
            update.selectionSet ||
            update.viewportChanged ||
            hasRefreshEffect
          ) {
            this.decorations = buildCaretAwareDecorations(
              update.view,
              enabled,
              currentTheme
            );
          }
        }
      },
      {
        decorations: (v) => v.decorations,
      }
    );
  }

  const accentMarkdownHighlight = HighlightStyle.define([
    { tag: tags.quote, color: 'var(--markup-quote-fg)', fontStyle: 'italic' },
    { tag: tags.heading1, color: 'var(--fg-heading)', fontWeight: '700' },
    { tag: tags.heading2, color: 'var(--fg-heading)', fontWeight: '700' },
    { tag: tags.heading3, color: 'var(--fg-subheading)', fontWeight: '700' },
    {
      tag: [tags.heading4, tags.heading5, tags.heading6],
      color: 'var(--fg-subheading)',
      fontWeight: '700',
    },
    { tag: tags.strong, fontWeight: '700', color: 'var(--markup-bold-fg)' },
    { tag: tags.emphasis, fontStyle: 'italic', color: 'var(--markup-emphasis-fg)' },
    { tag: tags.strikethrough, textDecoration: 'line-through', color: 'var(--fg-muted)' },
    {
      tag: tags.monospace,
      fontFamily: 'var(--code-font-stack)',
      color: 'var(--markup-code-fg)',
      backgroundColor: 'var(--bg-code-inline)',
      borderRadius: '4px',
    },
    { tag: tags.link, color: 'var(--accent-link)', textDecoration: 'underline' },
    { tag: tags.url, color: 'var(--markup-link-url)' },
    { tag: listMarkerTag, color: 'var(--markup-list-marker)', fontWeight: '700' },
    { tag: tags.processingInstruction, color: 'var(--fg-muted)' },
    { tag: tags.meta, color: 'var(--fg-muted)' },
    // Code block syntax highlighting (JSON, Rust, Bash, Python, TOML, YAML, HTML, CSS, JS/TS)
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

  export function focusEditor() {
    view?.focus();
  }

  export function scrollToCharOffset(charOffset: number) {
    if (!view) return;
    const clamped = Math.max(0, Math.min(charOffset, view.state.doc.length));
    suppressOutgoing = true;
    view.dispatch({
      selection: { anchor: clamped, head: clamped },
      effects: EditorView.scrollIntoView(clamped, { y: 'start', yMargin: 84 }),
    });
    suppressOutgoing = false;
    view.focus();
  }

  onMount(() => {
    if (!hostEl) return;
    lastLoadedTabId = activeTabId;
    lastKnownText = bufferText;
    lastAppliedSelStart = selectionStartChar;
    lastAppliedSelEnd = selectionEndChar;

    const startState = EditorState.create({
      doc: bufferText,
      selection: {
        anchor: Math.min(selectionStartChar, bufferText.length),
        head: Math.min(cursorChar, bufferText.length),
      },
      extensions: [
        highlightSpecialChars(),
        drawSelection(),
        EditorView.lineWrapping,
        markdown({
          base: markdownLanguage,
          codeLanguages: resolveCodeLanguage,
          extensions: [
            {
              props: [
                styleTags({
                  'OrderedList/... BulletList/...': tags.content,
                  ListMark: listMarkerTag,
                  CodeText: tags.content,
                  InlineCode: tags.monospace,
                }),
              ],
            },
          ],
        }),
        syntaxHighlighting(accentMarkdownHighlight),
        keymap.of([...defaultKeymap]),
        themeCompartment.of(buildEditorTheme()),
        readOnlyCompartment.of(EditorState.readOnly.of(readOnly)),
        livePreviewCompartment.of(createLivePreviewPlugin(viewWhileTyping, themeMode)),
        EditorView.updateListener.of((update) => {
          if (suppressOutgoing) return;
          if (update.docChanged || update.selectionSet) {
            const txt = update.state.doc.toString();
            lastKnownText = txt;
            const mainSel = update.state.selection.main;
            lastAppliedSelStart = Math.min(mainSel.anchor, mainSel.head);
            lastAppliedSelEnd = Math.max(mainSel.anchor, mainSel.head);
            onEdit(txt, mainSel.head, mainSel.anchor, update.docChanged);
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
    // React to tab switches, external buffer changes (formatting, file open, undo/redo), or external selection jumps (FindNext/Outline)
    const tabChanged = activeTabId !== lastLoadedTabId;
    const textChangedExternally = bufferText !== lastKnownText;
    const selChangedExternally =
      selectionStartChar !== lastAppliedSelStart || selectionEndChar !== lastAppliedSelEnd;
    const _rev = bufferRevision;

    if (view && (tabChanged || textChangedExternally)) {
      lastLoadedTabId = activeTabId;
      lastKnownText = bufferText;
      lastAppliedSelStart = selectionStartChar;
      lastAppliedSelEnd = selectionEndChar;
      suppressOutgoing = true;
      const anchor = Math.min(selectionStartChar, bufferText.length);
      const head = Math.min(
        selectionEndChar > selectionStartChar ? selectionEndChar : cursorChar,
        bufferText.length
      );
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: bufferText },
        selection: { anchor, head },
      });
      suppressOutgoing = false;
    } else if (view && selChangedExternally) {
      lastAppliedSelStart = selectionStartChar;
      lastAppliedSelEnd = selectionEndChar;
      suppressOutgoing = true;
      const anchor = Math.min(selectionStartChar, view.state.doc.length);
      const head = Math.min(
        selectionEndChar > selectionStartChar ? selectionEndChar : cursorChar,
        view.state.doc.length
      );
      view.dispatch({
        selection: { anchor, head },
        effects: EditorView.scrollIntoView(head, { y: 'center' }),
      });
      suppressOutgoing = false;
    }
  });

  $effect(() => {
    // Reconfigure typography, column geometry, view-while-typing, and theme dynamically
    const _pf = proseFont;
    const _cf = codeFont;
    const _fs = fontSize;
    const _lh = lineHeight;
    const _cw = columnMaxWidth;
    const _cp = columnPaddingX;
    const _ro = readOnly;
    const _vwt = viewWhileTyping;
    const _tm = themeMode;
    if (view) {
      view.dispatch({
        effects: [
          themeCompartment.reconfigure(buildEditorTheme()),
          readOnlyCompartment.reconfigure(EditorState.readOnly.of(readOnly)),
          livePreviewCompartment.reconfigure(
            createLivePreviewPlugin(viewWhileTyping, themeMode)
          ),
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
    color: var(--fg-heading);
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

  :global(.cm-live-blockquote-line) {
    border-left: 3px solid var(--markup-quote-bar);
    padding-left: 14px !important;
    color: var(--markup-quote-fg);
    font-style: italic;
  }

  :global(.cm-live-codeblock-line) {
    background-color: var(--bg-code-block);
    font-family: var(--code-font-stack) !important;
    padding-left: 12px !important;
    padding-right: 12px !important;
    border-radius: 3px;
  }

  :global(.cm-live-fence-muted) {
    opacity: 0.42;
    font-size: 0.85em;
  }

  :global(.cm-live-mermaid-hidden-line) {
    height: 0 !important;
    min-height: 0 !important;
    padding: 0 !important;
    margin: 0 !important;
    font-size: 0 !important;
    line-height: 0 !important;
    opacity: 0 !important;
    overflow: clip;
  }

  :global(.cm-live-inline-code),
  :global(.cm-live-inline-code *) {
    font-family: var(--code-font-stack);
    color: var(--markup-code-fg) !important;
    background-color: var(--bg-code-inline);
    padding: 1px 5px;
    border-radius: 4px;
  }

  :global(.cm-live-bold),
  :global(.cm-live-bold *) {
    font-weight: 700;
    color: var(--markup-bold-fg) !important;
  }

  :global(.cm-live-italic),
  :global(.cm-live-italic *) {
    font-style: italic;
    color: var(--markup-emphasis-fg) !important;
  }

  :global(.cm-live-list-marker),
  :global(.cm-live-list-marker *) {
    color: var(--markup-list-marker) !important;
    font-weight: 700;
  }

  :global(.cm-live-strike) {
    text-decoration: line-through;
    color: var(--fg-muted);
  }

  :global(.cm-live-link) {
    color: var(--accent-link);
    text-decoration: underline;
    cursor: pointer;
  }

  :global(.cm-task-checkbox) {
    color: var(--markup-list-marker);
    font-weight: 700;
    cursor: pointer;
  }

  :global(.cm-task-checkbox.is-checked) {
    color: var(--markup-task-done);
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

  :global(.cm-mermaid-widget svg) {
    display: block;
    max-width: 100%;
    height: auto;
  }
</style>
