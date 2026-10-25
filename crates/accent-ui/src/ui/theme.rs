use accent_session::ThemeMode;
use egui::{
    epaint::StrokeKind, Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Painter,
    Pos2, Rect, CornerRadius, Stroke, Vec2, Visuals,
};
use std::fs;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccentPalette {
    pub bg_canvas: Color32,
    pub bg_headerbar: Color32,
    pub border_headerbar: Color32,
    pub fg_primary: Color32,
    pub fg_heading: Color32,
    pub fg_muted: Color32,
    pub fg_focus_dimmed: Color32,
    pub bg_code_block: Color32,
    pub bg_pill_button: Color32,
    pub bg_popover: Color32,
    pub bg_preview: Color32,
    pub divider_preview: Color32,
    pub accent_orange: Color32,
    pub accent_link: Color32,
    pub spellcheck_red: Color32,
    pub scrollbar_thumb: Color32,
}

impl AccentPalette {
    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Sepia => Self::sepia(),
            ThemeMode::Dark => Self::dark(),
        }
    }

    /// Exact hex values sampled from `img/Apostrophe_screenshots/light_theme.png` and `split_preview_light.png`
    pub const fn light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xff, 0xff, 0xff),        // #ffffff
            bg_headerbar: Color32::from_rgb(0xff, 0xff, 0xff),     // #ffffff
            border_headerbar: Color32::from_rgb(0xe4, 0xe4, 0xe4), // #e4e4e4
            fg_primary: Color32::from_rgb(0x2e, 0x34, 0x36),       // #2e3436
            fg_heading: Color32::from_rgb(0x2e, 0x34, 0x36),       // #2e3436
            fg_muted: Color32::from_rgb(0x9a, 0x9a, 0x9a),         // #9a9a9a / #b1b1b1
            fg_focus_dimmed: Color32::from_rgb(0x88, 0x8a, 0x85),  // #888a85
            bg_code_block: Color32::from_rgb(0xe0, 0xe0, 0xe0),    // #e0e0e0
            bg_pill_button: Color32::from_rgb(0xf1, 0xf1, 0xf1),   // #f1f1f1
            bg_popover: Color32::from_rgb(0xff, 0xff, 0xff),       // #ffffff
            bg_preview: Color32::from_rgb(0xfa, 0xfa, 0xfa),       // #fafafa
            divider_preview: Color32::from_rgb(0xe4, 0xe4, 0xe4),  // #e4e4e4
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),    // #db632c
            accent_link: Color32::from_rgb(0x1c, 0x71, 0xd8),      // #1c71d8
            spellcheck_red: Color32::from_rgb(0xb8, 0x42, 0x42),   // #b84242
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xc6, 0xc6),  // #c6c6c6
        }
    }

    /// Exact hex values sampled from `img/Apostrophe_screenshots/sepia_theme.png` and `split_preview_sepia.png`
    pub const fn sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf9, 0xf3, 0xe9),        // #f9f3e9
            bg_headerbar: Color32::from_rgb(0xf9, 0xf3, 0xe9),     // #f9f3e9
            border_headerbar: Color32::from_rgb(0xde, 0xd9, 0xd0), // #ded9d0
            fg_primary: Color32::from_rgb(0x4f, 0x39, 0x15),       // #4f3915
            fg_heading: Color32::from_rgb(0x4f, 0x39, 0x15),       // #4f3915
            fg_muted: Color32::from_rgb(0x9e, 0x90, 0x78),         // #9e9078 / #a99c86
            fg_focus_dimmed: Color32::from_rgb(0x80, 0x80, 0x80),  // #808080
            bg_code_block: Color32::from_rgb(0xdf, 0xd7, 0xc9),    // #dfd7c9
            bg_pill_button: Color32::from_rgb(0xec, 0xe5, 0xda),   // #ece5da
            bg_popover: Color32::from_rgb(0xf9, 0xf3, 0xe9),       // #f9f3e9
            bg_preview: Color32::from_rgb(0xf2, 0xeb, 0xe0),       // #f2ebe0
            divider_preview: Color32::from_rgb(0xda, 0xd5, 0xcb),  // #dad5cb
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),    // #db632c
            accent_link: Color32::from_rgb(0x00, 0x80, 0x80),      // #008080
            spellcheck_red: Color32::from_rgb(0xa8, 0x3c, 0x32),   // #a83c32
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xc0, 0xb6),  // #c6c0b6
        }
    }

    /// Exact hex values sampled from `img/Apostrophe_screenshots/dark_theme.png` and `split_preview_dark.png`
    pub const fn dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2b, 0x2b, 0x2f),        // #2b2b2f
            bg_headerbar: Color32::from_rgb(0x2e, 0x2e, 0x32),     // #2e2e32
            border_headerbar: Color32::from_rgb(0x1e, 0x1e, 0x23), // #1e1e23
            fg_primary: Color32::from_rgb(0xee, 0xee, 0xee),       // #eeeeee
            fg_heading: Color32::from_rgb(0xee, 0xee, 0xee),       // #eeeeee
            fg_muted: Color32::from_rgb(0x6e, 0x6e, 0x74),         // #6e6e74 / #575757
            fg_focus_dimmed: Color32::from_rgb(0x80, 0x80, 0x80),  // #808080
            bg_code_block: Color32::from_rgb(0x3c, 0x3c, 0x42),    // #3c3c42
            bg_pill_button: Color32::from_rgb(0x3a, 0x3a, 0x3d),   // #3a3a3d
            bg_popover: Color32::from_rgb(0x36, 0x36, 0x3a),       // #36363a
            bg_preview: Color32::from_rgb(0x26, 0x26, 0x2a),       // #26262a
            divider_preview: Color32::from_rgb(0x16, 0x16, 0x1a),  // #16161a
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),    // #db632c
            accent_link: Color32::from_rgb(0x78, 0xae, 0xed),      // #78aeed
            spellcheck_red: Color32::from_rgb(0xb6, 0x36, 0x3c),   // #b6363c
            scrollbar_thumb: Color32::from_rgb(0x6c, 0x6c, 0x70),  // #6c6c70
        }
    }
}

