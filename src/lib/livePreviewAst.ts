import {
  EditorState,
  Facet,
  StateEffect,
  StateField,
  type Extension,
  type Range,
  type Text,
} from '@codemirror/state';
import {
  Decoration,
  EditorView,
  ViewPlugin,
  WidgetType,
  type DecorationSet,
  type ViewUpdate,
} from '@codemirror/view';
import { markdown, commonmarkLanguage } from '@codemirror/lang-markdown';
import {
  StreamLanguage,
  ensureSyntaxTree,
  syntaxTree,
  type Language,
  type LanguageSupport,
} from '@codemirror/language';
import { jsonLanguage } from '@codemirror/lang-json';
import { rustLanguage } from '@codemirror/lang-rust';
import { pythonLanguage } from '@codemirror/lang-python';
import { yamlLanguage, yamlFrontmatter } from '@codemirror/lang-yaml';
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
import type { SyntaxNode } from '@lezer/common';
import { Tag, styleTags, tags } from '@lezer/highlight';
import {
  GFM,
  type DelimiterType,
  type MarkdownConfig as LezerMarkdownConfig,
  type MarkdownExtension,
} from '@lezer/markdown';
import {
  appendSafeGfmTableCellDom,
  extractGfmTablesFromState,
  type ParsedGfmTable,
  type ParsedTableCell,
  type TableColumnAlign,
} from './gfmTable';
import { openExternalUrl, resolveLocalImageIpc } from './ipc';
import {
  getCachedMermaidSvg,
  getOrFetchMermaidSvg,
  renderLatexToHtml,
} from './mathAndMermaid';
import type { MarkdownConfig, ThemeMode } from './types';

const shellLanguage = StreamLanguage.define(shell);
const tomlLanguage = StreamLanguage.define(toml);

export const listMarkerTag = Tag.define();
export const inlineCodeTag = Tag.define();
export const codeBlockTextTag = Tag.define();

export const refreshLivePreviewEffect = StateEffect.define<boolean>();

export type GfmAlertKind = 'note' | 'tip' | 'important' | 'warning' | 'caution';

export interface ResolvedLinkRef {
  url: string;
  title: string;
}

export interface LivePreviewConfigState {
  enabled: boolean;
  theme: ThemeMode;
  markdownConfig: MarkdownConfig;
  currentFilePath: string | null;
}

export const DEFAULT_LIVE_PREVIEW_CONFIG: LivePreviewConfigState = {
  enabled: true,
  theme: 'Sepia',
  markdownConfig: {
    base: 'Gfm',
    extensions: {
      math: true,
      mermaid: true,
      alerts: true,
      footnotes: false,
      frontmatter: false,
    },
  },
  currentFilePath: null,
};

export const livePreviewConfigFacet = Facet.define<
  LivePreviewConfigState,
  LivePreviewConfigState
>({
  combine: (values) => values[0] ?? DEFAULT_LIVE_PREVIEW_CONFIG,
});

const ASCII_PUNCTUATION = new Set<string>(
  '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'.split('')
);
const PUNCTUATION_RE = /[\p{S}\p{P}]/u;

const NAMED_ENTITY_FALLBACKS: Record<string, string> = {
  amp: '&',
  lt: '<',
  gt: '>',
  quot: '"',
  apos: "'",
  nbsp: '\u00A0',
  copy: '\u00A9',
  reg: '\u00AE',
  trade: '\u2122',
  euro: '\u20AC',
  pound: '\u00A3',
  yen: '\u00A5',
  cent: '\u00A2',
  sect: '\u00A7',
  para: '\u00B6',
  middot: '\u00B7',
  bull: '\u2022',
  hellip: '\u2026',
  ndash: '\u2013',
  mdash: '\u2014',
  lsquo: '\u2018',
  rsquo: '\u2019',
  ldquo: '\u201C',
  rdquo: '\u201D',
  laquo: '\u00AB',
  raquo: '\u00BB',
  larr: '\u2190',
  uarr: '\u2191',
  rarr: '\u2192',
  darr: '\u2193',
  harr: '\u2194',
  times: '\u00D7',
  divide: '\u00F7',
  plusmn: '\u00B1',
  ne: '\u2260',
  le: '\u2264',
  ge: '\u2265',
  infin: '\u221E',
  deg: '\u00B0',
  alpha: '\u03B1',
  beta: '\u03B2',
  gamma: '\u03B3',
  delta: '\u03B4',
  lambda: '\u03BB',
  pi: '\u03C0',
  omega: '\u03C9',
  Alpha: '\u0391',
  Beta: '\u0392',
  Gamma: '\u0393',
  Delta: '\u0394',
  Lambda: '\u039B',
  Pi: '\u03A0',
  Omega: '\u03A9',
};

let entityTextareaEl: HTMLTextAreaElement | null = null;

export function resolveCodeLanguage(info: string): Language | null {
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

const SingleTildeStrikethroughDelim: DelimiterType = {
  resolve: 'Strikethrough',
  mark: 'StrikethroughMark',
};

const DoubleTildeStrikethroughDelim: DelimiterType = {
  resolve: 'Strikethrough',
  mark: 'StrikethroughMark',
};

/**
 * GFM `0.29-gfm` `§6.5` compliant strikethrough inline parser:
 * - Matches exact 1-tilde (`~text~`, Ex. 492) and exact 2-tilde (`~~text~~`, Ex. 491) pairs;
 * - Keeps 1-tilde and 2-tilde delimiter types distinct so `~foo~~` does not match;
 * - Strictly rejects runs of 3 or more tildes (`~~~text~~~`) without slicing inner tildes;
 * - Never enables Pandoc `Subscript` (`~...~`).
 */
const gfmStrikethroughOverride: LezerMarkdownConfig = {
  parseInline: [
    {
      name: 'Strikethrough',
      parse(cx, next, pos) {
        if (next !== 126 /* '~' */) {
          return -1;
        }
        // Check if preceded by an unescaped '~' (meaning we are inside a multi-tilde run >= 2)
        if (pos > cx.offset && cx.char(pos - 1) === 126) {
          let backslashes = 0;
          let scan = pos - 2;
          while (scan >= cx.offset && cx.char(scan) === 92 /* '\\' */) {
            backslashes++;
            scan--;
          }
          if (backslashes % 2 === 0) {
            return -1;
          }
        }

        let end = pos + 1;
        while (end < cx.end && cx.char(end) === 126) {
          end++;
        }
        const runLen = end - pos;
        if (runLen !== 1 && runLen !== 2) {
          return -1;
        }

        const before = pos > cx.offset ? cx.slice(pos - 1, pos) : '';
        const after = end < cx.end ? cx.slice(end, end + 1) : '';
        const sBefore = /\s|^$/.test(before);
        const sAfter = /\s|^$/.test(after);
        const pBefore = PUNCTUATION_RE.test(before);
        const pAfter = PUNCTUATION_RE.test(after);
        const canOpen = !sAfter && (!pAfter || sBefore || pBefore);
        const canClose = !sBefore && (!pBefore || sAfter || pAfter);
        const delimType =
          runLen === 1
            ? SingleTildeStrikethroughDelim
            : DoubleTildeStrikethroughDelim;
        return cx.addDelimiter(delimType, pos, end, canOpen, canClose);
      },
    },
  ],
};

const baseStyleExtension: MarkdownExtension = {
  props: [
    styleTags({
      'OrderedList/... BulletList/...': tags.content,
      ListMark: listMarkerTag,
      'CodeBlock/CodeText': codeBlockTextTag,
      'FencedCode/CodeText': tags.content,
      CodeInfo: tags.processingInstruction,
      InlineCode: inlineCodeTag,
      'InlineCode/...': inlineCodeTag,
    }),
  ],
};

export function buildMarkdownLanguageSupport(cfg: MarkdownConfig): LanguageSupport {
  if (cfg.base === 'CommonMark') {
    return markdown({
      base: commonmarkLanguage,
      codeLanguages: resolveCodeLanguage,
      extensions: [baseStyleExtension],
    });
  }

  const gfmSupport = markdown({
    base: commonmarkLanguage,
    codeLanguages: resolveCodeLanguage,
    extensions: [GFM, gfmStrikethroughOverride, baseStyleExtension],
  });

  return cfg.extensions.frontmatter
    ? yamlFrontmatter({ content: gfmSupport })
    : gfmSupport;
}

/**
 * Decodes CommonMark `§6.2` decimal (`&#35;`), hexadecimal (`&#x1F980;`), and HTML5 named
 * character references (`&copy;`, `&euro;`, `&Lambda;`) into Unicode characters.
 * Returns `null` if `raw` is not a recognized entity so unknown entities remain literal.
 */
export function decodeHtmlEntity(raw: string): string | null {
  const decMatch = raw.match(/^&#([0-9]{1,7});$/);
  if (decMatch) {
    const cp = Number.parseInt(decMatch[1], 10);
    return cp === 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)
      ? '\uFFFD'
      : String.fromCodePoint(cp);
  }

  const hexMatch = raw.match(/^&#[xX]([0-9a-fA-F]{1,6});$/);
  if (hexMatch) {
    const cp = Number.parseInt(hexMatch[1], 16);
    return cp === 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)
      ? '\uFFFD'
      : String.fromCodePoint(cp);
  }

  const namedMatch = raw.match(/^&([a-zA-Z][a-zA-Z0-9]{1,31});$/);
  if (!namedMatch) {
    return null;
  }

  const name = namedMatch[1];
  if (Object.prototype.hasOwnProperty.call(NAMED_ENTITY_FALLBACKS, name)) {
    return NAMED_ENTITY_FALLBACKS[name];
  }

  if (typeof document !== 'undefined') {
    entityTextareaEl ??= document.createElement('textarea');
    entityTextareaEl.innerHTML = raw;
    const decoded = entityTextareaEl.value;
    if (decoded !== raw) {
      return decoded;
    }
  }

  return null;
}

function unescapeAsciiBackslashes(raw: string): string {
  let out = '';
  for (let i = 0; i < raw.length; i++) {
    if (raw[i] === '\\' && i + 1 < raw.length && ASCII_PUNCTUATION.has(raw[i + 1])) {
      out += raw[i + 1];
      i++;
    } else {
      out += raw[i];
    }
  }
  return out;
}

export function normalizeLinkReferenceLabel(rawInsideBrackets: string): string {
  return rawInsideBrackets.replace(/[ \t\r\n]+/g, ' ').trim().toLowerCase();
}

export function sanitizeEditorLinkHref(rawHref: string): string | null {
  const cleaned = rawHref.trim().replace(/[\u0000-\u001F\u007F]/g, '');
  if (!cleaned || cleaned.startsWith('//')) {
    return null;
  }
  const schemeMatch = cleaned.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
  if (schemeMatch) {
    const scheme = schemeMatch[1].toLowerCase();
    if (
      scheme !== 'https' &&
      scheme !== 'http' &&
      scheme !== 'mailto' &&
      scheme !== 'xmpp'
    ) {
      return null;
    }
  }
  return cleaned;
}

const linkRefCache = new WeakMap<Text, Map<string, ResolvedLinkRef>>();

/**
 * Extracts all CommonMark `§4.7` `LinkReference` definitions (`[label]: url "title"`)
 * from `syntaxTree(state)` with first-definition-wins precedence.
 */
export function extractLinkReferenceMap(
  state: EditorState
): Map<string, ResolvedLinkRef> {
  const cached = linkRefCache.get(state.doc);
  if (cached) {
    return cached;
  }

  const tree = ensureSyntaxTree(state, state.doc.length, 50) ?? syntaxTree(state);
  const doc = state.doc;
  const map = new Map<string, ResolvedLinkRef>();

  tree.iterate({
    enter(nodeRef) {
      const name = nodeRef.name;
      if (
        name === 'Paragraph' ||
        name === 'FencedCode' ||
        name === 'CodeBlock' ||
        name === 'Table' ||
        name === 'HTMLBlock' ||
        name === 'CommentBlock' ||
        name === 'ProcessingInstructionBlock' ||
        name === 'ATXHeading1' ||
        name === 'ATXHeading2' ||
        name === 'ATXHeading3' ||
        name === 'ATXHeading4' ||
        name === 'ATXHeading5' ||
        name === 'ATXHeading6' ||
        name === 'SetextHeading1' ||
        name === 'SetextHeading2' ||
        name === 'Frontmatter'
      ) {
        return false;
      }

      if (name !== 'LinkReference') {
        return true;
      }

      let labelNode: SyntaxNode | null = null;
      let urlNode: SyntaxNode | null = null;
      let titleNode: SyntaxNode | null = null;

      for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
        if (ch.name === 'LinkLabel' && !labelNode) {
          labelNode = ch;
        } else if (ch.name === 'URL' && !urlNode) {
          urlNode = ch;
        } else if (ch.name === 'LinkTitle' && !titleNode) {
          titleNode = ch;
        }
      }

      if (!labelNode || !urlNode || labelNode.to - labelNode.from < 3) {
        return false;
      }

      const rawLabel = doc.sliceString(labelNode.from + 1, labelNode.to - 1);
      const normKey = normalizeLinkReferenceLabel(rawLabel);
      if (!normKey || map.has(normKey)) {
        return false;
      }

      let rawUrl = doc.sliceString(urlNode.from, urlNode.to).trim();
      if (rawUrl.startsWith('<') && rawUrl.endsWith('>') && rawUrl.length >= 2) {
        rawUrl = rawUrl.slice(1, -1);
      }
      const unescapedUrl = unescapeAsciiBackslashes(rawUrl);
      const safeHref = sanitizeEditorLinkHref(unescapedUrl);
      if (!safeHref) {
        return false;
      }

      let title = '';
      if (titleNode && titleNode.to - titleNode.from >= 2) {
        title = unescapeAsciiBackslashes(
          doc.sliceString(titleNode.from + 1, titleNode.to - 1)
        );
      }

      map.set(normKey, { url: safeHref, title });
      return false;
    },
  });

  linkRefCache.set(state.doc, map);
  return map;
}

export type ResolvedAstLink =
  | {
      kind: 'link';
      textFrom: number;
      textTo: number;
      closeBracketFrom: number;
      href: string;
      title: string;
    }
  | {
      kind: 'footnote-ref';
      id: string;
    }
  | {
      kind: 'unresolved-brackets';
    };

/**
 * Classifies a Lezer `Link` AST node into:
 * - Valid inline link `[text](url "title")` (including balanced parentheses and `<...>` URLs);
 * - Resolved reference link (`[text][ref]`, `[ref][]`, or shortcut `[ref]`);
 * - Modular footnote reference `[^id]` (only when `footnotesEnabled` is true and not followed by `:`);
 * - Plain literal bracketed text `[...]` (when shortcut `[...]` has no matching `LinkReference`).
 */
