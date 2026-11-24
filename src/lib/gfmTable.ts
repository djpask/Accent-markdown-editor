import type { EditorState } from '@codemirror/state';
import { ensureSyntaxTree, syntaxTree } from '@codemirror/language';
import { renderLatexIntoElement } from './mathAndMermaid';

export type TableColumnAlign = 'left' | 'center' | 'right' | null;

export interface ParsedTableCell {
  /** Trimmed raw Markdown content of the cell. */
  rawText: string;
  /** UTF-16 offset of this cell's caret target relative to `ParsedGfmTable.from`. */
  relOffset: number;
}

export interface ParsedGfmTable {
  /** Start of the first line containing the table (`startLine.from`). */
  from: number;
  /** End of the last line containing the table (`endLine.to`). */
  to: number;
  /** 1-based start line number in `EditorState.doc`. */
  startLineNum: number;
  /** 1-based end line number in `EditorState.doc`. */
  endLineNum: number;
  /** 1-based line number of `TableHeader`. */
  headerLineNum: number;
  /** 1-based line number of `TableDelimiter`. */
  delimiterLineNum: number;
  /** Nesting depth inside `Blockquote` containers (`0` for top-level tables). */
  blockquoteDepth: number;
  /** Per-column alignments fixed by `TableDelimiter` (`length === headers.length`). */
  alignments: TableColumnAlign[];
  /** Header cells (`TableHeader`). */
  headers: ParsedTableCell[];
  /** Body rows (`TableRow`), each normalized (padded/truncated) to `headers.length` cells. */
  rows: ParsedTableCell[][];
}

const ASCII_PUNCTUATION = new Set<string>(
  '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'.split('')
);

let entityDecoderEl: HTMLTextAreaElement | null = null;

function isHorizontalWs(ch: string | undefined): boolean {
  return ch === ' ' || ch === '\t';
}

function isUnicodeWordChar(ch: string | undefined): boolean {
  if (!ch) return false;
  return /[\p{L}\p{N}]/u.test(ch);
}

/**
 * Parses a single GFM `§4.10` delimiter cell (`:---`, `:---:`, `---:`, `---`).
 */
export function parseDelimiterCellAlignment(rawCell: string): TableColumnAlign {
  const trimmed = rawCell.trim();
  if (!/^:?-+:?$/.test(trimmed)) {
    return null;
  }
  const startsColon = trimmed.startsWith(':');
  const endsColon = trimmed.endsWith(':');
  if (startsColon && endsColon) return 'center';
  if (endsColon) return 'right';
  if (startsColon) return 'left';
  return null;
}

/**
 * Splits a single GFM `§4.10` table row (`TableHeader`, `TableDelimiter`, or `TableRow`)
 * into cells while respecting:
 * - Optional leading and trailing unescaped outer pipes (`|`);
 * - Escaped pipes `\|` (odd count of preceding `\`);
 * - Unescaped pipes `|` inside matched single- or multi-backtick code spans (e.g. `` `a | b` ``, ```` `` `x | y` `` ````).
 */
