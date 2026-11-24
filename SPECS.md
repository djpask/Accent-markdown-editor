# Accent — Functional Specifications & Development Roadmap (`SPECS.md`)

**Accent** is a multiplatform Markdown editor and viewer combining a **pure-Rust backend (`L1`–`L4`)** with a **Tauri v2 + Svelte 5 + Vite + CodeMirror 6** desktop interface (`L5`), engineered for crisp native typography across Linux and Windows, visual calm, strict standards compliance, and zero data loss.

*(For clean-room rules, layered invariants, and live GUI verification, always refer to **`AGENTS.md`**).*

---

## 1. Functional Specifications

1. **Rust Core + Tauri v2 + Svelte 5 + CodeMirror 6 Architecture**:
   - The backend (`accent-core`, `accent-storage`, `accent-mermaid`, `accent-export`, `accent-session`) is written in Rust around a unified `comrak` AST pipeline and exposed to the Svelte 5 frontend via non-blocking Tauri v2 commands and events.
   - **CodeMirror 6** handles editing and AST-driven *View-while-typing* using native OS font rendering (DirectWrite/ClearType on Windows, WebKitGTK/HarfBuzz on Linux).

2. **Frameless Window, Integrated CSD & Always-Visible Multi-Tab Bar**:
   - Native window decorations are disabled (`decorations: false`) in favor of an integrated **Client-Side Decoration (CSD)** top bar with drag support, double-click maximize, edge resize, non-blocking file menus (`Apri ▾`, `Salva`), and right-aligned **Minimize `─`**, **Maximize/Restore `□`**, and **Close `×`** buttons.
   - A dedicated **Multi-Tab Bar** sits directly beneath the CSD bar and **remains always visible**, supporting per-tab buffers/undo/dirty states, elastic tab sizing, overflow scrolling (`‹`/`›`), and a `+` new-tab button.

3. **Full Typing Auto-Hide Mode**:
   - Whenever the user types in the editor—even in a normal windowed state (not only fullscreen)—**all UI chrome** (CSD bar including `─ □ ×`, multi-tab bar, Document Outline panel, and both bottom floating bars) hides automatically so only the document canvas remains visible.
   - Moving the mouse or pressing `Esc`, `F11`, `Ctrl`/`Alt` combinations, or navigation keys (`Home`, `End`, `PageUp`, `PageDown`) immediately restores the UI.
   - Legacy *Hemingway* and *Focus/dimming* modes are **not** part of Accent.

4. **Dual Floating Bottom Islands**:
   - The bottom area hosts **two distinct floating rounded pill bars** separated by open canvas space:
     - **Left island (Formatting):** Markdown formatting actions (bold, italic, strikethrough, heading, bullet/ordered/task lists, blockquote, inline code, link, image, table) with a collapse/expand toggle (`‹` / `›`).
     - **Right island (Status & Stats):** Compact status indicator (autosave / WAL / file status) and the active statistic pill (`415 parole ▴`) opening the popover (characters, words, sentences, paragraphs, reading time).

5. **Themes, 4 Brightness Modes, Color Palettes & Dedicated Modal**:
   - Accent supports **4 brightness modes**: **Light**, **Sepia**, **Semi-Dark** (soft anthracite gray base designed to reduce both glare and deep-black halation), and **Dark**, paired with **5 color palette presets** (*Amber & Coral*, *Editorial Slate*, *Nordic Forest*, *Rosé Pine*, *Classic*) and structured for adding future themes/palettes.
   - The main menu (`☰`) places the **4 theme circles** (*Chiaro*, *Sepia*, *Semi-Dark*, *Scuro*) at the top for 1-click switching and provides a **`Temi e palette`** item that opens a **dedicated modal dialog** with brightness + palette selection, a **live combined preview**, and a fixed footer with **`Annulla`** and **`Salva`**.

