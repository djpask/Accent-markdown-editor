# Accent — Agent Operations, Technical Architecture & Constraints (`AGENTS.md`)

**Accent** is a multiplatform Markdown editor and viewer written in **100% pure Rust** using **`eframe` / `egui`**.
This document defines the **technical architecture, operational rules, clean-room constraints, and non-functional requirements**. For functional features and the milestone tracker, you MUST read **`SPECS.md`**.

---

## 0. Start Every Session Here (MANDATORY BOOTSTRAP)

At the beginning of **every** coding session, before writing or modifying any Rust code, follow this exact checklist:

1. **Read Functional Specs & Roadmap:** Open and read **`SPECS.md`** to identify the active Milestone (`[ ]` or `[~]`) and functional requirements. Never jump ahead to later phases before the current phase compiles, passes `cargo test`, and passes live interactive GUI verification.
2. **Inspect Reference Screenshots:** Check `img/Apostrophe_screenshots/`. If the directory is empty or missing reference views for the current task, attach to the running **Apostrophe** instance and capture them first (see Section 1.3).
3. **Verify Baseline Health:** If the Cargo workspace already exists, run `cargo test --workspace` to ensure you are starting from a green state.
4. **Work Incrementally & Test Interactively:** Implement one sub-feature at a time, launch Accent inside `Xvfb :99`, interact with it as a real human user via `xdotool` (mouse clicks, window dragging, keyboard typing), capture external screenshots via `maim` into `img/Accent_screenshots/` (Section 2), visually inspect them against `img/Apostrophe_screenshots/`, and update the checkboxes in **`SPECS.md`** as milestones are completed.

---

## 1. Clean Room & Reference Observation Protocol (CRITICAL)

Accent is a **clean-room implementation**. You (the AI Agent) must replicate the visual appearance, color palettes, spacing, and user-facing behaviors of **Apostrophe** strictly through empirical black-box observation:

1. **Strict Source Code Ban:**
   - **NEVER** read, inspect, search, or copy any source code, Python files, Blueprint/XML `.ui` templates, CSS stylesheets, or git repositories of *Apostrophe* (GPLv3), *Pandoc*, or any other GPL/LGPL/AGPL Markdown editor.
   - Do not inspect `/usr/share/apostrophe`, Flatpak bundles (`org.gnome.gitlab.somas.Apostrophe`), or system Python packages.

2. **Autonomous Visual Observation of Apostrophe (The Oracle):**
   - A live instance of **Apostrophe** is running (or can be launched by you) on the host Linux environment.
   - You will NOT be provided with pre-made screenshots by the user. **You must autonomously interact with the running Apostrophe instance, explore its views/menus/modes, capture screenshots, and save them into `img/Apostrophe_screenshots/`**.
   - Ensure `img/Apostrophe_screenshots/` is listed in `.gitignore` (never commit reference screenshots captured from external software into the git history, though keep them in the workspace directory for visual comparison).

3. **How to Interact with and Capture Apostrophe on Arch Linux (Headless Xvfb `:99` Sandbox):**
   - **CRITICAL:** Never run `xdotool` against the user's live GNOME Wayland/XWayland session, as GNOME Mutter blocks synthetic XTEST input with a Remote Desktop portal popup.
   - Instead, always run and control Apostrophe inside an isolated **Xvfb virtual display (`:99`)** with software rendering (`GSK_RENDERER=cairo`):
     ```bash
     # Start Xvfb :99 if not already running:
     pgrep -f "Xvfb :99" || (Xvfb :99 -screen 0 1920x1200x24 -ac +extension GLX +render -noreset &)
     # Launch Apostrophe on :99:
     DISPLAY=:99 GDK_BACKEND=x11 GSK_RENDERER=cairo apostrophe <sample_file.md> &
     ```
   - **Find Window ID on `:99`:**
     `WIN_ID=$(DISPLAY=:99 xdotool search --onlyvisible --class "apostrophe" | head -n 1)`
   - **Switch Themes / Modes Deterministically:**
     In addition to `DISPLAY=:99 xdotool` keystrokes (`F10`, `ctrl+comma`, `ctrl+shift+p`, `ctrl+d`) and mouse clicks, you can inspect and toggle Apostrophe's live state directly via **`gsettings`** (`gsettings list-recursively | grep -i apostrophe` or `dconf`) so the window updates immediately without fragile popover coordinates.
   - **Capture Clean Window Screenshot on `:99`:**
     `DISPLAY=:99 maim --window "$WIN_ID" --hidecursor "$PWD/img/Apostrophe_screenshots/<descriptive_name>.png"`
   - **Inspect your captured PNGs:** Always view the saved PNGs in `img/Apostrophe_screenshots/` using your multimodal vision capabilities to extract exact hex color palettes, padding ratios, typography contrast, and UI behaviors before implementing them in Accent.

