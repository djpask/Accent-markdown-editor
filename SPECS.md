# Accent — Functional Specifications & Reengineering Roadmap (`SPECS.md`)

**Accent** is a multiplatform Markdown editor and viewer combining a **pure-Rust backend (`L1`–`L4`)** with a **Tauri v2 + Svelte 5 + Vite + CodeMirror 6** desktop interface (`L5`), engineered for crisp native typography across Linux and Windows, visual calm, and zero data loss.

*(For clean-room rules, layered invariants, and live GUI verification, always refer to **`AGENTS.md`**).*

---

## 1. Functional Specifications

1. **Rust Core + Tauri v2 + Svelte 5 + CodeMirror 6 Architecture**:
   - The backend (`accent-core`, `accent-storage`, `accent-mermaid`, `accent-export`, `accent-session`) remains in Rust and is exposed to the Svelte 5 frontend via Tauri v2 commands and events.
   - **CodeMirror 6** handles editing and *View-while-typing* using native system font rendering (DirectWrite/ClearType on Windows, WebKitGTK/HarfBuzz on Linux), eliminating `egui`/`ab_glyph` alignment and cursor drift issues.

2. **Frameless Window, Integrated CSD & Always-Visible Multi-Tab Bar**:
   - Native window decorations are disabled (`decorations: false`) in favor of an integrated **Client-Side Decoration (CSD)** top bar with drag support, double-click maximize, edge resize, and right-aligned **Minimize `─`**, **Maximize/Restore `□`**, and **Close `×`** buttons.
   - A dedicated **Multi-Tab Bar** sits directly beneath the CSD bar and **remains always visible**, supporting per-tab buffers/undo/dirty states, elastic tab sizing, overflow scrolling (`‹`/`›`), and a `+` new-tab button.

3. **Full Typing Auto-Hide Mode**:
   - Whenever the user types in the editor—even in a normal windowed state (not only fullscreen)—**all UI chrome** (CSD bar including `─ □ ×`, multi-tab bar, Document Outline panel, and both bottom floating bars) hides automatically so only the document canvas remains visible.
   - Moving the mouse or pressing `Esc`, `F11`, `Ctrl`/`Alt` combinations, or navigation keys (`Home`, `End`, `PageUp`, `PageDown`) immediately restores the UI.
   - Legacy *Hemingway* and *Focus/dimming* modes are **not** part of Accent.

4. **Dual Floating Bottom Islands**:
   - Instead of a single full-width bottom bar, the bottom area hosts **two distinct floating rounded pill bars** separated by open canvas space:
     - **Left island (Formatting):** Markdown formatting actions (bold, italic, strikethrough, heading, bullet/ordered/task lists, blockquote, inline code, link, image, table) with a collapse/expand toggle (`‹` / `›`).
     - **Right island (Status & Stats):** Compact status indicator (autosave / WAL / file status) and the active statistic pill (`415 parole ▴`) opening the popover (characters, words, sentences, paragraphs, reading time).

5. **Themes, 4 Brightness Modes, Color Palettes & Dedicated Modal**:
   - Accent supports **4 brightness modes**: **Light**, **Sepia**, **Semi-Dark** (soft anthracite gray base designed to reduce both glare and deep-black halation), and **Dark**, paired with **5 color palette presets** (*Amber & Coral*, *Editorial Slate*, *Nordic Forest*, *Rosé Pine*, *Classic*) and structured for adding future themes/palettes.
   - The main menu (`☰`) places the **4 theme circles** (*Chiaro*, *Sepia*, *Semi-Dark*, *Scuro*) at the top for 1-click switching (replacing the old system-theme stub circle), and provides a **`Temi e palette`** item that opens a **dedicated modal dialog** (not a hover/flyout submenu) with brightness + palette selection, a **live combined preview**, and a fixed footer with **`Annulla`** and **`Salva`**.

6. **Tabbed Preferences Modal & Document Outline**:
   - The **Preferences** modal is organized into **2 tabs**:
     - **Tipografia e Pagina:** Prose font and Monospace code font (discovered via `fontdb`), font size, line height, line width (`80ch`, `110ch`, `140ch`, `Fluid`), and fluid lateral margin slider.
     - **Editor e Lingua:** Markdown dialect (*CommonMark*, *GFM*, *GLFM*), autosave/recovery options, and interface language (*Italian*, *English*, *Spanish*).
   - Each tab provides a **`Ripristina predefiniti`** action at the bottom of the tab, and the modal provides a **fixed footer** with **`Annulla`** and **`Salva`**.
   - A toggleable **Document Outline** panel displays clickable headings (`H1`–`H6`) extracted from the document AST to jump to sections.

7. **Preview Modes, KaTeX Math & Rust Mermaid Diagrams**:
   - The view-mode selector provides: *Modifica*, *Anteprima in-line •*, *Anteprima split a destra*, *Anteprima split sotto*, and *Solo lettura* (plus separate preview window where supported).
   - **KaTeX** (bundled locally for offline use) renders math expressions (`$...$` / `$$...$$`), while **`accent-mermaid`** renders `mermaid` blocks into crisp themed SVGs in Rust.

8. **100% Embedded Rust Exporters & Zero-Data-Loss Storage**:
   - Pure-Rust export pipeline for **PDF**, **DOCX**, **ODT**, **LaTeX (`.tex`)**, **EPUB**, and **HTML**.
   - `< 300ms` debounced WAL crash recovery, automatic restoration on startup, `write-temp -> fsync -> atomic rename` saving, single-instance file forwarding, and OS-native focused-file change detection.

---

## 2. Reengineering Roadmap & Milestone Tracker

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