export function classifyAstLinkNode(
  doc: Text,
  linkNode: SyntaxNode,
  refMap: Map<string, ResolvedLinkRef>,
  footnotesEnabled: boolean
): ResolvedAstLink {
  const firstChild = linkNode.firstChild;
  const lastChild = linkNode.lastChild;
  if (!firstChild || !lastChild || firstChild.name !== 'LinkMark') {
    return { kind: 'unresolved-brackets' };
  }

  // Locate the closing `]` of the link's visible label `[label]`
  let closeBracketNode: SyntaxNode | null = null;
  for (let ch = firstChild.nextSibling; ch; ch = ch.nextSibling) {
    if (
      ch.name === 'LinkMark' &&
      doc.sliceString(ch.from, ch.to) === ']'
    ) {
      closeBracketNode = ch;
      break;
    }
  }
  if (!closeBracketNode) {
    return { kind: 'unresolved-brackets' };
  }

  const textFrom = firstChild.to;
  const textTo = closeBracketNode.from;
  const closeBracketFrom = closeBracketNode.from;

  // 1. Inline link: `[label](dest "title")` -> ends with `LinkMark` `)` after `closeBracketNode`
  if (
    lastChild.from !== closeBracketNode.from &&
    lastChild.name === 'LinkMark' &&
    doc.sliceString(lastChild.from, lastChild.to) === ')'
  ) {
    let urlNode: SyntaxNode | null = null;
    let titleNode: SyntaxNode | null = null;
    for (let ch = closeBracketNode.nextSibling; ch; ch = ch.nextSibling) {
      if (ch.name === 'URL' && !urlNode) {
        urlNode = ch;
      } else if (ch.name === 'LinkTitle' && !titleNode) {
        titleNode = ch;
      }
    }

    let rawDest = '';
    if (urlNode) {
      rawDest = doc.sliceString(urlNode.from, urlNode.to).trim();
      if (rawDest.startsWith('<') && rawDest.endsWith('>') && rawDest.length >= 2) {
        rawDest = rawDest.slice(1, -1);
      }
      rawDest = unescapeAsciiBackslashes(rawDest);
    }

    const safeHref = rawDest ? sanitizeEditorLinkHref(rawDest) : '#';
    if (!safeHref) {
      return { kind: 'unresolved-brackets' };
    }

    const title =
      titleNode && titleNode.to - titleNode.from >= 2
        ? unescapeAsciiBackslashes(
            doc.sliceString(titleNode.from + 1, titleNode.to - 1)
          )
        : '';

    return {
      kind: 'link',
      textFrom,
      textTo,
      closeBracketFrom,
      href: safeHref,
      title,
    };
  }

  // 2. Full (`[text][ref]`) or collapsed (`[ref][]`) reference link
  if (lastChild.name === 'LinkLabel' && lastChild.from >= closeBracketNode.to) {
    const rawRefLabel = doc.sliceString(lastChild.from + 1, lastChild.to - 1);
    const lookupRaw =
      rawRefLabel.length === 0
        ? doc.sliceString(textFrom, textTo)
        : rawRefLabel;
    const resolved = refMap.get(normalizeLinkReferenceLabel(lookupRaw));
    if (!resolved) {
      return { kind: 'unresolved-brackets' };
    }
    return {
      kind: 'link',
      textFrom,
      textTo,
      closeBracketFrom,
      href: resolved.url,
      title: resolved.title,
    };
  }

  // 3. Shortcut reference link `[ref]` OR footnote reference `[^id]`
  if (
    lastChild.from === closeBracketNode.from &&
    lastChild.to === closeBracketNode.to
  ) {
    const rawInside = doc.sliceString(textFrom, textTo);

    if (footnotesEnabled && /^\^[^\s\[\]]+$/.test(rawInside)) {
      const nextChar = doc.sliceString(linkNode.to, linkNode.to + 1);
      if (nextChar !== ':') {
        return {
          kind: 'footnote-ref',
          id: rawInside.slice(1),
        };
      }
    }

    const resolved = refMap.get(normalizeLinkReferenceLabel(rawInside));
    if (resolved) {
      return {
        kind: 'link',
        textFrom,
        textTo,
        closeBracketFrom,
        href: resolved.url,
        title: resolved.title,
      };
    }
  }

  return { kind: 'unresolved-brackets' };
}

const ALLOWED_EDITOR_IMAGE_EXTENSIONS = new Set<string>([
  'png',
  'jpg',
  'jpeg',
  'gif',
  'webp',
  'svg',
  'bmp',
  'ico',
  'avif',
]);

export type CachedLocalImageEntry =
  | { status: 'ok'; src: string }
  | { status: 'missing' };

const localImageResolutionCache = new Map<string, CachedLocalImageEntry>();
const pendingLocalImageRequests = new Map<string, Promise<void>>();

function isTauriRuntimeAvailable(): boolean {
  return (
    typeof window !== 'undefined' &&
    '__TAURI_INTERNALS__' in (window as unknown as Record<string, unknown>)
  );
}

function tryPercentDecode(raw: string): string {
  try {
    return decodeURIComponent(raw);
  } catch {
    return raw;
  }
}

export function extractParentDirectory(filePath: string | null): string | null {
  if (!filePath) return null;
  const trimmed = filePath.trim();
  if (!trimmed) return null;
  const lastSlash = Math.max(trimmed.lastIndexOf('/'), trimmed.lastIndexOf('\\'));
  if (lastSlash < 0) return null;
  if (lastSlash === 0) return '/';
  if (lastSlash === 2 && /^[a-zA-Z]:/.test(trimmed)) {
    return trimmed.slice(0, 3);
  }
  return trimmed.slice(0, lastSlash);
}

function isSafeInlineRasterDataUri(rawUrl: string): boolean {
  const lower = rawUrl.trim().toLowerCase();
  return (
    lower.startsWith('data:image/png;base64,') ||
    lower.startsWith('data:image/jpeg;base64,') ||
    lower.startsWith('data:image/gif;base64,') ||
    lower.startsWith('data:image/webp;base64,') ||
    lower.startsWith('data:image/bmp;base64,') ||
    lower.startsWith('data:image/x-icon;base64,') ||
    lower.startsWith('data:image/avif;base64,')
  );
}

/**
 * Resolves a Markdown `§6.7` local relative image path (`diagram.png`, `./assets/img.jpg`,
 * `../img/logo.png`, including percent-encoding and spaces) against `currentFilePath`'s directory.
 * Returns `null` if `currentFilePath` is null, `rawUrl` is remote/non-relative, or the extension is not allowed.
 */
export function resolveRelativeImagePathForEditor(
  currentFilePath: string | null,
  rawUrl: string
): string | null {
  const parentDir = extractParentDirectory(currentFilePath);
  if (!parentDir) return null;

  let trimmed = rawUrl.trim();
  if (trimmed.startsWith('<') && trimmed.endsWith('>') && trimmed.length >= 2) {
    trimmed = trimmed.slice(1, -1).trim();
  }
  if (
    !trimmed ||
    trimmed.startsWith('#') ||
    trimmed.startsWith('/') ||
    trimmed.startsWith('\\') ||
    /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmed) ||
    /[\u0000-\u001F\u007F]/.test(trimmed)
  ) {
    return null;
  }

  const withoutHash = trimmed.split('#')[0] ?? trimmed;
  const cleanRel = (withoutHash.split('?')[0] ?? withoutHash).trim();
  if (!cleanRel) return null;

  const decoded = tryPercentDecode(cleanRel);
  if (
    !decoded ||
    decoded.startsWith('/') ||
    decoded.startsWith('\\') ||
    /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(decoded) ||
    /[\u0000-\u001F\u007F]/.test(decoded)
  ) {
    return null;
  }

  const dotIdx = decoded.lastIndexOf('.');
  const slashIdx = Math.max(decoded.lastIndexOf('/'), decoded.lastIndexOf('\\'));
  if (dotIdx <= slashIdx || dotIdx === decoded.length - 1) {
    return null;
  }
  const ext = decoded.slice(dotIdx + 1).toLowerCase();
  if (!ALLOWED_EDITOR_IMAGE_EXTENSIONS.has(ext)) {
    return null;
  }

  const isWindows = /^[a-zA-Z]:/.test(parentDir) || parentDir.includes('\\');
  const sep = isWindows ? '\\' : '/';
  const baseParts = parentDir.split(/[\\/]+/);
  const relParts = decoded.split(/[\\/]+/);
  const stack: string[] = [];

  for (let i = 0; i < baseParts.length; i++) {
    const seg = baseParts[i];
    if (i === 0 && (seg === '' || /^[a-zA-Z]:$/.test(seg))) {
      stack.push(seg);
      continue;
    }
    if (!seg || seg === '.') continue;
    stack.push(seg);
  }

  const rootFloor = stack.length > 0 && (stack[0] === '' || /^[a-zA-Z]:$/.test(stack[0])) ? 1 : 0;
  let netUp = 0;
  let depthBelow = 0;
  for (const seg of relParts) {
    if (!seg || seg === '.') continue;
    if (seg === '..') {
      if (depthBelow > 0) {
        depthBelow--;
      } else {
        netUp++;
        if (netUp > 2) {
          return null;
        }
      }
      if (stack.length > rootFloor) {
        stack.pop();
      } else {
        return null;
      }
    } else {
      if (seg.startsWith('.')) {
        return null;
      }
      depthBelow++;
      stack.push(seg);
    }
  }

  if (stack.length === 1 && stack[0] === '') {
    return '/';
  }
  return stack.join(sep);
}

function makeLocalImageCacheKey(currentFilePath: string, rawUrl: string): string {
  const normalized = resolveRelativeImagePathForEditor(currentFilePath, rawUrl);
  return normalized ?? `${currentFilePath}\0${rawUrl.trim()}`;
}

export function primeLocalImageResolutionCache(
  currentFilePath: string,
  rawUrl: string,
  entry: CachedLocalImageEntry
): void {
  localImageResolutionCache.set(makeLocalImageCacheKey(currentFilePath, rawUrl), entry);
}

export function clearLocalImageResolutionCache(): void {
  localImageResolutionCache.clear();
  pendingLocalImageRequests.clear();
}

export function lookupEditorImageResolution(
  currentFilePath: string | null,
  rawUrl: string
): CachedLocalImageEntry | 'pending' {
  if (isSafeInlineRasterDataUri(rawUrl)) {
    return { status: 'ok', src: rawUrl.trim() };
  }
  const normalized = resolveRelativeImagePathForEditor(currentFilePath, rawUrl);
  if (!normalized || !currentFilePath) {
    return { status: 'missing' };
  }
  const key = makeLocalImageCacheKey(currentFilePath, rawUrl);
  const cached = localImageResolutionCache.get(key);
  if (cached) {
    return cached;
  }
  if (!isTauriRuntimeAvailable()) {
    return { status: 'missing' };
  }
  return 'pending';
}

function triggerLocalImageIpcResolution(
  currentFilePath: string,
  rawUrl: string,
  onResolved: () => void
): void {
  const key = makeLocalImageCacheKey(currentFilePath, rawUrl);
  if (localImageResolutionCache.has(key)) {
    onResolved();
    return;
  }
  const existing = pendingLocalImageRequests.get(key);
  if (existing) {
    void existing.then(onResolved);
    return;
  }
  const req = resolveLocalImageIpc(currentFilePath, rawUrl)
    .then((payload) => {
      if (payload.ok && payload.src) {
        localImageResolutionCache.set(key, { status: 'ok', src: payload.src });
      } else {
        localImageResolutionCache.set(key, { status: 'missing' });
      }
    })
    .catch(() => {
      localImageResolutionCache.set(key, { status: 'missing' });
    })
    .finally(() => {
      pendingLocalImageRequests.delete(key);
      onResolved();
    });
  pendingLocalImageRequests.set(key, req);
}

function extractVisibleAltFromRange(doc: Text, from: number, to: number): string {
  if (to <= from) return '';
  const raw = doc.sliceString(from, to);
  let out = '';
  for (let i = 0; i < raw.length; i++) {
    if (raw[i] === '\\' && i + 1 < raw.length && ASCII_PUNCTUATION.has(raw[i + 1])) {
      out += raw[i + 1];
      i++;
      continue;
    }
    if (raw[i] === '&') {
      const semi = raw.indexOf(';', i + 1);
      if (semi !== -1 && semi - i <= 33) {
        const candidate = raw.slice(i, semi + 1);
        const dec = decodeHtmlEntity(candidate);
        if (dec !== null) {
          out += dec;
          i = semi;
          continue;
        }
      }
    }
    if (raw[i] === '*' || raw[i] === '_' || raw[i] === '`' || raw[i] === '~') {
      continue;
    }
    out += raw[i];
  }
  return out.replace(/\s+/g, ' ').trim();
}

export type ClassifiedAstImage =
  | {
      kind: 'image';
      alt: string;
      rawUrl: string;
      title: string;
      isStandalone: boolean;
    }
  | {
      kind: 'unresolved-image';
    };

