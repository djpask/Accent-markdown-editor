import { EditorState, type TransactionSpec } from '@codemirror/state';
import type { DecorationSet } from '@codemirror/view';
import { history, undo, redo } from '@codemirror/commands';
import {
  AlertBadgeWidget,
  DecodedEntityWidget,
  FootnoteRefWidget,
  KatexMathWidget,
  MermaidSvgWidget,
  TableWidget,
  TaskCheckboxWidget,
  ThematicBreakWidget,
  blockLivePreviewField,
  buildInlineAndLineLiveDecorations,
  buildMarkdownLanguageSupport,
  createLivePreviewExtensions,
  findHeadingAnchorPos,
  resolveLinkHrefAtPos,
  toggleTaskCheckboxInView,
} from '../src/lib/livePreviewAst';
import type { MarkdownConfig } from '../src/lib/types';

const GFM_DEFAULT: MarkdownConfig = {
  base: 'Gfm',
  extensions: {
    math: true,
    mermaid: true,
    alerts: true,
    footnotes: false,
    frontmatter: false,
  },
};

const COMMONMARK_STRICT: MarkdownConfig = {
  base: 'CommonMark',
  extensions: {
    math: true,
    mermaid: true,
    alerts: true,
    footnotes: true,
    frontmatter: true,
  },
};

interface DecoEntry {
  from: number;
  to: number;
  className: string;
  widgetName: string | null;
  widget: unknown;
  isBlock: boolean;
  isReplace: boolean;
  isLine: boolean;
  attrs: Record<string, string>;
}

function createTestState(
  doc: string,
  cursorPos = 0,
  cfg: MarkdownConfig = GFM_DEFAULT,
  livePreview = true
): EditorState {
  const clamped = Math.max(0, Math.min(cursorPos, doc.length));
  return EditorState.create({
    doc,
    selection: { anchor: clamped, head: clamped },
    extensions: [
      history(),
      buildMarkdownLanguageSupport(cfg),
      createLivePreviewExtensions(livePreview, 'Sepia', cfg),
    ],
  });
}

function collectDecorations(set: DecorationSet, docLen: number): DecoEntry[] {
  const entries: DecoEntry[] = [];
  set.between(0, docLen, (from, to, val) => {
    const spec = val.spec as {
      class?: string;
      widget?: { constructor: { name: string } };
      block?: boolean;
      attributes?: Record<string, string>;
    };
    const isLine = from === to && Boolean(spec.class) && !spec.widget;
    const isReplace =
      Boolean(spec.widget) || (!spec.class && !isLine && (to > from || Boolean(spec.block)));
    entries.push({
      from,
      to,
      className: spec.class ?? '',
      widgetName: spec.widget ? spec.widget.constructor.name : null,
      widget: spec.widget ?? null,
      isBlock: Boolean(spec.block),
      isReplace,
      isLine,
      attrs: spec.attributes ?? {},
    });
  });
  return entries;
}

function getLiveDecos(state: EditorState): {
  block: DecoEntry[];
  inline: DecoEntry[];
  all: DecoEntry[];
} {
  const blockSet = state.field(blockLivePreviewField, false);
  const block = blockSet ? collectDecorations(blockSet, state.doc.length) : [];
  const inlineSet = buildInlineAndLineLiveDecorations(state, [
    { from: 0, to: state.doc.length },
  ]);
  const inline = collectDecorations(inlineSet, state.doc.length);
  return { block, inline, all: [...block, ...inline] };
}

function assert(condition: unknown, message: string): void {
  if (!condition) {
    throw new Error(`Assertion failed: ${message}`);
  }
}

let passedTests = 0;
function runTest(name: string, fn: () => void): void {
  fn();
  passedTests++;
  console.log(`  ✓ ${name}`);
}

console.log('Running Phase 5.3 AST Live Preview & Interactive Task Lists Suite...');