export function splitGfmTableRow(
  rowText: string,
  rowDocStart: number,
  tableBlockFrom: number
): { cells: ParsedTableCell[]; fallbackEndRelOffset: number } {
  let firstNonWs = 0;
  while (firstNonWs < rowText.length && isHorizontalWs(rowText[firstNonWs])) {
    firstNonWs++;
  }
  let lastNonWs = rowText.length;
  while (lastNonWs > firstNonWs && isHorizontalWs(rowText[lastNonWs - 1])) {
    lastNonWs--;
  }

  if (firstNonWs >= lastNonWs) {
    const emptyRel = Math.max(0, rowDocStart + firstNonWs - tableBlockFrom);
    return { cells: [], fallbackEndRelOffset: emptyRel };
  }

  const innerStart = rowText[firstNonWs] === '|' ? firstNonWs + 1 : firstNonWs;
  let innerEnd = lastNonWs;
  if (innerEnd - 1 >= innerStart && rowText[innerEnd - 1] === '|') {
    let backslashes = 0;
    let k = innerEnd - 2;
    while (k >= innerStart && rowText[k] === '\\') {
      backslashes++;
      k--;
    }
    if (backslashes % 2 === 0) {
      innerEnd = innerEnd - 1;
    }
  }

  const segments: Array<[number, number]> = [];
  let segStart = innerStart;
  let i = innerStart;
  let escaped = false;

  while (i < innerEnd) {
    const ch = rowText[i];
    if (escaped) {
      escaped = false;
      i++;
      continue;
    }
    if (ch === '\\') {
      escaped = true;
      i++;
      continue;
    }
    if (ch === '`') {
      let tickLen = 0;
      while (i + tickLen < innerEnd && rowText[i + tickLen] === '`') {
        tickLen++;
      }
      let j = i + tickLen;
      let matchedCloseEnd = -1;
      while (j < innerEnd) {
        if (rowText[j] === '`') {
          let closeLen = 0;
          while (j + closeLen < innerEnd && rowText[j + closeLen] === '`') {
            closeLen++;
          }
          if (closeLen === tickLen) {
            matchedCloseEnd = j + closeLen;
            break;
          }
          j += closeLen;
        } else {
          j++;
        }
      }
      i = matchedCloseEnd !== -1 ? matchedCloseEnd : i + tickLen;
      continue;
    }
    if (ch === '|') {
      segments.push([segStart, i]);
      segStart = i + 1;
      i++;
      continue;
    }
    i++;
  }
  segments.push([segStart, innerEnd]);

  const cells: ParsedTableCell[] = segments.map(([sStart, sEnd]) => {
    let cStart = sStart;
    while (cStart < sEnd && isHorizontalWs(rowText[cStart])) {
      cStart++;
    }
    let cEnd = sEnd;
    while (cEnd > cStart && isHorizontalWs(rowText[cEnd - 1])) {
      cEnd--;
    }
    if (cStart < cEnd) {
      return {
        rawText: rowText.slice(cStart, cEnd),
        relOffset: Math.max(0, rowDocStart + cStart - tableBlockFrom),
      };
    }
    const emptyCaretInRow = Math.min(sStart + 1, sEnd);
    return {
      rawText: '',
      relOffset: Math.max(0, rowDocStart + emptyCaretInRow - tableBlockFrom),
    };
  });

  const fallbackEndRelOffset = Math.max(0, rowDocStart + innerEnd - tableBlockFrom);
  return { cells, fallbackEndRelOffset };
}

/**
 * Traverses Lezer's `syntaxTree(state)` to extract all GFM `§4.10` `Table` AST blocks
 * with normalized column counts, alignments, and relative caret offsets per cell.
 */
export function extractGfmTablesFromState(state: EditorState): ParsedGfmTable[] {
  const tree = ensureSyntaxTree(state, state.doc.length, 50) ?? syntaxTree(state);
  const doc = state.doc;
  const tables: ParsedGfmTable[] = [];

  tree.iterate({
    enter(nodeRef) {
      if (nodeRef.name !== 'Table') {
        return true;
      }

      const startLine = doc.lineAt(nodeRef.from);
      const endLine = doc.lineAt(nodeRef.to);
      const blockFrom = startLine.from;
      const blockTo = endLine.to;

      let blockquoteDepth = 0;
      let parent = nodeRef.node.parent;
      while (parent) {
        if (parent.name === 'Blockquote') {
          blockquoteDepth++;
        }
        parent = parent.parent;
      }

      let headerFrom = -1;
      let headerTo = -1;
      let delimFrom = -1;
      let delimTo = -1;
      const bodyRowRanges: Array<[number, number]> = [];

      let child = nodeRef.node.firstChild;
      while (child) {
        if (child.name === 'TableHeader') {
          headerFrom = child.from;
          headerTo = child.to;
        } else if (child.name === 'TableDelimiter') {
          delimFrom = child.from;
          delimTo = child.to;
        } else if (child.name === 'TableRow') {
          bodyRowRanges.push([child.from, child.to]);
        }
        child = child.nextSibling;
      }

      if (headerFrom < 0 || delimFrom < 0) {
        return false;
      }

      const headerSlice = doc.sliceString(headerFrom, headerTo);
      const { cells: headers } = splitGfmTableRow(headerSlice, headerFrom, blockFrom);
      const colCount = headers.length;
      if (colCount === 0) {
        return false;
      }

      const delimSlice = doc.sliceString(delimFrom, delimTo);
      const { cells: delimCells } = splitGfmTableRow(delimSlice, delimFrom, blockFrom);
      const alignments: TableColumnAlign[] = [];
      for (let c = 0; c < colCount; c++) {
        alignments.push(delimCells[c] ? parseDelimiterCellAlignment(delimCells[c].rawText) : null);
      }

      const rows: ParsedTableCell[][] = [];
      for (const [rFrom, rTo] of bodyRowRanges) {
        const rowSlice = doc.sliceString(rFrom, rTo);
        const { cells: rawCells, fallbackEndRelOffset } = splitGfmTableRow(
          rowSlice,
          rFrom,
          blockFrom
        );
        if (rawCells.length >= colCount) {
          rows.push(rawCells.slice(0, colCount));
        } else {
          const padded = rawCells.slice();
          while (padded.length < colCount) {
            padded.push({
              rawText: '',
              relOffset: fallbackEndRelOffset,
            });
          }
          rows.push(padded);
        }
      }

      tables.push({
        from: blockFrom,
        to: blockTo,
        startLineNum: startLine.number,
        endLineNum: endLine.number,
        headerLineNum: doc.lineAt(headerFrom).number,
        delimiterLineNum: doc.lineAt(delimFrom).number,
        blockquoteDepth,
        alignments,
        headers,
        rows,
      });

      return false;
    },
  });

  return tables;
}