export function classifyAstImageNode(
  doc: Text,
  imageNode: SyntaxNode,
  refMap: Map<string, ResolvedLinkRef>
): ClassifiedAstImage {
  const firstChild = imageNode.firstChild;
  const lastChild = imageNode.lastChild;
  if (
    !firstChild ||
    !lastChild ||
    firstChild.name !== 'LinkMark' ||
    doc.sliceString(firstChild.from, firstChild.to) !== '!['
  ) {
    return { kind: 'unresolved-image' };
  }

  let closeBracketNode: SyntaxNode | null = null;
  for (let ch = firstChild.nextSibling; ch; ch = ch.nextSibling) {
    if (ch.name === 'LinkMark' && doc.sliceString(ch.from, ch.to) === ']') {
      closeBracketNode = ch;
      break;
    }
  }
  if (!closeBracketNode) {
    return { kind: 'unresolved-image' };
  }

  const alt = extractVisibleAltFromRange(doc, firstChild.to, closeBracketNode.from);

  let isStandalone = false;
  const parent = imageNode.parent;
  if (parent && parent.name === 'Paragraph') {
    let onlyQuoteMarksAndImage = true;
    let ch: SyntaxNode | null = parent.firstChild;
    while (ch !== null) {
      if (ch.from !== imageNode.from || ch.to !== imageNode.to) {
        if (ch.name !== 'QuoteMark') {
          onlyQuoteMarksAndImage = false;
          break;
        }
      }
      ch = ch.nextSibling;
    }
    if (onlyQuoteMarksAndImage) {
      const before = doc.sliceString(parent.from, imageNode.from).replace(/>/g, '').trim();
      const after = doc.sliceString(imageNode.to, parent.to).trim();
      isStandalone = before.length === 0 && after.length === 0;
    }
  }

  // 1. Inline image `![alt](url "title")`
  if (
    lastChild.from !== closeBracketNode.from &&
    lastChild.name === 'LinkMark' &&
    doc.sliceString(lastChild.from, lastChild.to) === ')'
  ) {
    let urlNode: SyntaxNode | null = null;
    let titleNode: SyntaxNode | null = null;
    for (let ch = closeBracketNode.nextSibling; ch; ch = ch.nextSibling) {
      if (ch.name === 'URL' && !urlNode) {
        urlNode = ch;
      } else if (ch.name === 'LinkTitle' && !titleNode) {
        titleNode = ch;
      }
    }

    let rawUrl = '';
    if (urlNode) {
      rawUrl = doc.sliceString(urlNode.from, urlNode.to).trim();
      if (rawUrl.startsWith('<') && rawUrl.endsWith('>') && rawUrl.length >= 2) {
        rawUrl = rawUrl.slice(1, -1).trim();
      }
      rawUrl = unescapeAsciiBackslashes(rawUrl);
    }

    const title =
      titleNode && titleNode.to - titleNode.from >= 2
        ? unescapeAsciiBackslashes(
            doc.sliceString(titleNode.from + 1, titleNode.to - 1)
          )
        : '';

    return {
      kind: 'image',
      alt,
      rawUrl,
      title,
      isStandalone,
    };
  }

  // 2. Full `![alt][ref]` or collapsed `![ref][]` reference image
  if (lastChild.name === 'LinkLabel' && lastChild.from >= closeBracketNode.to) {
    const rawRefLabel = doc.sliceString(lastChild.from + 1, lastChild.to - 1);
    const lookupRaw =
      rawRefLabel.length === 0
        ? doc.sliceString(firstChild.to, closeBracketNode.from)
        : rawRefLabel;
    const resolved = refMap.get(normalizeLinkReferenceLabel(lookupRaw));
    if (!resolved) {
      return { kind: 'unresolved-image' };
    }
    return {
      kind: 'image',
      alt,
      rawUrl: resolved.url,
      title: resolved.title,
      isStandalone,
    };
  }

  // 3. Shortcut reference image `![ref]`
  if (
    lastChild.from === closeBracketNode.from &&
    lastChild.to === closeBracketNode.to
  ) {
    const rawInside = doc.sliceString(firstChild.to, closeBracketNode.from);
    const resolved = refMap.get(normalizeLinkReferenceLabel(rawInside));
    if (resolved) {
      return {
        kind: 'image',
        alt,
        rawUrl: resolved.url,
        title: resolved.title,
        isStandalone,
      };
    }
  }

  return { kind: 'unresolved-image' };
}

export function slugifyHeadingText(raw: string): string {
  return raw
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .replace(/\s+/g, '-');
}

function extractHeadingVisibleText(doc: Text, headingNode: SyntaxNode): string {
  const isSetext =
    headingNode.name === 'SetextHeading1' || headingNode.name === 'SetextHeading2';
  const endLimit = isSetext
    ? doc.line(doc.lineAt(headingNode.to).number - 1).to
    : headingNode.to;

  const excluded: Array<[number, number]> = [];
  let cursor = headingNode.cursor();
  if (cursor.firstChild()) {
    do {
      const walk = () => {
        const n = cursor.name;
        if (
          n === 'HeaderMark' ||
          n === 'QuoteMark' ||
          n === 'EmphasisMark' ||
          n === 'StrikethroughMark' ||
          n === 'CodeMark'
        ) {
          excluded.push([cursor.from, cursor.to]);
          return;
        }
        if (n === 'Escape') {
          excluded.push([cursor.from, cursor.from + 1]);
          return;
        }
        if (cursor.firstChild()) {
          do {
            walk();
          } while (cursor.nextSibling());
          cursor.parent();
        }
      };
      walk();
    } while (cursor.nextSibling());
  }

  excluded.sort((a, b) => a[0] - b[0]);
  let out = '';
  let pos = headingNode.from;
  for (const [exFrom, exTo] of excluded) {
    if (exFrom >= endLimit) break;
    if (exFrom > pos) {
      out += doc.sliceString(pos, Math.min(exFrom, endLimit));
    }
    pos = Math.max(pos, Math.min(exTo, endLimit));
  }
  if (pos < endLimit) {
    out += doc.sliceString(pos, endLimit);
  }
  return out.trim();
}

/**
 * Finds the document UTF-16 offset of the heading matching `#anchor` (`ATXHeading1..6` or `SetextHeading1..2`),
 * supporting duplicate-heading suffixes (`-1`, `-2`) per GFM / `comrak` anchor rules.
 */
