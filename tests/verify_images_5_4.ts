import { EditorState } from '@codemirror/state';
import type { DecorationSet } from '@codemirror/view';
import {
  ImageWidget,
  blockLivePreviewField,
  buildInlineAndLineLiveDecorations,
  buildMarkdownLanguageSupport,
  clearLocalImageResolutionCache,
  createLivePreviewExtensions,
  primeLocalImageResolutionCache,
  resolveRelativeImagePathForEditor,
} from '../src/lib/livePreviewAst';
import type { MarkdownConfig } from '../src/lib/types';

const GFM_CFG: MarkdownConfig = {
  base: 'Gfm',
  extensions: {
    math: true,
    mermaid: true,
    alerts: true,
    footnotes: false,
    frontmatter: false,
  },
};

const COMMONMARK_CFG: MarkdownConfig = {
  base: 'CommonMark',
  extensions: {
    math: false,
    mermaid: false,
    alerts: false,
    footnotes: false,
    frontmatter: false,
  },
};

interface DecoItem {
  from: number;
  to: number;
  widget: unknown;
}

function createEditorStateForImageTest(
  doc: string,
  cursorPos: number,
  cfg: MarkdownConfig = GFM_CFG,
  currentFilePath: string | null = '/workspace/tests/fixtures/gfm_images_export_5_4.md',
  livePreview = true
): EditorState {
  const clamped = Math.max(0, Math.min(cursorPos, doc.length));
  return EditorState.create({
    doc,
    selection: { anchor: clamped, head: clamped },
    extensions: [
      buildMarkdownLanguageSupport(cfg),
      createLivePreviewExtensions(livePreview, 'Sepia', cfg, currentFilePath),
    ],
  });
}

function collectWidgets(set: DecorationSet, docLen: number): DecoItem[] {
  const out: DecoItem[] = [];
  set.between(0, docLen, (from, to, val) => {
    const spec = val.spec as { widget?: unknown };
    if (spec.widget) {
      out.push({ from, to, widget: spec.widget });
    }
  });
  return out;
}

function getAllWidgets(state: EditorState): DecoItem[] {
  const blockSet = state.field(blockLivePreviewField, false);
  const blockItems = blockSet ? collectWidgets(blockSet, state.doc.length) : [];
  const inlineSet = buildInlineAndLineLiveDecorations(state, [
    { from: 0, to: state.doc.length },
  ]);
  const inlineItems = collectWidgets(inlineSet, state.doc.length);
  return [...blockItems, ...inlineItems];
}

function assert(cond: unknown, msg: string): void {
  if (!cond) {
    throw new Error(`Assertion failed: ${msg}`);
  }
}

let passed = 0;
function runTest(name: string, fn: () => void): void {
  clearLocalImageResolutionCache();
  fn();
  passed++;
  console.log(`  ✓ ${name}`);
}

console.log('Running Phase 5.4 Caret-Aware ImageWidget (§6.7) & Path Resolution Suite...');

runTest('Relative local image path resolution handles ./, ../, spaces, percent-encoding, and Windows/POSIX paths', () => {
  const docPath = '/home/user/docs/guide/chapter.md';
  assert(
    resolveRelativeImagePathForEditor(docPath, 'diagram.png') ===
      '/home/user/docs/guide/diagram.png',
    'Bare relative filename must resolve against parent directory'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, './assets/img.jpg') ===
      '/home/user/docs/guide/assets/img.jpg',
    'Dot-slash relative path must normalize cleanly'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, '../shared/logo.svg') ===
      '/home/user/docs/shared/logo.svg',
    'Parent-directory ../ segment must normalize without leaving ..'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, './my%20assets/schema%201.webp') ===
      '/home/user/docs/guide/my assets/schema 1.webp',
    'Percent-encoded spaces must decode cleanly'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, '<./assets/with spaces.avif>') ===
      '/home/user/docs/guide/assets/with spaces.avif',
    'Angle-bracketed URL with spaces must resolve cleanly'
  );
  assert(
    resolveRelativeImagePathForEditor('C:\\Users\\alice\\notes\\readme.md', '..\\img\\icon.ico') ===
      'C:\\Users\\alice\\img\\icon.ico',
    'Windows paths must preserve drive root and normalize ..\\'
  );

  // Security & fallback rejections
  assert(
    resolveRelativeImagePathForEditor(null, './assets/img.png') === null,
    'Unsaved buffer (currentFilePath === null) must return null'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, 'https://example.com/pic.png') === null,
    'Remote https:// URL must be rejected'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, '/etc/passwd') === null,
    'Absolute filesystem path must be rejected'
  );
  assert(
    resolveRelativeImagePathForEditor(docPath, './notes.md') === null,
    'Non-image file extension must be rejected'
  );
});