6. **Markdown Grammar Core (`GFM 0.29-gfm`) & Modular Extensions**:
   - Instead of rigid platform dialects, Accent uses a clean two-level **`MarkdownConfig`** shared across CodeMirror 6, the Preview Pane, Outline, and all Rust Exporters:
     - **Base Grammar (`MarkdownBase`):**
       - **`GFM Standard` (`0.29-gfm`, default):** Strict CommonMark `0.29` superset implementing all 5 formal GFM extensions:
         1. **Tables (`§4.10`):** Caret-aware `StateField` block `TableWidget` in *Anteprima in-line •* supporting `:---` / `:---:` / `---:` column alignments, escaped pipes `\|`, inline formatting inside cells, distinct header styling, and click-to-place-caret when the selection is outside the table; when the caret enters the table, it expands to raw source with inline marker collapsing suspended on table rows so monospace `|` columns stay vertically aligned.
         2. **Task List Items (`§5.3`):** Interactive checkboxes (`- [ ]`, `- [x]`, `- [X]`, including ordered lists and blockquotes) that toggle the Markdown buffer on click.
         3. **Strikethrough (`§6.5`):** `~~text~~` (and single-tilde GFM strikethrough without Pandoc `Subscript` `~...~` collision).
         4. **Extended Autolinks (`§6.9`):** Bare `https://`, `http://`, `www.` URLs and email addresses with trailing-punctuation trimming.
         5. **Disallowed Raw HTML (`§6.11`):** Filtering of `<title>`, `<textarea>`, `<style>`, `<xmp>`, `<iframe>`, `<noembed>`, `<noframes>`, `<script>`, and `<plaintext>` while preserving safe HTML tags, backed by CSS containment on `.preview-prose`.
       - **`CommonMark` (Strict `0.29`):** Pure CommonMark `0.29` mode with all 5 GFM extensions and all modular extensions disabled (zero syntax leakage).
     - **Modular Extensions (`MarkdownExtensions`, independently toggleable on `GFM Standard`):**
       - **Math (`KaTeX`):** Offline rendering of `$...$`, `` $`...`$ ``, `$$...$$`, and fenced `math` blocks *(enabled by default; when off, `$100 ... $200` remains literal text)*.
       - **Diagrams (`Mermaid`):** Fenced `mermaid` blocks rendered into crisp themed SVGs in Rust via `accent-mermaid` *(enabled by default)*.
       - **Callouts / Alerts:** Styled alert blockquotes for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]` *(enabled by default)*.
       - **Footnotes:** Reference footnotes `[^1]` and definitions `[^1]: ...` *(optional, disabled by default)*.
       - **YAML Frontmatter:** Leading `--- ... ---` metadata header *(optional, disabled by default so `---` strictly obeys CommonMark thematic break / Setext heading rules unless enabled)*.
     - **Settings Migration:** Legacy `settings.json` `"dialect"` entries migrate transparently (`"CommonMark"` → `CommonMark`; `"Gfm"` → `GFM Standard` + `math`, `mermaid`, `alerts`; `"Glfm"` → `GFM Standard` + all 5 modular extensions enabled).

7. **Tabbed Preferences Modal & Document Outline**:
   - The **Preferences** modal is organized into **2 tabs**:
     - **Tipografia e Pagina:** Prose font and Monospace code font (discovered via `fontdb`), font size, line height, line width (`80ch`, `110ch`, `140ch`, `Fluid`), and fluid lateral margin slider.
     - **Editor e Lingua:** Base Markdown grammar (*GFM Standard* / *CommonMark*), modular extension toggles (*Math KaTeX*, *Mermaid*, *Callouts/Alerts*, *Footnotes*, *YAML Frontmatter* — visually disabled when *CommonMark* is selected), autosave/recovery options, and interface language (*Italian*, *English*, *Spanish*).
   - Each tab provides a **`Ripristina predefiniti`** action at the bottom of the tab, and the modal provides a **fixed footer** with **`Annulla`** and **`Salva`**.
   - A toggleable **Document Outline** panel displays clickable headings (`H1`–`H6`) extracted from the `comrak` AST to jump to sections.

8. **Preview Modes, Local Images, 100% Embedded Rust Exporters & Zero-Data-Loss Storage**:
   - The view-mode selector provides: *Modifica*, *Anteprima in-line •*, *Anteprima split a destra*, *Anteprima split sotto*, and *Solo lettura*.
   - Relative local image paths (`![alt](./diagram.png)`, `../assets/photo.jpg`) resolve against the active document's parent directory via Tauri's `asset:` protocol in WebView previews/widgets and are embedded into exported documents.
   - Pure-Rust export pipeline for **PDF**, **DOCX**, **ODT**, **LaTeX (`.tex`)**, **EPUB**, and **HTML** preserving table alignments, inline formatting inside table cells, thematic breaks, hard line breaks, and nested blocks.
   - `< 300ms` debounced WAL crash recovery, automatic restoration on startup, `write-temp -> fsync -> atomic rename` saving, single-instance file forwarding, and OS-native focused-file change detection.

---

## 2. Development Roadmap & Milestone Tracker

- [x] **Phase 1: Backend Alignment & Spec Reengineering**
  - [x] Reengineer `AGENTS.md` and `SPECS.md` for Tauri v2 + Svelte 5 + CodeMirror 6.
  - [x] Add `ThemeMode::SemiDark` (and `MermaidTheme::SemiDark`) in Rust (`accent-session`, `accent-mermaid`, `accent-ui` palettes).
  - [x] Remove `hemingway_mode` and `focus_mode` from `accent-session` and `accent-ui`.

- [x] **Phase 2: Tauri v2 Shell, IPC Bridge & CSD + Always-Visible Tab Bar**
  - [x] Scaffold Tauri v2 (`decorations: false`) + Svelte 5 + Vite + CodeMirror 6 wired to `L1`–`L4` commands/events.
  - [x] Implement integrated CSD top bar (`─`, `□`, `×`, drag, double-click maximize, edge resize) and always-visible multi-tab bar.
  - [x] Implement full UI auto-hide while typing (windowed and fullscreen) and restore on pointer/key input.

- [x] **Phase 3: CodeMirror 6 Live Editing, Outline, KaTeX & Mermaid**
  - [x] Integrate CodeMirror 6 with `fontdb` system fonts, caret-aware *View-while-typing*, view modes (*Modifica*, *Anteprima in-line •*, *Split destra*, *Split sotto*, *Solo lettura*), and clickable AST *Document Outline*.
  - [x] Integrate offline **KaTeX** math rendering and Rust **`accent-mermaid`** SVG rendering.

- [x] **Phase 4: Dual Floating Bottom Bars, Menu `☰`, Modals & Exporters**
  - [x] Implement the two separate floating bottom pill bars (Formatting + collapse `‹`/`›` on left; Status + Stats on right).
  - [x] Implement Menu `☰` with the 4 theme circles and the dedicated **Temi e palette** modal (live combined preview + `Annulla` / `Salva` footer).
  - [x] Implement the 2-tab **Preferences** modal (*Tipografia e Pagina*, *Editor e Lingua*, per-tab *Ripristina predefiniti*, fixed *Annulla* / `Salva` footer), Search & Replace, WAL/disk-change banners, and all Rust exporters.

- [ ] **Phase 5: GFM `0.29-gfm` Conformance, Modular Extensions & AST Live Preview**
  - [x] **Phase 5.1 — `MarkdownConfig`, Unified `comrak` Backend & HTML Security (`§6.11`):**
    - Replace `MarkdownDialect` with `MarkdownConfig` (`MarkdownBase` + `MarkdownExtensions`) across `accent-core`, `accent-session` (with backward-compatible `settings.json` migration and i18n in `it`/`en`/`es`), IPC snapshots, and the *Editor e Lingua* Preferences tab.
    - Remove `pulldown-cmark` from `render_preview_html` / copy-HTML so Preview Pane, Copy HTML, and standalone HTML export all share `accent-core`'s `comrak` pipeline with Extended Autolinks (`§6.9`), Disallowed Raw HTML `tagfilter` (`§6.11`), and CSS scoping on `.preview-prose`.
    - Wire `markdownConfig` into `<CodeMirrorEditor />` via a dedicated language `Compartment` (`commonmarkLanguage` vs `GFM` without `Subscript` `~...~` conflict, gating KaTeX/Mermaid/Alerts/Footnotes/Frontmatter according to active toggles), and remove any remaining mentions of third-party editors outside `README.md`.
  - [x] **Phase 5.2 — Caret-Aware Live Table Rendering (`§4.10`):**
    - Implement a `StateField<DecorationSet>` reading Lezer `Table` AST nodes to render a themed `TableWidget` (`Decoration.replace({ widget, block: true })`) when the selection is outside `[table.from, table.to]`, supporting `:---` / `:---:` / `---:` alignments, escaped pipes `\|`, inline cell formatting, header styling (`tags.heading`), and click-to-place-caret.
    - Suspend inline delimiter collapsing on raw table lines when the caret enters a table so monospace `|` columns stay aligned.
  - [x] **Phase 5.3 — AST-Driven *View-while-typing* & Interactive Task Lists (`§5.3`):**
    - Migrate `buildCaretAwareDecorations` from per-line regexes and `isOccupied` to `syntaxTree(state)` traversal, fixing backslash escapes (`§6.1`), nested emphasis/strikethrough (`§6.4`–`§6.5`), multi-backtick code spans (`§6.3`), indented code blocks (`§4.4`), HTML blocks (`§4.6`), balanced-parenthesis & reference links (`§6.6`), thematic breaks (`§4.1`), Setext headings (`§4.3`), and clickable task checkboxes (`§5.3`).
  - [ ] **Phase 5.4 — Local Relative Images (`§6.7`) & Rich AST Exporters:**
    - Resolve relative local image paths against `current_file_path` via Tauri's `asset:` protocol in Preview/Editor and embed local images in PDF/DOCX/ODT/EPUB/HTML exports.
    - Enrich `PreviewBlock` (and use direct `comrak` HTML where appropriate) to preserve table column alignments, inline formatting inside table cells, `ThematicBreak`, `LineBreak`, and nested lists/blockquotes across all exporters.