pub fn font_editor_regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("EditorRegular".into()))
}

pub fn font_editor_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("EditorBold".into()))
}

pub fn font_editor_italic(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("EditorItalic".into()))
}

pub fn font_editor_bold_italic(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("EditorBoldItalic".into()))
}

pub fn font_ui_regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn font_ui_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("UiBold".into()))
}

pub fn resolve_prose_font(family: &str, size: f32, bold: bool, italic: bool) -> FontId {
    let lower = family.to_ascii_lowercase();
    if lower.contains("serif") || lower.contains("literata") || lower.contains("charter") || lower.contains("roman") {
        if bold {
            return FontId::new(size, FontFamily::Name("ProseSerifBold".into()));
        }
        return FontId::new(size, FontFamily::Name("ProseSerif".into()));
    }
    if lower.contains("sans") || lower.contains("inter") || lower.contains("cantarell") {
        if bold {
            return font_ui_bold(size);
        }
        return font_ui_regular(size);
    }
    match (bold, italic) {
        (true, true) => font_editor_bold_italic(size),
        (true, false) => font_editor_bold(size),
        (false, true) => font_editor_italic(size),
        (false, false) => font_editor_regular(size),
    }
}

pub fn resolve_code_font(_family: &str, size: f32, bold: bool, italic: bool) -> FontId {
    match (bold, italic) {
        (true, true) => font_editor_bold_italic(size),
        (true, false) => font_editor_bold(size),
        (false, true) => font_editor_italic(size),
        (false, false) => font_editor_regular(size),
    }
}

fn try_load_font(defs: &mut FontDefinitions, key: &str, paths: &[&str]) -> bool {
    for p in paths {
        if let Ok(bytes) = fs::read(p) {
            defs.font_data
                .insert(key.to_string(), Arc::new(FontData::from_owned(bytes)));
            return true;
        }
    }
    false
}