// 1. §6.1 Backslash Escapes & §6.12 HardBreak
runTest('§6.1 Backslash escapes hide leading backslash without false inline formatting', () => {
  const doc =
    'Intro line\n\\*non italic\\* \\*\\*non bold\\*\\* \\~\\~non strike\\~\\~ \\`non code\\` \\[non link\\](/local/doc.md) \\[non link 2\\](https://example.com)';
  const stateOutside = createTestState(doc, 0);
  const { inline: decosOut } = getLiveDecos(stateOutside);

  assert(
    !decosOut.some((d) =>
      /cm-live-(bold|italic|strike|inline-code)/.test(d.className)
    ),
    'Escaped delimiters must never produce bold/italic/strike/code classes'
  );

  const nonLinkLabelPos = doc.indexOf('non link\\](/local');
  const nonLink2LabelPos = doc.indexOf('non link 2');
  assert(
    !decosOut.some(
      (d) =>
        d.className.includes('cm-live-link') &&
        ((d.from <= nonLinkLabelPos && d.to > nonLinkLabelPos) ||
          (d.from <= nonLink2LabelPos && d.to > nonLink2LabelPos))
    ),
    'Escaped \\[non link\\] label must never be decorated as a link'
  );

  const firstStarEscapePos = doc.indexOf('\\*non italic');
  assert(
    decosOut.some(
      (d) => d.isReplace && d.from === firstStarEscapePos && d.to === firstStarEscapePos + 1
    ),
    'Leading backslash of Escape must be hidden when caret is outside span'
  );

  // Move caret onto the escape span -> backslash must re-expand
  const stateInside = createTestState(doc, firstStarEscapePos + 1);
  const { inline: decosIn } = getLiveDecos(stateInside);
  assert(
    !decosIn.some(
      (d) => d.isReplace && d.from === firstStarEscapePos && d.to === firstStarEscapePos + 1
    ),
    'Leading backslash must re-expand when caret enters Escape span'
  );
});

runTest('§6.12 Hard line break hides trailing backslash without replacing newline', () => {
  const doc = 'Caret line\nPrima riga con break\\\nSeconda riga';
  const slashPos = doc.indexOf('\\\n');
  const state = createTestState(doc, 0);
  const { inline } = getLiveDecos(state);

  assert(
    inline.some((d) => d.isReplace && d.from === slashPos && d.to === slashPos + 1),
    'Trailing backslash of HardBreak must be hidden when caret is off line'
  );
  assert(
    !inline.some((d) => d.from <= slashPos + 1 && d.to >= slashPos + 2),
    'Newline character of HardBreak must never be replaced'
  );
});

// 2. §6.2 Entities
runTest('§6.2 Character entities decode safely when caret is outside and expand on entry', () => {
  const doc = 'Top\n Diritti &copy; &euro; &#8594; &#x1F980; &unknownentity;';
  const stateOut = createTestState(doc, 0);
  const { inline } = getLiveDecos(stateOut);

  const entityWidgets = inline
    .filter((d) => d.widget instanceof DecodedEntityWidget)
    .map((d) => (d.widget as DecodedEntityWidget).decoded);

  assert(
    entityWidgets.includes('©') &&
      entityWidgets.includes('€') &&
      entityWidgets.includes('→') &&
      entityWidgets.includes('🦀'),
    `Expected decoded ©, €, →, 🦀 widgets, got: ${JSON.stringify(entityWidgets)}`
  );
  assert(
    entityWidgets.length === 4,
    'Unrecognized named entity &unknownentity; must remain literal'
  );

  const copyPos = doc.indexOf('&copy;');
  const stateIn = createTestState(doc, copyPos + 2);
  const { inline: inlineIn } = getLiveDecos(stateIn);
  assert(
    !inlineIn.some(
      (d) => d.widget instanceof DecodedEntityWidget && d.from === copyPos
    ),
    '&copy; must expand to raw source when caret enters entity span'
  );
});

// 3. §6.4 Nested Emphasis/Strong & §6.5 Strikethrough (double `~~`, single `~`, triple `~~~` immunity)
runTest('§6.4 & §6.5 Arbitrary nested spans (***bold italic***, **bold `code`**, ~~strike **bold**~~)', () => {
  const doc =
    'Top\n***bold italic*** e **bold con *italic* e `code`** e ~~strike **bold**~~';
  const state = createTestState(doc, 0);
  const { inline } = getLiveDecos(state);

  const biStart = doc.indexOf('bold italic');
  const biEnd = biStart + 'bold italic'.length;
  assert(
    inline.some(
      (d) => d.className === 'cm-live-bold' && d.from <= biStart && d.to >= biEnd
    ) &&
      inline.some(
        (d) => d.className === 'cm-live-italic' && d.from <= biStart && d.to >= biEnd
      ),
    '***bold italic*** must receive BOTH cm-live-bold and cm-live-italic'
  );

  const nestedCodeStart = doc.indexOf('code');
  const nestedCodeEnd = nestedCodeStart + 4;
  assert(
    inline.some(
      (d) =>
        d.className === 'cm-live-bold' &&
        d.from <= nestedCodeStart &&
        d.to >= nestedCodeEnd
    ) &&
      inline.some(
        (d) =>
          d.className === 'cm-live-inline-code' &&
          d.from === nestedCodeStart &&
          d.to === nestedCodeEnd
      ),
    '**bold con *italic* e `code`** must style both outer bold and inner inline code'
  );

  const strikeBoldStart = doc.lastIndexOf('bold');
  assert(
    inline.some(
      (d) =>
        d.className === 'cm-live-strike' &&
        d.from <= strikeBoldStart &&
        d.to >= strikeBoldStart + 4
    ) &&
      inline.some(
        (d) =>
          d.className === 'cm-live-bold' &&
          d.from === strikeBoldStart &&
          d.to === strikeBoldStart + 4
      ),
    '~~strike **bold**~~ must apply both strike and bold simultaneously'
  );
});

