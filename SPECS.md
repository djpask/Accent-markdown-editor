# Accent — Functional Specifications & Roadmap (`SPECS.md`)

Given that Apostrophe is a Markdown editor for GNOME that allows users to preview typed Markdown text in a side panel or a separate window, **Accent** is designed as a multiplatform evolution inspired by Apostrophe's visual cleanliness and feature set, while overcoming its structural limitations.

*(Agent Note: For architectural invariants, Clean-Room rules, and the visual verification loop, always refer to **`AGENTS.md`**).*

---

## 1. Functional Specifications (The 9 Pillars of Accent)

1. **100% Rust + `egui` Multiplatform Architecture**: Accent is written entirely in **Rust** using **`eframe` / `egui`** and distributed as a single self-contained binary with zero external system dependencies. The *GUI* replicates the minimalist elegance and color palettes of Apostrophe (*Light*, *Sepia*, and *Dark* themes), enhancing them with tones, contrasts, and spacing engineered to promote deep concentration (*focus mode*) and prolonged on-screen readability.

2. **Preview Modes & *View-while-typing* (Live Preview)**: In addition to all classic preview modes offered by Apostrophe — viewable in a side-by-side panel (*split view*) or in an **independent OS window** via `egui`'s native *Multi-Viewport* APIs — Accent integrates a **View-while-typing** mode (Typora/Obsidian style). As the user types, the editor parses and formats the text in real time. For example, as soon as the user finishes typing `**word**`, Accent hides the asterisks and directly renders **word** in bold; when the text cursor (*caret*) re-enters the word (via arrow keys or mouse click), the editor immediately reveals the raw markup `**word**` for editing, reverting to the rendered form as soon as the caret leaves the span.

3. **Configurable Line Width & Margins**: Accent overcomes Apostrophe's rigid ~70-character line limit by offering a quick selector in the top bar and Preferences with four modes:
   - **Compact**: ~80 characters per line
   - **Medium**: 100–120 characters per line
   - **Wide**: 140+ characters per line
   - **Full-Width (Fluid)**: dynamic adaptation to the window width with a lateral margin (percentage or pixels) freely adjustable via a slider.

4. **Screen-Optimized Typography & Native Font Management**: In Accent, the prose font (*Body/Prose*) is configurable independently from the code font (*Inline code* / *Code block*). Built-in presets feature high-legibility screen fonts (*Inter* / *Cantarell* for Sans-Serif, *Literata* / *Charter* for long-form Serif reading, and *JetBrains Mono* for code blocks or source mode). Inside the native `egui` Preferences panel, a font selector powered by the **`fontdb`** Rust crate allows browsing and selecting any font installed on the operating system, alongside granular controls for **Font Size** and **Line Height** (via `egui::TextFormat`).

5. **Native Multi-Dialect Markdown Support**: Through a pure-Rust parsing pipeline (based on `comrak` / `pulldown-cmark` with AST extensions), Accent natively supports:
   - **Standard CommonMark**
   - **GFM (GitHub Flavored Markdown)**: tables, task lists, strikethrough, autolinks, and alerts
   - **GLFM (GitLab Flavored Markdown)**: advanced extensions compatible with the native Rust pipeline (math blocks, footnotes, YAML/TOML frontmatter).

6. **Integrated Mermaid Support (Zero-WebView)**: Native support for authoring and rendering flowcharts and diagrams directly within the document. `mermaid` code blocks are converted to SVG in pure Rust and rasterized onto GPU textures via **`resvg`**, ensuring crisp rendering on HiDPI displays without relying on external browsers or WebView components.

7. **100% Embedded Rust Export Pipeline (Zero External Dependencies)**: Accent exports documents directly to **PDF**, **Word (`.docx`)**, **OpenDocument Text (`.odt`)**, **LaTeX (`.tex`)**, and **EPUB** without requiring any external software (such as Pandoc or a TeX distribution) installed on the user's system. The entire document generation pipeline is statically compiled into the executable using dedicated pure-Rust crates and serializers (`krilla` / `typst` for PDF, `docx-rs` for Word, ODF 1.2 XML packaged via `zip` with an uncompressed `mimetype` header for ODT, a direct AST-to-LaTeX serializer with strict special-character escaping for `.tex`, and `epub-builder` for EPUB).