pub fn configure_fonts(ctx: &Context) {
    let mut defs = FontDefinitions::default();

    let has_mono_reg = try_load_font(
        &mut defs,
        "editor_reg",
        &[
            "/usr/share/fonts/gsfonts/NimbusMonoPS-Regular.otf",
            "/usr/share/fonts/gnu-free/FreeMono.otf",
            "/usr/share/fonts/Adwaita/AdwaitaMono-Regular.ttf",
        ],
    );
    let has_mono_bold = try_load_font(
        &mut defs,
        "editor_bold",
        &[
            "/usr/share/fonts/gsfonts/NimbusMonoPS-Bold.otf",
            "/usr/share/fonts/gnu-free/FreeMonoBold.otf",
            "/usr/share/fonts/Adwaita/AdwaitaMono-Bold.ttf",
        ],
    );
    let has_mono_italic = try_load_font(
        &mut defs,
        "editor_italic",
        &[
            "/usr/share/fonts/gsfonts/NimbusMonoPS-Italic.otf",
            "/usr/share/fonts/gnu-free/FreeMonoOblique.otf",
            "/usr/share/fonts/Adwaita/AdwaitaMono-Italic.ttf",
        ],
    );
    let has_mono_bold_italic = try_load_font(
        &mut defs,
        "editor_bold_italic",
        &[
            "/usr/share/fonts/gsfonts/NimbusMonoPS-BoldItalic.otf",
            "/usr/share/fonts/gnu-free/FreeMonoBoldOblique.otf",
            "/usr/share/fonts/Adwaita/AdwaitaMono-BoldItalic.ttf",
        ],
    );

    let has_ui_reg = try_load_font(
        &mut defs,
        "ui_reg",
        &[
            "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
            "/usr/share/fonts/gnu-free/FreeSans.otf",
            "/usr/share/fonts/gsfonts/NimbusSans-Regular.otf",
        ],
    );
    let has_ui_bold = try_load_font(
        &mut defs,
        "ui_bold",
        &[
            "/usr/share/fonts/gnu-free/FreeSansBold.otf",
            "/usr/share/fonts/gsfonts/NimbusSans-Bold.otf",
            "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
        ],
    );

    let fallback_mono = defs
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    let fallback_prop = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    let mut reg_vec = Vec::new();
    if has_mono_reg {
        reg_vec.push("editor_reg".to_string());
    }
    reg_vec.extend(fallback_mono.clone());
    defs.families
        .insert(FontFamily::Name("EditorRegular".into()), reg_vec.clone());
    defs.families.insert(FontFamily::Monospace, reg_vec);

    let mut bold_vec = Vec::new();
    if has_mono_bold {
        bold_vec.push("editor_bold".to_string());
    } else if has_mono_reg {
        bold_vec.push("editor_reg".to_string());
    }
    bold_vec.extend(fallback_mono.clone());
    defs.families
        .insert(FontFamily::Name("EditorBold".into()), bold_vec);

    let mut italic_vec = Vec::new();
    if has_mono_italic {
        italic_vec.push("editor_italic".to_string());
    } else if has_mono_reg {
        italic_vec.push("editor_reg".to_string());
    }
    italic_vec.extend(fallback_mono.clone());
    defs.families
        .insert(FontFamily::Name("EditorItalic".into()), italic_vec);

    let mut bi_vec = Vec::new();
    if has_mono_bold_italic {
        bi_vec.push("editor_bold_italic".to_string());
    } else if has_mono_bold {
        bi_vec.push("editor_bold".to_string());
    }
    bi_vec.extend(fallback_mono);
    defs.families
        .insert(FontFamily::Name("EditorBoldItalic".into()), bi_vec);

    if has_ui_reg {
        if let Some(prop) = defs.families.get_mut(&FontFamily::Proportional) {
            prop.insert(0, "ui_reg".to_string());
        }
    }

    let has_serif_reg = try_load_font(
        &mut defs,
        "serif_reg",
        &[
            "/usr/share/fonts/gnu-free/FreeSerif.otf",
            "/usr/share/fonts/gsfonts/NimbusRoman-Regular.otf",
        ],
    );
    let has_serif_bold = try_load_font(
        &mut defs,
        "serif_bold",
        &[
            "/usr/share/fonts/gnu-free/FreeSerifBold.otf",
            "/usr/share/fonts/gsfonts/NimbusRoman-Bold.otf",
        ],
    );

    let mut ui_bold_vec = Vec::new();
    if has_ui_bold {
        ui_bold_vec.push("ui_bold".to_string());
    } else if has_ui_reg {
        ui_bold_vec.push("ui_reg".to_string());
    }
    ui_bold_vec.extend(fallback_prop.clone());
    defs.families
        .insert(FontFamily::Name("UiBold".into()), ui_bold_vec);

    let mut serif_vec = Vec::new();
    if has_serif_reg {
        serif_vec.push("serif_reg".to_string());
    }
    serif_vec.extend(fallback_prop.clone());
    defs.families
        .insert(FontFamily::Name("ProseSerif".into()), serif_vec);

    let mut serif_bold_vec = Vec::new();
    if has_serif_bold {
        serif_bold_vec.push("serif_bold".to_string());
    } else if has_serif_reg {
        serif_bold_vec.push("serif_reg".to_string());
    }
    serif_bold_vec.extend(fallback_prop);
    defs.families
        .insert(FontFamily::Name("ProseSerifBold".into()), serif_bold_vec);

    ctx.set_fonts(defs);
}

