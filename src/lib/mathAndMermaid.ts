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
 * Post-processes preview HTML to render display (`$$...$$`) and inline (`$...$`) KaTeX math
 * while preserving `<pre>`, `<code>`, and `<svg>` blocks untouched.
 */
export function renderMathInPreviewHtml(html: string): string {
  // Split HTML into protected tags (<pre...>, <code...>, <svg...>) vs normal HTML prose
  const protectedRegex = /(<pre[\s\S]*?<\/pre>|<code[\s\S]*?<\/code>|<svg[\s\S]*?<\/svg>)/gi;
  const parts = html.split(protectedRegex);

  return parts
    .map((part, idx) => {
      if (idx % 2 === 1) {
        return part;
      }
      // 1. Replace display math $$...$$
      let processed = part.replace(/\$\$([\s\S]+?)\$\$/g, (_match, expr: string) => {
        const clean = expr
          .replace(/<\/?p>/gi, '\n')
          .replace(/<br\s*\/?>/gi, '\n')
          .replace(/&lt;/g, '<')
          .replace(/&gt;/g, '>')
          .replace(/&amp;/g, '&');
        return `<div class="katex-display-block" data-testid="katex-display">${renderLatexToHtml(clean, true)}</div>`;
      });

      // 2. Replace inline math $...$ (non-empty, not starting/ending with space, single line)
      processed = processed.replace(
        /(^|[^\\$])\$([^\s$][^$\n]*?[^\s$]|[^\s$])\$/g,
        (_match, prefix: string, expr: string) => {
          const clean = expr
            .replace(/&lt;/g, '<')
            .replace(/&gt;/g, '>')
            .replace(/&amp;/g, '&');
          return `${prefix}<span class="katex-inline-span" data-testid="katex-inline">${renderLatexToHtml(clean, false)}</span>`;
        }
      );

      return processed;
    })
    .join('');
}