/**
 * Unescapes `\|` -> `|` inside a GFM `§4.10` table cell (odd preceding backslash count),
 * including inside code spans per GFM `0.29-gfm` `§4.10` Example 200, while leaving all
 * other backslash sequences intact for standard `§6.1` / `§6.3` inline rules.
 */
export function unescapeGfmTablePipes(cellText: string): string {
  let out = '';
  let i = 0;
  while (i < cellText.length) {
    if (cellText[i] === '\\') {
      let run = 0;
      while (i + run < cellText.length && cellText[i + run] === '\\') {
        run++;
      }
      if (cellText[i + run] === '|' && run % 2 === 1) {
        out += '\\'.repeat(run - 1) + '|';
        i += run + 1;
        continue;
      }
      out += '\\'.repeat(run);
      i += run;
      continue;
    }
    out += cellText[i];
    i++;
  }
  return out;
}

function sanitizeLinkHref(rawHref: string): string | null {
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

function decodeCharacterReferenceAt(
  text: string,
  pos: number
): { decoded: string; length: number } | null {
  const rest = text.slice(pos, pos + 40);
  const decMatch = rest.match(/^&#([0-9]{1,7});/);
  if (decMatch) {
    const cp = Number.parseInt(decMatch[1], 10);
    const safeCp =
      cp === 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)
        ? '\uFFFD'
        : String.fromCodePoint(cp);
    return { decoded: safeCp, length: decMatch[0].length };
  }

  const hexMatch = rest.match(/^&#[xX]([0-9a-fA-F]{1,6});/);
  if (hexMatch) {
    const cp = Number.parseInt(hexMatch[1], 16);
    const safeCp =
      cp === 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)
        ? '\uFFFD'
        : String.fromCodePoint(cp);
    return { decoded: safeCp, length: hexMatch[0].length };
  }

  const namedMatch = rest.match(/^&([a-zA-Z][a-zA-Z0-9]{1,31});/);
  if (namedMatch && typeof document !== 'undefined') {
    entityDecoderEl ??= document.createElement('textarea');
    entityDecoderEl.innerHTML = namedMatch[0];
    const decoded = entityDecoderEl.value;
    if (decoded !== namedMatch[0]) {
      return { decoded, length: namedMatch[0].length };
    }
  }

  return null;
}

function skipCodeSpanAt(text: string, pos: number): number {
  let tickLen = 0;
  while (pos + tickLen < text.length && text[pos + tickLen] === '`') {
    tickLen++;
  }
  let j = pos + tickLen;
  while (j < text.length) {
    if (text[j] === '`') {
      let closeLen = 0;
      while (j + closeLen < text.length && text[j + closeLen] === '`') {
        closeLen++;
      }
      if (closeLen === tickLen) {
        return j + closeLen;
      }
      j += closeLen;
    } else {
      j++;
    }
  }
  return pos + tickLen;
}

function parseInlineLinkAt(
  text: string,
  start: number
): { label: string; href: string; title: string; nextPos: number } | null {
  if (text[start] !== '[') return null;

  let i = start + 1;
  let bracketDepth = 1;
  while (i < text.length && bracketDepth > 0) {
    const ch = text[i];
    if (ch === '\\') {
      i += 2;
      continue;
    }
    if (ch === '`') {
      i = skipCodeSpanAt(text, i);
      continue;
    }
    if (ch === '[') {
      bracketDepth++;
    } else if (ch === ']') {
      bracketDepth--;
      if (bracketDepth === 0) break;
    }
    i++;
  }
  if (bracketDepth !== 0 || text[i] !== ']' || text[i + 1] !== '(') {
    return null;
  }

  const label = text.slice(start + 1, i);
  let p = i + 2;
  while (p < text.length && isHorizontalWs(text[p])) {
    p++;
  }

  let rawDest = '';
  if (text[p] === '<') {
    p++;
    const destStart = p;
    while (p < text.length && text[p] !== '>') {
      if (text[p] === '<' || text[p] === '\n') return null;
      if (text[p] === '\\') {
        p += 2;
        continue;
      }
      p++;
    }
    if (text[p] !== '>') return null;
    rawDest = unescapeAsciiBackslashes(text.slice(destStart, p));
    p++;
  } else {
    const destStart = p;
    let parenDepth = 0;
    while (p < text.length) {
      const ch = text[p];
      if (ch === '\\') {
        p += 2;
        continue;
      }
      if (isHorizontalWs(ch)) {
        break;
      }
      if (ch === '(') {
        parenDepth++;
      } else if (ch === ')') {
        if (parenDepth === 0) break;
        parenDepth--;
      }
      p++;
    }
    if (parenDepth !== 0) return null;
    rawDest = unescapeAsciiBackslashes(text.slice(destStart, p));
  }

  while (p < text.length && isHorizontalWs(text[p])) {
    p++;
  }

  let title = '';
  if (text[p] === '"' || text[p] === "'" || text[p] === '(') {
    const closeQuote = text[p] === '(' ? ')' : text[p];
    p++;
    const titleStart = p;
    while (p < text.length && text[p] !== closeQuote) {
      if (text[p] === '\\') {
        p += 2;
        continue;
      }
      p++;
    }
    if (text[p] !== closeQuote) return null;
    title = unescapeAsciiBackslashes(text.slice(titleStart, p));
    p++;
    while (p < text.length && isHorizontalWs(text[p])) {
      p++;
    }
  }

  if (text[p] !== ')') {
    return null;
  }

  const safeHref = sanitizeLinkHref(rawDest);
  if (!safeHref) {
    return null;
  }

  return {
    label,
    href: safeHref,
    title,
    nextPos: p + 1,
  };
}

function findMatchingDelim(
  text: string,
  contentStart: number,
  delimChar: '*' | '_' | '~',
  delimLen: number
): number {
  if (contentStart >= text.length || /\s/.test(text[contentStart])) {
    return -1;
  }

  let i = contentStart;
  while (i < text.length) {
    const ch = text[i];
    if (ch === '\\') {
      i += 2;
      continue;
    }
    if (ch === '`') {
      i = skipCodeSpanAt(text, i);
      continue;
    }
    if (ch === '[') {
      const maybeLink = parseInlineLinkAt(text, i);
      if (maybeLink) {
        i = maybeLink.nextPos;
        continue;
      }
    }
    if (ch === delimChar) {
      let run = 0;
      while (i + run < text.length && text[i + run] === delimChar) {
        run++;
      }
      if (run === delimLen && i > contentStart && !/\s/.test(text[i - 1])) {
        if (delimChar === '_' && isUnicodeWordChar(text[i + run])) {
          i += run;
          continue;
        }
        return i;
      }
      // Handle combined closing `***` / `___` runs (e.g. `**bold *italic***` or `*italic **bold***`):
      // leave `(3 - delimLen)` delimiters inside the slice so the nested emphasis closes cleanly.
      if (
        delimChar !== '~' &&
        (delimLen === 1 || delimLen === 2) &&
        run === 3 &&
        i > contentStart &&
        !/\s/.test(text[i - 1])
      ) {
        if (delimChar !== '_' || !isUnicodeWordChar(text[i + run])) {
          return i + (3 - delimLen);
        }
      }
      i += run;
      continue;
    }
    i++;
  }
  return -1;
}

function appendPlainTextWithExtendedAutolinks(plain: string, parent: HTMLElement): void {
  if (!plain) return;
  const autolinkRegex =
    /(^|[\s([*_~])((?:https?:\/\/|www\.)[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)+(?:\/[^\s<]*)?)/g;

  let cursor = 0;
  for (const match of plain.matchAll(autolinkRegex)) {
    const matchIndex = match.index ?? 0;
    const prefix = match[1];
    let urlPart = match[2];
    const urlStart = matchIndex + prefix.length;

    // GFM §6.9 trailing punctuation & unbalanced parenthesis trimming
    while (urlPart.length > 0) {
      const last = urlPart[urlPart.length - 1];
      if (/[?!.,:*_~]/.test(last)) {
        urlPart = urlPart.slice(0, -1);
        continue;
      }
      if (last === ')') {
        let opens = 0;
        let closes = 0;
        for (const c of urlPart) {
          if (c === '(') opens++;
          else if (c === ')') closes++;
        }
        if (closes > opens) {
          urlPart = urlPart.slice(0, -1);
          continue;
        }
      }
      break;
    }

    const normalizedHref = sanitizeLinkHref(
      urlPart.startsWith('www.') ? `http://${urlPart}` : urlPart
    );
    if (!normalizedHref || !urlPart) {
      continue;
    }

    if (urlStart > cursor) {
      parent.appendChild(document.createTextNode(plain.slice(cursor, urlStart)));
    }

    const a = document.createElement('a');
    a.className = 'cm-table-link';
    a.href = normalizedHref;
    a.title = `${normalizedHref} (Ctrl+Click)`;
    a.textContent = urlPart;
    parent.appendChild(a);

    cursor = urlStart + urlPart.length;
  }

  if (cursor < plain.length) {
    parent.appendChild(document.createTextNode(plain.slice(cursor)));
  }
}

function renderCellInlineSegment(
  text: string,
  parent: HTMLElement,
  mathEnabled: boolean
): void {
  let plainBuf = '';
  const flushPlain = () => {
    if (plainBuf.length > 0) {
      appendPlainTextWithExtendedAutolinks(plainBuf, parent);
      plainBuf = '';
    }
  };

  let pos = 0;
  while (pos < text.length) {
    const ch = text[pos];

    // 1. Backslash escapes (§6.1)
    if (ch === '\\') {
      if (pos + 1 < text.length && ASCII_PUNCTUATION.has(text[pos + 1])) {
        plainBuf += text[pos + 1];
        pos += 2;
      } else {
        plainBuf += '\\';
        pos += 1;
      }
      continue;
    }

    // 2. Backtick inline math ($`...`$) when mathEnabled
    if (mathEnabled && ch === '$' && text[pos + 1] === '`') {
      let tickLen = 0;
      while (pos + 1 + tickLen < text.length && text[pos + 1 + tickLen] === '`') {
        tickLen++;
      }
      let j = pos + 1 + tickLen;
      let closeTickStart = -1;
      while (j < text.length) {
        if (text[j] === '`') {
          let closeLen = 0;
          while (j + closeLen < text.length && text[j + closeLen] === '`') {
            closeLen++;
          }
          if (closeLen === tickLen && text[j + closeLen] === '$') {
            closeTickStart = j;
            break;
          }
          j += closeLen;
        } else {
          j++;
        }
      }
      if (closeTickStart !== -1) {
        const expr = text.slice(pos + 1 + tickLen, closeTickStart).trim();
        if (expr.length > 0) {
          flushPlain();
          const mathEl = document.createElement('span');
          mathEl.className = 'cm-table-math';
          mathEl.dataset.testid = 'cm-table-katex-inline';
          renderLatexIntoElement(expr, mathEl, false);
          parent.appendChild(mathEl);
          pos = closeTickStart + tickLen + 1;
          continue;
        }
      }
    }

    // 3. Multi-backtick code spans (§6.3)
    if (ch === '`') {
      let tickLen = 0;
      while (pos + tickLen < text.length && text[pos + tickLen] === '`') {
        tickLen++;
      }
      let j = pos + tickLen;
      let closeStart = -1;
      while (j < text.length) {
        if (text[j] === '`') {
          let closeLen = 0;
          while (j + closeLen < text.length && text[j + closeLen] === '`') {
            closeLen++;
          }
          if (closeLen === tickLen) {
            closeStart = j;
            break;
          }
          j += closeLen;
        } else {
          j++;
        }
      }
      if (closeStart !== -1) {
        flushPlain();
        let codeContent = text.slice(pos + tickLen, closeStart);
        if (
          codeContent.length >= 2 &&
          codeContent.startsWith(' ') &&
          codeContent.endsWith(' ') &&
          !/^ +$/.test(codeContent)
        ) {
          codeContent = codeContent.slice(1, -1);
        }
        const codeEl = document.createElement('code');
        codeEl.textContent = codeContent;
        parent.appendChild(codeEl);
        pos = closeStart + tickLen;
        continue;
      }
      plainBuf += text.slice(pos, pos + tickLen);
      pos += tickLen;
      continue;
    }

    // 4. Dollar inline math ($...$) — ONLY when mathEnabled === true
    if (
      mathEnabled &&
      ch === '$' &&
      text[pos + 1] !== '$' &&
      pos + 1 < text.length &&
      !/\s/.test(text[pos + 1]) &&
      (pos === 0 || text[pos - 1] !== '$')
    ) {
      let j = pos + 1;
      let closeDollar = -1;
      while (j < text.length) {
        if (text[j] === '\\') {
          j += 2;
          continue;
        }
        if (
          text[j] === '$' &&
          !/\s/.test(text[j - 1]) &&
          text[j + 1] !== '$' &&
          !/[0-9]/.test(text[j + 1] ?? '')
        ) {
          closeDollar = j;
          break;
        }
        j++;
      }
      if (closeDollar !== -1) {
        flushPlain();
        const expr = text.slice(pos + 1, closeDollar);
        const mathEl = document.createElement('span');
        mathEl.className = 'cm-table-math';
        mathEl.dataset.testid = 'cm-table-katex-inline';
        renderLatexIntoElement(expr, mathEl, false);
        parent.appendChild(mathEl);
        pos = closeDollar + 1;
        continue;
      }
    }

    // 5. HTML5 Entities & Numeric Character References (§6.2)
    if (ch === '&') {
      const decodedRef = decodeCharacterReferenceAt(text, pos);
      if (decodedRef) {
        plainBuf += decodedRef.decoded;
        pos += decodedRef.length;
        continue;
      }
    }

    // 6. Autolinks (§6.8), safe <br>/<kbd>/<sub>/<sup>, and XSS neutralization of raw HTML (§6.11)
    if (ch === '<') {
      const rest = text.slice(pos);

      const urlAuto = rest.match(/^<((?:https?:\/\/|mailto:|xmpp:)[^\s<>]+)>/i);
      if (urlAuto) {
        const safeHref = sanitizeLinkHref(urlAuto[1]);
        if (safeHref) {
          flushPlain();
          const a = document.createElement('a');
          a.className = 'cm-table-link';
          a.href = safeHref;
          a.title = `${safeHref} (Ctrl+Click)`;
          a.textContent = urlAuto[1].replace(/^mailto:/i, '');
          parent.appendChild(a);
          pos += urlAuto[0].length;
          continue;
        }
      }

      const emailAuto = rest.match(
        /^<([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)+)>/
      );
      if (emailAuto) {
        flushPlain();
        const a = document.createElement('a');
        a.className = 'cm-table-link';
        a.href = `mailto:${emailAuto[1]}`;
        a.title = `mailto:${emailAuto[1]} (Ctrl+Click)`;
        a.textContent = emailAuto[1];
        parent.appendChild(a);
        pos += emailAuto[0].length;
        continue;
      }

      const brMatch = rest.match(/^<br\s*\/?>/i);
      if (brMatch) {
        flushPlain();
        parent.appendChild(document.createElement('br'));
        pos += brMatch[0].length;
        continue;
      }

      const commentMatch = rest.match(/^<!--[\s\S]*?-->/);
      if (commentMatch) {
        pos += commentMatch[0].length;
        continue;
      }

      const safeInlineTag = rest.match(/^<(kbd|sub|sup)>([\s\S]*?)<\/\1>/i);
      if (safeInlineTag) {
        flushPlain();
        const tagEl = document.createElement(safeInlineTag[1].toLowerCase());
        renderCellInlineSegment(safeInlineTag[2], tagEl, mathEnabled);
        parent.appendChild(tagEl);
        pos += safeInlineTag[0].length;
        continue;
      }

      // All other raw HTML tags (<script>, <style>, <iframe>, <img onerror=...>, etc.)
      // remain inert plain text via Text nodes (zero XSS surface).
      plainBuf += '<';
      pos += 1;
      continue;
    }

    // 7. Inline Links [label](url "title") (§6.6)
    if (ch === '[') {
      const parsedLink = parseInlineLinkAt(text, pos);
      if (parsedLink) {
        flushPlain();
        const a = document.createElement('a');
        a.className = 'cm-table-link';
        a.href = parsedLink.href;
        a.title = parsedLink.title
          ? `${parsedLink.title} (${parsedLink.href})`
          : `${parsedLink.href} (Ctrl+Click)`;
        renderCellInlineSegment(parsedLink.label, a, mathEnabled);
        parent.appendChild(a);
        pos = parsedLink.nextPos;
        continue;
      }
    }

    // 8. Emphasis & Strong Emphasis (§6.4: ***, **, *, ___, __, _)
    if (ch === '*' || ch === '_') {
      let run = 0;
      while (pos + run < text.length && text[pos + run] === ch) {
        run++;
      }
      const prevChar = pos > 0 ? text[pos - 1] : undefined;
      const canOpen = ch === '*' || !isUnicodeWordChar(prevChar);

      if (canOpen) {
        if (run >= 3) {
          const close3 = findMatchingDelim(text, pos + 3, ch, 3);
          if (close3 !== -1) {
            flushPlain();
            const strongEl = document.createElement('strong');
            const emEl = document.createElement('em');
            renderCellInlineSegment(text.slice(pos + 3, close3), emEl, mathEnabled);
            strongEl.appendChild(emEl);
            parent.appendChild(strongEl);
            pos = close3 + 3;
            continue;
          }
        }
        if (run >= 2) {
          const openOffset = run === 2 ? 2 : 0;
          if (openOffset === 2) {
            const close2 = findMatchingDelim(text, pos + 2, ch, 2);
            if (close2 !== -1) {
              flushPlain();
              const strongEl = document.createElement('strong');
              renderCellInlineSegment(text.slice(pos + 2, close2), strongEl, mathEnabled);
              parent.appendChild(strongEl);
              pos = close2 + 2;
              continue;
            }
          }
        }
        if (run === 1) {
          const close1 = findMatchingDelim(text, pos + 1, ch, 1);
          if (close1 !== -1) {
            flushPlain();
            const emEl = document.createElement('em');
            renderCellInlineSegment(text.slice(pos + 1, close1), emEl, mathEnabled);
            parent.appendChild(emEl);
            pos = close1 + 1;
            continue;
          }
        }
      }

      plainBuf += text.slice(pos, pos + run);
      pos += run;
      continue;
    }

    // 9. GFM Strikethrough (§6.5: ~ or ~~, never 3+ tildes ~~~)
    if (ch === '~') {
      let run = 0;
      while (pos + run < text.length && text[pos + run] === '~') {
        run++;
      }
      if (run === 1 || run === 2) {
        const closeTilde = findMatchingDelim(text, pos + run, '~', run);
        if (closeTilde !== -1) {
          flushPlain();
          const delEl = document.createElement('del');
          renderCellInlineSegment(text.slice(pos + run, closeTilde), delEl, mathEnabled);
          parent.appendChild(delEl);
          pos = closeTilde + run;
          continue;
        }
      }
      plainBuf += text.slice(pos, pos + run);
      pos += run;
      continue;
    }

    plainBuf += ch;
    pos += 1;
  }

  flushPlain();
}

/**
 * Safely renders a GFM `§4.10` table cell's inline Markdown into `cellEl` (`<th>` or `<td>`)
 * using DOM node construction (`createElement` / `createTextNode`), preventing XSS via raw HTML.
 */
export function appendSafeGfmTableCellDom(
  rawCellMarkdown: string,
  cellEl: HTMLElement,
  mathEnabled: boolean
): void {
  if (!rawCellMarkdown) {
    return;
  }
  const normalized = unescapeGfmTablePipes(rawCellMarkdown);
  renderCellInlineSegment(normalized, cellEl, mathEnabled);
}