runTest('§6.5 Single-tilde GFM strikethrough (~text~) works while triple-tilde (~~~text~~~) stays literal', () => {
  const doc = 'Top\n~barrato singolo~ e ~~~tre tilde letterali~~~';
  const state = createTestState(doc, 0);
  const { inline } = getLiveDecos(state);

  const singleStart = doc.indexOf('barrato singolo');
  const singleEnd = singleStart + 'barrato singolo'.length;
  assert(
    inline.some(
      (d) =>
        d.className === 'cm-live-strike' &&
        d.from === singleStart &&
        d.to === singleEnd
    ),
    'Single-tilde ~barrato singolo~ must receive cm-live-strike per GFM §6.5 Ex. 492'
  );
  assert(
    inline.some((d) => d.isReplace && d.from === singleStart - 1 && d.to === singleStart) &&
      inline.some((d) => d.isReplace && d.from === singleEnd && d.to === singleEnd + 1),
    'Single-tilde delimiters must collapse when caret is outside'
  );

  const tripleStart = doc.indexOf('~~~tre tilde');
  assert(
    !inline.some((d) => d.from >= tripleStart && d.className === 'cm-live-strike'),
    'Triple-tilde ~~~tre tilde letterali~~~ must never be styled as strikethrough'
  );
});

// 4. §6.3 Multi-backtick & Guard-space Code Spans
runTest('§6.3 Multi-backtick code spans preserve inner backticks and strip guard spaces', () => {
  const doc = 'Top\n``usa `codice` qui`` e `` `racchiuso` ``';
  const state = createTestState(doc, 0);
  const { inline } = getLiveDecos(state);

  const firstContent = 'usa `codice` qui';
  const firstStart = doc.indexOf(firstContent);
  assert(
    inline.some(
      (d) =>
        d.className === 'cm-live-inline-code' &&
        d.from === firstStart &&
        d.to === firstStart + firstContent.length
    ),
    'Double-backtick code span must span full inner content including single backticks'
  );

  const guardedInner = '`racchiuso`';
  const guardedStart = doc.indexOf(guardedInner);
  assert(
    inline.some(
      (d) =>
        d.className === 'cm-live-inline-code' &&
        d.from === guardedStart &&
        d.to === guardedStart + guardedInner.length
    ),
    'CommonMark §6.3 guard spaces inside `` `racchiuso` `` must be stripped from code mark'
  );
});

// 5. §4.4 Indented Code Blocks, §4.5 Fenced Code Blocks & §4.6 HTML Blocks + StateField Consolidation
runTest('§4.4 Indented code blocks never trigger heading, task list, or blockquote decorations', () => {
  const doc =
    'Paragrafo iniziale\n\n    # non heading\n    - [x] non task\n    > non citazione';
  const state = createTestState(doc, 0);
  const { all } = getLiveDecos(state);

  assert(
    !all.some((d) => d.className.includes('cm-live-heading-line')),
    'Indented `    # non heading` must never become a heading'
  );
  assert(
    !all.some((d) => d.widget instanceof TaskCheckboxWidget),
    'Indented `    - [x] non task` must never become a TaskCheckboxWidget'
  );
  assert(
    !all.some((d) => d.className.includes('cm-live-blockquote-line')),
    'Indented `    > non citazione` must never become a blockquote'
  );

  const codeLine3 = state.doc.line(3).from;
  assert(
    all.some(
      (d) =>
        d.isLine &&
        d.from === codeLine3 &&
        d.className.includes('cm-live-codeblock-line') &&
        d.className.includes('cm-live-indented-code-line')
    ),
    'Indented code lines must receive cm-live-codeblock-line styling'
  );
});

