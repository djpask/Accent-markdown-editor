/**
 * Converts a JavaScript UTF-16 code-unit offset (`0..text.length`, used by CodeMirror 6)
 * into a Unicode scalar (`char`) index (`0..len_chars`, used by `ropey::Rope` in Rust).
 */
export function utf16ToCharOffset(text: string, utf16Pos: number): number {
  const limit = Math.max(0, Math.min(utf16Pos, text.length));
  let i = 0;
  let chars = 0;
  while (i < limit) {
    const c1 = text.charCodeAt(i);
    if (c1 >= 0xd800 && c1 <= 0xdbff && i + 1 < text.length) {
      const c2 = text.charCodeAt(i + 1);
      if (c2 >= 0xdc00 && c2 <= 0xdfff) {
        if (i + 1 >= limit) {
          break;
        }
        i += 2;
        chars += 1;
        continue;
      }
    }
    i += 1;
    chars += 1;
  }
  return chars;
}

/**
 * Converts a Unicode scalar (`char`) index (`0..len_chars`, used by `ropey::Rope` in Rust)
 * into a JavaScript UTF-16 code-unit offset (`0..text.length`, used by CodeMirror 6).
 */
export function charToUtf16Offset(text: string, charPos: number): number {
  if (charPos <= 0) return 0;
  let i = 0;
  let chars = 0;
  const len = text.length;
  while (i < len && chars < charPos) {
    const c1 = text.charCodeAt(i);
    if (c1 >= 0xd800 && c1 <= 0xdbff && i + 1 < len) {
      const c2 = text.charCodeAt(i + 1);
      if (c2 >= 0xdc00 && c2 <= 0xdfff) {
        i += 2;
        chars += 1;
        continue;
      }
    }
    i += 1;
    chars += 1;
  }
  return i;
}