runTest('§6.7 ImageWidget renders loaded local image in both GFM and CommonMark when caret is outside', () => {
  const docPath = '/workspace/tests/fixtures/gfm_images_export_5_4.md';
  primeLocalImageResolutionCache(docPath, './assets/accent_pipeline_5_4.svg', {
    status: 'ok',
    src: 'data:image/svg+xml;base64,PHN2Zy8+',
  });
  primeLocalImageResolutionCache(docPath, '../../img/brand/accent_logo.png', {
    status: 'ok',
    src: 'asset://localhost/%2Fworkspace%2Fimg%2Fbrand%2Faccent_logo.png',
  });

  const md =
    'Caret here\n\n![Pipeline 5.4](./assets/accent_pipeline_5_4.svg "Titolo SVG")\n\nTesto con ![Logo](../../img/brand/accent_logo.png "Logo PNG") inline.';

  for (const cfg of [GFM_CFG, COMMONMARK_CFG]) {
    const state = createEditorStateForImageTest(md, 0, cfg, docPath);
    const imgWidgets = getAllWidgets(state)
      .filter((w) => w.widget instanceof ImageWidget)
      .map((w) => w.widget as ImageWidget);

    assert(
      imgWidgets.length === 2,
      `Expected 2 ImageWidgets in ${cfg.base}, got ${imgWidgets.length}`
    );
    assert(
      imgWidgets[0].alt === 'Pipeline 5.4' &&
        imgWidgets[0].title === 'Titolo SVG' &&
        imgWidgets[0].isStandalone === true &&
        imgWidgets[0].resolvedSrc === 'data:image/svg+xml;base64,PHN2Zy8+',
      'First standalone SVG ImageWidget must preserve alt, title, standalone flag, and resolvedSrc'
    );
    assert(
      imgWidgets[1].alt === 'Logo' &&
        imgWidgets[1].title === 'Logo PNG' &&
        imgWidgets[1].isStandalone === false &&
        imgWidgets[1].resolvedSrc ===
          'asset://localhost/%2Fworkspace%2Fimg%2Fbrand%2Faccent_logo.png',
      'Second inline PNG ImageWidget must preserve alt, title, inline flag, and asset:// src'
    );
  }
});

runTest('§6.7 ImageWidget expands source on caret entry (click targetPos) and collapses on caret exit', () => {
  const docPath = '/workspace/tests/fixtures/gfm_images_export_5_4.md';
  primeLocalImageResolutionCache(docPath, './assets/logo.png', {
    status: 'ok',
    src: 'asset://localhost/%2Fworkspace%2Ftests%2Ffixtures%2Fassets%2Flogo.png',
  });

  const md = 'Introduzione\n\n![Schema Architettura](./assets/logo.png "Schema")\n\nConclusione';
  const imgFrom = md.indexOf('![Schema');
  const imgTo = imgFrom + '![Schema Architettura](./assets/logo.png "Schema")'.length;

  // 1. Caret outside -> ImageWidget active with targetPos === imgFrom
  const stateOut = createEditorStateForImageTest(md, 0, GFM_CFG, docPath);
  const outWidgets = getAllWidgets(stateOut).filter((w) => w.widget instanceof ImageWidget);
  assert(outWidgets.length === 1, 'ImageWidget must be active when caret is outside');
  const widget = outWidgets[0].widget as ImageWidget;
  assert(
    widget.targetPos === imgFrom && outWidgets[0].from === imgFrom && outWidgets[0].to === imgTo,
    `ImageWidget must replace [${imgFrom}, ${imgTo}] and target ${imgFrom}`
  );

  // 2. Simulate click on ImageWidget -> selection moves to widget.targetPos -> source re-expands!
  const stateClicked = stateOut.update({
    selection: { anchor: widget.targetPos, head: widget.targetPos },
  }).state;
  const inWidgets = getAllWidgets(stateClicked).filter((w) => w.widget instanceof ImageWidget);
  assert(
    inWidgets.length === 0,
    'ImageWidget must immediately disappear when caret enters the Image node'
  );

  // 3. Move caret to "Conclusione" -> ImageWidget collapses back!
  const endPos = md.indexOf('Conclusione');
  const stateAfter = stateClicked.update({
    selection: { anchor: endPos, head: endPos },
  }).state;
  const afterWidgets = getAllWidgets(stateAfter).filter((w) => w.widget instanceof ImageWidget);
  assert(
    afterWidgets.length === 1,
    'ImageWidget must re-collapse when caret leaves the Image node'
  );
});