runTest('§4.5 & §4.6 Fenced fake closures, HTML blocks immunity, and StateField Mermaid/Math widgets', () => {
  const doc = [
    'Top line',
    '',
    '```mermaid',
    'graph LR;',
    '  A --> B;',
    '```',
    '',
    '$$',
    '\\int_0^1 x^2 dx = \\frac{1}{3}',
    '$$',
    '',
    '<div class="raw-html">',
    '  **non bold in html** $x + y$',
    '</div>',
    '',
    '````markdown',
    '``` fake closure with text',
    'still inside 4-backtick block',
    '````',
  ].join('\n');

  const state = createTestState(doc, 0);
  const { block, inline } = getLiveDecos(state);

  assert(
    block.some((d) => d.isBlock && d.widget instanceof MermaidSvgWidget),
    'Mermaid fenced block must be rendered as a block-level StateField widget'
  );
  assert(
    block.some(
      (d) =>
        d.isBlock &&
        d.widget instanceof KatexMathWidget &&
        (d.widget as KatexMathWidget).displayMode
    ),
    'Multi-line $$...$$ display math must be rendered as a block-level StateField widget'
  );

  const htmlStart = doc.indexOf('<div');
  const htmlEnd = doc.indexOf('</div>') + 6;
  assert(
    !inline.some((d) => d.from >= htmlStart && d.to <= htmlEnd),
    'HTMLBlock (§4.6) must not receive Markdown or KaTeX decorations'
  );

  const fourTickEndLine = state.doc.line(19).from;
  assert(
    inline.some(
      (d) =>
        d.isLine &&
        d.from === fourTickEndLine &&
        d.className.includes('cm-live-codeblock-last')
    ),
    '4-backtick FencedCode must ignore inner 3-backtick fake closure (§4.5 Ex. 97)'
  );
});

// 6. §4.1 ThematicBreak, §4.2 Empty/Closing ATX, §4.3 Setext Headings, §5.1 Alerts & Lazy Blockquotes
runTest('§4.1 ThematicBreak, §4.2 ATX closing/empty, §4.3 Setext via StateField, and §5.1 Callouts', () => {
  const doc = [
    'Top caret',
    '',
    '---',
    '',
    '### Titolo Chiuso ###',
    '',
    '###',
    '',
    'Titolo Setext Riga 1',
    'Titolo Setext Riga 2',
    '--------------------',
    '',
    '> [!WARNING]',
    '> Prima riga avviso',
    'continuazione pigra del callout',
  ].join('\n');

  const stateOut = createTestState(doc, 0);
  const { block: blockOut, inline: inlineOut } = getLiveDecos(stateOut);

  assert(
    inlineOut.some((d) => d.widget instanceof ThematicBreakWidget),
    'HorizontalRule `---` must render ThematicBreakWidget when caret is off line'
  );

  const closedH3Pos = doc.indexOf('### Titolo Chiuso ###');
  const closedTailPos = closedH3Pos + '### Titolo Chiuso'.length;
  assert(
    inlineOut.some(
      (d) => d.isReplace && d.from === closedTailPos && d.to === closedTailPos + 4
    ),
    'Closing ATX ` ###` sequence must be hidden when caret is off line'
  );

  // Setext multi-line title + StateField underline collapse
  const setextLine1 = stateOut.doc.line(9).from;
  const setextLine2 = stateOut.doc.line(10);
  const setextUnderline = stateOut.doc.line(11);
  assert(
    inlineOut.some(
      (d) => d.isLine && d.from === setextLine1 && d.className.includes('cm-live-h2')
    ) &&
      inlineOut.some(
        (d) =>
          d.isLine && d.from === setextLine2.from && d.className.includes('cm-live-h2')
      ),
    'Both lines of multi-line SetextHeading2 must receive cm-live-h2'
  );
  assert(
    blockOut.some(
      (d) => d.from === setextLine2.to && d.to === setextUnderline.to
    ),
    'Setext underline line must be collapsed via StateField when caret is outside SetextHeading'
  );

  // Enter Setext title line -> underline re-expands
  const stateInSetext = createTestState(doc, setextLine1 + 2);
  const { block: blockInSetext } = getLiveDecos(stateInSetext);
  assert(
    !blockInSetext.some(
      (d) => d.from === setextLine2.to && d.to === setextUnderline.to
    ),
    'Setext underline must re-expand as soon as caret enters SetextHeading title'
  );

  // Callout Alert + lazy continuation line
  const lazyLine = stateOut.doc.line(15).from;
  assert(
    inlineOut.some(
      (d) =>
        d.widget instanceof AlertBadgeWidget &&
        (d.widget as AlertBadgeWidget).kind === 'warning'
    ),
    '`> [!WARNING]` must render AlertBadgeWidget when caret is outside blockquote'
  );
  assert(
    inlineOut.some(
      (d) =>
        d.isLine &&
        d.from === lazyLine &&
        d.className.includes('cm-live-blockquote-line') &&
        d.className.includes('cm-live-alert-warning')
    ),
    'Lazy continuation line of blockquote must inherit blockquote & alert classes'
  );
});