---

## 2. Unified Live GUI Verification Protocol (`Xvfb :99` + `xdotool` + `maim`)

Never guess whether your `egui` UI code looks right or responds to user input. Compilation (`cargo check`) only proves syntax and types—it does NOT prove visual correctness or real interactivity. **For all UI work, you must run Accent as a live desktop app inside `Xvfb :99`, drive it with `xdotool`, capture external screenshots with `maim`, and visually inspect the PNGs.**

### 2.1 Strictly No CLI State-Shortcut or Screenshot Flags
- **CRITICAL BAN:** Building or maintaining CLI state-shortcut or one-frame screenshot flags (such as `--shot`, `--overlay`, `--theme`, `--width`, `--dialect`, `--script`, or `--control`) inside `accent-ui` is **strictly prohibited**, as they encourage static visual mockups ("Potëmkin UI") disconnected from real mouse and keyboard events.
- **Clean Production CLI Surface:** `accent-ui` must ONLY accept an optional file path argument (`accent-ui [path/to/file.md]`). All UI states (themes, line widths, popovers, Preferences modal, preview modes) must be reached exclusively through real user interaction or persisted user preferences.

### 2.2 The Mandatory External Black-Box UI Workflow
Whenever you implement or modify a UI feature in Accent:
1. Look at the corresponding reference screenshot in `img/Apostrophe_screenshots/` (or capture a new one from the running Apostrophe instance if missing).
2. Implement the feature in Rust (`accent-ui` / `accent-core`) with full mouse `.clicked()` and keyboard handlers.
3. Launch Accent normally inside `Xvfb :99` (`DISPLAY=:99 cargo run -p accent-ui -- <sample.md> &`), interact with it using `DISPLAY=:99 xdotool` (clicking buttons, opening popovers, typing text), and capture the live window using `DISPLAY=:99 maim --window "$ACCENT_WIN" --hidecursor "$PWD/img/Accent_screenshots/<descriptive_name>.png"`.
4. **View the captured PNGs in `img/Accent_screenshots/` and compare them side-by-side with `img/Apostrophe_screenshots/*.png`.**
5. Iterate on the Rust code until Accent matches or exceeds the reference quality and responds to every mouse/keyboard action.

---

## 3. Layered Workspace Architecture & Non-Functional Constraints (STRICT)

To prevent spaghetti code and `egui` borrow-checker conflicts, Accent is organized into strict architectural layers. **Lower layers must NEVER depend on higher layers.**

- **L1 — `accent-core` (Document Model, AST, & Live-Span Engine):**
  - Rope/piece-table text buffer (`ropey`), Markdown parser (`comrak`), AST-to-byte-offset mapping, and the Caret-Aware Live Preview span calculator.
  - **INVARIANT:** Zero GUI dependencies (`no egui`, `no eframe`, `no winit`). 100% unit-tested via `cargo test`.

- **L2 — `accent-storage` (Atomic I/O & Crash Recovery WAL):**
  - **Zero Data Loss Constraint:** Handles the `< 300ms` debounced write-ahead recovery journal, crash detection on startup, and `write-temp -> fsync -> atomic rename` file persistence. Never overwrite user files non-atomically.