runTest('§6.7 Fallback badge activates cleanly for missing local file, unsaved buffer, and remote https:// URL', () => {
  const docPath = '/workspace/tests/fixtures/gfm_images_export_5_4.md';
  primeLocalImageResolutionCache(docPath, './assets/missing.png', { status: 'missing' });

  const md =
    'Top\n\n![Mancante](./assets/missing.png)\n\n![Remota](https://example.com/pic.png)';

  // Saved buffer with missing file + remote URL -> both have resolvedSrc === null (fallback mode)
  const stateSaved = createEditorStateForImageTest(md, 0, GFM_CFG, docPath);
  const widgetsSaved = getAllWidgets(stateSaved)
    .filter((w) => w.widget instanceof ImageWidget)
    .map((w) => w.widget as ImageWidget);
  assert(widgetsSaved.length === 2, 'Both missing and remote images must produce ImageWidget');
  assert(
    widgetsSaved[0].resolvedSrc === null && widgetsSaved[0].alt === 'Mancante',
    'Missing local image must enter clean fallback badge state (resolvedSrc === null)'
  );
  assert(
    widgetsSaved[1].resolvedSrc === null && widgetsSaved[1].alt === 'Remota',
    'Remote https:// image must enter clean fallback badge state (resolvedSrc === null)'
  );

  // Unsaved buffer (currentFilePath === null) -> even valid-looking relative path enters fallback badge state
  const stateUnsaved = createEditorStateForImageTest(
    'Top\n\n![Bozza](./assets/logo.png)',
    0,
    GFM_CFG,
    null
  );
  const widgetsUnsaved = getAllWidgets(stateUnsaved)
    .filter((w) => w.widget instanceof ImageWidget)
    .map((w) => w.widget as ImageWidget);
  assert(
    widgetsUnsaved.length === 1 && widgetsUnsaved[0].resolvedSrc === null,
    'Unsaved buffer without currentFilePath must render fallback badge'
  );
});

runTest('§6.7 Reference-style images and multi-line alt images resolve via AST and StateField', () => {
  const docPath = '/workspace/tests/fixtures/gfm_images_export_5_4.md';
  primeLocalImageResolutionCache(docPath, './assets/ref_diagram.png', {
    status: 'ok',
    src: 'asset://localhost/%2Fworkspace%2Ftests%2Ffixtures%2Fassets%2Fref_diagram.png',
  });

  const md =
    'Caret\n\n![Schema su\ndue righe](./assets/ref_diagram.png)\n\n![Immagine Ref][diag]\n\n[diag]: ./assets/ref_diagram.png "Titolo da Ref"';

  const state = createEditorStateForImageTest(md, 0, COMMONMARK_CFG, docPath);
  const blockSet = state.field(blockLivePreviewField, false);
  const blockImgs = blockSet
    ? collectWidgets(blockSet, state.doc.length).filter((w) => w.widget instanceof ImageWidget)
    : [];
  assert(
    blockImgs.length === 1 &&
      (blockImgs[0].widget as ImageWidget).alt === 'Schema su due righe',
    'Multi-line Image node must be replaced via StateField (blockLivePreviewField)'
  );

  const allImgs = getAllWidgets(state)
    .filter((w) => w.widget instanceof ImageWidget)
    .map((w) => w.widget as ImageWidget);
  assert(allImgs.length === 2, 'Both multi-line and reference-style images must render');
  assert(
    allImgs[1].alt === 'Immagine Ref' &&
      allImgs[1].title === 'Titolo da Ref' &&
      allImgs[1].resolvedSrc !== null,
    'Reference-style image ![Alt][diag] must resolve URL and title from [diag]: definition'
  );
});

console.log(`\nAll ${passed} Phase 5.4 ImageWidget & Path Resolution tests passed!`);