// 7. §5.3 Interactive TaskCheckboxWidget (`- [ ]`, `1. [ ]`, `> - [ ]`) + Transactional Undo/Redo
runTest('§5.3 Interactive TaskCheckboxWidget toggles `- [ ]` <-> `- [x]` on bullet, ordered, and quoted tasks with Undo/Redo', () => {
  const initialDoc = [
    'Caret stays here on line 1',
    '',
    '- [ ] Bullet task',
    '1. [ ] Ordered task',
    '> - [ ] Quoted task',
  ].join('\n');

  // Lightweight EditorView-compatible harness backed by real EditorState + history()
  let currentState = createTestState(initialDoc, 0);
  const mockView = {
    get state() {
      return currentState;
    },
    dispatch(...specs: TransactionSpec[]) {
      currentState = currentState.update(...specs).state;
    },
    focus() {},
  };

  const decosInitial = getLiveDecos(currentState).inline;
  const checkboxes = decosInitial
    .filter((d) => d.widget instanceof TaskCheckboxWidget)
    .map((d) => d.widget as TaskCheckboxWidget);

  assert(
    checkboxes.length === 3,
    `Expected 3 TaskCheckboxWidgets (bullet, ordered, quoted), got ${checkboxes.length}`
  );
  assert(
    checkboxes.every((cb) => !cb.checked),
    'All 3 checkboxes should initially be unchecked'
  );

  // Ordered task must retain its `1.` ListMark styling
  const orderedLine = currentState.doc.line(4);
  assert(
    decosInitial.some(
      (d) =>
        d.className === 'cm-live-list-marker' &&
        d.from === orderedLine.from &&
        d.to === orderedLine.from + 2
    ),
    'Ordered task `1. [ ]` must preserve `1.` list marker styling'
  );

  // Toggle bullet task `- [ ]` -> `- [x]`
  const toggled1 = toggleTaskCheckboxInView(
    mockView as never,
    checkboxes[0].markerFrom,
    checkboxes[0].markerTo
  );
  assert(toggled1, 'toggleTaskCheckboxInView on bullet task must succeed');
  assert(
    currentState.doc.line(3).text === '- [x] Bullet task',
    `Expected '- [x] Bullet task', got '${currentState.doc.line(3).text}'`
  );
  assert(
    currentState.selection.main.head === 0,
    'Toggling checkbox must not move cursor onto task line'
  );

  // Widget on line 3 should now report checked === true while remaining collapsed
  const afterBulletWidgets = getLiveDecos(currentState)
    .inline.filter((d) => d.widget instanceof TaskCheckboxWidget)
    .map((d) => d.widget as TaskCheckboxWidget);
  assert(
    afterBulletWidgets[0].checked === true,
    'Bullet TaskCheckboxWidget must flip to checked === true immediately'
  );

  // Toggle ordered task `1. [ ]` -> `1. [x]` and quoted task `> - [ ]` -> `> - [x]`
  toggleTaskCheckboxInView(
    mockView as never,
    afterBulletWidgets[1].markerFrom,
    afterBulletWidgets[1].markerTo
  );
  toggleTaskCheckboxInView(
    mockView as never,
    afterBulletWidgets[2].markerFrom,
    afterBulletWidgets[2].markerTo
  );
  assert(
    currentState.doc.line(4).text === '1. [x] Ordered task' &&
      currentState.doc.line(5).text === '> - [x] Quoted task',
    'Ordered and quoted task list items must toggle to [x]'
  );

  // Verify Undo reverts quoted, ordered, and bullet task mutations cleanly
  undo({ state: currentState, dispatch: (tr) => (currentState = tr.state) });
  assert(
    currentState.doc.line(5).text === '> - [ ] Quoted task',
    'First undo must revert quoted task back to [ ]'
  );
  undo({ state: currentState, dispatch: (tr) => (currentState = tr.state) });
  undo({ state: currentState, dispatch: (tr) => (currentState = tr.state) });
  assert(
    currentState.doc.toString() === initialDoc,
    'Undo stack must restore initial Markdown buffer completely'
  );
  redo({ state: currentState, dispatch: (tr) => (currentState = tr.state) });
  assert(
    currentState.doc.line(3).text === '- [x] Bullet task',
    'Redo must re-apply checkbox toggle'
  );
});