pub fn apply_theme(ctx: &Context, mode: ThemeMode) -> AccentPalette {
    let pal = AccentPalette::for_mode(mode);
    let mut visuals = match mode {
        ThemeMode::Dark => Visuals::dark(),
        ThemeMode::Light | ThemeMode::Sepia => Visuals::light(),
    };

    visuals.panel_fill = pal.bg_canvas;
    visuals.window_fill = pal.bg_popover;
    visuals.extreme_bg_color = pal.bg_canvas;
    visuals.faint_bg_color = pal.bg_code_block;
    visuals.override_text_color = Some(pal.fg_primary);
    visuals.widgets.noninteractive.bg_fill = pal.bg_canvas;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, pal.fg_primary);
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.4, pal.fg_primary);
    visuals.widgets.hovered.bg_fill = pal.bg_pill_button;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, pal.fg_primary);
    visuals.widgets.active.bg_fill = pal.bg_pill_button;
    visuals.widgets.active.fg_stroke = Stroke::new(1.6, pal.fg_primary);
    visuals.selection.bg_fill = pal.accent_orange.gamma_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.5, pal.accent_orange);

    ctx.set_visuals(visuals);
    pal
}

// ============================================================================
// Crisp Vector Icon Painters matching Apostrophe's GTK symbolic icons
// ============================================================================

pub fn draw_new_doc_icon(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.6, color);
    // Rounded page outline with top-right fold and bottom-right '+'
    let r = Rect::from_center_size(Pos2::new(center.x - 1.0, center.y - 1.0), Vec2::new(12.0, 13.0));
    painter.line_segment([r.left_top(), Pos2::new(r.right() - 4.0, r.top())], s);
    painter.line_segment(
        [Pos2::new(r.right() - 4.0, r.top()), Pos2::new(r.right(), r.top() + 4.0)],
        s,
    );
    painter.line_segment([Pos2::new(r.right(), r.top() + 4.0), Pos2::new(r.right(), r.bottom() - 5.0)], s);
    painter.line_segment([r.left_top(), r.left_bottom()], s);
    painter.line_segment([r.left_bottom(), Pos2::new(r.right() - 5.0, r.bottom())], s);
    // Small '+' at bottom right
    let pc = Pos2::new(r.right() + 0.5, r.bottom() - 0.5);
    painter.line_segment([Pos2::new(pc.x - 3.5, pc.y), Pos2::new(pc.x + 3.5, pc.y)], s);
    painter.line_segment([Pos2::new(pc.x, pc.y - 3.5), Pos2::new(pc.x, pc.y + 3.5)], s);
}

pub fn draw_chevron_down(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.6, color);
    painter.line_segment(
        [
            Pos2::new(center.x - 4.0, center.y - 1.5),
            Pos2::new(center.x, center.y + 2.5),
        ],
        s,
    );
    painter.line_segment(
        [
            Pos2::new(center.x, center.y + 2.5),
            Pos2::new(center.x + 4.0, center.y - 1.5),
        ],
        s,
    );
}

pub fn draw_chevron_up(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.6, color);
    painter.line_segment(
        [
            Pos2::new(center.x - 4.0, center.y + 2.0),
            Pos2::new(center.x, center.y - 2.0),
        ],
        s,
    );
    painter.line_segment(
        [
            Pos2::new(center.x, center.y - 2.0),
            Pos2::new(center.x + 4.0, center.y + 2.0),
        ],
        s,
    );
}

pub fn draw_chevron_left(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.7, color);
    painter.line_segment(
        [
            Pos2::new(center.x + 2.0, center.y - 4.5),
            Pos2::new(center.x - 2.5, center.y),
        ],
        s,
    );
    painter.line_segment(
        [
            Pos2::new(center.x - 2.5, center.y),
            Pos2::new(center.x + 2.0, center.y + 4.5),
        ],
        s,
    );
}

