# Accent — Agent Operations, Technical Architecture & Constraints (`AGENTS.md`)

**Accent** is a local-first, multiplatform Markdown editor and viewer combining a **pure-Rust backend (`L1`–`L4`)** with a **Tauri v2 + Svelte 5 + Vite + CodeMirror 6** desktop frontend (`L5`).
This document defines the **technical architecture, operational rules, clean-room constraints, and verification standards**. For functional specifications and the active development roadmap, read **`SPECS.md`**.

---

## 0. Start Every Session Here (MANDATORY BOOTSTRAP)

At the beginning of **every** coding session, before writing or modifying code, follow this checklist:

1. **Read Functional Specs & Roadmap:** Open and read **`SPECS.md`** to identify the active phase (`[ ]` or `[~]`). Never jump ahead before the active phase compiles, passes tests, and passes interactive GUI verification.
2. **Verify Workspace Baseline & Exact Build Commands:**
   - Rust workspace tests & checks: `cargo test --workspace` and `cargo check --workspace`.
   - Frontend build & type check: `npm run build` (and `npm run check` if configured).
   - Launch desktop binary: `./target/debug/accent-ui` or `cargo run -p accent-ui -- [optional/path/to/file.md]` (do not rely on `cargo-tauri` CLI; note that `cargo run accent-ui` without `-p` passes `"accent-ui"` as a file path argument).
3. **Work Incrementally & Verify Visually:** Implement one cohesive sub-feature at a time, run live verification against the actual Tauri app window, capture external screenshots into `img/Accent_screenshots/`, visually inspect them, and update **`SPECS.md`** checkboxes as phases complete.

---

## 1. Clean-Room & Sovereignty Rules (CRITICAL)

1. **Strict Clean-Room Ban:**
   - **NEVER** read, inspect, search, copy, or derive source code, UI templates, stylesheets, or assets from any third-party Markdown editor, viewer, or document converter distributed under copyleft (`GPL`, `LGPL`, `AGPL`) or proprietary licenses.
   - All implementations must be original clean-room code or use exclusively the permissive open-source crates and npm packages declared in `Cargo.toml` and `package.json`.
   - Do not introduce references to third-party desktop editors in code comments, identifiers, internal docs, or UI strings (historical inspiration credits belong solely in `README.md`).
2. **Local-First, Zero Telemetry & Preview Sandboxing:**
   - All document parsing, persistence, WAL crash recovery, Mermaid rendering, exports, KaTeX math rendering, local image resolution, and settings stay 100% local and offline.
   - Never add external CDN links, analytics, telemetry, or cloud dependencies.
   - Enforce GFM `§6.11` (*Disallowed Raw HTML* tagfilter) on all rendered HTML previews and scope `.preview-prose` styles so raw HTML/CSS inside a user Markdown document can never alter Accent's application chrome or execute scripts.

---

## 2. Layered Architecture & Non-Functional Invariants (STRICT)

Lower layers must **NEVER** depend on higher layers:

- **L1 — `accent-core` (Buffer, Unified `comrak` AST & Markdown Configuration):**
  - `ropey::Rope` buffer, **single source-of-truth Rust Markdown parser built on `comrak`**, byte/char/UTF-16/line conversions, document & selection statistics, outline heading extraction, and rich AST export blocks (`PreviewBlock` / `PreviewInline`).
  - Implements the two-tier **`MarkdownConfig`** model:
    - **`MarkdownBase`**: `Gfm` (default — strict CommonMark `0.29` + the 5 formal `0.29-gfm` extensions: *Tables* `§4.10`, *Task list items* `§5.3`, *Strikethrough* `§6.5`, *Extended Autolinks* `§6.9`, and *Disallowed Raw HTML* `§6.11`) or `CommonMark` (pure CommonMark `0.29` with all extensions disabled).
    - **`MarkdownExtensions`** (independently toggleable when `base == Gfm`): `math`, `mermaid`, `alerts`, `footnotes`, and `frontmatter`.
  - **INVARIANT:** Zero GUI or Tauri dependencies. 100% unit-tested via `cargo test`.

- **L2 — `accent-storage` (Atomic I/O, WAL Recovery, Single-Instance & File Watcher):**
  - `< 300ms` debounced write-ahead recovery journal, crash detection on startup, `write-temp -> fsync -> atomic rename` persistence, single-instance IPC, and OS-native focused-file change notifications.

- **L3 — `accent-mermaid` & `accent-export` (Diagram Renderer, Unified HTML Preview & Exporters):**
  - Pure-Rust Mermaid-to-SVG/raster renderer (`resvg` + LRU cache) supporting `Light`, `Sepia`, `SemiDark`, and `Dark`.
  - **INVARIANT (No Split-Brain Parser):** Do not use a secondary Markdown parser (`pulldown-cmark`) for preview or copy-HTML. All HTML preview, copy-HTML, and standalone HTML export paths must consume `accent-core`'s `comrak` pipeline configured with the active `MarkdownConfig`.
  - Pure-Rust document exporters for **PDF**, **DOCX**, **ODT**, **LaTeX (`.tex`)**, **EPUB**, and **HTML** preserving table column alignments, inline formatting inside table cells, thematic breaks, hard line breaks, nested lists/blockquotes, and local relative images resolved against the document directory.