// 8. §6.6 Balanced-paren Links, Reference Links, Autolinks, Anchor Slugs & Raw Table Protection
runTest('§6.6–§6.9 Balanced-paren links, reference links, autolinks, #slug navigation, and raw Table protection', () => {
  const doc = [
    '# Sezione Destinazione',
    '',
    '[gfm-ref]: https://github.github.com/gfm/ "GFM Spec"',
    '[rust-home]: <https://www.rust-lang.org>',
    '',
    'Link [Wiki (Rust)](https://en.wikipedia.org/wiki/Rust_(programming_language)) e [Spec][gfm-ref] e [rust-home][] e [gfm-ref] e [non-un-link] e <https://example.org> e www.commonmark.org.',
    '',
    '| Col A | Col B |',
    '| :--- | ---: |',
    '| **bold** | `code` |',
  ].join('\n');

  const stateOut = createTestState(doc, 0);
  const { block: blockOut, inline: inlineOut } = getLiveDecos(stateOut);

  const wikiPos = doc.indexOf('Wiki (Rust)');
  assert(
    resolveLinkHrefAtPos(stateOut, wikiPos + 2, GFM_DEFAULT) ===
      'https://en.wikipedia.org/wiki/Rust_(programming_language)',
    'Balanced-parenthesis inline link URL must resolve completely'
  );

  const fullRefPos = doc.indexOf('Spec][gfm-ref]');
  assert(
    resolveLinkHrefAtPos(stateOut, fullRefPos + 1, GFM_DEFAULT) ===
      'https://github.github.com/gfm/',
    'Full reference link [Spec][gfm-ref] must resolve via LinkReference map'
  );

  const collapsedRefPos = doc.indexOf('rust-home][]');
  assert(
    resolveLinkHrefAtPos(stateOut, collapsedRefPos + 2, GFM_DEFAULT) ===
      'https://www.rust-lang.org',
    'Collapsed reference link [rust-home][] must resolve via LinkReference map'
  );

  const shortcutRefPos = doc.indexOf('e [gfm-ref] e') + 3;
  assert(
    resolveLinkHrefAtPos(stateOut, shortcutRefPos + 2, GFM_DEFAULT) ===
      'https://github.github.com/gfm/',
    'Shortcut reference link [gfm-ref] must resolve via LinkReference map'
  );

  const nonLinkPos = doc.indexOf('[non-un-link]');
  assert(
    resolveLinkHrefAtPos(stateOut, nonLinkPos + 2, GFM_DEFAULT) === null &&
      !inlineOut.some((d) => d.isReplace && d.from === nonLinkPos),
    'Unresolved [non-un-link] must remain literal brackets without collapsing'
  );

  const bareUrlPos = doc.indexOf('www.commonmark.org');
  assert(
    resolveLinkHrefAtPos(stateOut, bareUrlPos + 4, GFM_DEFAULT) ===
      'http://www.commonmark.org',
    'Bare GFM §6.9 www. autolink must normalize to http://'
  );

  assert(
    findHeadingAnchorPos(stateOut, '#sezione-destinazione') === 0,
    '#sezione-destinazione anchor must resolve to heading offset 0'
  );

  // Table collapsed when caret is outside, and no inline collapsing when caret enters table
  assert(
    blockOut.some((d) => d.widget instanceof TableWidget),
    'Table must collapse to TableWidget when caret is outside'
  );
  const tableCellPos = doc.indexOf('**bold**') + 2;
  const stateInTable = createTestState(doc, tableCellPos);
  const { block: blockInTbl, inline: inlineInTbl } = getLiveDecos(stateInTable);
  assert(
    !blockInTbl.some((d) => d.widget instanceof TableWidget),
    'TableWidget must expand to raw source when caret enters table'
  );
  const tableStart = stateInTable.doc.line(8).from;
  assert(
    !inlineInTbl.some((d) => d.isReplace && d.from >= tableStart),
    'Phase 5.2 invariant: zero inline delimiter replacements allowed on raw table rows'
  );
});

