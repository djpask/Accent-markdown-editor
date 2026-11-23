import katex from 'katex';
import 'katex/dist/katex.min.css';
import { renderMermaidDiagram } from './ipc';
import type { ThemeMode } from './types';

const mermaidSvgCache = new Map<string, string>();
const mermaidInFlight = new Set<string>();

export function renderLatexToHtml(latex: string, displayMode: boolean): string {
  try {
    return katex.renderToString(latex.trim(), {
      displayMode,
      throwOnError: false,
      strict: false,
    });
  } catch {
    return `<code class="katex-fallback">${latex}</code>`;
  }
}

export function getOrFetchMermaidSvg(
  code: string,
  theme: ThemeMode,
  onLoaded?: (svg: string) => void
): string | null {
  const normalized = code.trim();
  if (!normalized) return null;
  const key = `${theme}::${normalized}`;
  const cached = mermaidSvgCache.get(key);
  if (cached) {
    return cached;
  }
  if (!mermaidInFlight.has(key)) {
    mermaidInFlight.add(key);
    void renderMermaidDiagram(normalized, theme)
      .then((res) => {
        mermaidSvgCache.set(key, res.svg);
        mermaidInFlight.delete(key);
        onLoaded?.(res.svg);
      })
      .catch(() => {
        mermaidInFlight.delete(key);
      });
  }
  return null;
}

/**
 * Post-processes `comrak` preview HTML to render display (`$$...$$`, fenced `math`) and inline
 * (`$...$`, `` $`...`$ ``) KaTeX math when `mathEnabled` is active, while preserving `<pre>`,
 * `<code>`, and `<svg>` blocks untouched.
 */
export function renderMathInPreviewHtml(html: string, mathEnabled = true): string {
  if (!mathEnabled) {
    return html;
  }

  const decodeEntities = (raw: string) =>
    raw
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'")
      .replace(/&apos;/g, "'")
      .replace(/&amp;/g, '&');

  // 1. Render comrak fenced ```math blocks (<pre><code class="language-math" data-math-style="display">...</code></pre>)
  //    and backtick inline math $`...`$ (<code data-math-style="inline">...</code>) before splitting protected code tags.
  let stage1 = html.replace(
    /<pre[^>]*><code[^>]*\bdata-math-style="display"[^>]*>([\s\S]*?)<\/code><\/pre>/gi,
    (_match, expr: string) =>
      `<div class="katex-display-block" data-testid="katex-display">${renderLatexToHtml(decodeEntities(expr), true)}</div>`
  );
  stage1 = stage1.replace(
    /<code[^>]*\bdata-math-style="inline"[^>]*>([\s\S]*?)<\/code>/gi,
    (_match, expr: string) =>
      `<span class="katex-inline-span" data-testid="katex-inline">${renderLatexToHtml(decodeEntities(expr), false)}</span>`
  );

  // 2. Split remaining HTML into protected tags (<pre...>, <code...>, <svg...>) vs normal HTML prose
  const protectedRegex = /(<pre[\s\S]*?<\/pre>|<code[\s\S]*?<\/code>|<svg[\s\S]*?<\/svg>)/gi;
  const parts = stage1.split(protectedRegex);

  return parts
    .map((part, idx) => {
      if (idx % 2 === 1) {
        return part;
      }

      // 3. Replace comrak dollar math spans (<span data-math-style="display|inline">)
      let processed = part.replace(
        /<span[^>]*\bdata-math-style="display"[^>]*>([\s\S]*?)<\/span>/gi,
        (_match, expr: string) =>
          `<div class="katex-display-block" data-testid="katex-display">${renderLatexToHtml(decodeEntities(expr), true)}</div>`
      );
      processed = processed.replace(
        /<span[^>]*\bdata-math-style="inline"[^>]*>([\s\S]*?)<\/span>/gi,
        (_match, expr: string) =>
          `<span class="katex-inline-span" data-testid="katex-inline">${renderLatexToHtml(decodeEntities(expr), false)}</span>`
      );

      // 4. Multi-paragraph display math fallback ($$...$$ spanning blank lines)
      processed = processed.replace(/\$\$([\s\S]+?)\$\$/g, (_match, expr: string) => {
        const clean = decodeEntities(
          expr.replace(/<\/?p>/gi, '\n').replace(/<br\s*\/?>/gi, '\n')
        );
        return `<div class="katex-display-block" data-testid="katex-display">${renderLatexToHtml(clean, true)}</div>`;
      });

      return processed;
    })
    .join('');
}