- **L4 — `accent-session` (Session State, Actions, Persistence & i18n):**
  - Manages multi-tab state, per-tab undo/redo, `MarkdownConfig` (including transparent backward-compatible deserialization from legacy `settings.json` `"dialect"` fields), `ThemeMode` (`Light`, `Sepia`, `SemiDark`, `Dark`), `ColorPalettePreset`, `LineWidthMode`, `TypographyConfig` + `fontdb` discovery, `Locale` (`en`, `it`, `es`), `settings.json` persistence, and `session.apply(Action)`.
  - **INVARIANT:** Modes `Hemingway` and `FocusMode` (line dimming) are **removed** and must not be reintroduced.

- **L5 — `accent-ui` / Tauri v2 Shell + Svelte 5 + CodeMirror 6:**
  - Tauri v2 commands expose `Session` actions/queries cleanly over IPC without duplicating business logic in TypeScript.
  - **INVARIANT (Non-Blocking Main Thread):** Never invoke synchronous/blocking native file dialogs or hold `Mutex<Session>` locks across `.await` points or disk I/O on the GTK/WebKitGTK main thread. Close open UI dropdowns before launching native file dialogs.
  - **Svelte 5** renders the integrated CSD headerbar, always-visible multi-tab bar, optional Document Outline drawer, dual floating bottom islands, popovers, and dedicated modal dialogs (*Preferences*, *Themes & Palettes*, *Export*, *Shortcuts*, *About*).
  - **CodeMirror 6 Architecture Invariants:**
    1. **Dynamic Language Compartment:** `<CodeMirrorEditor />` must synchronize with `MarkdownConfig` via a dedicated `Compartment` (`commonmarkLanguage` when `base == CommonMark`; `GFM` without conflicting Pandoc `Subscript` `~...~` overrides when `base == Gfm`; modular extensions gated by `MarkdownExtensions`).
    2. **AST-Driven Caret-Aware Decorations:** *View-while-typing* decorations must traverse `syntaxTree(state)` rather than per-line regex scanners or flat `isOccupied` arrays, respecting backslash escapes (`§6.1`), indented code blocks (`§4.4`), HTML blocks (`§4.6`), multi-backtick code spans (`§6.3`), balanced-parenthesis/reference links (`§6.6`), and nested inline styles (`§6.4`–`§6.5`).
    3. **`StateField` for Multi-Line Block Widgets:** Decorations that replace line breaks or render block-level widgets (such as GFM `§4.10` `TableWidget`, multi-line display math, or Mermaid SVG blocks) **must** be registered through a `StateField<DecorationSet>` (`EditorView.decorations.from(field)`), never through a `ViewPlugin`.

---

## 3. UI & Interaction Invariants (No "Potëmkin UI")

1. **Real Editable Surface & Interactive Widgets:** The editor must always be a live **CodeMirror 6** instance supporting real typing, selection, cursor navigation, clipboard, undo/redo, scrolling, click-to-place-caret on rendered table widgets, and real buffer mutations on interactive GFM task checkboxes.
2. **Real CSD Window Controls:** With native decorations disabled (`decorations: false`), the top bar provides native drag (`data-tauri-drag-region`), double-click maximize toggle, edge/corner resize support, and working window buttons: **Minimize `─`**, **Maximize/Restore `□`**, and **Close `×`**.
3. **Full Typing Auto-Hide:** As soon as the user types in the editor (even when the window is not fullscreen), **all chrome**—CSD bar (including `─ □ ×`), multi-tab bar, Outline panel, and both floating bottom bars—fades out cleanly (`opacity: 0; pointer-events: none`) until pointer movement or command/navigation/escape keys restore it.
4. **No CLI State-Shortcut Flags:** Never add CLI flags that force overlays, themes, or modes for screenshots. The production binary accepts only an optional file path argument (`accent-ui [path/to/file.md]`).

---

## 4. Live GUI Verification Protocol (`Xvfb :99` + `xdotool` + `maim`)

Before marking any UI milestone as `[x]`, run the built Tauri application inside the isolated **`Xvfb :99`** display (never driving the user's live GNOME desktop session directly with `xdotool`):

1. Ensure `Xvfb :99` is active (`1920x1200x24`), launch `./target/debug/accent-ui` inside `:99`, and locate the window ID via `xdotool`.
2. Verify real mouse clicks on CSD controls (`─`, `□`, `×`), tab switching/creation, non-blocking `Apri ▾` / `Salva` menus, menu `☰` + the 4 theme circles + **Temi e palette** modal + **Preferenze** (`MarkdownConfig` base & extension toggles), formatting pills, and statistics popover.
3. Click inside CodeMirror 6, verify caret-aware live rendering (including GFM tables expanding cleanly to aligned monospace source on caret entry and collapsing back to rendered HTML tables on caret exit), verify full UI auto-hide while typing, and save external screenshots to `img/Accent_screenshots/*.png` for visual inspection.
4. Terminate temporary test processes cleanly once verification completes.