export function findHeadingAnchorPos(
  state: EditorState,
  rawAnchor: string
): number | null {
  const cleaned = decodeURIComponent(rawAnchor.replace(/^#/, '')).trim().toLowerCase();
  if (!cleaned) return null;

  const tree = ensureSyntaxTree(state, state.doc.length, 50) ?? syntaxTree(state);
  const slugCounts = new Map<string, number>();
  let foundPos: number | null = null;

  tree.iterate({
    enter(nodeRef) {
      if (foundPos !== null) return false;
      if (!/^(?:ATXHeading[1-6]|SetextHeading[12])$/.test(nodeRef.name)) {
        return true;
      }
      const visible = extractHeadingVisibleText(state.doc, nodeRef.node);
      const baseSlug = slugifyHeadingText(visible);
      if (!baseSlug) return false;

      const seen = slugCounts.get(baseSlug) ?? 0;
      slugCounts.set(baseSlug, seen + 1);
      const effectiveSlug = seen === 0 ? baseSlug : `${baseSlug}-${seen}`;

      if (effectiveSlug === cleaned || (seen === 0 && baseSlug === cleaned)) {
        foundPos = state.doc.lineAt(nodeRef.from).from;
      }
      return false;
    },
  });

  return foundPos;
}

function normalizeBareOrAngleAutolinkHref(rawUrl: string): string | null {
  const trimmed = rawUrl.trim();
  if (!trimmed) return null;
  if (trimmed.startsWith('www.')) {
    return sanitizeEditorLinkHref(`http://${trimmed}`);
  }
  if (
    trimmed.includes('@') &&
    !/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmed)
  ) {
    return sanitizeEditorLinkHref(`mailto:${trimmed}`);
  }
  return sanitizeEditorLinkHref(trimmed);
}

/**
 * Resolves the clickable link `href` at UTF-16 `pos` in `state` using `syntaxTree(state)`
 * (`Link` inline/reference, `Autolink` `<...>`, or GFM `§6.9` bare `URL`).
 */
export function resolveLinkHrefAtPos(
  state: EditorState,
  pos: number,
  cfg: MarkdownConfig
): string | null {
  const tree = ensureSyntaxTree(state, state.doc.length, 50) ?? syntaxTree(state);
  const isGfm = cfg.base === 'Gfm';
  const footnotesEnabled = isGfm && cfg.extensions.footnotes;

  for (const side of [1, -1] as const) {
    let cur: SyntaxNode | null = tree.resolveInner(pos, side);
    while (cur) {
      if (cur.name === 'Autolink') {
        const inner = state.doc.sliceString(cur.from + 1, cur.to - 1);
        return normalizeBareOrAngleAutolinkHref(inner);
      }
      if (cur.name === 'Link') {
        const refMap = extractLinkReferenceMap(state);
        const classified = classifyAstLinkNode(
          state.doc,
          cur,
          refMap,
          footnotesEnabled
        );
        if (classified.kind === 'link') {
          return classified.href;
        }
        return null;
      }
      if (
        isGfm &&
        cur.name === 'URL' &&
        cur.parent &&
        cur.parent.name !== 'Link' &&
        cur.parent.name !== 'Image' &&
        cur.parent.name !== 'Autolink' &&
        cur.parent.name !== 'LinkReference'
      ) {
        return normalizeBareOrAngleAutolinkHref(
          state.doc.sliceString(cur.from, cur.to)
        );
      }
      cur = cur.parent;
    }
  }
  return null;
}

export function activateEditorLinkHref(view: EditorView, href: string): boolean {
  const trimmed = href.trim();
  if (!trimmed) return false;

  if (trimmed.startsWith('#')) {
    const targetPos = findHeadingAnchorPos(view.state, trimmed);
    if (targetPos !== null) {
      view.dispatch({
        selection: { anchor: targetPos, head: targetPos },
        effects: EditorView.scrollIntoView(targetPos, { y: 'start', yMargin: 84 }),
      });
      view.focus();
      return true;
    }
    return false;
  }

  if (trimmed.startsWith('https://') || trimmed.startsWith('http://')) {
    void openExternalUrl(trimmed).catch((err) => {
      console.error('openExternalUrl failed:', err);
    });
    return true;
  }

  const withoutHash = (trimmed.split('#')[0] ?? trimmed).split('?')[0] ?? trimmed;
  if (
    !trimmed.includes('://') &&
    !trimmed.startsWith('javascript:') &&
    !trimmed.startsWith('data:') &&
    /\.(md|markdown|mdown|mkd|txt)$/i.test(withoutHash)
  ) {
    view.dom.dispatchEvent(
      new CustomEvent('accent:open-relative-link', {
        bubbles: true,
        detail: { href: trimmed },
      })
    );
    return true;
  }

  return false;
}

/**
 * Performs a real transactional toggle of a GFM `§5.3` `TaskMarker` (`[ ]` <-> `[x]`, `[X]` -> `[ ]`)
 * on `view`, preserving undo history (`Ctrl+Z`) and propagating changes to Rust Session / WAL / Preview
 * without forcing the caret onto the task line.
 */
export function toggleTaskCheckboxInView(
  view: EditorView,
  markerFrom: number,
  markerTo: number
): boolean {
  if (view.state.readOnly) {
    return false;
  }
  if (markerFrom < 0 || markerTo > view.state.doc.length || markerTo - markerFrom !== 3) {
    return false;
  }
  const currentSlice = view.state.doc.sliceString(markerFrom, markerTo);
  if (!/^\[[ xX]\]$/.test(currentSlice)) {
    return false;
  }
  const isChecked = currentSlice[1].toLowerCase() === 'x';
  const nextInsert = isChecked ? '[ ]' : '[x]';
  view.dispatch({
    changes: { from: markerFrom, to: markerTo, insert: nextInsert },
    userEvent: 'input.toggle-task',
  });
  return true;
}

export class KatexMathWidget extends WidgetType {
  latex: string;
  displayMode: boolean;
  targetPos: number;
  blockquoteDepth: number;

  constructor(
    latex: string,
    displayMode: boolean,
    targetPos: number,
    blockquoteDepth = 0
  ) {
    super();
    this.latex = latex;
    this.displayMode = displayMode;
    this.targetPos = targetPos;
    this.blockquoteDepth = blockquoteDepth;
  }

  eq(other: KatexMathWidget): boolean {
    return (
      this.latex === other.latex &&
      this.displayMode === other.displayMode &&
      this.targetPos === other.targetPos &&
      this.blockquoteDepth === other.blockquoteDepth
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const el: HTMLElement = document.createElement(this.displayMode ? 'div' : 'span');
    if (this.displayMode) {
      el.className =
        this.blockquoteDepth > 0
          ? 'cm-katex-display-widget is-in-blockquote'
          : 'cm-katex-display-widget';
    } else {
      el.className = 'cm-katex-inline-widget';
    }
    el.dataset.testid = this.displayMode ? 'cm-katex-display' : 'cm-katex-inline';
    el.innerHTML = renderLatexToHtml(this.latex, this.displayMode);
    el.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });
    return el;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

export class MermaidSvgWidget extends WidgetType {
  code: string;
  theme: ThemeMode;
  targetPos: number;
  svgHtml: string | null;
  blockquoteDepth: number;

  constructor(
    code: string,
    theme: ThemeMode,
    targetPos: number,
    svgHtml: string | null,
    blockquoteDepth = 0
  ) {
    super();
    this.code = code;
    this.theme = theme;
    this.targetPos = targetPos;
    this.svgHtml = svgHtml;
    this.blockquoteDepth = blockquoteDepth;
  }

  eq(other: MermaidSvgWidget): boolean {
    return (
      this.code === other.code &&
      this.theme === other.theme &&
      this.targetPos === other.targetPos &&
      this.svgHtml === other.svgHtml &&
      this.blockquoteDepth === other.blockquoteDepth
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className =
      this.blockquoteDepth > 0
        ? 'cm-mermaid-widget is-in-blockquote'
        : 'cm-mermaid-widget';
    wrap.dataset.testid = 'cm-mermaid-widget';
    wrap.title = 'Clicca per modificare il diagramma Mermaid';

    const cachedSvg =
      this.svgHtml ??
      getOrFetchMermaidSvg(this.code, this.theme, () => {
        queueMicrotask(() => {
          view.dispatch({ effects: refreshLivePreviewEffect.of(true) });
        });
      });

    if (cachedSvg) {
      wrap.innerHTML = cachedSvg;
    } else {
      wrap.textContent = 'Rendering Mermaid diagram...';
    }

    wrap.addEventListener('mousedown', (e) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });
    return wrap;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

export class TaskCheckboxWidget extends WidgetType {
  checked: boolean;
  markerFrom: number;
  markerTo: number;

  constructor(checked: boolean, markerFrom: number, markerTo: number) {
    super();
    this.checked = checked;
    this.markerFrom = markerFrom;
    this.markerTo = markerTo;
  }

  eq(other: TaskCheckboxWidget): boolean {
    return (
      this.checked === other.checked &&
      this.markerFrom === other.markerFrom &&
      this.markerTo === other.markerTo
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const span = document.createElement('span');
    span.className = this.checked
      ? 'cm-task-checkbox is-checked'
      : 'cm-task-checkbox';
    span.dataset.testid = 'cm-task-checkbox';
    span.dataset.checked = this.checked ? 'true' : 'false';
    span.setAttribute('role', 'checkbox');
    span.setAttribute('aria-checked', this.checked ? 'true' : 'false');
    span.textContent = this.checked ? '☑' : '☐';

    span.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      toggleTaskCheckboxInView(view, this.markerFrom, this.markerTo);
      view.focus();
    });

    span.addEventListener('click', (e: MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
    });

    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

export class ThematicBreakWidget extends WidgetType {
  targetPos: number;

  constructor(targetPos: number) {
    super();
    this.targetPos = targetPos;
  }

  eq(other: ThematicBreakWidget): boolean {
    return this.targetPos === other.targetPos;
  }

  toDOM(view: EditorView): HTMLElement {
    const span = document.createElement('span');
    span.className = 'cm-thematic-break-widget';
    span.dataset.testid = 'cm-thematic-break';
    span.setAttribute('role', 'separator');
    span.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });
    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

export class DecodedEntityWidget extends WidgetType {
  decoded: string;
  targetPos: number;

  constructor(decoded: string, targetPos: number) {
    super();
    this.decoded = decoded;
    this.targetPos = targetPos;
  }

  eq(other: DecodedEntityWidget): boolean {
    return this.decoded === other.decoded && this.targetPos === other.targetPos;
  }

  toDOM(view: EditorView): HTMLElement {
    const span = document.createElement('span');
    span.className = 'cm-entity-widget';
    span.dataset.testid = 'cm-entity-widget';
    span.appendChild(document.createTextNode(this.decoded));
    span.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });
    return span;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

function populateImageFallbackBadgeDom(
  host: HTMLElement,
  alt: string,
  rawUrl: string,
  title: string
): void {
  const displayLabel = alt || rawUrl || 'Immagine';
  const tooltip = title
    ? rawUrl
      ? `${title} (${rawUrl})`
      : title
    : rawUrl || displayLabel;

  host.classList.remove('is-loaded');
  host.classList.add('is-fallback', 'cm-image-fallback-badge');
  host.dataset.testid = 'cm-image-fallback';
  host.setAttribute('role', 'img');
  host.setAttribute('aria-label', displayLabel);
  host.title = tooltip;

  while (host.firstChild) {
    host.removeChild(host.firstChild);
  }

  const iconEl = document.createElement('span');
  iconEl.className = 'cm-image-fallback-icon';
  iconEl.setAttribute('aria-hidden', 'true');
  iconEl.appendChild(document.createTextNode('🖼'));

  const textEl = document.createElement('span');
  textEl.className = 'cm-image-fallback-alt';
  textEl.appendChild(document.createTextNode(displayLabel));

  host.appendChild(iconEl);
  host.appendChild(textEl);
}

/**
 * Caret-aware CommonMark/GFM `§6.7` Image widget (`![alt](url "title")`):
 * - Renders a responsive themed `<img>` when `rawUrl` resolves against `currentFilePath`;
 * - Renders a compact fallback badge (`🖼 alt`) without broken browser icons when missing, unsaved, or remote (`https://...`);
 * - Places the caret at `targetPos` on click to immediately re-expand the Markdown source.
 */
export class ImageWidget extends WidgetType {
  alt: string;
  rawUrl: string;
  title: string;
  targetPos: number;
  currentFilePath: string | null;
  isStandalone: boolean;
  resolvedSrc: string | null;
  pendingIpc: boolean;

  constructor(
    alt: string,
    rawUrl: string,
    title: string,
    targetPos: number,
    currentFilePath: string | null,
    isStandalone: boolean
  ) {
    super();
    this.alt = alt;
    this.rawUrl = rawUrl;
    this.title = title;
    this.targetPos = targetPos;
    this.currentFilePath = currentFilePath;
    this.isStandalone = isStandalone;

    const lookup = lookupEditorImageResolution(currentFilePath, rawUrl);
    if (lookup === 'pending') {
      this.resolvedSrc = null;
      this.pendingIpc = true;
    } else if (lookup.status === 'ok') {
      this.resolvedSrc = lookup.src;
      this.pendingIpc = false;
    } else {
      this.resolvedSrc = null;
      this.pendingIpc = false;
    }
  }

  eq(other: ImageWidget): boolean {
    return (
      this.alt === other.alt &&
      this.rawUrl === other.rawUrl &&
      this.title === other.title &&
      this.targetPos === other.targetPos &&
      this.currentFilePath === other.currentFilePath &&
      this.isStandalone === other.isStandalone &&
      this.resolvedSrc === other.resolvedSrc &&
      this.pendingIpc === other.pendingIpc
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const wrap = document.createElement('span');
    wrap.className = this.isStandalone
      ? 'cm-image-widget is-standalone'
      : 'cm-image-widget is-inline';

    if (this.pendingIpc && this.currentFilePath) {
      populateImageFallbackBadgeDom(wrap, this.alt, this.rawUrl, this.title);
      triggerLocalImageIpcResolution(this.currentFilePath, this.rawUrl, () => {
        queueMicrotask(() => {
          view.dispatch({ effects: refreshLivePreviewEffect.of(true) });
        });
      });
    } else if (this.resolvedSrc) {
      wrap.classList.add('is-loaded');
      wrap.dataset.testid = 'cm-image-widget';
      if (this.title) {
        wrap.title = this.title;
      }

      const img = document.createElement('img');
      img.className = 'cm-image-widget-img';
      img.src = this.resolvedSrc;
      img.alt = this.alt;
      if (this.title) {
        img.title = this.title;
      }
      img.decoding = 'async';
      img.draggable = false;
      img.addEventListener('error', () => {
        if (this.currentFilePath) {
          primeLocalImageResolutionCache(this.currentFilePath, this.rawUrl, {
            status: 'missing',
          });
        }
        populateImageFallbackBadgeDom(wrap, this.alt, this.rawUrl, this.title);
      });
      wrap.appendChild(img);
    } else {
      populateImageFallbackBadgeDom(wrap, this.alt, this.rawUrl, this.title);
    }

    wrap.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });

    return wrap;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

const ALERT_LABELS: Record<GfmAlertKind, { icon: string; label: string }> = {
  note: { icon: 'ℹ', label: 'Note' },
  tip: { icon: '✦', label: 'Tip' },
  important: { icon: '⚑', label: 'Important' },
  warning: { icon: '⚠', label: 'Warning' },
  caution: { icon: '⛔', label: 'Caution' },
};

export class AlertBadgeWidget extends WidgetType {
  kind: GfmAlertKind;
  targetPos: number;

  constructor(kind: GfmAlertKind, targetPos: number) {
    super();
    this.kind = kind;
    this.targetPos = targetPos;
  }

  eq(other: AlertBadgeWidget): boolean {
    return this.kind === other.kind && this.targetPos === other.targetPos;
  }

  toDOM(view: EditorView): HTMLElement {
    const meta = ALERT_LABELS[this.kind];
    const badge = document.createElement('span');
    badge.className = `cm-alert-badge cm-alert-badge-${this.kind}`;
    badge.dataset.testid = `cm-alert-badge-${this.kind}`;

    const iconSpan = document.createElement('span');
    iconSpan.className = 'cm-alert-badge-icon';
    iconSpan.setAttribute('aria-hidden', 'true');
    iconSpan.appendChild(document.createTextNode(meta.icon));

    const labelSpan = document.createElement('span');
    labelSpan.className = 'cm-alert-badge-label';
    labelSpan.appendChild(document.createTextNode(meta.label));

    badge.appendChild(iconSpan);
    badge.appendChild(labelSpan);

    badge.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });
    return badge;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

export class FootnoteRefWidget extends WidgetType {
  footnoteId: string;
  targetPos: number;

  constructor(footnoteId: string, targetPos: number) {
    super();
    this.footnoteId = footnoteId;
    this.targetPos = targetPos;
  }

  eq(other: FootnoteRefWidget): boolean {
    return (
      this.footnoteId === other.footnoteId && this.targetPos === other.targetPos
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const sup = document.createElement('sup');
    sup.className = 'cm-footnote-ref';
    sup.dataset.testid = 'cm-footnote-ref';
    sup.appendChild(document.createTextNode(`[${this.footnoteId}]`));
    sup.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;
      e.preventDefault();
      e.stopPropagation();
      const clamped = Math.max(0, Math.min(this.targetPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: clamped, head: clamped },
      });
      view.focus();
    });
    return sup;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

export class TableWidget extends WidgetType {
  alignments: TableColumnAlign[];
  headers: ParsedTableCell[];
  rows: ParsedTableCell[][];
  mathEnabled: boolean;
  blockquoteDepth: number;

  constructor(
    alignments: TableColumnAlign[],
    headers: ParsedTableCell[],
    rows: ParsedTableCell[][],
    mathEnabled: boolean,
    blockquoteDepth: number
  ) {
    super();
    this.alignments = alignments;
    this.headers = headers;
    this.rows = rows;
    this.mathEnabled = mathEnabled;
    this.blockquoteDepth = blockquoteDepth;
  }

  eq(other: TableWidget): boolean {
    if (
      this.mathEnabled !== other.mathEnabled ||
      this.blockquoteDepth !== other.blockquoteDepth ||
      this.alignments.length !== other.alignments.length ||
      this.headers.length !== other.headers.length ||
      this.rows.length !== other.rows.length
    ) {
      return false;
    }
    for (let c = 0; c < this.alignments.length; c++) {
      if (this.alignments[c] !== other.alignments[c]) {
        return false;
      }
    }
    for (let c = 0; c < this.headers.length; c++) {
      const a = this.headers[c];
      const b = other.headers[c];
      if (a.rawText !== b.rawText || a.relOffset !== b.relOffset) {
        return false;
      }
    }
    for (let r = 0; r < this.rows.length; r++) {
      const rowA = this.rows[r];
      const rowB = other.rows[r];
      if (rowA.length !== rowB.length) {
        return false;
      }
      for (let c = 0; c < rowA.length; c++) {
        if (
          rowA[c].rawText !== rowB[c].rawText ||
          rowA[c].relOffset !== rowB[c].relOffset
        ) {
          return false;
        }
      }
    }
    return true;
  }

  toDOM(view: EditorView): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className =
      this.blockquoteDepth > 0
        ? 'cm-table-widget is-in-blockquote'
        : 'cm-table-widget';
    wrap.dataset.testid = 'cm-table-widget';

    const scrollWrap = document.createElement('div');
    scrollWrap.className = 'cm-table-scroll';

    const tableEl = document.createElement('table');
    tableEl.className = 'cm-table-grid';

    const thead = document.createElement('thead');
    const headerTr = document.createElement('tr');
    for (let c = 0; c < this.headers.length; c++) {
      const cell = this.headers[c];
      const th = document.createElement('th');
      th.scope = 'col';
      const align = this.alignments[c];
      if (align) {
        th.dataset.align = align;
      }
      th.dataset.cellRelOffset = String(cell.relOffset);
      th.dataset.colIndex = String(c);
      appendSafeGfmTableCellDom(cell.rawText, th, this.mathEnabled);
      headerTr.appendChild(th);
    }
    thead.appendChild(headerTr);
    tableEl.appendChild(thead);

    if (this.rows.length > 0) {
      const tbody = document.createElement('tbody');
      for (let r = 0; r < this.rows.length; r++) {
        const rowCells = this.rows[r];
        const tr = document.createElement('tr');
        for (let c = 0; c < rowCells.length; c++) {
          const cell = rowCells[c];
          const td = document.createElement('td');
          const align = this.alignments[c];
          if (align) {
            td.dataset.align = align;
          }
          td.dataset.cellRelOffset = String(cell.relOffset);
          td.dataset.rowIndex = String(r);
          td.dataset.colIndex = String(c);
          appendSafeGfmTableCellDom(cell.rawText, td, this.mathEnabled);
          tr.appendChild(td);
        }
        tbody.appendChild(tr);
      }
      tableEl.appendChild(tbody);
    }

    scrollWrap.appendChild(tableEl);
    wrap.appendChild(scrollWrap);

    wrap.addEventListener('mousedown', (e: MouseEvent) => {
      if (e.button !== 0) return;

      if (
        e.target === scrollWrap &&
        (e.offsetY > scrollWrap.clientHeight || e.offsetX > scrollWrap.clientWidth)
      ) {
        return;
      }

      const anchor = (e.target as HTMLElement | null)?.closest(
        'a.cm-table-link'
      ) as HTMLAnchorElement | null;
      if (anchor && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        e.stopPropagation();
        const href = anchor.getAttribute('href')?.trim() ?? '';
        activateEditorLinkHref(view, href);
        return;
      }

      e.preventDefault();
      e.stopPropagation();
      const widgetDocStart = view.posAtDOM(wrap, 0);
      const cellEl = (e.target as HTMLElement | null)?.closest(
        'th, td'
      ) as HTMLElement | null;
      const relAttr = cellEl?.dataset.cellRelOffset;
      const relOffset = relAttr !== undefined ? Number.parseInt(relAttr, 10) : 0;
      const rawPos = widgetDocStart + (Number.isFinite(relOffset) ? relOffset : 0);
      const targetPos = Math.max(0, Math.min(rawPos, view.state.doc.length));
      view.dispatch({
        selection: { anchor: targetPos, head: targetPos },
      });
      view.focus();
    });

    wrap.addEventListener('click', (e: MouseEvent) => {
      const anchor = (e.target as HTMLElement | null)?.closest('a');
      if (anchor) {
        e.preventDefault();
        e.stopPropagation();
      }
    });

    return wrap;
  }

  ignoreEvent(): boolean {
    return true;
  }
}

function countBlockquoteAncestors(node: SyntaxNode): number {
  let depth = 0;
  let p = node.parent;
  while (p) {
    if (p.name === 'Blockquote') {
      depth++;
    }
    p = p.parent;
  }
  return depth;
}

function inspectFencedCodeNode(doc: Text, fencedNode: SyntaxNode): {
  startLineNum: number;
  endLineNum: number;
  blockFrom: number;
  blockTo: number;
  fenceLang: string;
  hasClosingFence: boolean;
  bodyCode: string;
  blockquoteDepth: number;
} {
  const startLine = doc.lineAt(fencedNode.from);
  const endLine = doc.lineAt(fencedNode.to);
  let infoNode: SyntaxNode | null = null;
  let lastCodeMark: SyntaxNode | null = null;
  const quoteMarkEndByLine = new Map<number, number>();

  for (let ch = fencedNode.firstChild; ch; ch = ch.nextSibling) {
    if (ch.name === 'CodeInfo' && !infoNode) {
      infoNode = ch;
    } else if (ch.name === 'CodeMark') {
      lastCodeMark = ch;
    } else if (ch.name === 'QuoteMark') {
      const qLine = doc.lineAt(ch.from).number;
      quoteMarkEndByLine.set(qLine, ch.to);
    }
  }

  const fenceLang = infoNode
    ? doc
        .sliceString(infoNode.from, infoNode.to)
        .trim()
        .split(/\s+/)[0]
        .toLowerCase()
    : '';

  const hasClosingFence = Boolean(
    lastCodeMark && doc.lineAt(lastCodeMark.from).number > startLine.number
  );

  const bodyEndLine = hasClosingFence ? endLine.number - 1 : endLine.number;
  const bodyLines: string[] = [];
  for (let ln = startLine.number + 1; ln <= bodyEndLine; ln++) {
    const l = doc.line(ln);
    const qEnd = quoteMarkEndByLine.get(ln);
    if (qEnd !== undefined) {
      const afterQuote =
        qEnd < l.to && doc.sliceString(qEnd, qEnd + 1) === ' '
          ? qEnd + 1
          : qEnd;
      bodyLines.push(doc.sliceString(afterQuote, l.to));
    } else {
      bodyLines.push(l.text);
    }
  }

  return {
    startLineNum: startLine.number,
    endLineNum: endLine.number,
    blockFrom: startLine.from,
    blockTo: endLine.to,
    fenceLang,
    hasClosingFence,
    bodyCode: bodyLines.join('\n'),
    blockquoteDepth: countBlockquoteAncestors(fencedNode),
  };
}

interface DisplayMathBlockInfo {
  blockFrom: number;
  blockTo: number;
  latex: string;
  blockquoteDepth: number;
}

function collectDisplayMathBlocksFromParagraph(
  doc: Text,
  paragraphNode: SyntaxNode
): DisplayMathBlockInfo[] {
  const results: DisplayMathBlockInfo[] = [];
  const startLineNum = doc.lineAt(paragraphNode.from).number;
  const endLineNum = doc.lineAt(paragraphNode.to).number;
  const blockquoteDepth = countBlockquoteAncestors(paragraphNode);

  // Collect protected spans (Escape, InlineCode, HTMLTag, Comment, Link URLs) inside Paragraph
  const protectedSpans: Array<[number, number]> = [];
  const quoteMarkEndByLine = new Map<number, number>();
  for (let ch = paragraphNode.firstChild; ch; ch = ch.nextSibling) {
    if (ch.name === 'QuoteMark') {
      quoteMarkEndByLine.set(doc.lineAt(ch.from).number, ch.to);
    } else if (
      ch.name === 'Escape' ||
      ch.name === 'InlineCode' ||
      ch.name === 'HTMLTag' ||
      ch.name === 'Comment' ||
      ch.name === 'ProcessingInstruction'
    ) {
      protectedSpans.push([ch.from, ch.to]);
    }
  }

  const isProtected = (pos: number): boolean =>
    protectedSpans.some(([a, b]) => pos >= a && pos < b);

  const getCleanLineSlice = (ln: number): { contentStart: number; trimmed: string; rawContent: string } => {
    const l = doc.line(ln);
    let contentStart = l.from;
    if (ln === startLineNum && paragraphNode.from > l.from) {
      contentStart = paragraphNode.from;
    } else {
      const qEnd = quoteMarkEndByLine.get(ln);
      if (qEnd !== undefined) {
        contentStart =
          qEnd < l.to && doc.sliceString(qEnd, qEnd + 1) === ' ' ? qEnd + 1 : qEnd;
      }
    }
    const rawContent = doc.sliceString(contentStart, l.to);
    return { contentStart, trimmed: rawContent.trim(), rawContent };
  };

  let ln = startLineNum;
  while (ln <= endLineNum) {
    const { contentStart, trimmed, rawContent } = getCleanLineSlice(ln);
    const firstNonWsOffset = rawContent.search(/\S/);
    const dollarDocPos =
      firstNonWsOffset >= 0 ? contentStart + firstNonWsOffset : contentStart;

    if (trimmed.startsWith('$$') && !isProtected(dollarDocPos)) {
      // Single-line display math `$$...$$`
      if (trimmed.length > 4 && trimmed.endsWith('$$')) {
        const closeDollarPos = doc.line(ln).to - (rawContent.length - rawContent.trimEnd().length) - 2;
        if (!isProtected(closeDollarPos)) {
          const inner = trimmed.slice(2, -2).trim();
          if (inner.length > 0 && !inner.includes('$$')) {
            const l = doc.line(ln);
            results.push({
              blockFrom: l.from,
              blockTo: l.to,
              latex: inner,
              blockquoteDepth,
            });
            ln++;
            continue;
          }
        }
      }

      // Multi-line display math `$$\n...\n$$` within the Paragraph
      if (trimmed === '$$' || !trimmed.slice(2).includes('$$')) {
        const mathLines: string[] = [];
        const firstRest = trimmed.slice(2).trim();
        if (firstRest) {
          mathLines.push(firstRest);
        }
        let closeLine = -1;
        for (let scan = ln + 1; scan <= endLineNum; scan++) {
          const scanSlice = getCleanLineSlice(scan);
          if (scanSlice.trimmed.endsWith('$$')) {
            const closePos =
              doc.line(scan).to -
              (scanSlice.rawContent.length - scanSlice.rawContent.trimEnd().length) -
              2;
            if (!isProtected(closePos)) {
              const beforeEnd = scanSlice.trimmed.slice(0, -2).trim();
              if (beforeEnd) {
                mathLines.push(beforeEnd);
              }
              closeLine = scan;
              break;
            }
          }
          mathLines.push(scanSlice.rawContent);
        }

        if (closeLine > ln) {
          results.push({
            blockFrom: doc.line(ln).from,
            blockTo: doc.line(closeLine).to,
            latex: mathLines.join('\n'),
            blockquoteDepth,
          });
          ln = closeLine + 1;
          continue;
        }
      }
    }
    ln++;
  }

  return results;
}

function isSelectionIntersecting(
  state: EditorState,
  from: number,
  to: number
): boolean {
  return state.selection.ranges.some((r) => r.from <= to && r.to >= from);
}

interface CachedSetextBlock {
  blockFrom: number;
  blockTo: number;
  collapseFrom: number;
  collapseTo: number;
}

interface CachedFencedDiagramOrMath {
  blockFrom: number;
  blockTo: number;
  fenceLang: 'mermaid' | 'math';
  bodyCode: string;
  blockquoteDepth: number;
}

interface CachedMultiLineImage {
  from: number;
  to: number;
  alt: string;
  rawUrl: string;
  title: string;
  isStandalone: boolean;
}

interface CachedDocBlockStructure {
  tables: ParsedGfmTable[];
  setextHeadings: CachedSetextBlock[];
  fencedSpecial: CachedFencedDiagramOrMath[];
  displayMathBlocks: DisplayMathBlockInfo[];
  multiLineImages: CachedMultiLineImage[];
}

const docBlockStructureCache = new WeakMap<Text, Map<string, CachedDocBlockStructure>>();

function getCachedDocBlockStructure(
  state: EditorState,
  isGfm: boolean,
  frontmatterEnabled: boolean
): CachedDocBlockStructure {
  const modeKey = `${isGfm ? 'gfm' : 'cm'}:${frontmatterEnabled ? 'fm' : 'nofm'}`;
  let perDoc = docBlockStructureCache.get(state.doc);
  if (perDoc) {
    const hit = perDoc.get(modeKey);
    if (hit) {
      return hit;
    }
  } else {
    perDoc = new Map();
    docBlockStructureCache.set(state.doc, perDoc);
  }

  const tree = ensureSyntaxTree(state, state.doc.length, 50) ?? syntaxTree(state);
  const doc = state.doc;
  const tables = isGfm ? extractGfmTablesFromState(state) : [];
  const setextHeadings: CachedSetextBlock[] = [];
  const fencedSpecial: CachedFencedDiagramOrMath[] = [];
  const displayMathBlocks: DisplayMathBlockInfo[] = [];
  const multiLineImages: CachedMultiLineImage[] = [];
  const refMapForBlockImages = extractLinkReferenceMap(state);

  tree.iterate({
    enter(nodeRef) {
      const name = nodeRef.name;
      if (
        name === 'Frontmatter' ||
        name === 'Table' ||
        name === 'CodeBlock' ||
        name === 'HTMLBlock' ||
        name === 'CommentBlock' ||
        name === 'ProcessingInstructionBlock' ||
        name === 'LinkReference' ||
        name === 'HorizontalRule' ||
        /^ATXHeading[1-6]$/.test(name)
      ) {
        return false;
      }

      if (name === 'SetextHeading1' || name === 'SetextHeading2') {
        const startLine = doc.lineAt(nodeRef.from);
        const endLine = doc.lineAt(nodeRef.to);
        if (endLine.number > startLine.number) {
          const prevLine = doc.line(endLine.number - 1);
          if (endLine.to > prevLine.to) {
            setextHeadings.push({
              blockFrom: startLine.from,
              blockTo: endLine.to,
              collapseFrom: prevLine.to,
              collapseTo: endLine.to,
            });
          }
        }
        return false;
      }

      if (name === 'FencedCode') {
        if (isGfm) {
          const info = inspectFencedCodeNode(doc, nodeRef.node);
          if (
            info.hasClosingFence &&
            info.blockTo > info.blockFrom &&
            (info.fenceLang === 'mermaid' || info.fenceLang === 'math')
          ) {
            fencedSpecial.push({
              blockFrom: info.blockFrom,
              blockTo: info.blockTo,
              fenceLang: info.fenceLang,
              bodyCode: info.bodyCode,
              blockquoteDepth: info.blockquoteDepth,
            });
          }
        }
        return false;
      }

      if (name === 'Paragraph') {
        if (isGfm) {
          const mbs = collectDisplayMathBlocksFromParagraph(doc, nodeRef.node);
          for (const mb of mbs) {
            if (mb.blockTo > mb.blockFrom) {
              displayMathBlocks.push(mb);
            }
          }
        }
        for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
          if (ch.name === 'Image') {
            const sLine = doc.lineAt(ch.from).number;
            const eLine = doc.lineAt(ch.to).number;
            if (eLine > sLine && ch.to > ch.from) {
              const classifiedImg = classifyAstImageNode(doc, ch, refMapForBlockImages);
              if (classifiedImg.kind === 'image') {
                multiLineImages.push({
                  from: ch.from,
                  to: ch.to,
                  alt: classifiedImg.alt,
                  rawUrl: classifiedImg.rawUrl,
                  title: classifiedImg.title,
                  isStandalone: classifiedImg.isStandalone,
                });
              }
            }
          }
        }
        return false;
      }

      return true;
    },
  });

  const entry: CachedDocBlockStructure = {
    tables,
    setextHeadings,
    fencedSpecial,
    displayMathBlocks,
    multiLineImages,
  };
  perDoc.set(modeKey, entry);
  return entry;
}

/**
 * Builds all multi-line and block-level replacements (`TableWidget`, `SetextHeading` underline
 * collapse, `MermaidSvgWidget`, fenced `math`, and `$$...$$` display math) via a `StateField`
 * per `AGENTS.md` §2.L5.3, reusing a `WeakMap<Text, ...>` block structure cache so selection
 * movements are $O(B)$ in the number of blocks rather than re-parsing the full document.
 */
export function buildBlockLiveDecorations(state: EditorState): DecorationSet {
  const liveCfg = state.facet(livePreviewConfigFacet);
  if (!liveCfg.enabled) {
    return Decoration.none;
  }

  const isGfm = liveCfg.markdownConfig.base === 'Gfm';
  const mathEnabled = isGfm && liveCfg.markdownConfig.extensions.math;
  const mermaidEnabled = isGfm && liveCfg.markdownConfig.extensions.mermaid;
  const frontmatterEnabled = isGfm && liveCfg.markdownConfig.extensions.frontmatter;

  const cached = getCachedDocBlockStructure(state, isGfm, frontmatterEnabled);
  const decoRanges: Range<Decoration>[] = [];

  if (isGfm) {
    for (const tbl of cached.tables) {
      if (!isSelectionIntersecting(state, tbl.from, tbl.to) && tbl.to > tbl.from) {
        decoRanges.push(
          Decoration.replace({
            widget: new TableWidget(
              tbl.alignments,
              tbl.headers,
              tbl.rows,
              mathEnabled,
              tbl.blockquoteDepth
            ),
            block: true,
          }).range(tbl.from, tbl.to)
        );
      }
    }
  }

  for (const sh of cached.setextHeadings) {
    if (!isSelectionIntersecting(state, sh.blockFrom, sh.blockTo)) {
      decoRanges.push(Decoration.replace({}).range(sh.collapseFrom, sh.collapseTo));
    }
  }

  for (const fc of cached.fencedSpecial) {
    if (!isSelectionIntersecting(state, fc.blockFrom, fc.blockTo)) {
      if (mermaidEnabled && fc.fenceLang === 'mermaid') {
        const cachedSvg = getCachedMermaidSvg(fc.bodyCode, liveCfg.theme);
        decoRanges.push(
          Decoration.replace({
            widget: new MermaidSvgWidget(
              fc.bodyCode,
              liveCfg.theme,
              fc.blockFrom,
              cachedSvg,
              fc.blockquoteDepth
            ),
            block: true,
          }).range(fc.blockFrom, fc.blockTo)
        );
      } else if (mathEnabled && fc.fenceLang === 'math') {
        decoRanges.push(
          Decoration.replace({
            widget: new KatexMathWidget(
              fc.bodyCode,
              true,
              fc.blockFrom,
              fc.blockquoteDepth
            ),
            block: true,
          }).range(fc.blockFrom, fc.blockTo)
        );
      }
    }
  }

  if (mathEnabled) {
    for (const mb of cached.displayMathBlocks) {
      if (!isSelectionIntersecting(state, mb.blockFrom, mb.blockTo)) {
        decoRanges.push(
          Decoration.replace({
            widget: new KatexMathWidget(
              mb.latex,
              true,
              mb.blockFrom,
              mb.blockquoteDepth
            ),
            block: true,
          }).range(mb.blockFrom, mb.blockTo)
        );
      }
    }
  }

  for (const img of cached.multiLineImages) {
    if (!isSelectionIntersecting(state, img.from, img.to)) {
      decoRanges.push(
        Decoration.replace({
          widget: new ImageWidget(
            img.alt,
            img.rawUrl,
            img.title,
            img.from,
            liveCfg.currentFilePath,
            img.isStandalone
          ),
        }).range(img.from, img.to)
      );
    }
  }

  return decoRanges.length > 0 ? Decoration.set(decoRanges, true) : Decoration.none;
}

export const blockLivePreviewField = StateField.define<DecorationSet>({
  create(state) {
    return buildBlockLiveDecorations(state);
  },
  update(decorations, tr) {
    const hasRefresh = tr.effects.some((e) => e.is(refreshLivePreviewEffect));
    if (
      hasRefresh ||
      tr.docChanged ||
      tr.selection !== undefined ||
      tr.reconfigured ||
      syntaxTree(tr.startState) !== syntaxTree(tr.state)
    ) {
      return buildBlockLiveDecorations(tr.state);
    }
    return decorations;
  },
  provide: (f) => EditorView.decorations.from(f),
});

interface CoalescedRange {
  from: number;
  to: number;
}

function coalesceRanges(
  ranges: readonly { from: number; to: number }[],
  docLen: number
): CoalescedRange[] {
  if (ranges.length === 0) {
    return [{ from: 0, to: docLen }];
  }
  const sorted = ranges
    .map((r) => ({
      from: Math.max(0, Math.min(r.from, docLen)),
      to: Math.max(0, Math.min(r.to, docLen)),
    }))
    .sort((a, b) => a.from - b.from);
  const out: CoalescedRange[] = [sorted[0]];
  for (let i = 1; i < sorted.length; i++) {
    const prev = out[out.length - 1];
    const cur = sorted[i];
    if (cur.from <= prev.to) {
      prev.to = Math.max(prev.to, cur.to);
    } else {
      out.push(cur);
    }
  }
  return out;
}

interface PendingReplace {
  from: number;
  to: number;
  deco: Decoration;
}

function pushLineSplitReplaces(
  list: PendingReplace[],
  doc: Text,
  from: number,
  to: number
): void {
  if (to <= from) return;
  const startLine = doc.lineAt(from);
  const endLine = doc.lineAt(to);
  if (startLine.number === endLine.number) {
    list.push({ from, to, deco: Decoration.replace({}) });
    return;
  }
  if (startLine.to > from) {
    list.push({ from, to: startLine.to, deco: Decoration.replace({}) });
  }
  for (let ln = startLine.number + 1; ln < endLine.number; ln++) {
    const l = doc.line(ln);
    if (l.to > l.from) {
      list.push({ from: l.from, to: l.to, deco: Decoration.replace({}) });
    }
  }
  if (to > endLine.from) {
    list.push({ from: endLine.from, to, deco: Decoration.replace({}) });
  }
}

function detectBlockquoteAlert(
  doc: Text,
  bqNode: SyntaxNode
): { kind: GfmAlertKind; markerFrom: number; markerTo: number } | null {
  let ch: SyntaxNode | null = bqNode.firstChild;
  while (ch !== null) {
    if (ch.name === 'QuoteMark') {
      ch = ch.nextSibling;
      continue;
    }
    if (ch.name !== 'Paragraph') {
      return null;
    }
    const firstLine = doc.lineAt(ch.from);
    const slice = doc.sliceString(ch.from, firstLine.to);
    const m = slice.match(/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\](?=\s|$)/i);
    if (!m) {
      return null;
    }
    const kind = m[1].toLowerCase() as GfmAlertKind;
    return {
      kind,
      markerFrom: ch.from,
      markerTo: ch.from + m[0].length,
    };
  }
  return null;
}

interface InlineMathMatch {
  from: number;
  to: number;
  latex: string;
}

function collectInlineDollarMathInContainer(
  doc: Text,
  containerFrom: number,
  containerTo: number,
  protectedSpans: Array<[number, number]>
): InlineMathMatch[] {
  const matches: InlineMathMatch[] = [];
  const isOverlappingProtected = (a: number, b: number): boolean =>
    protectedSpans.some(([x, y]) => a < y && b > x);

  const startLineNum = doc.lineAt(containerFrom).number;
  const endLineNum = doc.lineAt(containerTo).number;

  for (let ln = startLineNum; ln <= endLineNum; ln++) {
    const line = doc.line(ln);
    const segFrom = Math.max(line.from, containerFrom);
    const segTo = Math.min(line.to, containerTo);
    if (segTo - segFrom < 3) continue;

    const text = doc.sliceString(segFrom, segTo);
    let pos = 0;
    while (pos < text.length) {
      if (text[pos] !== '$') {
        pos++;
        continue;
      }
      const docOpen = segFrom + pos;
      if (isOverlappingProtected(docOpen, docOpen + 1)) {
        pos++;
        continue;
      }
      if (
        text[pos + 1] === '$' ||
        (pos > 0 && text[pos - 1] === '$') ||
        pos + 1 >= text.length ||
        /\s/.test(text[pos + 1])
      ) {
        pos++;
        continue;
      }

      let closeIdx = -1;
      for (let j = pos + 1; j < text.length; j++) {
        const docPosJ = segFrom + j;
        if (isOverlappingProtected(docPosJ, docPosJ + 1)) {
          // If this is an Escape node `\$`, skip past it; otherwise abort math scan
          if (text[j] === '\\' && j + 1 < text.length && text[j + 1] === '$') {
            j++;
            continue;
          }
          break;
        }
        if (text[j] !== '$') continue;
        if (
          !/\s/.test(text[j - 1]) &&
          text[j + 1] !== '$' &&
          !/[0-9]/.test(text[j + 1] ?? '')
        ) {
          closeIdx = j;
        }
        // Inline `$...$` cannot contain unescaped `$`; stop at the first unescaped `$`
        break;
      }

      if (closeIdx !== -1) {
        const expr = text.slice(pos + 1, closeIdx);
        matches.push({
          from: docOpen,
          to: segFrom + closeIdx + 1,
          latex: expr,
        });
        pos = closeIdx + 1;
      } else {
        pos++;
      }
    }
  }

  return matches;
}

/**
 * Traverses `syntaxTree(state)` across `visibleRanges` to produce all caret-aware line and inline
 * decorations without per-line regex scanners or flat `isOccupied` arrays, coordinated with
 * `blockLivePreviewField` and preserving raw GFM `Table` column alignment when the caret enters a table.
 */
export function buildInlineAndLineLiveDecorations(
  state: EditorState,
  visibleRanges?: readonly { from: number; to: number }[],
  overrideCfg?: LivePreviewConfigState
): DecorationSet {
  const liveCfg = overrideCfg ?? state.facet(livePreviewConfigFacet);
  const enabled = liveCfg.enabled;
  const cfg = liveCfg.markdownConfig;

  const isGfm = cfg.base === 'Gfm';
  const mathEnabled = isGfm && cfg.extensions.math;
  const alertsEnabled = isGfm && cfg.extensions.alerts;
  const footnotesEnabled = isGfm && cfg.extensions.footnotes;

  const doc = state.doc;
  const tree = ensureSyntaxTree(state, doc.length, 50) ?? syntaxTree(state);
  const coalescedVis = coalesceRanges(
    visibleRanges ?? [{ from: 0, to: doc.length }],
    doc.length
  );

  const selRanges = state.selection.ranges;
  const selLineIntervals: Array<[number, number]> = selRanges.map((r) => [
    doc.lineAt(r.from).number,
    doc.lineAt(r.to).number,
  ]);

  const isCaretInSpan = (from: number, to: number): boolean =>
    selRanges.some((r) => r.from <= to && r.to >= from);

  const isCaretOnLine = (ln: number): boolean =>
    selLineIntervals.some(([s, e]) => s <= ln && e >= ln);

  const isCaretOnLines = (l1: number, l2: number): boolean =>
    selLineIntervals.some(([s, e]) => s <= l2 && e >= l1);

  // Collect active multi-line block replacements from `blockLivePreviewField` (or compute if absent)
  const blockDecoSet =
    state.field(blockLivePreviewField, false) ?? buildBlockLiveDecorations(state);
  const replacedBlockIntervals: Array<[number, number]> = [];
  blockDecoSet.between(0, doc.length, (from, to, val) => {
    if (val.spec.block) {
      replacedBlockIntervals.push([from, to]);
    } else {
      // Setext underline collapse `[prevLine.to, endLine.to]`: skip decorations on `endLine`
      replacedBlockIntervals.push([from + 1, to]);
    }
  });

  const isCoveredByBlockReplace = (from: number, to: number): boolean =>
    replacedBlockIntervals.some(([bf, bt]) => from >= bf && to <= bt);

  const isLineCoveredByBlockReplace = (ln: number): boolean => {
    const l = doc.line(ln);
    return replacedBlockIntervals.some(([bf, bt]) => l.from >= bf && l.to <= bt);
  };

  const gfmTables: ParsedGfmTable[] =
    enabled && isGfm
      ? getCachedDocBlockStructure(state, isGfm, cfg.extensions.frontmatter).tables
      : [];
  const activeRawTableIntervals: Array<[number, number]> = [];
  const tableByNodeStartLine = new Map<number, ParsedGfmTable>();
  for (const tbl of gfmTables) {
    tableByNodeStartLine.set(tbl.startLineNum, tbl);
    if (isCaretInSpan(tbl.from, tbl.to)) {
      activeRawTableIntervals.push([tbl.from, tbl.to]);
    }
  }

  const isInsideActiveRawTable = (from: number, to: number): boolean =>
    activeRawTableIntervals.some(([tf, tt]) => from >= tf && to <= tt);

  const refMap = enabled ? extractLinkReferenceMap(state) : new Map<string, ResolvedLinkRef>();

  const lineClassMap = new Map<number, Set<string>>();
  const lineAttrMap = new Map<number, Record<string, string>>();
  const markRanges: Range<Decoration>[] = [];
  const pendingReplaces: PendingReplace[] = [];

  const addLineClass = (ln: number, ...cls: string[]) => {
    if (isLineCoveredByBlockReplace(ln)) return;
    let set = lineClassMap.get(ln);
    if (!set) {
      set = new Set();
      lineClassMap.set(ln, set);
    }
    for (const c of cls) {
      if (c) set.add(c);
    }
  };

  const setLineAttr = (ln: number, key: string, val: string) => {
    if (isLineCoveredByBlockReplace(ln)) return;
    let attrs = lineAttrMap.get(ln);
    if (!attrs) {
      attrs = {};
      lineAttrMap.set(ln, attrs);
    }
    attrs[key] = val;
  };

  const blockquoteDepthByLine = new Map<number, number>();
  const alertLineKeepQuoteVisible = new Set<number>();
  const bulletTaskSuppressedListMarks = new Set<number>();
  const visitedNodeKeys = new Set<string>();
  const inlineProtectedForDollarMath: Array<[number, number]> = [];
  const dollarMathContainers: Array<{ from: number; to: number }> = [];

  for (const vis of coalescedVis) {
    const visStartLine = doc.lineAt(vis.from).number;
    const visEndLine = doc.lineAt(vis.to).number;

    tree.iterate({
      from: vis.from,
      to: vis.to,
      enter(nodeRef) {
        const name = nodeRef.name;
        const from = nodeRef.from;
        const to = nodeRef.to;
        const nodeKey = `${name}:${from}:${to}`;

        if (isCoveredByBlockReplace(from, to)) {
          return false;
        }

        // 0. YAML Frontmatter (`yamlFrontmatter` wrapper when `extensions.frontmatter` is true)
        if (name === 'Frontmatter') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const fmStart = doc.lineAt(from).number;
            const fmEnd = doc.lineAt(Math.max(from, to - 1)).number;
            const caretInsideFm = isCaretOnLines(fmStart, fmEnd);
            const lo = Math.max(fmStart, visStartLine);
            const hi = Math.min(fmEnd, visEndLine);
            for (let ln = lo; ln <= hi; ln++) {
              addLineClass(ln, 'cm-live-codeblock-line');
              if (ln === fmStart) {
                addLineClass(ln, 'cm-live-codeblock-first');
                setLineAttr(ln, 'data-code-lang', 'yaml');
              }
              if (ln === fmEnd) {
                addLineClass(ln, 'cm-live-codeblock-last');
              }
              if (enabled && !caretInsideFm && (ln === fmStart || ln === fmEnd)) {
                const l = doc.line(ln);
                if (l.to > l.from) {
                  markRanges.push(
                    Decoration.mark({ class: 'cm-live-fence-muted' }).range(l.from, l.to)
                  );
                }
              }
            }
          }
          return false;
        }

        // 1. Fenced Code Blocks (§4.5 — including unclosed EOF Ex. 95 & fake closures Ex. 97)
        if (name === 'FencedCode') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const info = inspectFencedCodeNode(doc, nodeRef.node);
            const caretInsideFence = isCaretOnLines(info.startLineNum, info.endLineNum);
            const lo = Math.max(info.startLineNum, visStartLine);
            const hi = Math.min(info.endLineNum, visEndLine);
            for (let ln = lo; ln <= hi; ln++) {
              addLineClass(ln, 'cm-live-codeblock-line');
              if (ln === info.startLineNum) {
                addLineClass(ln, 'cm-live-codeblock-first');
                if (info.fenceLang) {
                  setLineAttr(ln, 'data-code-lang', info.fenceLang);
                }
              }
              if (ln === info.endLineNum) {
                addLineClass(ln, 'cm-live-codeblock-last');
              }
              const isBoundary =
                ln === info.startLineNum ||
                (info.hasClosingFence && ln === info.endLineNum);
              if (enabled && isBoundary && !caretInsideFence) {
                const l = doc.line(ln);
                if (l.to > l.from) {
                  markRanges.push(
                    Decoration.mark({ class: 'cm-live-fence-muted' }).range(l.from, l.to)
                  );
                }
              }
            }

            // Still hide QuoteMark inside FencedCode if inside a Blockquote and caret outside line
            if (enabled && info.blockquoteDepth > 0) {
              for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
                if (ch.name === 'QuoteMark') {
                  const qLine = doc.lineAt(ch.from);
                  if (
                    qLine.number >= visStartLine &&
                    qLine.number <= visEndLine &&
                    !isCaretOnLine(qLine.number)
                  ) {
                    const hideTo =
                      ch.to < qLine.to && doc.sliceString(ch.to, ch.to + 1) === ' '
                        ? ch.to + 1
                        : ch.to;
                    pendingReplaces.push({
                      from: ch.from,
                      to: hideTo,
                      deco: Decoration.replace({}),
                    });
                  }
                }
              }
            }
          }
          return false;
        }

        // 2. Indented Code Blocks (§4.4)
        if (name === 'CodeBlock') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const cbStart = doc.lineAt(from).number;
            const cbEnd = doc.lineAt(to).number;
            const lo = Math.max(cbStart, visStartLine);
            const hi = Math.min(cbEnd, visEndLine);
            for (let ln = lo; ln <= hi; ln++) {
              addLineClass(ln, 'cm-live-codeblock-line', 'cm-live-indented-code-line');
              if (ln === cbStart) addLineClass(ln, 'cm-live-codeblock-first');
              if (ln === cbEnd) addLineClass(ln, 'cm-live-codeblock-last');
            }
            if (enabled) {
              for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
                if (ch.name === 'QuoteMark') {
                  const qLine = doc.lineAt(ch.from);
                  if (
                    qLine.number >= visStartLine &&
                    qLine.number <= visEndLine &&
                    !isCaretOnLine(qLine.number)
                  ) {
                    const hideTo =
                      ch.to < qLine.to && doc.sliceString(ch.to, ch.to + 1) === ' '
                        ? ch.to + 1
                        : ch.to;
                    pendingReplaces.push({
                      from: ch.from,
                      to: hideTo,
                      deco: Decoration.replace({}),
                    });
                  }
                }
              }
            }
          }
          return false;
        }

        // In raw editing mode (`!enabled`), stop here after codeblock boxes
        if (!enabled) {
          return true;
        }

        // 3. HTML Blocks (§4.6) — never apply Markdown or KaTeX decorations inside HTML blocks
        if (
          name === 'HTMLBlock' ||
          name === 'CommentBlock' ||
          name === 'ProcessingInstructionBlock'
        ) {
          for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
            if (ch.name === 'QuoteMark') {
              const qLine = doc.lineAt(ch.from);
              if (
                qLine.number >= visStartLine &&
                qLine.number <= visEndLine &&
                !isCaretOnLine(qLine.number)
              ) {
                const hideTo =
                  ch.to < qLine.to && doc.sliceString(ch.to, ch.to + 1) === ' '
                    ? ch.to + 1
                    : ch.to;
                pendingReplaces.push({
                  from: ch.from,
                  to: hideTo,
                  deco: Decoration.replace({}),
                });
              }
            }
          }
          return false;
        }

        // 4. GFM §4.10 Tables when caret is inside the table: monospace aligned source lines,
        //    suspending all inline delimiter collapsing on table rows!
        if (name === 'Table' && isGfm) {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const tblStartLine = doc.lineAt(from).number;
            const tbl = tableByNodeStartLine.get(tblStartLine);
            if (tbl && isCaretInSpan(tbl.from, tbl.to)) {
              const lo = Math.max(tbl.startLineNum, visStartLine);
              const hi = Math.min(tbl.endLineNum, visEndLine);
              for (let ln = lo; ln <= hi; ln++) {
                addLineClass(ln, 'cm-live-table-source-line');
                if (ln === tbl.startLineNum) addLineClass(ln, 'cm-live-table-source-first');
                if (ln === tbl.headerLineNum) addLineClass(ln, 'cm-live-table-source-header');
                if (ln === tbl.delimiterLineNum)
                  addLineClass(ln, 'cm-live-table-source-delimiter');
                if (ln === tbl.endLineNum) addLineClass(ln, 'cm-live-table-source-last');
              }
            }
          }
          return false;
        }

        // 5. Blockquotes (§5.1: nested `> >`, lazy continuation lines, and GFM Alerts/Callouts)
        if (name === 'Blockquote') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const bqStartLine = doc.lineAt(from);
            const bqEndLine = doc.lineAt(to);
            const bqBlockFrom = bqStartLine.from;
            const bqBlockTo = bqEndLine.to;
            const caretInsideBq = isCaretInSpan(bqBlockFrom, bqBlockTo);

            const lo = Math.max(bqStartLine.number, visStartLine);
            const hi = Math.min(bqEndLine.number, visEndLine);
            for (let ln = lo; ln <= hi; ln++) {
              const nextDepth = (blockquoteDepthByLine.get(ln) ?? 0) + 1;
              blockquoteDepthByLine.set(ln, nextDepth);
            }

            const alertInfo =
              alertsEnabled ? detectBlockquoteAlert(doc, nodeRef.node) : null;
            if (alertInfo) {
              if (!caretInsideBq) {
                for (let ln = lo; ln <= hi; ln++) {
                  addLineClass(
                    ln,
                    'cm-live-alert-line',
                    `cm-live-alert-${alertInfo.kind}`
                  );
                  if (ln === bqStartLine.number) {
                    addLineClass(ln, 'cm-live-alert-first');
                  }
                  if (ln === bqEndLine.number) {
                    addLineClass(ln, 'cm-live-alert-last');
                  }
                }
                const markerLine = doc.lineAt(alertInfo.markerFrom).number;
                if (markerLine >= visStartLine && markerLine <= visEndLine) {
                  pendingReplaces.push({
                    from: alertInfo.markerFrom,
                    to: alertInfo.markerTo,
                    deco: Decoration.replace({
                      widget: new AlertBadgeWidget(
                        alertInfo.kind,
                        alertInfo.markerFrom
                      ),
                    }),
                  });
                }
              } else {
                // Caret is inside the Alert blockquote: keep `>` visible on the `[!TYPE]` line too
                alertLineKeepQuoteVisible.add(doc.lineAt(alertInfo.markerFrom).number);
              }
            }
          }
          return true;
        }

        if (name === 'QuoteMark') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const qLine = doc.lineAt(from);
            if (
              qLine.number >= visStartLine &&
              qLine.number <= visEndLine &&
              !isCaretOnLine(qLine.number) &&
              !alertLineKeepQuoteVisible.has(qLine.number)
            ) {
              const hideTo =
                to < qLine.to && doc.sliceString(to, to + 1) === ' '
                  ? to + 1
                  : to;
              pendingReplaces.push({
                from,
                to: hideTo,
                deco: Decoration.replace({}),
              });
            }
          }
          return false;
        }

        // 6. Thematic breaks (§4.1 HorizontalRule)
        if (name === 'HorizontalRule') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const hrLine = doc.lineAt(from);
            if (
              hrLine.number >= visStartLine &&
              hrLine.number <= visEndLine &&
              !isCaretOnLine(hrLine.number)
            ) {
              addLineClass(hrLine.number, 'cm-live-hr-line');
              const prefixSlice = doc.sliceString(hrLine.from, from);
              const replaceFrom = /^[ \t]*$/.test(prefixSlice) ? hrLine.from : from;
              if (hrLine.to > replaceFrom) {
                pendingReplaces.push({
                  from: replaceFrom,
                  to: hrLine.to,
                  deco: Decoration.replace({
                    widget: new ThematicBreakWidget(hrLine.from),
                  }),
                });
              }
            }
          }
          return false;
        }

        // 7. ATX Headings (§4.2 ATXHeading1..6 — including empty headings `###` & closing `###`)
        if (/^ATXHeading([1-6])$/.test(name)) {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const level = Number(name.slice(10));
            const hLine = doc.lineAt(from);
            if (hLine.number >= visStartLine && hLine.number <= visEndLine) {
              addLineClass(hLine.number, 'cm-live-heading-line', `cm-live-h${level}`);

              let openMark: SyntaxNode | null = null;
              let closeMark: SyntaxNode | null = null;
              for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
                if (ch.name === 'HeaderMark') {
                  if (!openMark) openMark = ch;
                  else closeMark = ch;
                }
              }

              if (openMark && !isCaretOnLine(hLine.number)) {
                const prefixSlice = doc.sliceString(hLine.from, openMark.from);
                const openHideFrom = /^[ \t]*$/.test(prefixSlice)
                  ? hLine.from
                  : openMark.from;
                const innerEndBound = closeMark ? closeMark.from : hLine.to;
                const afterOpenSlice = doc.sliceString(openMark.to, innerEndBound);

                if (/^[ \t]*$/.test(afterOpenSlice)) {
                  // Empty ATX heading (`###`, `###   `, `### ###`) per GFM §4.2 Ex. 36
                  if (hLine.to > openHideFrom) {
                    pendingReplaces.push({
                      from: openHideFrom,
                      to: hLine.to,
                      deco: Decoration.replace({}),
                    });
                  }
                } else {
                  const wsMatch = afterOpenSlice.match(/^[ \t]+/);
                  const openHideTo = openMark.to + (wsMatch ? wsMatch[0].length : 0);
                  if (openHideTo > openHideFrom) {
                    pendingReplaces.push({
                      from: openHideFrom,
                      to: openHideTo,
                      deco: Decoration.replace({}),
                    });
                  }
                  if (closeMark) {
                    let closeHideFrom = closeMark.from;
                    while (
                      closeHideFrom > openHideTo &&
                      /[ \t]/.test(doc.sliceString(closeHideFrom - 1, closeHideFrom))
                    ) {
                      closeHideFrom--;
                    }
                    if (hLine.to > closeHideFrom) {
                      pendingReplaces.push({
                        from: closeHideFrom,
                        to: hLine.to,
                        deco: Decoration.replace({}),
                      });
                    }
                  }
                }
              }

              if (mathEnabled) {
                dollarMathContainers.push({ from, to });
              }
            }
          }
          return true;
        }

        // 8. Setext Headings (§4.3 SetextHeading1..2 — multi-line H1/H2 title lines; underline collapsed by StateField)
        if (name === 'SetextHeading1' || name === 'SetextHeading2') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const level = name === 'SetextHeading1' ? 1 : 2;
            const startLine = doc.lineAt(from);
            const endLine = doc.lineAt(to);
            const titleEndLine = Math.max(startLine.number, endLine.number - 1);
            const lo = Math.max(startLine.number, visStartLine);
            const hi = Math.min(titleEndLine, visEndLine);
            for (let ln = lo; ln <= hi; ln++) {
              addLineClass(
                ln,
                'cm-live-heading-line',
                `cm-live-h${level}`,
                'cm-live-setext-heading'
              );
            }
            if (
              endLine.number >= visStartLine &&
              endLine.number <= visEndLine &&
              isCaretInSpan(startLine.from, endLine.to)
            ) {
              addLineClass(endLine.number, 'cm-live-setext-underline-line');
            }
            if (mathEnabled) {
              dollarMathContainers.push({
                from,
                to: doc.line(titleEndLine).to,
              });
            }
          }
          return true;
        }

        // 9. Interactive Task Lists (§5.3: `- [ ]`, `1. [x]`, `> - [ ]`) & List Markers (§5.2 / §5.4)
        if (name === 'ListItem') {
          if (isGfm && !visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            let listMarkNode: SyntaxNode | null = null;
            let taskNode: SyntaxNode | null = null;
            for (let ch = nodeRef.node.firstChild; ch; ch = ch.nextSibling) {
              if (ch.name === 'ListMark' && !listMarkNode) {
                listMarkNode = ch;
              } else if (ch.name === 'Task' && !taskNode) {
                taskNode = ch;
              }
            }
            const taskMarkerNode = taskNode?.firstChild;
            if (
              listMarkNode &&
              taskMarkerNode &&
              taskMarkerNode.name === 'TaskMarker'
            ) {
              const tLine = doc.lineAt(taskMarkerNode.from);
              const isBulletParent = nodeRef.node.parent?.name === 'BulletList';
              if (
                tLine.number >= visStartLine &&
                tLine.number <= visEndLine &&
                !isCaretOnLine(tLine.number)
              ) {
                const rawMarker = doc.sliceString(
                  taskMarkerNode.from,
                  taskMarkerNode.to
                );
                const checked = rawMarker[1]?.toLowerCase() === 'x';
                const replaceFrom = isBulletParent
                  ? listMarkNode.from
                  : taskMarkerNode.from;
                if (isBulletParent) {
                  bulletTaskSuppressedListMarks.add(listMarkNode.from);
                }
                pendingReplaces.push({
                  from: replaceFrom,
                  to: taskMarkerNode.to,
                  deco: Decoration.replace({
                    widget: new TaskCheckboxWidget(
                      checked,
                      taskMarkerNode.from,
                      taskMarkerNode.to
                    ),
                  }),
                });
              }
            }
          }
          return true;
        }

        if (name === 'ListMark') {
          if (
            !visitedNodeKeys.has(nodeKey) &&
            !bulletTaskSuppressedListMarks.has(from)
          ) {
            visitedNodeKeys.add(nodeKey);
            const mLine = doc.lineAt(from).number;
            if (mLine >= visStartLine && mLine <= visEndLine && to > from) {
              markRanges.push(
                Decoration.mark({ class: 'cm-live-list-marker' }).range(from, to)
              );
            }
          }
          return false;
        }

        if (name === 'Task') {
          if (!visitedNodeKeys.has(nodeKey) && mathEnabled) {
            visitedNodeKeys.add(nodeKey);
            dollarMathContainers.push({ from, to });
          }
          return true;
        }

        if (name === 'Paragraph') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            if (footnotesEnabled) {
              const pLine = doc.lineAt(from);
              if (pLine.number >= visStartLine && pLine.number <= visEndLine) {
                const slice = doc.sliceString(from, pLine.to);
                const fnDefMatch = slice.match(/^(\[\^[^\s\[\]]+\]:)/);
                if (fnDefMatch) {
                  addLineClass(pLine.number, 'cm-live-footnote-def-line');
                  markRanges.push(
                    Decoration.mark({ class: 'cm-live-footnote-label' }).range(
                      from,
                      from + fnDefMatch[1].length
                    )
                  );
                }
              }
            }
            if (mathEnabled) {
              dollarMathContainers.push({ from, to });
            }
          }
          return true;
        }

        if (name === 'LinkReference') {
          if (!visitedNodeKeys.has(nodeKey) && footnotesEnabled) {
            visitedNodeKeys.add(nodeKey);
            const rLine = doc.lineAt(from);
            if (rLine.number >= visStartLine && rLine.number <= visEndLine) {
              const slice = doc.sliceString(from, rLine.to);
              const fnDefMatch = slice.match(/^(\[\^[^\s\[\]]+\]:)/);
              if (fnDefMatch) {
                addLineClass(rLine.number, 'cm-live-footnote-def-line');
                markRanges.push(
                  Decoration.mark({ class: 'cm-live-footnote-label' }).range(
                    from,
                    from + fnDefMatch[1].length
                  )
                );
              }
            }
          }
          return false;
        }

        // Skip inline delimiter collapsing if inside an expanded raw Table
        if (isInsideActiveRawTable(from, to)) {
          return false;
        }

        // 10. Inline constructs (§6.1–§6.12)

        // §6.1 Backslash Escape (`\*` -> `*`)
        if (name === 'Escape') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            inlineProtectedForDollarMath.push([from, to]);
            if (!isCaretInSpan(from, to) && to - from >= 2) {
              pendingReplaces.push({
                from,
                to: from + 1,
                deco: Decoration.replace({}),
              });
            }
          }
          return false;
        }

        // §6.12 HardBreak (`\` before `\n` — NEVER replace the `\n` itself!)
        if (name === 'HardBreak') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const hbLine = doc.lineAt(from);
            if (
              hbLine.number >= visStartLine &&
              hbLine.number <= visEndLine &&
              !isCaretOnLine(hbLine.number) &&
              doc.sliceString(from, from + 1) === '\\'
            ) {
              pendingReplaces.push({
                from,
                to: from + 1,
                deco: Decoration.replace({}),
              });
            }
          }
          return false;
        }

        // §6.2 Entity (`&copy;`, `&#8594;`, `&#x1F980;`)
        if (name === 'Entity') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            inlineProtectedForDollarMath.push([from, to]);
            if (!isCaretInSpan(from, to) && to > from) {
              const rawEntity = doc.sliceString(from, to);
              const decoded = decodeHtmlEntity(rawEntity);
              if (decoded !== null) {
                pendingReplaces.push({
                  from,
                  to,
                  deco: Decoration.replace({
                    widget: new DecodedEntityWidget(decoded, from),
                  }),
                });
              }
            }
          }
          return false;
        }

        // §6.3 Code spans (`InlineCode` + `CodeMark` arbitrary backtick count & multi-line)
        // + Backtick inline math `` $`...`$ `` when `mathEnabled`
        if (name === 'InlineCode') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const openMark = nodeRef.node.firstChild;
            const closeMark = nodeRef.node.lastChild;
            if (
              openMark &&
              closeMark &&
              openMark.from !== closeMark.from &&
              openMark.name === 'CodeMark' &&
              closeMark.name === 'CodeMark'
            ) {
              // Check backtick math `$` + InlineCode + `$` (unescaped leading `$`)
              let isBacktickMath = false;
              if (
                mathEnabled &&
                from > 0 &&
                to < doc.length &&
                doc.sliceString(from - 1, from) === '$' &&
                doc.sliceString(to, to + 1) === '$' &&
                doc.lineAt(from - 1).number === doc.lineAt(to + 1).number
              ) {
                let backslashes = 0;
                let scan = from - 2;
                const lineStart = doc.lineAt(from - 1).from;
                while (scan >= lineStart && doc.sliceString(scan, scan + 1) === '\\') {
                  backslashes++;
                  scan--;
                }
                if (backslashes % 2 === 0) {
                  const expr = doc.sliceString(openMark.to, closeMark.from).trim();
                  if (expr.length > 0) {
                    isBacktickMath = true;
                    inlineProtectedForDollarMath.push([from - 1, to + 1]);
                    if (!isCaretInSpan(from - 1, to + 1)) {
                      pendingReplaces.push({
                        from: from - 1,
                        to: to + 1,
                        deco: Decoration.replace({
                          widget: new KatexMathWidget(expr, false, from - 1),
                        }),
                      });
                    }
                  }
                }
              }

              if (!isBacktickMath) {
                inlineProtectedForDollarMath.push([from, to]);
                let contentFrom = openMark.to;
                let contentTo = closeMark.from;
                // CommonMark §6.3 single leading & trailing guard space stripping
                if (contentTo - contentFrom >= 2) {
                  const inner = doc.sliceString(contentFrom, contentTo);
                  if (
                    inner.startsWith(' ') &&
                    inner.endsWith(' ') &&
                    !/^ +$/.test(inner)
                  ) {
                    contentFrom += 1;
                    contentTo -= 1;
                  }
                }

                if (contentTo > contentFrom) {
                  markRanges.push(
                    Decoration.mark({ class: 'cm-live-inline-code' }).range(
                      contentFrom,
                      contentTo
                    )
                  );
                }
                if (!isCaretInSpan(from, to) && contentTo >= contentFrom) {
                  pendingReplaces.push({
                    from: openMark.from,
                    to: contentFrom,
                    deco: Decoration.replace({}),
                  });
                  pendingReplaces.push({
                    from: contentTo,
                    to: closeMark.to,
                    deco: Decoration.replace({}),
                  });
                }
              }
            }
          }
          return false;
        }

        // §6.10 Raw inline HTML — protect from `$...$` math
        if (
          name === 'HTMLTag' ||
          name === 'Comment' ||
          name === 'ProcessingInstruction'
        ) {
          inlineProtectedForDollarMath.push([from, to]);
          return false;
        }

        // §6.4 StrongEmphasis & Emphasis (nesting-friendly: returns `true` for children!)
        if (name === 'StrongEmphasis' || name === 'Emphasis') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const openMark = nodeRef.node.firstChild;
            const closeMark = nodeRef.node.lastChild;
            const cls = name === 'StrongEmphasis' ? 'cm-live-bold' : 'cm-live-italic';
            if (
              openMark &&
              closeMark &&
              openMark.from !== closeMark.from &&
              openMark.name === 'EmphasisMark' &&
              closeMark.name === 'EmphasisMark' &&
              closeMark.from > openMark.to
            ) {
              markRanges.push(
                Decoration.mark({ class: cls }).range(openMark.to, closeMark.from)
              );
              if (!isCaretInSpan(from, to)) {
                pendingReplaces.push({
                  from: openMark.from,
                  to: openMark.to,
                  deco: Decoration.replace({}),
                });
                pendingReplaces.push({
                  from: closeMark.from,
                  to: closeMark.to,
                  deco: Decoration.replace({}),
                });
              }
            }
          }
          return true;
        }

        // §6.5 Strikethrough (`~~text~~` & `~text~`, GFM only, nesting-friendly!)
        if (name === 'Strikethrough' && isGfm) {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const openMark = nodeRef.node.firstChild;
            const closeMark = nodeRef.node.lastChild;
            if (
              openMark &&
              closeMark &&
              openMark.from !== closeMark.from &&
              openMark.name === 'StrikethroughMark' &&
              closeMark.name === 'StrikethroughMark' &&
              closeMark.from > openMark.to
            ) {
              markRanges.push(
                Decoration.mark({ class: 'cm-live-strike' }).range(
                  openMark.to,
                  closeMark.from
                )
              );
              if (!isCaretInSpan(from, to)) {
                pendingReplaces.push({
                  from: openMark.from,
                  to: openMark.to,
                  deco: Decoration.replace({}),
                });
                pendingReplaces.push({
                  from: closeMark.from,
                  to: closeMark.to,
                  deco: Decoration.replace({}),
                });
              }
            }
          }
          return true;
        }

        // §6.6 Links (inline balanced-parens, `<...>`, nested inline formatting, & full/collapsed/shortcut reference links)
        // + Modular Footnotes `[^1]`
        if (name === 'Link') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            const classified = classifyAstLinkNode(
              doc,
              nodeRef.node,
              refMap,
              footnotesEnabled
            );

            if (classified.kind === 'link') {
              // Protect the URL/ref tail `[closeBracketFrom, to]` from `$...$` math
              inlineProtectedForDollarMath.push([classified.closeBracketFrom, to]);

              const titleAttr = classified.title
                ? `${classified.title} (${classified.href} — Ctrl+Click)`
                : `${classified.href} (Ctrl+Click)`;

              if (classified.textTo > classified.textFrom) {
                markRanges.push(
                  Decoration.mark({
                    class: 'cm-live-link',
                    attributes: {
                      'data-link-href': classified.href,
                      title: titleAttr,
                    },
                  }).range(classified.textFrom, classified.textTo)
                );
              }

              if (!isCaretInSpan(from, to)) {
                pendingReplaces.push({
                  from,
                  to: classified.textFrom,
                  deco: Decoration.replace({}),
                });
                pushLineSplitReplaces(
                  pendingReplaces,
                  doc,
                  classified.closeBracketFrom,
                  to
                );
              }
              return true;
            }

            if (classified.kind === 'footnote-ref') {
              inlineProtectedForDollarMath.push([from, to]);
              if (!isCaretInSpan(from, to)) {
                pendingReplaces.push({
                  from,
                  to,
                  deco: Decoration.replace({
                    widget: new FootnoteRefWidget(classified.id, from),
                  }),
                });
                return false;
              }
              return false;
            }

            // Unresolved `[...]` (e.g. `[spazio inseparabile]` or `[^1]` when footnotes off):
            // neutralize Lezer's `Link/...` underline so literal brackets look like normal prose!
            if (to > from) {
              markRanges.push(
                Decoration.mark({ class: 'cm-live-plain-brackets' }).range(from, to)
              );
            }
          }
          return true;
        }

        // Protect Link/Reference URLs and Titles from `$...$` scanning & skip inner children when link tail is collapsed
        if (name === 'LinkLabel' || name === 'LinkTitle') {
          inlineProtectedForDollarMath.push([from, to]);
          return false;
        }

        // §6.7 Images (CommonMark & GFM core): caret-aware ImageWidget with local relative path resolution & clean fallback badge
        if (name === 'Image') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            inlineProtectedForDollarMath.push([from, to]);
            if (
              doc.lineAt(from).number === doc.lineAt(to).number &&
              to > from &&
              !isCaretInSpan(from, to)
            ) {
              const classifiedImg = classifyAstImageNode(doc, nodeRef.node, refMap);
              if (classifiedImg.kind === 'image') {
                pendingReplaces.push({
                  from,
                  to,
                  deco: Decoration.replace({
                    widget: new ImageWidget(
                      classifiedImg.alt,
                      classifiedImg.rawUrl,
                      classifiedImg.title,
                      from,
                      liveCfg.currentFilePath,
                      classifiedImg.isStandalone
                    ),
                  }),
                });
              }
            }
          }
          return false;
        }

        // §6.8 Standard Autolinks (`<https://...>`, `<user@host>`)
        if (name === 'Autolink') {
          if (!visitedNodeKeys.has(nodeKey)) {
            visitedNodeKeys.add(nodeKey);
            inlineProtectedForDollarMath.push([from, to]);
            if (to - from > 2) {
              const rawInner = doc.sliceString(from + 1, to - 1);
              const href = normalizeBareOrAngleAutolinkHref(rawInner);
              if (href) {
                markRanges.push(
                  Decoration.mark({
                    class: 'cm-live-link',
                    attributes: {
                      'data-link-href': href,
                      title: `${href} (Ctrl+Click)`,
                    },
                  }).range(from + 1, to - 1)
                );
              }
              if (!isCaretInSpan(from, to)) {
                pendingReplaces.push({
                  from,
                  to: from + 1,
                  deco: Decoration.replace({}),
                });
                pendingReplaces.push({
                  from: to - 1,
                  to,
                  deco: Decoration.replace({}),
                });
              }
            }
          }
          return false;
        }

        // §6.9 GFM Extended Bare Autolinks (`www.`, `https://`, bare email, `xmpp:`)
        if (name === 'URL') {
          inlineProtectedForDollarMath.push([from, to]);
          const pName = nodeRef.node.parent?.name;
          if (
            isGfm &&
            !visitedNodeKeys.has(nodeKey) &&
            pName !== 'Link' &&
            pName !== 'Image' &&
            pName !== 'Autolink' &&
            pName !== 'LinkReference' &&
            to > from
          ) {
            visitedNodeKeys.add(nodeKey);
            const rawBare = doc.sliceString(from, to);
            const href = normalizeBareOrAngleAutolinkHref(rawBare);
            if (href) {
              markRanges.push(
                Decoration.mark({
                  class: 'cm-live-link',
                  attributes: {
                    'data-link-href': href,
                    title: `${href} (Ctrl+Click)`,
                  },
                }).range(from, to)
              );
            }
          }
          return false;
        }

        return true;
      },
    });
  }

  // Apply Blockquote line classes across all covered lines (including lazy continuation lines & nested levels)
  for (const [ln, depth] of blockquoteDepthByLine.entries()) {
    addLineClass(ln, 'cm-live-blockquote-line');
    if (depth >= 2) {
      addLineClass(ln, `cm-live-blockquote-d${Math.min(depth, 4)}`);
    }
  }

  // Collect AST-scoped inline `$...$` KaTeX math spans when `mathEnabled` is active
  if (enabled && mathEnabled && dollarMathContainers.length > 0) {
    inlineProtectedForDollarMath.sort((a, b) => a[0] - b[0]);
    for (const container of dollarMathContainers) {
      const mathSpans = collectInlineDollarMathInContainer(
        doc,
        container.from,
        container.to,
        inlineProtectedForDollarMath
      );
      for (const ms of mathSpans) {
        if (
          isCoveredByBlockReplace(ms.from, ms.to) ||
          isInsideActiveRawTable(ms.from, ms.to)
        ) {
          continue;
        }
        // Check if visible
        if (!coalescedVis.some((v) => ms.from <= v.to && ms.to >= v.from)) {
          continue;
        }
        if (!isCaretInSpan(ms.from, ms.to)) {
          pendingReplaces.push({
            from: ms.from,
            to: ms.to,
            deco: Decoration.replace({
              widget: new KatexMathWidget(ms.latex, false, ms.from),
            }),
          });
        }
      }
    }
  }

  // Filter `pendingReplaces` so outer atomic replacements (e.g. `$a *b* c$` or `[txt](url\(x\))`)
  // always win over any inner overlapping `Decoration.replace` (preventing RangeSet replace collisions).
  pendingReplaces.sort((a, b) => {
    if (a.from !== b.from) return a.from - b.from;
    return (b.to - b.from) - (a.to - a.from);
  });

  const safeReplaces: Range<Decoration>[] = [];
  let lastReplaceTo = -1;
  for (const pr of pendingReplaces) {
    if (pr.to <= pr.from) continue;
    if (pr.from < lastReplaceTo) {
      continue;
    }
    safeReplaces.push(pr.deco.range(pr.from, pr.to));
    lastReplaceTo = pr.to;
  }

  // Combine line decorations + mark decorations (excluding marks strictly inside replaced widgets) + safe replace decorations
  const finalRanges: Range<Decoration>[] = [];
  for (const [ln, classSet] of lineClassMap.entries()) {
    if (classSet.size === 0) continue;
    const l = doc.line(ln);
    const attrs = lineAttrMap.get(ln);
    finalRanges.push(
      Decoration.line({
        class: Array.from(classSet).join(' '),
        ...(attrs ? { attributes: attrs } : {}),
      }).range(l.from)
    );
  }
  for (const mr of markRanges) {
    if (safeReplaces.some((sr) => mr.from >= sr.from && mr.to <= sr.to)) {
      continue;
    }
    finalRanges.push(mr);
  }
  for (const rr of safeReplaces) {
    finalRanges.push(rr);
  }

  return finalRanges.length > 0 ? Decoration.set(finalRanges, true) : Decoration.none;
}

