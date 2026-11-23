# Assessment Tecnico di Conformità GFM (`0.29-gfm`) in Accent

**Data:** 2026-11-22  
**Versione analizzata:** Accent `v0.8.23` (`HEAD` su `master`, post-migrazione Tauri v2 + Svelte 5 + CodeMirror 6)  
**Specifica di riferimento:** [GitHub Flavored Markdown Spec — Version `0.29-gfm`](https://github.github.com/gfm/)  
**Modalità di verifica:** Analisi statica in sola lettura su `src/` e `crates/` (`cargo test --workspace`: 60/60 test verdi)

---

## Sintesi Esecutiva

L'architettura attuale di Accent (`v0.8.23`) presenta una **frammentazione in tre pipeline di parsing Markdown distinte** che non condividono né lo stesso parser né la stessa configurazione di dialetto:

1. **Editor CodeMirror 6 (`src/lib/CodeMirrorEditor.svelte`):** utilizza `@codemirror/lang-markdown` (`markdownLanguage`, ossia `@lezer/markdown` + `GFM` + `Subscript` + `Superscript` + `Emoji`) per AST e syntax highlighting, affiancato da uno scanner regex/procedurale riga-per-riga (`buildCaretAwareDecorations`) per la modalità *View-while-typing* (*Anteprima in-line •*). **Non riceve mai il dialetto selezionato dall'utente** e **non implementa il rendering live delle tabelle GFM (`§4.10`)**.
2. **Pannello Anteprima Split / Solo Lettura e Copia HTML (`crates/accent-ui/src/lib.rs` → `crates/accent-export/src/lib.rs`):** utilizza **`pulldown-cmark` v0.12** con flag cablati (`ENABLE_TABLES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS | ENABLE_MATH`), **ignorando `session.dialect`** e mancando del supporto per *Extended Autolinks* (`§6.9`) e *Disallowed Raw HTML* (`§6.11`).
3. **Indice Documento (Outline) ed Esportatori Nativi (`crates/accent-core` → `crates/accent-export`):** utilizzano **`comrak` v0.35** rispettando `session.dialect`, ma gli esportatori (incluso `ExportFormat::Html` → `export_to_standalone_html`) passano attraverso una rappresentazione intermedia ridotta (`Vec<PreviewBlock>`) che perde allineamenti e formattazione inline nelle celle delle tabelle, interruzioni tematiche (`---`), blocchi/tag HTML, *hard line breaks*, immagini inline nel testo e sotto-liste annidate.

---

## 1. Censimento dei Dialetti e Pipeline di Parsing

### 1.1 Dialetti Markdown dichiarati nel progetto

In `SPECS.md` (§1.6, riga 36) sono previsti **3 dialetti selezionabili** (*CommonMark*, *GFM*, *GLFM*; nelle note concettuali storiche in `docs/Accent_conceptual_features.md:48` compariva anche *Pandoc Markdown*, poi rimosso dalle specifiche attive).

| Livello | File e Righe | Tipo / Enum | Valori Dichiarati | Default |
| :--- | :--- | :--- | :--- | :--- |
| **L1 — Core Rust** | `crates/accent-core/src/lib.rs:128-180` | `pub enum MarkdownDialect` | `CommonMark`, `Gfm`, `Glfm` (`from_str_name`: `"commonmark"\|"cm"`, `"gfm"\|"github"`, `"glfm"\|"gitlab"`) | `MarkdownDialect::Gfm` |
| **L4 — Sessione & Persistenza** | `crates/accent-session/src/lib.rs:1, 781, 905, 932, 1041, 1177, 2050` | `PersistedSettings.dialect`, `Session.dialect`, `Action::SetDialect` | Riesporta `accent_core::MarkdownDialect`; persistito in `settings.json` | `MarkdownDialect::Gfm` |
| **L5 — Snapshot IPC Tauri** | `crates/accent-ui/src/ui/mod.rs:78, 593` | `UiSessionSnapshot.dialect` | Serializzato via Serde verso il frontend (`"CommonMark"`, `"Gfm"`, `"Glfm"`) | `session.dialect` |
| **L5 — Tipi TypeScript** | `src/lib/types.ts:12, 157, 196` | `export type MarkdownDialect` | `'CommonMark' \| 'Gfm' \| 'Glfm'` | `'Gfm'` (da snapshot) |
| **L5 — UI Preferenze Svelte 5** | `src/App.svelte:43, 524, 539, 564, 2261-2270` | Selettore a 3 pillole (*Preferenze → Editor e Lingua*) | `CommonMark`, `GFM` (`'Gfm'`), `GLFM` (`'Glfm'`) con rollback su *Annulla* e reset a `'Gfm'` | `'Gfm'` |

#### Mapping delle opzioni `comrak` in `MarkdownDialect::to_comrak_options` (`crates/accent-core/src/lib.rs:153-179`)

- **`CommonMark`**: `Options::default()` con `opts.parse.smart = false` (tutte le estensioni disattivate; `opts.render.unsafe_ = false`).
- **`Gfm`**: abilita `extension.table = true`, `extension.tasklist = true`, `extension.strikethrough = true`, `extension.autolink = true`, `extension.alerts = true`.
  - *Disallineamento rispetto a GFM `0.29-gfm`:* `extension.alerts` (`> [!NOTE]`) è un'estensione GitHub successiva non parte della specifica `0.29-gfm`, mentre **`extension.tagfilter` (`§6.11 Disallowed Raw HTML`) è lasciato a `false`** (e `render.unsafe_ = false`).
- **`Glfm`**: abilita tutte le opzioni di `Gfm` più `math_dollars = true`, `math_code = true`, `footnotes = true`, `front_matter_delimiter = Some("---")`, `multiline_block_quotes = true`.

---

### 1.2 Verifica della propagazione del dialetto nei sottosistemi

```
[Preferenze UI: CommonMark | GFM | GLFM] (src/App.svelte:2261-2270)
        │
        ▼ dispatchAction({ SetDialect })
[Session.dialect in settings.json] (crates/accent-session/src/lib.rs:2050)
        │
        ├──► 1. CodeMirror 6 Editor (src/App.svelte:1283-1307 -> src/lib/CodeMirrorEditor.svelte)
        │       ❌ INTERROTTO: `snapshot.dialect` NON è passato nelle Props di <CodeMirrorEditor />.
        │       Usa sempre `markdownLanguage` (GFM + Subscript + Superscript + Emoji) + regex fisse.
        │
        ├──► 2. Preview Pane & Copia HTML (crates/accent-ui/src/lib.rs:705-710)
        │       ❌ INTERROTTO: `render_preview_html` ignora `session.dialect` e invoca
        │       `accent_export::render_markdown_to_html(text)` (`pulldown-cmark` con flag fissi).
        │
        ├──► 3. Document Outline (crates/accent-ui/src/ui/mod.rs:433-461)
        │       ✅ PROPAGATO: `session.buffer.compute_live_spans(session.dialect, false, 0..0)` (`comrak`).
        │
        ├──► 4. Esportatori su File PDF/DOCX/ODT/LaTeX/EPUB/HTML (crates/accent-session/src/lib.rs:3007-3012)
        │       ✅ PROPAGATO al parser `comrak` (`buffer.parse_preview_blocks(self.dialect)`),
        │       ⚠️ MA degradato dalla conversione intermedia in `Vec<PreviewBlock>`.
        │
        └──► 5. Codice Rust Legacy Non Invocato dalla UI Tauri (`crates/accent-core/src/lib.rs`)
                ⚪ ORFANO: `DocumentBuffer::render_html(dialect)` (riga 789), `build_editor_segments*`
                (riga 1717) e `styled_lines*` (riga 2320) rispettano `dialect`, ma erano usati da `egui`.
```

#### Dettaglio delle disconnessioni rilevate

1. **Mancata propagazione a CodeMirror 6 (`@codemirror/lang-markdown` / `@lezer/markdown`):**
   - In `src/App.svelte:1283-1307`, il componente `<CodeMirrorEditor />` non riceve `dialect={snapshot.dialect}`.
   - In `src/lib/CodeMirrorEditor.svelte:99-143`, `Props` non include `dialect`; non esiste un `Compartment` dedicato al linguaggio (sono presenti solo `themeCompartment`, `readOnlyCompartment`, `livePreviewCompartment`, `searchHighlightCompartment` alle righe `151-154`).
   - Alle righe `16` e `924-941`, l'editor istanzia staticamente `markdown({ base: markdownLanguage, ... })`. In `@codemirror/lang-markdown` v6.3.1, `markdownLanguage` abilita sempre `[GFM, Subscript, Superscript, Emoji]`, mentre `commonmarkLanguage` (esportato dallo stesso pacchetto) non viene mai importato.
   - Inoltre `buildCaretAwareDecorations` (`src/lib/CodeMirrorEditor.svelte:284-647`) esegue espressioni regolari direttamente sulle righe del documento senza consultare `dialect`: anche scegliendo `CommonMark`, l'editor continua a trasformare `- [x]` in checkbox, `~~testo~~` in barrato e `$...$` in formule KaTeX.
2. **Mancata propagazione a `render_preview_html` (`pulldown-cmark`):**
   - Il comando Tauri `render_preview_html` (`crates/accent-ui/src/lib.rs:705-710`) chiama `accent_export::render_markdown_to_html(session.buffer.as_str())` (`crates/accent-export/src/lib.rs:61-71`) senza passare `session.dialect`.
   - Anche `renderMathInPreviewHtml` (`src/lib/mathAndMermaid.ts:52-101`) applica KaTeX su `$...$` e `$$...$$` nel pannello di anteprima indipendentemente dal fatto che il dialetto sia `CommonMark`, `Gfm` o `Glfm`.

---

## 2. Diagnosi Specifica sulla Regressione delle Tabelle (`§4.10`)

### 2.1 Origine storica della regressione

Fino a `v0.8.14` (interfaccia nativa `egui`, es. commit `790a643`), `accent-core` estraeva `AstSpanKind::Table { headers, rows }` con `delimiter_byte_ranges` sull'intero blocco tabella (`crates/accent-core/src/lib.rs:1201-1208, 1554-1596`) e il livello UI disegnava una griglia tabellare quando il cursore era esterno (`is_collapsed`), riespandendo i pipe `|` quando il cursore entrava nel blocco (`crates/accent-core/src/lib.rs:2241-2254`).

Con la migrazione a **Tauri v2 + Svelte 5 + CodeMirror 6** (`v0.8.20`–`v0.8.23`, commit `c3ee220` → `9c63d05`), la logica *View-while-typing* è stata riscritta in TypeScript dentro `src/lib/CodeMirrorEditor.svelte`, **omettendo il supporto per le tabelle**.

### 2.2 Cause tecniche esatte (file e righe)

1. **L'estensione Lezer `GFM` / `Table` È presente nel parser, ma il suo output AST non viene letto dalla Live Preview:**
   - In `src/lib/CodeMirrorEditor.svelte:16` e `924-926`, `markdown({ base: markdownLanguage })` attiva già `GFM = [Table, TaskList, Strikethrough, Autolink]` (`node_modules/@codemirror/lang-markdown/dist/index.js:63-74`). Il syntax tree Lezer contiene quindi correttamente i nodi `Table`, `TableHeader`, `TableDelimiter`, `TableRow` e `TableCell`.
   - Tuttavia `buildCaretAwareDecorations` (`src/lib/CodeMirrorEditor.svelte:284-647`) **non interroga `syntaxTree(view.state)`**, ma scansiona manualmente `doc.line(lineNum)` implementando solo 7 costrutti:
     1. Fenced code blocks e Mermaid (`righe 315-409`)
     2. Display math multi-riga `$$...$$` (`righe 417-469`)
     3. Display math singola riga `$$...$$` (`righe 472-481`)
     4. ATX Headings `#..######` (`righe 484-498`)
     5. Blockquotes `>` (`righe 501-511`)
     6. Task list `- [ ]` / `- [x]` e bullet/ordered markers (`righe 514-533`)
     7. Costrutti inline: codice, math inline, link, bold, strikethrough, italic (`righe 536-641`)
   - **Non esiste alcuna classe `TableWidget extends WidgetType`** (sono definite esclusivamente `KatexMathWidget` alle righe `157-195`, `MermaidSvgWidget` alle righe `197-253` e `TaskCheckboxWidget` alle righe `255-282`) né alcun ramo di riconoscimento delle righe tabella `| ... |`.

2. **Vincolo architetturale di CodeMirror 6: uso di `ViewPlugin` anziché `StateField` per widget multi-riga (`block: true`):**
   - Il plugin live preview è registrato tramite `ViewPlugin.fromClass(...)` in `createLivePreviewPlugin` (`src/lib/CodeMirrorEditor.svelte:649-678`) dentro `livePreviewCompartment` (`righe 153, 946, 1018-1020`).
   - In `@codemirror/view`, **un `ViewPlugin` non può fornire decorazioni `Decoration.replace({ widget, block: true })` né sostituire intervalli che attraversano interruzioni di riga (`\n`)** (CodeMirror solleva `RangeError: Decorations that replace line breaks may not be specified via plugins`).
   - Per Mermaid (`righe 345-377`) e Display Math multi-riga (`righe 441-464`) è stato usato un workaround intra-riga (widget inline sulla prima riga + `Decoration.line({ class: 'cm-live-mermaid-hidden-line' })` e `Decoration.replace({})` su ciascuna riga successiva). Per sostituire in modo pulito e atomico un blocco `Table` multi-riga quando il cursore è fuori (`sel.to < tableFrom || sel.from > tableTo`), **manca uno `StateField<DecorationSet>` registrato con `provide: (f) => EditorView.decorations.from(f)`**.

3. **Assenza di Syntax Highlighting su `TableHeader` e assenza di regole CSS nell'editor:**
   - In `@lezer/markdown` (`node_modules/@lezer/markdown/dist/index.js:2182`), `TableHeader` è mappato sul tag generico `tags.heading` (non `tags.heading1`..`tags.heading6`).
   - In `accentMarkdownHighlight` (`src/lib/CodeMirrorEditor.svelte:682-693`) sono configurati solo `tags.heading1` e `tags.heading2..tags.heading6`, mentre **`tags.heading` non è registrato**: le celle di intestazione non ricevono né grassetto né colore semantico nell'editor (solo i `|` e la riga `:---:` vengono attenuati tramite `TableDelimiter -> tags.processingInstruction` a riga `707`).
   - La variabile CSS `--bg-table-header` è definita in `src/lib/ipc.ts:149` e usata in `.preview-prose th` (`src/App.svelte:3169-3185`), ma nel blocco `<style>` di `src/lib/CodeMirrorEditor.svelte:1040-1251` **non è presente alcuna regola CSS per tabelle**.
   - **Effetto collaterale visivo attuale:** quando *Anteprima in-line •* è attiva e il cursore è fuori dalla tabella, le righe rimangono sorgente `| ... |`, ma lo step 7 nasconde comunque i delimitatori inline dentro le celle (es. `` ` `` o `**` o `~~`), accorciando il testo visualizzato e **rompendo l'allineamento verticale in colonne monospace dei caratteri `|`**.

---

## 3. Matrice di Conformità GFM Completa (`0.29-gfm`)

**Legenda stati:**
- ✅ **Conforme:** comportamento aderente alla specifica `0.29-gfm`.
- ⚠️ **Parziale / Con limiti:** supportato nei casi base, ma con divergenze, edge case non gestiti o dipendenza dalla pipeline.
- ❌ **Assente / Non conforme:** funzionalità mancante o soggetta a regressione.
- **Estensioni GFM ufficiali** evidenziate con badge **`[GFM Ext]`**. Nella colonna **(d)** si distinguono i due percorsi attivi del backend: **Preview** (`render_preview_html` via `pulldown-cmark` 0.12) ed **Export HTML** (`export_to_standalone_html` via `comrak` 0.35 + `PreviewBlock`).

### 3.1 Capitolo 4 — Leaf Blocks (`§4.1`–`§4.10`)

| Sezione GFM `0.29-gfm` | (a) Parsing AST Frontend (Lezer) | (b) Syntax Highlighting Editor | (c) Live Preview / Hybrid Widget (cursore fuori vs dentro) | (d) Backend Rust / Esportazione HTML |
| :--- | :--- | :--- | :--- | :--- |
| **§4.1 Thematic breaks** (`---`, `***`, `___`) | ✅ Conforme (`HorizontalRule`, precedenza corretta su Setext `---`). | ❌ Non stilizzato: `tags.contentSeparator` assente in `accentMarkdownHighlight` (`CodeMirrorEditor.svelte:680-772`). | ❌ Assente: nessun widget `<hr>` o linea divisoria con cursore fuori; resta testo grezzo. | ⚠️ **Preview:** ✅ `<hr />`. **Export HTML (e PDF/DOCX/ODT/LaTeX/EPUB):** ❌ **Scartato** (`NodeValue::ThematicBreak` ignorato in `collect_preview_block`, `accent-core/src/lib.rs:2767`). |
| **§4.2 ATX headings** (`#`..`######`) | ✅ Conforme (`ATXHeading1..6`, `HeaderMark` di apertura e chiusura). | ✅ Conforme (`H1` `--fg-heading` bold; `H2..H6` `--fg-subheading` bold; `#` `--fg-muted`). | ⚠️ Parziale (`righe 484-498`): fuori nasconde `# ` iniziale e scala font (`1.44em`..`1.05em`), dentro mostra `# `. Non gestisce heading vuoti (`###`), `###` di chiusura opzionali, né ignora righe indentate ≥4 spazi (`    # code`). | ✅ **Preview:** `<h1>..<h6>`. **Outline:** estratto via `comrak`. **Export HTML:** `<h1>..<h6>`. |
| **§4.3 Setext headings** (sottolineatura `===` / `---`) | ✅ Conforme (`SetextHeading1`, `SetextHeading2`, multi-riga). | ✅ Conforme (`SetextHeading1/2` mappati su `tags.heading1/2`; underline `--fg-muted`). | ❌ Assente: nessuna classe `.cm-live-h1/h2` né occultamento della riga `===`/`---` con cursore fuori. | ✅ **Preview & Export HTML:** `<h1>` / `<h2>` conformi (in Outline prende solo la prima riga del titolo). |
| **§4.4 Indented code blocks** (4 spazi / 1 tab) | ✅ Conforme (`CodeBlock` + `CodeText`). | ⚠️ Parziale: `CodeText` è rimappato su `tags.content` (`riga 933`), perdendo font monospace e colore codice. | ❌ Assente + conflitto regex: nessuno sfondo blocco codice; inoltre righe come `    # nota` o `    - voce` vengono erroneamente decorate come heading/lista perché le regex usano `^\s*`. | ✅ **Preview & Export HTML:** `<pre><code>...</code></pre>` conforme. |
| **§4.5 Fenced code blocks** (```` ``` ````, `~~~`, info string) | ✅ Conforme (`FencedCode`, `CodeMark`, `CodeInfo`, nested parser per 10 famiglie di linguaggi, `righe 47-97`). | ✅ Conforme per i linguaggi registrati (`json`, `rust`, `sh`, `py`, `toml`, `yaml`, `html`, `css`, `js/ts`); `CodeInfo`/`CodeMark` in `--fg-muted`. | ⚠️ Buono (`righe 315-409`): box `.cm-live-codeblock-line` + badge `data-code-lang`, fence attenuati fuori (`opacity: 0.28`) e pieni dentro; per `mermaid` renderizza `MermaidSvgWidget`. **Limiti:** fallisce su blocchi non chiusi a EOF (§4.5 Ex. 95) e su finte chiusure ```` ```testo ```` (§4.5 Ex. 97). | ⚠️ **Preview:** ✅ `<pre data-code-lang="..."><code class="language-...">` + SVG `mermaid`. **Export HTML:** ✅ SVG `mermaid`, ma omette `class="language-..."` sugli altri blocchi (`accent-export/src/lib.rs:677`). |
| **§4.6 HTML blocks** (Tipi CommonMark 1–7) | ✅ Conforme (`HTMLBlock`, `CommentBlock`, `ProcessingInstructionBlock` + `@codemirror/lang-html`). | ✅ Conforme (tag, attributi, stringhe e commenti colorati). | ⚠️ Nessun widget DOM live; inoltre lo scanner regex non salta gli `HTMLBlock`, alterando eventuale sintassi Markdown/`$...$` interna al blocco HTML. | ⚠️ **Preview:** emesso grezzo in `{@html}` senza filtro `§6.11`. **Export HTML (e altri):** ❌ **Scartato** (`NodeValue::HtmlBlock` ignorato in `collect_preview_block`). |
| **§4.7 Link reference definitions** (`[id]: url "title"`) | ✅ Conforme (`LinkReference`, multi-riga, `<url>`, titoli `'...'`/`"..."`/`(...)`). | ✅ Conforme (`LinkLabel`, `URL`, `LinkTitle`, `LinkMark` colorati distintamente). | ⚠️ Non collassate con cursore fuori; i reference link che vi fanno riferimento non vengono risolti in live preview. | ✅ **Preview & Export HTML:** definizioni risolte e non emesse nell'output HTML. |
| **§4.8 Paragraphs** | ✅ Conforme (`Paragraph`). | ✅ Conforme (`--fg-primary`). | ✅ Conforme (delega agli span inline). | ✅ **Preview & Export HTML:** `<p>...</p>`. |
| **§4.9 Blank lines** | ✅ Conforme. | ✅ N/A. | ✅ Conforme (preservate 1:1). | ⚠️ **Preview:** ✅ conforme. **Export HTML:** distinzione tight/loose list non preservata in `PreviewBlock::List`. |
| **§4.10 Tables `[GFM Ext]`** (`\| col \|`, `:---:`) | ✅ Conforme (`Table`, `TableHeader`, `TableDelimiter`, `TableRow`, `TableCell`, gestione `\|` escapato). Attivo anche se il dialetto è `CommonMark`. | ⚠️ Parziale: `TableDelimiter` è `--fg-muted`, ma `TableHeader` (`tags.heading`) non è mappato in `accentMarkdownHighlight` (niente bold né colore header). | ❌ **Assente (Regressione):** nessun `StateField` né `TableWidget` in `CodeMirrorEditor.svelte`; le tabelle restano testo grezzo con pipe disallineati quando le celle contengono inline collassati. | ⚠️ **Preview (`pulldown-cmark`):** ✅ `<table>` completo con `style="text-align:..."`, `\|` e inline nelle celle (ma ignora `CommonMark`). **Export HTML (`PreviewBlock`):** ❌ perde allineamenti colonne e appiattisce tutto il markup inline delle celle in testo semplice (`accent-core/src/lib.rs:2737-2766`). |

---

### 3.2 Capitolo 5 — Container Blocks (`§5.1`–`§5.4`)

| Sezione GFM `0.29-gfm` | (a) Parsing AST Frontend (Lezer) | (b) Syntax Highlighting Editor | (c) Live Preview / Hybrid Widget (cursore fuori vs dentro) | (d) Backend Rust / Esportazione HTML |
| :--- | :--- | :--- | :--- | :--- |
| **§5.1 Block quotes** (`> ...`, annidamento, lazy continuation) | ✅ Conforme (`Blockquote` + `QuoteMark`). | ✅ Conforme (`--markup-quote-fg` sul testo, `--fg-muted` su `>`). | ⚠️ Parziale (`righe 501-511`): fuori nasconde il primo `> ` e mostra barra laterale `.cm-live-blockquote-line`; dentro rivela `> `. **Limiti:** non gestisce `> >` annidati (nasconde solo il primo `>`), perde la barra sulle *lazy continuation lines* e non stilizza `[!NOTE]`. | ⚠️ **Preview:** ✅ `<blockquote>` annidabili e multi-blocco. **Export HTML:** ❌ `PreviewBlock::BlockQuote` appiattisce tutti i figli in un singolo `<p>`, perdendo liste, codice e paragrafi multipli interni (`accent-core/src/lib.rs:2708-2714`). |
| **§5.2 List items** (regole indentazione, blocchi interni) | ✅ Conforme (`ListItem`, `ListMark`, continuazione automatica su `Enter`). | ✅ Conforme (`listMarkerTag` → `--markup-list-marker` bold). | ⚠️ Parziale (`righe 524-532`): applica solo colore `.cm-live-list-marker` al marcatore (sia fuori che dentro); non ignora righe indentate ≥4 spazi né item vuoti (`-`). | ⚠️ **Preview:** ✅ `<li>` multi-blocco conforme. **Export HTML:** ❌ `PreviewListItem` appiattisce i figli in `Vec<PreviewInline>`, fondendo eventuali sotto-liste o blocchi interni su un'unica riga. |
| **§5.3 Task list items `[GFM Ext]`** (`- [ ]`, `- [x]`, `- [X]`) | ⚠️ Conforme (`Task` + `TaskMarker` sia in liste puntate che numerate), salvo item vuoti `- [ ]` privi di spazio/testo. Sempre attivo anche in `CommonMark`. | ✅ Conforme (`ListMark` `--markup-list-marker`, `TaskMarker` `--syntax-number`). | ⚠️ Parziale (`righe 255-282, 514-522`): fuori sostituisce `- [ ] ` / `- [x] ` con `☐ ` / `☑ `, dentro espande il sorgente. **Limiti:** (1) **non interattiva** (il click su `☐` non commuta `[x]` ma sposta il cursore); (2) non supporta liste ordinate `1. [x]`; (3) non funziona dentro blockquote `> - [x]`. | ⚠️ **Preview:** ✅ `<input disabled="" type="checkbox" [checked=""]/>` conforme a GFM statico, ma non cliccabile per aggiornare il buffer (e attivo anche in `CommonMark`). **Export HTML:** usa caratteri Unicode `☑ ` / `☐ ` invece di `<input type="checkbox" disabled>`. |
| **§5.4 Lists** (`BulletList`, `OrderedList`, `start`, tight/loose) | ✅ Conforme (incluso cambio delimitatore `-` vs `*` e `.` vs `)`). | ✅ Conforme. | ⚠️ Parziale (come `§5.2`). | ⚠️ **Preview:** ✅ `<ul/ol>` con attributo `start="..."` e tight/loose conformi. **Export HTML:** ❌ perde `start` su `<ol>`, sotto-liste annidate e tight/loose (`accent-export/src/lib.rs:682-696`). |

---

### 3.3 Capitolo 6 — Inlines (`§6.1`–`§6.14`)

| Sezione GFM `0.29-gfm` | (a) Parsing AST Frontend (Lezer) | (b) Syntax Highlighting Editor | (c) Live Preview / Hybrid Widget (cursore fuori vs dentro) | (d) Backend Rust / Esportazione HTML |
| :--- | :--- | :--- | :--- | :--- |
| **§6.1 Backslash escapes** (`\*`, `\|`, `\[`, `\\`, 32 segni ASCII) | ✅ Conforme (`Escape` su tutti i 32 caratteri ASCII di punteggiatura). | ⚠️ Non distinto: `tags.escape` non ha regola in `accentMarkdownHighlight`. | ❌ **Non gestito:** con cursore fuori il `\` resta visibile (`\*testo\*`) e le regex inline (`Code`, `Bold`, `Strike`, `Link`) **ignorano `\`**, formattando per errore `\*\*non bold\*\*` o `\~\~non strike\~\~`! | ✅ **Preview & Export HTML:** tutti i 32 escape ASCII risolti correttamente. |
| **§6.2 Entity & numeric character references** (`&copy;`, `&#35;`, `&#x1F680;`) | ✅ Conforme (`Entity`). | ⚠️ Non distinto (`tags.character` non mappato). | ❌ Assente: le entità non vengono decodificate visivamente con cursore fuori; restano `&copy;`. | ✅ **Preview & Export HTML:** decodificate secondo HTML5 / CommonMark (in export PDF i glifi extra-WinAnsi diventano `?`). |
| **§6.3 Code spans** (`` `code` ``, ```` ``a ` b`` ````) | ✅ Conforme (`InlineCode` + `CodeMark` con conteggio arbitrario di backtick). | ✅ Conforme (`inlineCodeTag` → `--code-font-stack`, `--markup-code-fg`, `--bg-code-inline`). | ⚠️ Parziale (`righe 544-557`): fuori nasconde i backtick singoli `` `...` `` e dentro li rivela, ma **fallisce su doppi/multipli backtick** ```` ``con ` interno`` ```` e su code span multi-riga. | ✅ **Preview & Export HTML:** `<code>...</code>` conforme (multi-backtick, multi-riga e space stripping). |
| **§6.4 Emphasis & strong emphasis** (`*em*`, `_em_`, `**strong**`, `__strong__`) | ✅ Conforme (regole flanking CommonMark complete e intraword underscore `a_b_c`). | ✅ Conforme (`--markup-bold-fg` bold, `--markup-emphasis-fg` italic, delimitatori `--fg-muted`). | ⚠️ Parziale (`righe 595-607, 625-641`): funziona su span semplici e rispetta intraword `_`, ma **il filtro di mutua esclusione `isOccupied` (`righe 536-541`) rompe `***bold+italic***` e qualsiasi annidamento** (`**bold con *italic***` o `**bold con `code`**`). | ✅ **Preview & Export HTML:** `<em>` / `<strong>` pienamente conformi (inclusa immunità su `campo__c`). |
| **§6.5 Strikethrough `[GFM Ext]`** (`~~del~~`, `~del~`) | ⚠️ Parziale: `Strikethrough` in Lezer riconosce solo `~~...~~` (non `~~~`), mentre **`~singola~` (valida in GFM §6.5 Ex. 492) viene catturata come `Subscript` Pandoc** da `markdownLanguage`! Attivo anche in `CommonMark`. | ⚠️ Conforme per `~~...~~` (`line-through`, `--fg-muted`), assente per `~...~`. | ⚠️ Parziale (`righe 610-623`): fuori nasconde `~~` e applica `.cm-live-strike`, dentro espande `~~`. Non supporta `~singola~`, soffre di collisione `isOccupied` su annidamenti ed è attivo anche in `CommonMark`. | ✅ **Preview & Export HTML:** `<del>...</del>` sia per `~~...~~` che per `~...~` (nel Preview Pane resta però attivo anche selezionando `CommonMark`). |
| **§6.6 Links** (`[txt](url "title")`, `[txt][ref]`) | ✅ Conforme (inline, `<url>`, parentesi bilanciate, titoli, reference full/collapsed/shortcut). | ✅ Conforme (`--accent-link` underline, URL `--markup-link-url`, titolo `--syntax-string`). | ⚠️ Parziale (`righe 576-593`): fuori nasconde `[` e `](url)` su link inline semplici. **Limiti:** (1) non cliccabile nell'editor; (2) si rompe su URL con parentesi `(foo)` (es. Wikipedia); (3) non collassa formattazione interna `[**bold**](url)`; (4) ignora tutti i reference link `[txt][ref]`. | ⚠️ **Preview:** ✅ `<a href="..." title="...">` cliccabile (`http/https`, `#slug`, `.md` relativi). **Export HTML:** `<a href="...">` ma perde `title` (e in DOCX/ODT/PDF perde l'`href`). |
| **§6.7 Images** (`![alt](url "title")`) | ✅ Conforme (`Image`). | ✅ Conforme (`tags.link` + `tags.url`). | ❌ **Assente:** nessun widget immagine né collasso di `![alt](url)` nell'editor (`(?<!!)` esclude esplicitamente le immagini a riga `576`). | ❌ **Critico sui path locali:** nel **Preview Pane** `<img src="relativo.png">` non viene risolto rispetto a `current_file_path` né convertito con protocollo `asset:` Tauri → **immagine rotta**; immagini `https://` sono bloccate dalla CSP (`tauri.conf.json:25`). In **Export** le immagini inline in un paragrafo perdono il tag `<img>`, e PDF/DOCX/ODT/EPUB stampano solo il testo `[alt (url)]`. |
| **§6.8 Autolinks** (`<https://...>`, `<user@host>`) | ✅ Conforme (`Autolink` + `URL` + `LinkMark`). | ✅ Conforme (`--markup-link-url`, `<`/`>` `--fg-muted`). | ❌ Assente: `<` e `>` restano sempre visibili anche con cursore fuori. | ✅ **Preview & Export HTML:** `<a href="...">...</a>` conforme (nota: in `handlePreviewClick` `mailto:` non apre il client email). |
| **§6.9 Autolinks `[GFM Ext]`** (`www.`, `https://`, bare email, `xmpp:`) | ✅ Conforme (`Autolink` GFM in `@lezer/markdown:2228-2307` con trimming punteggiatura finale e validazione dominio). Attivo anche in `CommonMark`. | ⚠️ Parziale: evidenziato con `tags.url` (`--markup-link-url`), senza sottolineatura link. | ❌ Assente: nessuna sottolineatura `.cm-live-link` né apertura URL da editor. | ❌ **Preview Pane (`pulldown-cmark`):** **NON SUPPORTATO** — URL nudi `https://...`, `www....` ed email nude restano testo piatto non cliccabile! **Export HTML (`comrak`):** ✅ convertiti in `<a href="...">`. |
| **§6.10 Raw HTML** (`<kbd>`, `<span>`, `<!-- -->`) | ✅ Conforme (`HTMLTag`, `Comment`, `ProcessingInstruction`). | ✅ Conforme. | ❌ Assente: tag inline e commenti `<!-- ... -->` restano sorgente visibile anche con cursore fuori. | ⚠️ **Preview:** ✅ renderizzato nel DOM. **Export HTML:** ❌ `HtmlInline` viene passato a `xml_escape` (`&lt;kbd&gt;`), apparendo come codice escapato anziché HTML! |
| **§6.11 Disallowed Raw HTML `[GFM Ext]`** (9 tag filtrati) | ❌ Non distinto da `HTMLBlock`/`HTMLTag` standard. | ❌ Nessuna evidenziazione di tag vietato. | N/A (nessun rendering HTML nell'editor). | ❌ **NON IMPLEMENTATO:** né `pulldown-cmark` (Preview Pane) né `to_comrak_options` (`extension.tagfilter == false`) filtrano `<title>`, `<textarea>`, `<style>`, `<xmp>`, `<iframe>`, `<noembed>`, `<noframes>`, `<script>`, `<plaintext>`. Nel Preview Pane `<style>` può alterare il CSS globale dell'app! |
| **§6.12 Hard line breaks** (`\` a fine riga o `>= 2` spazi) | ✅ Conforme (`HardBreak`). | ✅ Conforme (`\` finale in `--fg-muted`). | ⚠️ Parziale: il `\` finale non viene nascosto con cursore fuori riga. | ⚠️ **Preview:** ✅ `<br />`. **Export HTML (e altri):** ❌ `NodeValue::LineBreak` è convertito in `" "` (`accent-core/src/lib.rs:2791`), fondendo le righe! |
| **§6.13 Soft line breaks** (`\n` singolo) | ✅ Conforme. | ✅ Conforme. | ✅ Conforme. | ✅ **Preview & Export HTML:** conforme. |
| **§6.14 Textual content** (Unicode, emoji, simboli) | ✅ Conforme (+ nodo `Emoji` `:shortcode:`; conversione accurata UTF-16 ↔ `char` in `src/lib/textOffsets.ts`). | ✅ Conforme. | ⚠️ Conforme per Unicode/emoji, ma coppie di simboli di valuta `$100 ... $200` sulla stessa riga attivano per errore `KatexMathWidget` (`riga 560`) anche in `CommonMark`/`Gfm`. | ⚠️ **Preview:** UTF-8 conforme, ma `renderMathInPreviewHtml` (`mathAndMermaid.ts:91`) intercetta `$...$` anche fuori da `Glfm`. **Export:** `comrak` espande `:shortcode:` emoji; PDF limita il testo a `WinAnsiEncoding`. |

---

## 4. Analisi Approfondita dei Casi Limite Critici

### 4.1 Escape `\|`, allineamento `:---:` / `---:` e formattazione inline nelle Tabelle (`§4.10`)
- **Editor (Live Preview):** completamente assente la trasformazione in widget tabella (`CodeMirrorEditor.svelte:284-647`). Inoltre, poiché lo step 7 processa le regex inline sulle righe della tabella, una cella contenente `` `a | b` `` o `**testo**` nasconde i delimitatori `` ` `` / `**` quando il cursore è fuori riga, spostando orizzontalmente tutti i `|` successivi sulla riga e rendendo illeggibile l'incolonnamento monospace.
- **Preview Pane (`pulldown-cmark`):** gestisce correttamente `\|` come pipe letterale, rispetta `:---` / `:---:` / `---:` emettendo `style="text-align: left|center|right"` (che prevale su `.preview-prose th, .preview-prose td { text-align: left; }` in `src/App.svelte:3175-3180`) e preserva grassetto/corsivo/codice/barrato/link dentro le celle.
- **Esportazione File (`PreviewBlock::Table` in `crates/accent-core/src/lib.rs:2737-2766` → `crates/accent-export/src/lib.rs:698-716`):**
  1. Scarta il vettore `alignments` di `comrak::nodes::NodeTable` (`NodeValue::Table(_)`), forzando tutte le colonne ad allineamento sinistro in HTML, EPUB, ODT, DOCX, LaTeX (`"l".repeat(cols)` a riga `212`) e PDF.
  2. Concatena i nodi figli della cella estraendo solo `i.text.as_str()` (`riga 2756`): qualunque `**bold**`, `*italic*`, `` `code` ``, `~~strike~~` o `[link](url)` dentro una cella di tabella **perde completamente la formattazione** nell'esportazione!

### 4.2 Immagini con path relativi locali e CSP Tauri (`§6.7`)
- **Editor:** nessun `ImageWidget` né collasso sintattico (`CodeMirrorEditor.svelte:576`).
- **Preview Pane (`src/App.svelte:1335` + `crates/accent-ui/src/lib.rs:705-710`):**
  - `render_preview_html` non risolve gli `src` relativi (es. `accent_architettura.png`, `./assets/schema.png`, `../../img/brand/accent_logo.png`) rispetto alla directory di `session.current_file_path`. Il WebView cerca l'immagine sull'origine interna dell'app anziché sul filesystem locale, mostrando un'immagine non caricata (*broken image*).
  - Anche qualora venisse calcolato il path assoluto su disco, in `crates/accent-ui/tauri.conf.json` la CSP consente `img-src 'self' data: blob: asset: http://asset.localhost;` (`riga 25`), ma non risulta abilitato `assetProtocol` né invocato `convertFileSrc()` (o embedding `data:` lato Rust). Le immagini remote `https://...` sono invece bloccate intenzionalmente dalla CSP offline-first.
- **Esportazione File (`crates/accent-core/src/lib.rs:2676-2698` e `crates/accent-export/src/lib.rs`):**
  - Un'immagine posta inline accanto a testo nello stesso paragrafo (`Icona ![logo](a.png) qui`) non soddisfa `node.children().count() == 1` e viene degradata al solo testo `alt`.
  - Nei formati binari/impaginati (`PDF`, `DOCX`, `ODT`, `EPUB`) i file immagine utente non vengono letti né incorporati (a differenza dei diagrammi Mermaid), ma sostituiti dal testo segnaposto `[Immagine: alt (url)]`.

### 4.3 Task list interattive, task list numerate e annidate (`§5.3`)
- **Assenza di toggle al click nell'Editor:** `TaskCheckboxWidget.toDOM` (`src/lib/CodeMirrorEditor.svelte:269-281`) intercetta `mousedown` ma esegue solo `view.dispatch({ selection: { anchor: this.targetPos, head: this.targetPos } })` invece di sostituire `[ ]` con `[x]` (o viceversa) tramite una transazione `changes` sul documento CodeMirror.
- **Assenza di toggle nel Preview Pane:** `pulldown-cmark` genera `<input disabled="" type="checkbox"/>` e `handlePreviewClick` (`src/App.svelte:622-673`) gestisce esclusivamente elementi `<a>`.
- **Liste ordinate e citazioni:** la regex `/^(\s*[-*+]\s+\[([ xX])\]\s+)/` (`CodeMirrorEditor.svelte:514`) esclude sia `1. [x] Task` (valido in GFM `§5.3`) sia `> - [x] Task` dentro un blockquote.

### 4.4 Sanitizzazione dei 9 tag HTML vietati (`§6.11 Disallowed Raw HTML`)
- La specifica GFM `0.29-gfm` `§6.11` impone di sostituire il `<` iniziale con `&lt;` (case-insensitively, in apertura e chiusura) esclusivamente per i **9 tag**:
  `<title>`, `<textarea>`, `<style>`, `<xmp>`, `<iframe>`, `<noembed>`, `<noframes>`, `<script>`, `<plaintext>`, lasciando intatti tutti gli altri tag HTML validi (`<table>`, `<details>`, `<kbd>`, `<sup>`, ecc.).
- **Impatto pratico in Accent:** nel Preview Pane (`{@html previewHtml}` in `src/App.svelte:1335`), poiché la CSP (`tauri.conf.json:25`) include `style-src 'self' 'unsafe-inline'` e `.preview-prose` vive nello stesso DOM della shell applicativa (senza Shadow DOM), un tag `<style>` in un documento Markdown **modifica gli stili CSS dell'intera interfaccia di Accent** (top bar CSD, tab bar, modali), mentre `<textarea>` o `<plaintext>` alterano il flusso DOM dell'anteprima.

### 4.5 Collisioni dello scanner regex inline in `CodeMirrorEditor.svelte` (`§6.1`, `§6.4`, `§6.5`, `§6.6`)
- L'array `occupied` (`src/lib/CodeMirrorEditor.svelte:536-541`) impedisce qualsiasi decorazione gerarchica o combinata sulla stessa riga:
  - `***grassetto corsivo***`, `**grassetto con *corsivo***`, `**grassetto con `codice`**`, `~~barrato con **grassetto**~~` e `[**link in grassetto**](https://example.com)` lasciano marcatori grezzi visibili o perdono uno dei due stili quando il cursore è fuori.
- L'uso di `markdownLanguage` in CodeMirror include `Subscript` e `Superscript` di `@lezer/markdown`, facendo sì che `~barrato~` con singola tilde (conforme a GFM `§6.5`) venga invece parsato come pedice.

---

## 5. Riepilogo dei Punti d'Intervento Raccomandati (Fasi Successive)

1. **Propagazione end-to-end di `MarkdownDialect`:**
   - Passare `dialect={snapshot.dialect}` a `<CodeMirrorEditor />` usando un `languageCompartment` (`commonmarkLanguage` per `CommonMark`; parser configurato con `GFM` — senza `Subscript` che confligge con `~...~` — per `Gfm`/`Glfm`) e condizionare KaTeX `$...$` al dialetto `Glfm` (o ad apposita opzione).
   - Unificare `render_preview_html` su `comrak` (o passare `session.dialect` abilitando *Extended Autolinks* `§6.9` e *Tagfilter* `§6.11` con `render.unsafe_ = true` + `extension.tagfilter = true` e scoping CSS sul pannello preview).
2. **Ripristino Live Preview Tabelle (`§4.10`) in CodeMirror 6:**
   - Introdurre uno `StateField<DecorationSet>` (con `provide: f => EditorView.decorations.from(f)`) guidato dai nodi `Table` del `syntaxTree(state)` Lezer, sostituendo il blocco con un `TableWidget` HTML (`Decoration.replace({ widget, block: true })`) quando la selezione non interseca `[node.from, node.to]`, con gestione di `:---`/`:---:`/`---:`, `\|`, formattazione inline nelle celle e click-to-place-caret sulla riga/cella cliccata.
   - Aggiungere `tags.heading` in `accentMarkdownHighlight` e gli stili `.cm-table-widget` coerenti con `--bg-table-header` e `--border-headerbar`.
3. **Migrazione di `buildCaretAwareDecorations` da regex a `syntaxTree(state)` Lezer:**
   - Eliminare i falsi positivi su backslash escapes (`§6.1`), indented code blocks (`§4.4`), blocchi HTML (`§4.6`), code span multi-backtick (`§6.3`), link con parentesi (`§6.6`) e span annidati (`§6.4`/`§6.5`), aggiungendo toggle interattivo al click su `TaskCheckboxWidget` (`§5.3`) e rendering `ThematicBreak` (`§4.1`) / `SetextHeading` (`§4.3`).
4. **Risoluzione immagini locali (`§6.7`) e arricchimento `PreviewBlock` per gli Esportatori:**
   - Risolvere i path relativi delle immagini rispetto alla cartella di `session.current_file_path` nel Preview Pane e negli esportatori, e preservare in `PreviewBlock` allineamenti e `Vec<PreviewInline>` per cella di tabella, `ThematicBreak`, `LineBreak` e liste annidate.