pub fn draw_search_icon(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.7, color);
    let c = Pos2::new(center.x - 1.5, center.y - 1.5);
    painter.circle_stroke(c, 5.2, s);
    painter.line_segment(
        [Pos2::new(c.x + 3.7, c.y + 3.7), Pos2::new(c.x + 7.5, c.y + 7.5)],
        Stroke::new(2.0, color),
    );
}

pub fn draw_preview_book_icon(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.5, color);
    let r = Rect::from_center_size(center, Vec2::new(14.0, 12.0));
    painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
    painter.line_segment(
        [Pos2::new(center.x, r.top()), Pos2::new(center.x, r.bottom())],
        s,
    );
    // Tiny horizontal lines inside pages
    for dy in [-2.5_f32, 0.0, 2.5] {
        painter.line_segment(
            [
                Pos2::new(r.left() + 2.2, center.y + dy),
                Pos2::new(center.x - 1.8, center.y + dy),
            ],
            Stroke::new(1.1, color),
        );
        painter.line_segment(
            [
                Pos2::new(center.x + 1.8, center.y + dy),
                Pos2::new(r.right() - 2.2, center.y + dy),
            ],
            Stroke::new(1.1, color),
        );
    }
}

pub fn draw_hamburger_icon(painter: &Painter, center: Pos2, color: Color32) {
    let s = Stroke::new(1.7, color);
    for dy in [-4.5_f32, 0.0, 4.5] {
        painter.line_segment(
            [
                Pos2::new(center.x - 6.5, center.y + dy),
                Pos2::new(center.x + 6.5, center.y + dy),
            ],
            s,
        );
    }
}

pub fn draw_window_button(
    painter: &Painter,
    center: Pos2,
    pill_bg: Color32,
    fg: Color32,
    kind: char,
) {
    painter.circle_filled(center, 12.0, pill_bg);
    let s = Stroke::new(1.6, fg);
    match kind {
        '-' => {
            painter.line_segment(
                [
                    Pos2::new(center.x - 4.0, center.y + 2.5),
                    Pos2::new(center.x + 4.0, center.y + 2.5),
                ],
                s,
            );
        }
        '□' => {
            let r = Rect::from_center_size(center, Vec2::new(7.5, 7.5));
            painter.rect_stroke(r, CornerRadius::ZERO, s, StrokeKind::Middle);
        }
        'x' => {
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.5, center.y - 3.5),
                    Pos2::new(center.x + 3.5, center.y + 3.5),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.5, center.y + 3.5),
                    Pos2::new(center.x + 3.5, center.y - 3.5),
                ],
                s,
            );
        }
        _ => {}
    }
}