export function createLivePreviewPlugin() {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = buildInlineAndLineLiveDecorations(
          view.state,
          view.visibleRanges
        );
      }
      update(update: ViewUpdate) {
        const hasRefreshEffect = update.transactions.some((tr) =>
          tr.effects.some((e) => e.is(refreshLivePreviewEffect))
        );
        if (
          update.docChanged ||
          update.selectionSet ||
          update.viewportChanged ||
          hasRefreshEffect ||
          syntaxTree(update.startState) !== syntaxTree(update.state)
        ) {
          this.decorations = buildInlineAndLineLiveDecorations(
            update.view.state,
            update.view.visibleRanges
          );
        }
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );
}

const editorCtrlClickLinkHandler = EditorView.domEventHandlers({
  mousedown(e: MouseEvent, view: EditorView) {
    if (e.button !== 0 || !(e.ctrlKey || e.metaKey)) {
      return false;
    }

    const targetEl = e.target as HTMLElement | null;
    const domLinkEl = targetEl?.closest('[data-link-href]') as HTMLElement | null;
    const domHref = domLinkEl?.getAttribute('data-link-href')?.trim() ?? '';
    if (domHref) {
      e.preventDefault();
      e.stopPropagation();
      activateEditorLinkHref(view, domHref);
      return true;
    }

    const pos = view.posAtCoords({ x: e.clientX, y: e.clientY });
    if (pos !== null) {
      const liveCfg = view.state.facet(livePreviewConfigFacet);
      const astHref = resolveLinkHrefAtPos(view.state, pos, liveCfg.markdownConfig);
      if (astHref) {
        e.preventDefault();
        e.stopPropagation();
        activateEditorLinkHref(view, astHref);
        return true;
      }
    }

    return false;
  },
});

export function createLivePreviewExtensions(
  enabled: boolean,
  currentTheme: ThemeMode,
  cfg: MarkdownConfig,
  currentFilePath: string | null = null
): Extension {
  const extensions: Extension[] = [
    livePreviewConfigFacet.of({
      enabled,
      theme: currentTheme,
      markdownConfig: cfg,
      currentFilePath,
    }),
    createLivePreviewPlugin(),
    editorCtrlClickLinkHandler,
  ];
  if (enabled) {
    extensions.unshift(blockLivePreviewField);
  }
  return extensions;
}