- **L3 — `accent-export` & `accent-mermaid` (Renderers & Exporters):**
  - **Zero External Runtime Dependencies:** Everything is compiled into the single Rust binary (no Pandoc, no WebView).
  - Pure Rust Mermaid-to-SVG/Pixmap renderer (`resvg`) with content-hash caching.
  - Pure Rust exporters for PDF (`krilla`/`typst`), DOCX (`docx-rs`), and EPUB (`epub-builder`) consuming the L1 AST.

- **L4 — `accent-session` (Commands, Config, & i18n):**
  - Holds `Session` state, user preferences, undo/redo history, i18n localization (`en`, `it`, `es`), and the central `Action` / `Command` dispatcher (`session.apply(Action)`).

- **L5 — `accent-ui` (Thin `eframe` / `egui` Shell):**
  - Implements `eframe::App`.
  - **Borrow-Checker Rule:** During `egui` panel rendering, treat `Session` as read-only where possible and push user interactions into a frame-local `Vec<Action>`. Drain and apply all actions at the end of `update()`.
  - Centralizes all Apostrophe-inspired color palettes, custom headers, rounded popovers, and typography inside `ui/theme.rs`.

---

## 4. No "Potëmkin UI" Rule & Mandatory Human-Simulation Testing (CRITICAL)

Never build static, non-interactive visual mockups. The application MUST be 100% operable by a human using mouse and keyboard, with zero reliance on CLI flags to open menus or change views.

### 4.1 Architectural Interactivity Requirements
1. **Real Editable Text Widget:** The editor canvas must always be a genuinely interactive `egui::TextEdit` (with custom AST `layouter`) supporting mouse click cursor placement, text selection, keyboard typing, copy/paste, and scrolling—never static `ui.label()` rows.
2. **Real CSD Window Controls:** Dragging the top headerbar background must trigger `ViewportCommand::StartDrag`, double-clicking must toggle maximize, window edges must support resizing (`ViewportCommand::BeginResize`), and the `-`, `□`, `×` buttons must dispatch real viewport commands.
3. **Every Button Must Work on Click:** Every button, pill (`80ch`/`110ch`/`140ch`/`Fluid`), and counter in the headerbar, bottom toolbar, and modals must have a working `.clicked()` handler that toggles its popover, dialog, or formatting action interactively with the mouse.

### 4.2 Mandatory Live Human-Simulation Verification (`xdotool` + `maim` on Accent)
Before marking any UI milestone as `[x]`, you MUST launch Accent as a normal interactive desktop app inside `Xvfb :99` (`DISPLAY=:99 cargo run -p accent-ui -- <sample.md> &`) and simulate a real human user interacting with it via `DISPLAY=:99 xdotool`:
1. **Launch Live Accent on `:99`:**
   `DISPLAY=:99 cargo run -p accent-ui &` and locate its X11 window via `ACCENT_WIN=$(DISPLAY=:99 xdotool search --onlyvisible --name ".*" | tail -n 1)`.
2. **Simulate Human Window Dragging:**
   Use `DISPLAY=:99 xdotool` to drag the headerbar and verify via `xdotool getwindowgeometry "$ACCENT_WIN"` (or `xwininfo -id "$ACCENT_WIN"`) that the window coordinates actually changed.
3. **Simulate Human Mouse Clicks on Headerbar & Bottom Bar:**
   Move the mouse cursor and perform real left-clicks (`xdotool mousemove --window "$ACCENT_WIN" <x> <y> click 1`) on the Line-Width pill, Preview button (book icon), Main Menu (`☰`), and bottom formatting buttons, capturing screenshots with `DISPLAY=:99 maim --window "$ACCENT_WIN" --hidecursor` into `img/Accent_screenshots/human_sim_*.png` to prove the popovers open from real mouse clicks.
4. **Simulate Human Typing & Caret Movement in the Editor:**
   Click inside the editor canvas (`xdotool mousemove --window "$ACCENT_WIN" 600 400 click 1`), type live Markdown (`xdotool type --delay 20 " **live human test**"`), move the caret with arrow keys (`xdotool key Left Left`), and capture `img/Accent_screenshots/human_sim_typing.png` to visually verify that typed characters appear in the buffer and Caret-Aware *View-while-typing* reacts to real keyboard navigation.
5. Kill the background test instance cleanly once verified.