8. **Uncompromising Reliability & Crash Recovery**: Accent is engineered around a strict *zero data loss* principle: under no circumstances is data loss or user file corruption acceptable, even during a sudden power outage or system crash. The application implements:
   - ***Save-while-typing* Journaling**: on every edit (debounced under 300ms), the editor state is saved to a local recovery journal, allowing the application on next launch to resume work from the exact last word typed without ever leaving the original file half-written.
   - **Atomic Saving (*Write-Temp $\rightarrow$ Fsync $\rightarrow$ Atomic Rename*)**: every save to the target `.md` file writes first to a temporary sibling file on the same partition, forces a physical hardware buffer flush (`sync_all`), and performs an atomic filesystem-level rename.

9. **Internationalization (i18n)**: Accent is built with a native Rust localization system (`fluent` / `rust-i18n`) to make adding translations effortless. From the very first release, three languages are fully supported: **English**, **Italian**, and **Spanish**.

---

## 2. Implementation Roadmap & Milestone Tracker

Keep these checkboxes updated as phases complete (`[ ]` Pending, `[~]` In Progress, `[x]` Done & Visually Verified):

- [x] **Phase 0: Oracle Reconnaissance & Visual Baseline**
  - [x] Attach to running Apostrophe instance via XWayland (`xdotool` + `maim`).
  - [x] Capture reference screenshots into `img/Apostrophe_screenshots/` (Light, Sepia, Dark themes, Headerbar popovers, Hemingway/Focus mode, Split Preview, Preferences, Export dialog).
  - [x] Extract exact hex palettes, typography proportions, and layout metrics from captured PNGs.

- [x] **Phase 1: Workspace Skeleton, Theme Engine & Shell Chrome**
  - [x] Initialize Cargo workspace (`accent-core`, `accent-storage`, `accent-mermaid`, `accent-export`, `accent-session`, `accent-ui`).
  - [x] Verify live window rendering and external screenshot capture via `Xvfb :99` + `xdotool` + `maim` into `img/Accent_screenshots/`.
  - [x] Implement `ui/theme.rs` with Apostrophe *Light*, *Sepia*, and *Dark* palettes, custom headerbar, and i18n scaffolding (`en`, `it`, `es`).

- [x] **Phase 2: Core Buffer, Multi-Dialect AST & Configurable Typography/Margins**
  - [x] Implement `ropey` text buffer and `comrak` AST parser (CommonMark, GFM, GLFM).
  - [x] Implement configurable line widths (*Compact*, *Medium*, *Wide*, *Fluid* slider).
  - [x] Integrate `fontdb` system font discovery and independent Prose vs. Code font selectors + Line Height controls.

- [x] **Phase 3: Caret-Aware *View-while-typing* & Multi-Viewport Preview**
  - [x] Build the Caret-Aware span engine in `accent-core` (hiding syntax delimiters when caret is outside span, expanding raw Markdown when caret enters span).
  - [x] Implement inline live formatting in `accent-ui` editor canvas.
  - [x] Implement side-by-side Split Preview and separate OS Window Preview (`ctx.show_viewport_immediate`).

- [ ] **Phase 4: Uncompromising Reliability (Crash Recovery WAL & Atomic Save)**
  - [ ] Implement `< 300ms` debounced crash-recovery journaling in `accent-storage`.
  - [ ] Implement automatic dirty-session restoration on startup after simulated crash.
  - [ ] Implement `write-temp -> fsync -> atomic rename` file saving with comprehensive unit tests.

- [ ] **Phase 5: Native Mermaid GPU Rendering & 100% Rust Export Pipeline**
  - [ ] Integrate pure-Rust Mermaid SVG generation + `resvg` texture rasterization with hash caching.
  - [ ] Implement embedded PDF (`krilla`/`typst`), Word (`docx-rs`), OpenDocument Text (`.odt` via ODF 1.2 XML + `zip`), LaTeX (`.tex` AST serializer), and EPUB (`epub-builder`) exporters.
