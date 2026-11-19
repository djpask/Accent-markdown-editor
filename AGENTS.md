# Accent — Agent Operations, Technical Architecture & Constraints (`AGENTS.md`)

**Accent** is a local-first, multiplatform Markdown editor and viewer combining a **pure-Rust backend (`L1`–`L4`)** with a **Tauri v2 + Svelte 5 + Vite + CodeMirror 6** desktop frontend (`L5`).
This document defines the **technical architecture, operational rules, clean-room constraints, and verification standards**. For functional specifications and the reengineering roadmap, read **`SPECS.md`**.

---

## 0. Start Every Session Here (MANDATORY BOOTSTRAP)

At the beginning of **every** coding session, before writing or modifying code, follow this checklist:

1. **Read Functional Specs & Roadmap:** Open and read **`SPECS.md`** to identify the active phase (`[ ]` or `[~]`). Never jump ahead before the active phase compiles, passes tests, and passes interactive GUI verification.
2. **Verify Workspace Baseline:** Run `cargo test --workspace` (and frontend type-check/build checks once the Tauri/Svelte frontend is initialized) to confirm a clean starting state.
3. **Work Incrementally & Verify Visually:** Implement one cohesive sub-feature at a time, run live verification against the actual Tauri app window, capture external screenshots into `img/Accent_screenshots/`, visually inspect them, and update **`SPECS.md`** checkboxes as phases complete.

---

## 1. Clean-Room & Sovereignty Rules (CRITICAL)

1. **Strict Source Code Ban:**
   - **NEVER** read, inspect, search, or copy source code, `.ui`/`.blp` templates, CSS, or repositories of *Apostrophe* (GPLv3), *Pandoc*, or any GPL/LGPL/AGPL Markdown editor.
2. **Local-First & Zero Telemetry:**
   - All document parsing, persistence, WAL crash recovery, Mermaid rendering, exports, KaTeX math rendering, and settings stay 100% local and offline.
   - Do not add external CDN links, analytics, telemetry, or cloud dependencies.

---

## 2. Layered Architecture & Non-Functional Invariants (STRICT)

Lower layers must **NEVER** depend on higher layers:

- **L1 — `accent-core` (Buffer, AST & Caret-Aware Spans):**
  - `ropey::Rope` buffer, `comrak` AST parser (`CommonMark`, `GFM`, `GLFM`), byte/char/line conversions, document & selection statistics, and caret-aware live spans.
  - **INVARIANT:** Zero GUI or Tauri dependencies. 100% unit-tested via `cargo test`.

- **L2 — `accent-storage` (Atomic I/O, WAL Recovery, Single-Instance & File Watcher):**
  - `< 300ms` debounced write-ahead recovery journal, crash detection on startup, `write-temp -> fsync -> atomic rename` persistence, single-instance IPC, and OS-native focused-file change notifications.

- **L3 — `accent-mermaid` & `accent-export` (Diagram Renderer & Exporters):**
  - Pure-Rust Mermaid-to-SVG/raster renderer (`resvg` + LRU cache) supporting `Light`, `Sepia`, `SemiDark`, and `Dark`.
  - Pure-Rust document exporters for **PDF**, **DOCX**, **ODT**, **LaTeX (`.tex`)**, **EPUB**, and **HTML** consuming the L1 AST.

- **L4 — `accent-session` (Session State, Actions, Persistence & i18n):**
  - Manages multi-tab state, per-tab undo/redo, `ThemeMode` (`Light`, `Sepia`, `SemiDark`, `Dark`), `ColorPalettePreset`, `LineWidthMode`, `TypographyConfig` + `fontdb` discovery, `Locale` (`en`, `it`, `es`), `settings.json` persistence, and `session.apply(Action)`.
  - **INVARIANT:** Modes `Hemingway` and `FocusMode` (line dimming) are **removed** and must not be reintroduced.

- **L5 — `accent-ui` / Tauri v2 Shell + Svelte 5 + CodeMirror 6:**
  - Tauri v2 commands expose `Session` actions/queries cleanly over IPC without duplicating business logic in JavaScript/TypeScript.
  - **Svelte 5** renders the integrated CSD headerbar, always-visible multi-tab bar, optional Document Outline drawer, dual floating bottom islands, popovers, and dedicated modal dialogs (*Preferences*, *Themes & Palettes*, *Export*, *Shortcuts*, *About*).
  - **CodeMirror 6** provides native OS text shaping/hinting (DirectWrite/ClearType on Windows, WebKitGTK/HarfBuzz on Linux), caret-aware *View-while-typing*, inline KaTeX math rendering, and Rust-rendered Mermaid SVG blocks.

---

## 3. UI & Interaction Invariants (No "Potëmkin UI")

1. **Real Editable Surface:** The editor must always be a live, interactive **CodeMirror 6** instance supporting real typing, selection, cursor navigation, clipboard, undo/redo, and scrolling.
2. **Real CSD Window Controls:** With native decorations disabled (`decorations: false`), the top bar provides native drag (`data-tauri-drag-region`), double-click maximize toggle, edge/corner resize support, and working window buttons: **Minimize `─`**, **Maximize/Restore `□`**, and **Close `×`**.
3. **Full Typing Auto-Hide:** As soon as the user types in the editor (even when the window is not fullscreen), **all chrome**—CSD bar (including `─ □ ×`), multi-tab bar, Outline panel, and both floating bottom bars—fades out cleanly (`opacity: 0; pointer-events: none`) until pointer movement or command/navigation/escape keys restore it.
4. **No CLI State-Shortcut Flags:** Never add CLI flags that force overlays, themes, or modes for screenshots. The production binary accepts only an optional file path argument (`accent [path/to/file.md]`).

---

## 4. Live GUI Verification Protocol (`Xvfb :99` + `xdotool` + `maim`)

Before marking any UI milestone as `[x]`, run the built Tauri application inside the isolated **`Xvfb :99`** display (never driving the user's live GNOME desktop session directly with `xdotool`):

1. Ensure `Xvfb :99` is active (`1920x1200x24`), launch Accent inside `:99`, and locate the window ID via `xdotool`.
2. Verify real mouse clicks on CSD controls (`─`, `□`, `×`), tab switching/creation, menu `☰` + the 4 theme circles + opening the **Temi e palette** modal + opening **Preferenze**, formatting pills, and statistics popover.
3. Click inside CodeMirror 6, type live Markdown, verify full UI auto-hide while typing and smooth restoration on pointer/key input, and save external screenshots to `img/Accent_screenshots/*.png` for visual inspection.
4. Terminate temporary test processes cleanly once verification completes.