/// Draws the 12 bottom formatting toolbar icons (`0..12`) matching Apostrophe's bottom-left bar.
pub fn draw_toolbar_icon(painter: &Painter, center: Pos2, fg: Color32, idx: usize) {
    let s = Stroke::new(1.5, fg);
    match idx {
        // 0: Bold 'a'
        0 => {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "a",
                font_ui_bold(16.5),
                fg,
            );
        }
        // 1: Italic 'a'
        1 => {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "a",
                font_editor_bold_italic(17.0),
                fg,
            );
        }
        // 2: Strikethrough 'a' with horizontal bar
        2 => {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "a",
                font_ui_regular(15.5),
                fg,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 6.0, center.y + 0.5),
                    Pos2::new(center.x + 6.0, center.y + 0.5),
                ],
                Stroke::new(1.6, fg),
            );
        }
        // 3: Heading 'H1'
        3 => {
            painter.text(
                Pos2::new(center.x - 2.0, center.y),
                egui::Align2::CENTER_CENTER,
                "H",
                font_ui_bold(13.5),
                fg,
            );
            painter.text(
                Pos2::new(center.x + 4.5, center.y + 1.5),
                egui::Align2::CENTER_CENTER,
                "1",
                font_ui_bold(9.5),
                fg,
            );
        }
        // 4: Unordered bullet list
        4 => {
            for dy in [-4.5_f32, 0.0, 4.5] {
                painter.circle_filled(Pos2::new(center.x - 5.5, center.y + dy), 1.3, fg);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 2.0, center.y + dy),
                        Pos2::new(center.x + 6.0, center.y + dy),
                    ],
                    s,
                );
            }
        }
        // 5: Ordered numbered list (1 2 3 + lines)
        5 => {
            for (i, dy) in [-4.5_f32, 0.0, 4.5].into_iter().enumerate() {
                painter.text(
                    Pos2::new(center.x - 5.5, center.y + dy),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", i + 1),
                    font_ui_bold(6.5),
                    fg,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - 1.5, center.y + dy),
                        Pos2::new(center.x + 6.0, center.y + dy),
                    ],
                    s,
                );
            }
        }
        // 6: Task list checkbox [v]
        6 => {
            let r = Rect::from_center_size(center, Vec2::new(12.5, 12.5));
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.2, center.y + 0.2),
                    Pos2::new(center.x - 0.8, center.y + 2.8),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 0.8, center.y + 2.8),
                    Pos2::new(center.x + 3.6, center.y - 2.4),
                ],
                s,
            );
        }
        // 7: Blockquote “”
        7 => {
            painter.text(
                Pos2::new(center.x, center.y + 3.0),
                egui::Align2::CENTER_CENTER,
                "“”",
                font_ui_bold(18.0),
                fg,
            );
        }
        // 8: Code <>
        8 => {
            painter.line_segment(
                [
                    Pos2::new(center.x - 2.0, center.y - 4.0),
                    Pos2::new(center.x - 6.0, center.y),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 6.0, center.y),
                    Pos2::new(center.x - 2.0, center.y + 4.0),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 2.0, center.y - 4.0),
                    Pos2::new(center.x + 6.0, center.y),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 6.0, center.y),
                    Pos2::new(center.x + 2.0, center.y + 4.0),
                ],
                s,
            );
        }
        // 9: Link icon
        9 => {
            painter.circle_stroke(Pos2::new(center.x, center.y), 5.0, s);
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.5, center.y + 3.5),
                    Pos2::new(center.x + 3.5, center.y - 3.5),
                ],
                s,
            );
        }
        // 10: Image frame icon
        10 => {
            let r = Rect::from_center_size(center, Vec2::new(13.0, 11.0));
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.circle_filled(Pos2::new(r.left() + 3.5, r.top() + 3.5), 1.3, fg);
            painter.line_segment(
                [
                    Pos2::new(r.left() + 2.0, r.bottom() - 2.0),
                    Pos2::new(center.x, center.y + 0.5),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x, center.y + 0.5),
                    Pos2::new(r.right() - 2.0, r.bottom() - 2.0),
                ],
                s,
            );
        }
        // 11: Table grid icon
        _ => {
            let r = Rect::from_center_size(center, Vec2::new(12.5, 11.5));
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(r.left(), r.top() + 4.0),
                    Pos2::new(r.right(), r.top() + 4.0),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(r.left(), r.top() + 7.8),
                    Pos2::new(r.right(), r.top() + 7.8),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x, r.top() + 4.0),
                    Pos2::new(center.x, r.bottom()),
                ],
                s,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palettes_match_sampled_hex() {
        let light = AccentPalette::light();
        assert_eq!(light.bg_canvas, Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(light.fg_primary, Color32::from_rgb(0x2e, 0x34, 0x36));
        assert_eq!(light.bg_code_block, Color32::from_rgb(0xe0, 0xe0, 0xe0));

        let sepia = AccentPalette::sepia();
        assert_eq!(sepia.bg_canvas, Color32::from_rgb(0xf9, 0xf3, 0xe9));
        assert_eq!(sepia.fg_primary, Color32::from_rgb(0x4f, 0x39, 0x15));
        assert_eq!(sepia.bg_code_block, Color32::from_rgb(0xdf, 0xd7, 0xc9));

        let dark = AccentPalette::dark();
        assert_eq!(dark.bg_canvas, Color32::from_rgb(0x2b, 0x2b, 0x2f));
        assert_eq!(dark.bg_headerbar, Color32::from_rgb(0x2e, 0x2e, 0x32));
        assert_eq!(dark.fg_primary, Color32::from_rgb(0xee, 0xee, 0xee));
        assert_eq!(dark.bg_code_block, Color32::from_rgb(0x3c, 0x3c, 0x42));
    }
}