// 9. Modular Extensions Gating (`footnotes`, `frontmatter`, `math`) & Strict CommonMark Isolation
runTest('Modular toggles (footnotes, frontmatter, math) and strict CommonMark zero-leakage isolation', () => {
  const modDoc = [
    '---',
    'title: Frontmatter Test',
    '---',
    '',
    'Nota[^1] con prezzo $100 - $250 e formula $E=mc^2$ e ~strike~ e ~~strike2~~ e www.example.com',
    '',
    '- [ ] Task item',
    '',
    '> [!NOTE]',
    '> Callout body',
    '',
    '| A | B |',
    '| - | - |',
    '| 1 | 2 |',
    '',
    '[^1]: Definizione nota',
  ].join('\n');

  // A) GFM with footnotes & frontmatter enabled
  const gfmAllCfg: MarkdownConfig = {
    base: 'Gfm',
    extensions: {
      math: true,
      mermaid: true,
      alerts: true,
      footnotes: true,
      frontmatter: true,
    },
  };
  const stateGfmAll = createTestState(modDoc, modDoc.length, gfmAllCfg);
  const decosGfmAll = getLiveDecos(stateGfmAll);
  assert(
    decosGfmAll.inline.some(
      (d) =>
        d.isLine &&
        d.from === 0 &&
        d.className.includes('cm-live-codeblock-first') &&
        d.attrs['data-code-lang'] === 'yaml'
    ),
    'Leading --- must be parsed as YAML Frontmatter when frontmatter === true'
  );
  assert(
    decosGfmAll.inline.some((d) => d.widget instanceof FootnoteRefWidget),
    '[^1] must render FootnoteRefWidget when footnotes === true'
  );
  const katexInlineWidgets = decosGfmAll.inline
    .filter((d) => d.widget instanceof KatexMathWidget)
    .map((d) => (d.widget as KatexMathWidget).latex);
  assert(
    katexInlineWidgets.length === 1 && katexInlineWidgets[0] === 'E=mc^2',
    `Currency $100 - $250 must not trigger KaTeX; expected ['E=mc^2'], got ${JSON.stringify(katexInlineWidgets)}`
  );

  // B) Strict CommonMark mode: ZERO GFM or modular extension leakage!
  const stateCm = createTestState(modDoc, modDoc.length, COMMONMARK_STRICT);
  const decosCm = getLiveDecos(stateCm);

  assert(
    !decosCm.all.some((d) => d.widget instanceof TableWidget),
    'CommonMark must never render TableWidget'
  );
  assert(
    !decosCm.all.some((d) => d.widget instanceof TaskCheckboxWidget),
    'CommonMark must never render TaskCheckboxWidget'
  );
  assert(
    !decosCm.all.some((d) => d.className.includes('cm-live-strike')),
    'CommonMark must never style ~strike~ or ~~strike2~~'
  );
  assert(
    !decosCm.all.some((d) => d.widget instanceof AlertBadgeWidget) &&
      !decosCm.all.some((d) => d.className.includes('cm-live-alert-line')),
    'CommonMark must never render GFM Alert callouts'
  );
  assert(
    !decosCm.all.some((d) => d.widget instanceof KatexMathWidget),
    'CommonMark must never render KaTeX math widgets'
  );
  assert(
    !decosCm.all.some((d) => d.widget instanceof FootnoteRefWidget),
    'CommonMark must never render FootnoteRefWidget'
  );
  const bareWwwPos = modDoc.indexOf('www.example.com');
  assert(
    resolveLinkHrefAtPos(stateCm, bareWwwPos + 3, COMMONMARK_STRICT) === null,
    'CommonMark must not autolink bare www.example.com'
  );
  assert(
    decosCm.inline.some(
      (d) => d.from === 0 && d.widget instanceof ThematicBreakWidget
    ),
    'In CommonMark, leading `---` must strictly render as a ThematicBreak (§4.1)'
  );
});

console.log(`\nAll ${passedTests} Phase 5.3 AST Live Preview tests passed!`);
