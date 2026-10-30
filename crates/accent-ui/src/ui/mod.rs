pub mod theme;

use accent_core::{MarkdownDialect, PreviewBlock, PreviewInline};
use accent_session::{
    Action, ActiveOverlay, ExportFormat, FormatCommand, I18n, LineWidthMode, Locale,
    PreviewLayout, Session, StatMetric, ThemeMode,
};
use egui::{
    epaint::StrokeKind, text::LayoutJob, Align2, Color32, Context, CornerRadius, FontId, Id,
    Margin, Order, Painter, Pos2, Rect, ScrollArea, Sense, Stroke, TextEdit, TextFormat, Ui, Vec2,
    ViewportId,
};
use std::sync::Arc;
use theme::{
    apply_theme, draw_chevron_down, draw_chevron_left, draw_chevron_up, draw_hamburger_icon,
    draw_new_doc_icon, draw_search_icon, draw_toolbar_icon, draw_view_mode_icon,
    draw_window_button, font_editor_regular, font_ui_bold, font_ui_regular, resolve_code_font,
    resolve_prose_font, AccentPalette,
};

fn trigger_open_file_dialog(session: &Session, actions: &mut Vec<Action>) {
    let is_headless_xvfb = std::env::var("DISPLAY").as_deref() == Ok(":99");
    if !is_headless_xvfb {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "txt"])
            .set_directory(&session.file_dialog_dir)
            .pick_file()
        {
            actions.push(Action::OpenFile(path));
            actions.push(Action::SetOverlay(ActiveOverlay::None));
            return;
        }
    }
    actions.push(Action::SetOverlay(ActiveOverlay::OpenFileDialog));
}

fn trigger_save_as_file_dialog(session: &Session, actions: &mut Vec<Action>) {
    let is_headless_xvfb = std::env::var("DISPLAY").as_deref() == Ok(":99");
    if !is_headless_xvfb {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .set_directory(&session.file_dialog_dir)
            .set_file_name(&session.document_title)
            .save_file()
        {
            actions.push(Action::SaveAsFile(path));
            actions.push(Action::SetOverlay(ActiveOverlay::None));
            return;
        }
    }
    actions.push(Action::SetOverlay(ActiveOverlay::SaveFileDialog));
}

fn active_view_mode_index(session: &Session) -> usize {
    if !session.preview_active {
        if session.view_while_typing {
            1 // Anteprima in-line • (default)
        } else {
            0 // Modifica (raw source)
        }
    } else {
        match session.preview_layout {
            PreviewLayout::HalfWidth => 2,  // Anteprima split a destra
            PreviewLayout::HalfHeight => 3, // Anteprima split sotto
            PreviewLayout::FullWidth | PreviewLayout::Windowed => 4, // Solo lettura
        }
    }
}

pub fn render_ui(
    ctx: &Context,
    session: &Session,
    actions: &mut Vec<Action>,
    sync_cursor_to_widget: &mut bool,
) {
    let pal = apply_theme(ctx, session.theme);

    // Wake up auto-hidden top/bottom bars when the mouse pointer approaches the top or bottom window edge
    if session.chrome_autohidden {
        let win_rect = ctx.screen_rect();
        let wake_up = ctx.input(|i| {
            let moved = i.pointer.delta().length_sq() > 0.25;
            if let Some(pos) = i.pointer.hover_pos().or_else(|| i.pointer.latest_pos()) {
                let near_top = pos.y <= win_rect.top() + 54.0;
                let near_bottom = pos.y >= win_rect.bottom() - 54.0;
                near_top || near_bottom || (moved && (pos.y <= 72.0 || pos.y >= win_rect.bottom() - 72.0))
            } else {
                false
            }
        });
        if wake_up {
            actions.push(Action::SetChromeAutohidden(false));
        }
    }

    // Global keyboard shortcuts
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            if session.overlay != ActiveOverlay::None {
                actions.push(Action::SetOverlay(ActiveOverlay::None));
            } else if session.search_bar_open {
                actions.push(Action::ToggleSearchBar);
            } else if session.chrome_autohidden {
                actions.push(Action::SetChromeAutohidden(false));
            }
        }
        if i.key_pressed(egui::Key::F10) {
            actions.push(Action::SetOverlay(ActiveOverlay::MainMenu));
        }
        if i.modifiers.ctrl && !i.modifiers.shift && !i.modifiers.alt && i.key_pressed(egui::Key::L)
        {
            actions.push(Action::ToggleViewWhileTyping);
        }
        if i.modifiers.ctrl && !i.modifiers.shift && !i.modifiers.alt && i.key_pressed(egui::Key::E)
        {
            actions.push(Action::SetOverlay(ActiveOverlay::ExportDialog));
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::E) {
            actions.push(Action::SetEditorOnlyRaw);
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::P) {
            let toggle = !(session.preview_active && session.preview_layout == PreviewLayout::HalfWidth);
            actions.push(Action::SetPreviewMode {
                active: toggle,
                layout: PreviewLayout::HalfWidth,
            });
        }
        if i.modifiers.ctrl && i.modifiers.alt && i.key_pressed(egui::Key::P) {
            let toggle = !(session.preview_active && session.preview_layout == PreviewLayout::FullWidth);
            actions.push(Action::SetPreviewMode {
                active: toggle,
                layout: PreviewLayout::FullWidth,
            });
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::J) {
            let toggle = !(session.preview_active && session.preview_layout == PreviewLayout::HalfHeight);
            actions.push(Action::SetPreviewMode {
                active: toggle,
                layout: PreviewLayout::HalfHeight,
            });
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::W) {
            let toggle = !(session.preview_active && session.preview_layout == PreviewLayout::Windowed);
            actions.push(Action::SetPreviewMode {
                active: toggle,
                layout: PreviewLayout::Windowed,
            });
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::Comma) {
            actions.push(Action::SetOverlay(ActiveOverlay::PreferencesDialog));
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
            actions.push(Action::ToggleFocusMode);
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::T) {
            actions.push(Action::ToggleHemingway);
        }
        if i.modifiers.ctrl && (i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::F)) {
            actions.push(Action::ToggleSearchBar);
        }
        if i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::S) {
            actions.push(Action::SetChromeAutohidden(false));
            actions.push(Action::SaveDocument);
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S) {
            actions.push(Action::SetChromeAutohidden(false));
            trigger_save_as_file_dialog(session, actions);
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
            actions.push(Action::SetChromeAutohidden(false));
            actions.push(Action::NewDocument);
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
            actions.push(Action::SetChromeAutohidden(false));
            trigger_open_file_dialog(session, actions);
        }
        if i.key_pressed(egui::Key::Home)
            || i.key_pressed(egui::Key::End)
            || i.key_pressed(egui::Key::PageUp)
            || i.key_pressed(egui::Key::PageDown)
        {
            actions.push(Action::SetChromeAutohidden(false));
        }
        if i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::B) {
            actions.push(Action::ApplyFormat(FormatCommand::Bold));
        }
        if i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::I) {
            actions.push(Action::ApplyFormat(FormatCommand::Italic));
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::X) {
            actions.push(Action::ApplyFormat(FormatCommand::Strikethrough));
        }
    });

    let chrome_hidden = session.chrome_autohidden
        && session.overlay == ActiveOverlay::None
        && !session.search_bar_open
        && !session.recovery_banner_active;
    let chrome_alpha =
        ctx.animate_bool_with_time(Id::new("accent_zen_chrome_alpha"), !chrome_hidden, 0.16);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(pal.bg_canvas))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter_at(rect);

            // 1. Render Main Content Area (Editor Canvas, Split Preview, or Full Preview)
            render_workspace_area(
                ui,
                &painter,
                rect,
                session,
                &pal,
                actions,
                sync_cursor_to_widget,
            );

            // 2. Render Unified Slim Top Bar (fades to 0% opacity in Zen Mode while typing)
            if chrome_alpha > 0.01 {
                let mut faded_pal = pal;
                if chrome_alpha < 0.99 {
                    faded_pal.bg_headerbar = pal.bg_headerbar.gamma_multiply(chrome_alpha);
                    faded_pal.border_headerbar = pal.border_headerbar.gamma_multiply(chrome_alpha);
                    faded_pal.fg_primary = pal.fg_primary.gamma_multiply(chrome_alpha);
                    faded_pal.fg_muted = pal.fg_muted.gamma_multiply(chrome_alpha);
                    faded_pal.bg_pill_button = pal.bg_pill_button.gamma_multiply(chrome_alpha);
                    faded_pal.accent_orange = pal.accent_orange.gamma_multiply(chrome_alpha);
                }
                render_headerbar(ctx, ui, &painter, rect, session, &faded_pal, actions);
            }

            // 3. Render Crash Recovery Notification Banner if restored from dirty WAL journal
            if session.recovery_banner_active {
                render_recovery_banner(ui, &painter, rect, session, &pal, actions);
            }

            // 4. Render Floating Bottom Bar (fades to 0% opacity in Zen Mode while typing)
            if chrome_alpha > 0.01 {
                let mut faded_pal = pal;
                if chrome_alpha < 0.99 {
                    faded_pal.bg_headerbar = pal.bg_headerbar.gamma_multiply(chrome_alpha);
                    faded_pal.bg_popover = pal.bg_popover.gamma_multiply(chrome_alpha);
                    faded_pal.bg_canvas = pal.bg_canvas.gamma_multiply(chrome_alpha);
                    faded_pal.border_headerbar = pal.border_headerbar.gamma_multiply(chrome_alpha);
                    faded_pal.fg_primary = pal.fg_primary.gamma_multiply(chrome_alpha);
                    faded_pal.fg_muted = pal.fg_muted.gamma_multiply(chrome_alpha);
                    faded_pal.bg_pill_button = pal.bg_pill_button.gamma_multiply(chrome_alpha);
                    faded_pal.accent_orange = pal.accent_orange.gamma_multiply(chrome_alpha);
                }
                render_bottom_bar(ui, &painter, rect, session, &faded_pal, actions);
            }
        });

    // 5. Render Slim Floating Search & Replace Panel in Foreground Layer when open
    if session.search_bar_open {
        let win_rect = ctx.screen_rect();
        let search_bounds = Rect::from_min_max(
            Pos2::new((win_rect.right() - 404.0).max(16.0), 54.0),
            Pos2::new(win_rect.right() - 16.0, 146.0),
        );
        egui::Area::new(Id::new("accent_floating_search_panel"))
            .order(Order::Foreground)
            .fixed_pos(search_bounds.left_top())
            .show(ctx, |ui| {
                ui.set_min_size(search_bounds.size());
                let painter = ui.painter().clone();
                render_search_bar(ui, &painter, search_bounds, session, &pal, actions);
            });
    }

    // 6. Render Active Popover or Modal Dialog in Foreground Layer so clicks are never stolen by TextEdit
    if session.overlay != ActiveOverlay::None {
        let win_rect = ctx.screen_rect();
        let bounds = overlay_bounds(win_rect, session);
        egui::Area::new(Id::new((
            "accent_active_overlay_layer",
            format!("{:?}", session.overlay),
        )))
        .order(Order::Foreground)
        .fixed_pos(bounds.left_top())
        .show(ctx, |ui| {
            ui.set_min_size(bounds.size());
            let painter = ui.painter().clone();
            render_popovers(ctx, ui, &painter, win_rect, session, &pal, actions);
        });
    }

    // 7. Separate OS Window Preview via Multi-Viewport API when PreviewLayout::Windowed is active
    if session.preview_active && session.preview_layout == PreviewLayout::Windowed {
        let vp_id = ViewportId::from_hash_of("accent_separate_preview_window");
        let builder = egui::ViewportBuilder::default()
            .with_title(format!("{} — Preview", session.document_title))
            .with_inner_size([720.0, 920.0]);
        ctx.show_viewport_immediate(vp_id, builder, |vp_ctx, _class| {
            let vp_pal = apply_theme(vp_ctx, session.theme);
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(vp_pal.bg_preview))
                .show(vp_ctx, |vp_ui| {
                    let vp_rect = vp_ui.max_rect();
                    let vp_painter = vp_ui.painter_at(vp_rect);
                    render_preview_pane(vp_ui, &vp_painter, vp_rect, session, &vp_pal);
                });
            if vp_ctx.input(|i| i.viewport().close_requested()) {
                actions.push(Action::SetPreviewMode {
                    active: false,
                    layout: PreviewLayout::Windowed,
                });
            }
        });
    }
}

fn overlay_bounds(win_rect: Rect, session: &Session) -> Rect {
    match session.overlay {
        ActiveOverlay::None => Rect::NOTHING,
        ActiveOverlay::OpenMenu => {
            let n_recent = session.recent_files.len().max(1);
            let h = 136.0 + (n_recent as f32) * 38.0;
            Rect::from_min_max(Pos2::new(18.0, 44.0), Pos2::new(316.0, 44.0 + h))
        }
        ActiveOverlay::SaveMenu => {
            Rect::from_min_max(Pos2::new(124.0, 44.0), Pos2::new(374.0, 326.0))
        }
        ActiveOverlay::PreviewMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 352.0, 44.0),
            Pos2::new(win_rect.right() - 36.0, 240.0),
        ),
        ActiveOverlay::MainMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 302.0, 44.0),
            Pos2::new(win_rect.right() - 6.0, 444.0),
        ),
        ActiveOverlay::LineWidthMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 310.0, 44.0),
            Pos2::new(win_rect.right() - 70.0, 232.0),
        ),
        ActiveOverlay::StatsMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 266.0, win_rect.bottom() - 232.0),
            Pos2::new(win_rect.right() - 18.0, win_rect.bottom() - 58.0),
        ),
        ActiveOverlay::PreferencesDialog
        | ActiveOverlay::ExportDialog
        | ActiveOverlay::OpenFileDialog
        | ActiveOverlay::SaveFileDialog
        | ActiveOverlay::ShortcutsDialog
        | ActiveOverlay::AboutDialog => {
            Rect::from_min_max(Pos2::new(win_rect.left(), 44.0), win_rect.right_bottom())
        }
    }
}

fn render_headerbar(
    _ctx: &Context,
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    let hb_height = 44.0;
    let hb_rect = Rect::from_min_max(
        win_rect.left_top(),
        Pos2::new(win_rect.right(), win_rect.top() + hb_height),
    );
    painter.rect_filled(hb_rect, CornerRadius::ZERO, pal.bg_headerbar);
    painter.line_segment(
        [
            Pos2::new(win_rect.left(), win_rect.top() + hb_height + 0.5),
            Pos2::new(win_rect.right(), win_rect.top() + hb_height + 0.5),
        ],
        Stroke::new(1.0, pal.border_headerbar),
    );

    let y_mid = win_rect.top() + 22.0;
    let fg = pal.fg_primary;
    let w = win_rect.right();

    // Left controls: New Doc (`+`), Open (`Apri ▾`), Split Save (`Salva • | ▾`)
    let new_doc_rect = Rect::from_center_size(Pos2::new(26.0, y_mid), Vec2::new(32.0, 32.0));
    let open_rect = Rect::from_min_max(Pos2::new(50.0, 6.0), Pos2::new(112.0, 38.0));
    let save_btn_rect = Rect::from_min_max(Pos2::new(124.0, 6.0), Pos2::new(186.0, 38.0));
    let save_menu_rect = Rect::from_min_max(Pos2::new(186.0, 6.0), Pos2::new(214.0, 38.0));

    // Right controls (OS handles window buttons natively via `with_decorations(true)`):
    // Line-Width Pill (`w - 166.0`), Search (`w - 116.0`), Compact Editing Mode Pill (`w - 70.0`), Main Menu `☰` (`w - 26.0`)
    let menu_center = Pos2::new(w - 26.0, y_mid);
    let menu_rect = Rect::from_center_size(menu_center, Vec2::new(32.0, 32.0));
    let preview_pill_rect =
        Rect::from_center_size(Pos2::new(w - 70.0, y_mid), Vec2::new(44.0, 30.0));
    let search_rect = Rect::from_center_size(Pos2::new(w - 114.0, y_mid), Vec2::new(32.0, 32.0));
    let lw_rect = Rect::from_center_size(Pos2::new(w - 164.0, y_mid), Vec2::new(52.0, 28.0));

    // 1. Left: New doc icon (x=26)
    let new_hit = ui
        .interact(new_doc_rect, ui.id().with("new_doc_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_new_doc"));
    if new_hit.hovered() {
        painter.rect_filled(new_doc_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    draw_new_doc_icon(painter, Pos2::new(26.0, y_mid), fg);
    if new_hit.clicked() {
        actions.push(Action::NewDocument);
    }

    // 2. Left: "Apri ▾" (x=50..112)
    let open_hit = ui
        .interact(open_rect, ui.id().with("open_menu_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_open"));
    if session.overlay == ActiveOverlay::OpenMenu || open_hit.hovered() {
        painter.rect_filled(open_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    let open_label = I18n::tr(session.locale, "open");
    painter.text(
        Pos2::new(58.0, y_mid),
        Align2::LEFT_CENTER,
        open_label,
        font_ui_bold(14.0),
        fg,
    );
    draw_chevron_down(painter, Pos2::new(99.0, y_mid + 0.5), fg);
    if open_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::OpenMenu));
    }

    // 3. Left: Split-button "Salva • | ▾" (x=124..214)
    let save_hit = ui
        .interact(save_btn_rect, ui.id().with("save_main_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_save"));
    let save_menu_hit = ui
        .interact(
            save_menu_rect,
            ui.id().with("save_menu_btn"),
            Sense::click(),
        )
        .on_hover_text(I18n::tr(session.locale, "tooltip_save_menu"));
    let save_combined = Rect::from_min_max(Pos2::new(124.0, 6.0), Pos2::new(214.0, 38.0));
    painter.rect_filled(
        save_combined,
        CornerRadius::same(6),
        if session.overlay == ActiveOverlay::SaveMenu
            || save_hit.hovered()
            || save_menu_hit.hovered()
        {
            pal.bg_pill_button
        } else {
            pal.bg_pill_button.gamma_multiply(0.55)
        },
    );
    let save_label = I18n::tr(session.locale, "save");
    painter.text(
        Pos2::new(134.0, y_mid),
        Align2::LEFT_CENTER,
        save_label,
        font_ui_bold(14.0),
        fg,
    );
    if session.is_dirty {
        painter.circle_filled(Pos2::new(178.0, y_mid), 3.4, pal.accent_orange);
    }
    painter.line_segment(
        [Pos2::new(186.0, 11.0), Pos2::new(186.0, 33.0)],
        Stroke::new(1.0, pal.border_headerbar),
    );
    draw_chevron_down(painter, Pos2::new(199.0, y_mid + 0.5), fg);
    if save_hit.clicked() {
        actions.push(Action::SaveDocument);
    }
    if save_menu_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::SaveMenu));
    }

    // 4. Center: Discreet document filename keeping the 3-column balance
    painter.text(
        Pos2::new(win_rect.center().x, y_mid),
        Align2::CENTER_CENTER,
        &session.document_title,
        font_ui_regular(13.8),
        pal.fg_primary.gamma_multiply(0.85),
    );

    // 5. Right: Line-Width selector pill (`80ch`, `110ch`, `140ch`, `Fluid`)
    let lw_hit = ui
        .interact(lw_rect, ui.id().with("lw_header_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_line_width"));
    let lw_fill = if session.overlay == ActiveOverlay::LineWidthMenu || lw_hit.hovered() {
        pal.bg_pill_button
    } else {
        pal.bg_pill_button.gamma_multiply(0.65)
    };
    painter.rect_filled(lw_rect, CornerRadius::same(6), lw_fill);
    painter.text(
        lw_rect.center(),
        Align2::CENTER_CENTER,
        session.line_width.short_badge(),
        font_ui_bold(11.5),
        fg,
    );
    if lw_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::LineWidthMenu));
    }

    // 6. Right: Search icon button `🔍`
    let search_hit = ui
        .interact(search_rect, ui.id().with("search_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_search"));
    if session.search_bar_open || search_hit.hovered() {
        painter.rect_filled(search_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    draw_search_icon(painter, search_rect.center(), fg);
    if search_hit.clicked() {
        actions.push(Action::ToggleSearchBar);
    }

    // 7. Right: Compact Editing Mode Pill (Icon + `▾` ONLY, no text)
    let prev_hit = ui
        .interact(
            preview_pill_rect,
            ui.id().with("preview_btn"),
            Sense::click(),
        )
        .on_hover_text(I18n::tr(session.locale, "tooltip_preview"));
    let prev_fill = if session.overlay == ActiveOverlay::PreviewMenu || prev_hit.hovered() {
        pal.bg_pill_button
    } else {
        pal.bg_pill_button.gamma_multiply(0.65)
    };
    painter.rect_filled(preview_pill_rect, CornerRadius::same(6), prev_fill);
    let mode_idx = active_view_mode_index(session);
    draw_view_mode_icon(
        painter,
        Pos2::new(preview_pill_rect.left() + 14.5, y_mid),
        fg,
        pal.accent_orange,
        mode_idx,
    );
    draw_chevron_down(
        painter,
        Pos2::new(preview_pill_rect.right() - 9.5, y_mid + 0.5),
        fg,
    );
    if prev_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::PreviewMenu));
    }

    // 8. Far Right: Hamburger Main Menu button `☰`
    let menu_hit = ui
        .interact(menu_rect, ui.id().with("main_menu_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_main_menu"));
    if session.overlay == ActiveOverlay::MainMenu || menu_hit.hovered() {
        painter.rect_filled(menu_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    draw_hamburger_icon(painter, menu_center, fg);
    if menu_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::MainMenu));
    }
}

fn render_search_bar(
    ui: &mut Ui,
    painter: &Painter,
    panel_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    // Soft ambient shadow under the floating search & replace panel
    painter.rect_filled(
        panel_rect.translate(Vec2::new(0.0, 4.0)),
        CornerRadius::same(12),
        Color32::from_rgba_unmultiplied(0, 0, 0, 32),
    );
    painter.rect_filled(panel_rect, CornerRadius::same(12), pal.bg_popover);
    painter.rect_stroke(
        panel_rect,
        CornerRadius::same(12),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let find_hint = match session.locale {
        Locale::It => "Trova nel documento...",
        Locale::Es => "Buscar en el documento...",
        Locale::En => "Find in document...",
    };
    let replace_hint = match session.locale {
        Locale::It => "Sostituisci con...",
        Locale::Es => "Reemplazar con...",
        Locale::En => "Replace with...",
    };

    // Compute match counter ("1 di 4")
    let (cur_match, total_matches) = if session.search_query.is_empty() {
        (0usize, 0usize)
    } else {
        let text_lower = session.buffer.as_str().to_lowercase();
        let q_lower = session.search_query.to_lowercase();
        let cur_b = session.buffer.cursor_byte();
        let mut total = 0usize;
        let mut current = 1usize;
        for (byte_pos, _) in text_lower.match_indices(&q_lower) {
            total += 1;
            if byte_pos <= cur_b {
                current = total;
            }
        }
        if total == 0 {
            (0, 0)
        } else {
            (current, total)
        }
    };
    let counter_str = match session.locale {
        Locale::It => format!("{cur_match} di {total_matches}"),
        Locale::Es => format!("{cur_match} de {total_matches}"),
        Locale::En => format!("{cur_match} of {total_matches}"),
    };

    // Row 1: Find input + "1 di 4" counter + ↑ / ↓ + ×
    let row1_y = panel_rect.top() + 24.0;
    let find_box = Rect::from_min_max(
        Pos2::new(panel_rect.left() + 10.0, row1_y - 14.0),
        Pos2::new(panel_rect.left() + 266.0, row1_y + 14.0),
    );
    painter.rect_filled(find_box, CornerRadius::same(7), pal.bg_canvas);
    painter.rect_stroke(
        find_box,
        CornerRadius::same(7),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );
    draw_search_icon(
        painter,
        Pos2::new(find_box.left() + 14.0, row1_y),
        pal.fg_muted,
    );

    let mut q = session.search_query.clone();
    let mut find_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                Pos2::new(find_box.left() + 28.0, find_box.top() + 4.0),
                Pos2::new(find_box.right() - 56.0, find_box.bottom() - 4.0),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let q_resp = find_ui.add(
        TextEdit::singleline(&mut q)
            .id(Id::new("accent_floating_search_input"))
            .frame(false)
            .hint_text(find_hint)
            .desired_width(164.0)
            .font(font_ui_regular(13.0)),
    );
    if q_resp.changed() {
        actions.push(Action::SetSearchQuery(q));
    }

    painter.text(
        Pos2::new(find_box.right() - 8.0, row1_y),
        Align2::RIGHT_CENTER,
        &counter_str,
        font_ui_bold(11.5),
        if total_matches > 0 {
            pal.accent_orange
        } else {
            pal.fg_muted
        },
    );

    // Compact ↑ / ↓ / × buttons on Row 1
    let prev_r = Rect::from_center_size(Pos2::new(panel_rect.right() - 94.0, row1_y), Vec2::new(26.0, 26.0));
    let next_r = Rect::from_center_size(Pos2::new(panel_rect.right() - 62.0, row1_y), Vec2::new(26.0, 26.0));
    let close_r = Rect::from_center_size(Pos2::new(panel_rect.right() - 26.0, row1_y), Vec2::new(26.0, 26.0));

    for (btn_r, id_salt, symbol, act) in [
        (prev_r, "search_prev_btn", "↑", Action::FindPrev),
        (next_r, "search_next_btn", "↓", Action::FindNext),
        (close_r, "search_close_btn", "×", Action::ToggleSearchBar),
    ] {
        let hit = ui.interact(btn_r, ui.id().with(id_salt), Sense::click());
        painter.rect_filled(
            btn_r,
            CornerRadius::same(6),
            if hit.hovered() {
                pal.bg_pill_button
            } else {
                pal.bg_pill_button.gamma_multiply(0.55)
            },
        );
        painter.text(
            btn_r.center(),
            Align2::CENTER_CENTER,
            symbol,
            font_ui_bold(13.5),
            pal.fg_primary,
        );
        if hit.clicked() {
            actions.push(act);
        }
    }

    // Row 2: Replace input + "Sostituisci" + "Tutti"
    let row2_y = panel_rect.top() + 64.0;
    let rep_box = Rect::from_min_max(
        Pos2::new(panel_rect.left() + 10.0, row2_y - 14.0),
        Pos2::new(panel_rect.left() + 236.0, row2_y + 14.0),
    );
    painter.rect_filled(rep_box, CornerRadius::same(7), pal.bg_canvas);
    painter.rect_stroke(
        rep_box,
        CornerRadius::same(7),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let mut r = session.replace_text.clone();
    let mut rep_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rep_box.shrink2(Vec2::new(10.0, 4.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let r_resp = rep_ui.add(
        TextEdit::singleline(&mut r)
            .id(Id::new("accent_floating_replace_input"))
            .frame(false)
            .hint_text(replace_hint)
            .desired_width(200.0)
            .font(font_ui_regular(13.0)),
    );
    if r_resp.changed() {
        actions.push(Action::SetReplaceText(r));
    }
    if !q_resp.has_focus() && !r_resp.has_focus() {
        q_resp.request_focus();
    }

    let rep_one_r = Rect::from_min_max(
        Pos2::new(panel_rect.left() + 244.0, row2_y - 13.0),
        Pos2::new(panel_rect.right() - 64.0, row2_y + 13.0),
    );
    let rep_all_r = Rect::from_min_max(
        Pos2::new(panel_rect.right() - 58.0, row2_y - 13.0),
        Pos2::new(panel_rect.right() - 10.0, row2_y + 13.0),
    );
    let h_one = ui.interact(rep_one_r, ui.id().with("search_rep_one"), Sense::click());
    painter.rect_filled(
        rep_one_r,
        CornerRadius::same(6),
        if h_one.hovered() {
            pal.accent_orange
        } else {
            pal.bg_pill_button
        },
    );
    painter.text(
        rep_one_r.center(),
        Align2::CENTER_CENTER,
        I18n::tr(session.locale, "replace_btn"),
        font_ui_bold(12.0),
        if h_one.hovered() {
            Color32::WHITE
        } else {
            pal.fg_primary
        },
    );
    if h_one.clicked() {
        actions.push(Action::ReplaceNext);
    }

    let all_lbl = match session.locale {
        Locale::It => "Tutti",
        Locale::Es => "Todos",
        Locale::En => "All",
    };
    let h_all = ui.interact(rep_all_r, ui.id().with("search_rep_all"), Sense::click());
    painter.rect_filled(
        rep_all_r,
        CornerRadius::same(6),
        if h_all.hovered() {
            pal.accent_orange
        } else {
            pal.bg_pill_button
        },
    );
    painter.text(
        rep_all_r.center(),
        Align2::CENTER_CENTER,
        all_lbl,
        font_ui_bold(12.0),
        if h_all.hovered() {
            Color32::WHITE
        } else {
            pal.fg_primary
        },
    );
    if h_all.clicked() {
        actions.push(Action::ReplaceNext);
    }
}

fn render_recovery_banner(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    let top_offset = 44.0;
    let banner_rect = Rect::from_min_max(
        Pos2::new(win_rect.left(), top_offset),
        Pos2::new(win_rect.right(), top_offset + 44.0),
    );
    let banner_bg = match session.theme {
        ThemeMode::Dark => Color32::from_rgb(0x42, 0x35, 0x22),
        ThemeMode::Sepia => Color32::from_rgb(0xf2, 0xe4, 0xca),
        ThemeMode::Light => Color32::from_rgb(0xfd, 0xf2, 0xdb),
    };
    let banner_border = Color32::from_rgb(0xc8, 0x8a, 0x2e);
    painter.rect_filled(banner_rect, CornerRadius::ZERO, banner_bg);
    painter.line_segment(
        [
            Pos2::new(banner_rect.left(), banner_rect.bottom()),
            Pos2::new(banner_rect.right(), banner_rect.bottom()),
        ],
        Stroke::new(1.0, banner_border),
    );

    let y_mid = banner_rect.center().y;
    let msg = I18n::tr(session.locale, "recovery_banner_msg");
    painter.text(
        Pos2::new(banner_rect.left() + 24.0, y_mid),
        Align2::LEFT_CENTER,
        msg,
        font_ui_bold(13.0),
        pal.fg_primary,
    );

    let discard_rect = Rect::from_center_size(
        Pos2::new(banner_rect.right() - 102.0, y_mid),
        Vec2::new(148.0, 28.0),
    );
    let confirm_rect = Rect::from_center_size(
        Pos2::new(banner_rect.right() - 264.0, y_mid),
        Vec2::new(152.0, 28.0),
    );

    let confirm_hit = ui.interact(
        confirm_rect,
        ui.id().with("recovery_confirm_save_btn"),
        Sense::click(),
    );
    let confirm_fill = if confirm_hit.hovered() {
        Color32::from_rgb(0x28, 0x7a, 0x4b)
    } else {
        Color32::from_rgb(0x2e, 0x8b, 0x57)
    };
    painter.rect_filled(confirm_rect, CornerRadius::same(6), confirm_fill);
    painter.text(
        confirm_rect.center(),
        Align2::CENTER_CENTER,
        I18n::tr(session.locale, "recovery_confirm_save"),
        font_ui_bold(12.5),
        Color32::WHITE,
    );
    if confirm_hit.clicked() {
        actions.push(Action::ConfirmRecoverySave);
    }

    let discard_hit = ui.interact(
        discard_rect,
        ui.id().with("recovery_discard_btn"),
        Sense::click(),
    );
    let discard_fill = if discard_hit.hovered() {
        pal.bg_pill_button
    } else {
        pal.bg_pill_button.gamma_multiply(0.75)
    };
    painter.rect_filled(discard_rect, CornerRadius::same(6), discard_fill);
    painter.rect_stroke(
        discard_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Inside,
    );
    painter.text(
        discard_rect.center(),
        Align2::CENTER_CENTER,
        I18n::tr(session.locale, "recovery_discard"),
        font_ui_bold(12.5),
        pal.fg_primary,
    );
    if discard_hit.clicked() {
        actions.push(Action::DiscardRecovery);
    }
}

fn render_workspace_area(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
    sync_cursor_to_widget: &mut bool,
) {
    let chrome_hidden = session.chrome_autohidden
        && session.overlay == ActiveOverlay::None
        && !session.search_bar_open
        && !session.recovery_banner_active;
    let mut top_y = if chrome_hidden { 8.0 } else { 44.0 };
    if session.recovery_banner_active {
        top_y += 44.0;
    }
    let bot_y = if chrome_hidden {
        win_rect.bottom() - 8.0
    } else {
        win_rect.bottom() - 56.0
    };
    let content_rect = Rect::from_min_max(
        Pos2::new(win_rect.left(), top_y),
        Pos2::new(win_rect.right(), bot_y),
    );

    if !session.preview_active || session.preview_layout == PreviewLayout::Windowed {
        render_editor_pane(
            ui,
            painter,
            content_rect,
            session,
            pal,
            actions,
            sync_cursor_to_widget,
        );
        return;
    }

    match session.preview_layout {
        PreviewLayout::HalfWidth => {
            let mid_x = win_rect.center().x.round();
            let left_rect = Rect::from_min_max(
                content_rect.left_top(),
                Pos2::new(mid_x, content_rect.bottom()),
            );
            let right_rect = Rect::from_min_max(
                Pos2::new(mid_x + 1.0, content_rect.top()),
                content_rect.right_bottom(),
            );
            render_editor_pane(
                ui,
                painter,
                left_rect,
                session,
                pal,
                actions,
                sync_cursor_to_widget,
            );
            painter.line_segment(
                [
                    Pos2::new(mid_x + 0.5, content_rect.top()),
                    Pos2::new(mid_x + 0.5, win_rect.bottom()),
                ],
                Stroke::new(1.0, pal.divider_preview),
            );
            render_preview_pane(ui, painter, right_rect, session, pal);
        }
        PreviewLayout::HalfHeight => {
            let mid_y = content_rect.center().y.round();
            let top_rect = Rect::from_min_max(
                content_rect.left_top(),
                Pos2::new(content_rect.right(), mid_y),
            );
            let bot_rect = Rect::from_min_max(
                Pos2::new(content_rect.left(), mid_y + 1.0),
                content_rect.right_bottom(),
            );
            render_editor_pane(
                ui,
                painter,
                top_rect,
                session,
                pal,
                actions,
                sync_cursor_to_widget,
            );
            painter.line_segment(
                [
                    Pos2::new(content_rect.left(), mid_y + 0.5),
                    Pos2::new(content_rect.right(), mid_y + 0.5),
                ],
                Stroke::new(1.0, pal.divider_preview),
            );
            render_preview_pane(ui, painter, bot_rect, session, pal);
        }
        PreviewLayout::FullWidth => {
            render_preview_pane(ui, painter, content_rect, session, pal);
            // Floating "Back to Editor" pill in FullWidth reader mode so user never gets trapped
            let back_rect = Rect::from_min_max(
                Pos2::new(content_rect.right() - 178.0, content_rect.top() + 14.0),
                Pos2::new(content_rect.right() - 24.0, content_rect.top() + 44.0),
            );
            painter.rect_filled(back_rect, CornerRadius::same(8), pal.accent_orange);
            painter.text(
                back_rect.center(),
                Align2::CENTER_CENTER,
                I18n::tr(session.locale, "back_to_editor"),
                font_ui_bold(12.5),
                Color32::WHITE,
            );
            let back_hit = ui.interact(
                back_rect,
                ui.id().with("fullwidth_back_to_editor_btn"),
                Sense::click(),
            );
            if back_hit.clicked() {
                actions.push(Action::SetEditorOnlyRaw);
            }
        }
        PreviewLayout::Windowed => {}
    }
}

fn render_editor_pane(
    ui: &mut Ui,
    _painter: &Painter,
    pane_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
    sync_cursor_to_widget: &mut bool,
) {
    let pane_w = pane_rect.width();
    let (x_body_rel, col_width) = if session.preview_active
        && session.preview_layout == PreviewLayout::HalfWidth
    {
        let left_pad = 86.0_f32;
        let cw = (pane_w - left_pad - 48.0).max(200.0);
        (left_pad, cw)
    } else {
        session.line_width.compute_column_bounds(pane_w)
    };

    let x_body = pane_rect.left() + x_body_rel;
    let x_right = x_body + col_width;

    let font_size = session.typography.font_size;
    let line_step = (font_size * session.typography.line_height).max(18.0);

    // Align `TextEdit` left edge directly at `x_body` (`base_indent = 0.0`) so soft-wrapped
    // continuation lines in paragraphs, list items, and blockquotes ALWAYS align at `x_body`
    // without jumping left into the gutter.
    let te_left = x_body;
    let base_indent = 0.0_f32;

    // Genuine interactive vertical `egui::ScrollArea` spanning the full editor pane!
    let mut pane_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(pane_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    let mut editable_text = session.buffer.as_str().to_owned();
    let buf_rev = session.buffer.revision();
    let dialect = session.dialect;
    let view_while_typing = session.view_while_typing;
    let focus_mode = session.focus_mode;
    let theme_variant = session.theme;
    let prose_family = session.typography.prose_font_family.clone();
    let code_family = session.typography.code_font_family.clone();
    let selection_bytes = session.buffer.selection_byte_range();
    let pre_selection_bytes = selection_bytes.clone();
    let eff_caret_key = if view_while_typing {
        Some(selection_bytes.clone())
    } else {
        None
    };
    let active_cursor_line = if focus_mode {
        session.buffer.cursor_line()
    } else {
        0
    };
    let pal_copy = *pal;

    #[derive(Clone, PartialEq, Eq)]
    struct EditorGalleyCacheKey {
        buf_rev: u64,
        dialect: accent_core::MarkdownDialect,
        view_while_typing: bool,
        eff_caret_key: Option<std::ops::Range<usize>>,
        focus_mode: bool,
        active_cursor_line: usize,
        theme_variant: ThemeMode,
        wrap_w_bits: u32,
        font_size_bits: u32,
        line_step_bits: u32,
        ppp_bits: u32,
        atlas_size: [usize; 2],
        prose_family: String,
        code_family: String,
    }

    let mut layouter = move |ui: &Ui, text_str: &str, wrap_width: f32| -> Arc<egui::Galley> {
        let eff_wrap_w = wrap_width.max(col_width + base_indent + 16.0);
        let is_session_buf = text_str == session.buffer.as_str();
        let cache_id = Id::new("accent_editor_galley_cache");
        let gen_id = Id::new("accent_editor_atlas_gen");
        let pre_atlas = ui.fonts(|f| f.font_image_size());
        let ppp_bits = ui.ctx().pixels_per_point().to_bits();
        let pass_nr = ui.ctx().cumulative_pass_nr();

        let key = EditorGalleyCacheKey {
            buf_rev: if is_session_buf { buf_rev } else { 0 },
            dialect,
            view_while_typing,
            eff_caret_key: eff_caret_key.clone(),
            focus_mode,
            active_cursor_line,
            theme_variant,
            wrap_w_bits: eff_wrap_w.to_bits(),
            font_size_bits: font_size.to_bits(),
            line_step_bits: line_step.to_bits(),
            ppp_bits,
            atlas_size: pre_atlas,
            prose_family: prose_family.clone(),
            code_family: code_family.clone(),
        };

        if is_session_buf && pass_nr >= 2 {
            if let Some((cached_key, cached_galley)) = ui.ctx().data_mut(|d| {
                d.get_temp::<(EditorGalleyCacheKey, Arc<egui::Galley>)>(cache_id)
            }) {
                if cached_key == key {
                    return cached_galley;
                }
            }
        }

        let temp_buf;
        let segments = if is_session_buf {
            session.buffer.build_editor_segments_arc(
                dialect,
                view_while_typing,
                selection_bytes.clone(),
            )
        } else {
            temp_buf = accent_core::DocumentBuffer::from_str_content(text_str);
            temp_buf.build_editor_segments_arc(dialect, view_while_typing, selection_bytes.clone())
        };
        let mut job = LayoutJob::default();
        job.wrap.max_width = eff_wrap_w;

        for seg in segments.iter() {
            let leading_space = if seg.is_line_start && seg.code_block {
                8.0
            } else {
                0.0
            };

            let seg_line_step = match seg.heading_level {
                Some(1) if view_while_typing => line_step * 1.30,
                Some(2) if view_while_typing => line_step * 1.16,
                _ => line_step,
            };

            if seg.collapsed_delimiter {
                // Zero-Cursor-Drift collapsed syntax delimiter:
                // Keeps 1:1 char/byte indices in LayoutJob while shrinking delimiter to 0.01pt transparent.
                let collapsed_lh = if seg.mermaid_collapsed {
                    44.0
                } else if seg.code_block {
                    6.0
                } else {
                    seg_line_step
                };
                job.append(
                    &seg.text,
                    leading_space,
                    TextFormat {
                        font_id: FontId::new(0.01, egui::FontFamily::Monospace),
                        color: Color32::TRANSPARENT,
                        line_height: Some(collapsed_lh),
                        ..Default::default()
                    },
                );
                continue;
            }

            let eff_font_size = match seg.heading_level {
                Some(1) if view_while_typing && !seg.is_heading_hashes => font_size * 1.42,
                Some(2) if view_while_typing && !seg.is_heading_hashes => font_size * 1.24,
                Some(3) if view_while_typing && !seg.is_heading_hashes => font_size * 1.12,
                Some(_) if view_while_typing && !seg.is_heading_hashes => font_size * 1.04,
                _ => font_size,
            };

            let font_id = if seg.code_inline || seg.code_block {
                resolve_code_font(&code_family, eff_font_size, seg.bold, seg.italic)
            } else {
                resolve_prose_font(&prose_family, eff_font_size, seg.bold, seg.italic)
            };

            let fg_base = if focus_mode && seg.line_index != active_cursor_line {
                pal_copy.fg_focus_dimmed
            } else if seg.heading_level.is_some() && view_while_typing {
                pal_copy.fg_heading
            } else {
                pal_copy.fg_primary
            };

            let color = if seg.muted {
                pal_copy.fg_muted
            } else if seg.link && view_while_typing {
                pal_copy.accent_link
            } else {
                fg_base
            };

            let background = if seg.code_inline && !seg.code_block && view_while_typing && !seg.muted
            {
                pal_copy.bg_code_block
            } else {
                Color32::TRANSPARENT
            };

            let strikethrough = if seg.strikethrough && !seg.collapsed_delimiter {
                Stroke::new(1.3, color)
            } else {
                Stroke::NONE
            };
            let underline = if seg.link && view_while_typing && !seg.collapsed_delimiter && !seg.muted
            {
                Stroke::new(1.0, pal_copy.accent_link)
            } else {
                Stroke::NONE
            };

            job.append(
                &seg.text,
                leading_space,
                TextFormat {
                    font_id,
                    color,
                    background,
                    strikethrough,
                    underline,
                    line_height: Some(seg_line_step),
                    ..Default::default()
                },
            );
        }

        let (mut last_seen_atlas, mut atlas_gen) = ui
            .ctx()
            .data_mut(|d| d.get_temp::<([usize; 2], u32)>(gen_id))
            .unwrap_or((pre_atlas, 0));
        if last_seen_atlas != pre_atlas {
            atlas_gen = atlas_gen.wrapping_add(1);
            last_seen_atlas = pre_atlas;
            ui.ctx()
                .data_mut(|d| d.insert_temp(gen_id, (last_seen_atlas, atlas_gen)));
        }

        let caret_jitter = if view_while_typing {
            ((selection_bytes
                .start
                .wrapping_mul(31)
                .wrapping_add(selection_bytes.end))
                % 17) as f32
                * 0.0002
        } else {
            0.0
        };
        job.wrap.max_width = eff_wrap_w + (atlas_gen as f32) * 0.0005 + caret_jitter;
        let mut galley = ui.fonts(|f| f.layout_job(job.clone()));
        let post_atlas = ui.fonts(|f| f.font_image_size());
        if post_atlas != pre_atlas {
            // The Font Atlas texture grew mid-way through `layout_job`, invalidating earlier rows' UVs!
            // Re-layout immediately with incremented `atlas_gen` against the new `post_atlas` dimensions.
            atlas_gen = atlas_gen.wrapping_add(1);
            ui.ctx()
                .data_mut(|d| d.insert_temp(gen_id, (post_atlas, atlas_gen)));
            job.wrap.max_width = eff_wrap_w + (atlas_gen as f32) * 0.0005 + caret_jitter;
            galley = ui.fonts(|f| f.layout_job(job));
            ui.ctx().request_repaint();
        }

        if is_session_buf && pass_nr >= 2 && post_atlas == pre_atlas {
            let mut final_key = key;
            final_key.atlas_size = post_atlas;
            ui.ctx()
                .data_mut(|d| d.insert_temp(cache_id, (final_key, Arc::clone(&galley))));
        } else if pass_nr < 2 {
            ui.ctx().request_repaint();
        }
        galley
    };

    ScrollArea::vertical()
        .id_salt("accent_editor_scroll_area")
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(&mut pane_ui, |ui| {
            ui.add_space(18.0);
            ui.horizontal_top(|ui| {
                ui.add_space((te_left - pane_rect.left()).max(0.0));
                let scroll_painter = ui.painter().clone();
                let code_bg_shape_idx = scroll_painter.add(egui::Shape::Noop);

                // Suppress egui's oversized full-line-height cursor bar so we can draw a crisp
                // character-proportioned caret directly on `scroll_painter`.
                let prev_cursor_stroke = ui.visuals().text_cursor.stroke;
                ui.visuals_mut().text_cursor.stroke = Stroke::NONE;

                let te_id = Id::new("accent_live_markdown_editor");
                let mut output = TextEdit::multiline(&mut editable_text)
                    .id(te_id)
                    .frame(false)
                    .margin(Margin::ZERO)
                    .desired_width(col_width + base_indent + 16.0)
                    .desired_rows(28)
                    .text_color(pal.fg_primary)
                    .layouter(&mut layouter)
                    .show(ui);

                ui.visuals_mut().text_cursor.stroke = prev_cursor_stroke;

                // Capture post-TextEdit cursor range immediately so mouse clicks on collapsed spans
                // convert CCursor char indices to UTF-8 byte offsets and trigger an immediate repaint (<16ms).
                let click_anchor_id = Id::new("accent_editor_click_anchor");
                let (primary_pressed, primary_down, pointer_pos, press_origin) =
                    ui.ctx().input(|i| {
                        (
                            i.pointer.primary_pressed(),
                            i.pointer.primary_down(),
                            i.pointer.interact_pos(),
                            i.pointer.press_origin(),
                        )
                    });

                let mut active_cursor_range = output
                    .cursor_range
                    .map(|cr| cr.as_ccursor_range())
                    .or_else(|| output.state.cursor.char_range());

                if primary_pressed {
                    if let Some(cr) = active_cursor_range {
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(click_anchor_id, (pointer_pos, cr.primary.index));
                        });
                    }
                } else if primary_down {
                    let is_stationary_click = match (press_origin, pointer_pos) {
                        (Some(orig), Some(cur)) => orig.distance(cur) < 3.5,
                        _ => false,
                    };
                    if is_stationary_click {
                        if let Some((_, anchored_char)) = ui
                            .ctx()
                            .data_mut(|d| d.get_temp::<(Option<Pos2>, usize)>(click_anchor_id))
                        {
                            let locked = egui::text::CCursorRange::one(egui::text::CCursor::new(
                                anchored_char,
                            ));
                            output.state.cursor.set_char_range(Some(locked));
                            output.state.clone().store(ui.ctx(), output.response.id);
                            active_cursor_range = Some(locked);
                        }
                    }
                } else {
                    ui.ctx()
                        .data_mut(|d| d.remove::<(Option<Pos2>, usize)>(click_anchor_id));
                }

                let post_selection_bytes = if let Some(cr) = active_cursor_range {
                    let p = cr.primary.index;
                    let s = cr.secondary.index;
                    session.buffer.char_to_byte(p.min(s))..session.buffer.char_to_byte(p.max(s))
                } else {
                    pre_selection_bytes.clone()
                };

                // Populate block/inline visual overlays on `scroll_painter` with O(1) viewport culling
                let live_spans = session.buffer.compute_live_spans_arc(
                    session.dialect,
                    session.view_while_typing,
                    post_selection_bytes.clone(),
                );
                let vis_min_y = (pane_rect.top() - output.galley_pos.y - 400.0).max(0.0);
                let vis_max_y = (pane_rect.bottom() - output.galley_pos.y + 400.0).max(0.0);
                let min_vis_char = output
                    .galley
                    .cursor_from_pos(Vec2::new(0.0, vis_min_y))
                    .ccursor
                    .index;
                let max_vis_char = output
                    .galley
                    .cursor_from_pos(Vec2::new(col_width + 64.0, vis_max_y))
                    .ccursor
                    .index;

                let row_top = |r: Rect| -> f32 { r.min.y.min(r.max.y - font_size * 1.18) };
                let row_center_y = |r: Rect| -> f32 { (row_top(r) + r.max.y) * 0.5 };

                let mut bg_shapes = Vec::new();
                for ls in live_spans.iter() {
                    if ls.span.char_range.end < min_vis_char
                        || ls.span.char_range.start > max_vis_char
                    {
                        continue;
                    }
                    let is_collapsed =
                        ls.visibility == accent_core::DelimiterVisibility::Collapsed;
                    match &ls.span.kind {
                        accent_core::AstSpanKind::Heading { level: 1 } if is_collapsed => {
                            let r_bot = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.end.saturating_sub(1),
                            ));
                            let rule_y = output.galley_pos.y + r_bot.max.y + 1.5;
                            scroll_painter.line_segment(
                                [Pos2::new(x_body, rule_y), Pos2::new(x_right, rule_y)],
                                Stroke::new(1.0, pal.border_headerbar),
                            );
                        }
                        accent_core::AstSpanKind::CodeBlock { info, .. } => {
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let r_bot = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.end.saturating_sub(1),
                            ));
                            let sy = output.galley_pos.y + r_top.min.y - 2.0;
                            let ey = if is_collapsed {
                                (output.galley_pos.y + r_bot.min.y + 2.0).max(sy + 26.0)
                            } else {
                                output.galley_pos.y + r_bot.max.y + 2.0
                            };
                            if ey > sy {
                                let is_mermaid = info
                                    .trim()
                                    .to_ascii_lowercase()
                                    .starts_with("mermaid");
                                bg_shapes.push(egui::Shape::rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(x_body, sy),
                                        Pos2::new(x_right, ey),
                                    ),
                                    CornerRadius::same(4),
                                    pal.bg_code_block,
                                ));
                                if is_collapsed && is_mermaid {
                                    let full_slice =
                                        &session.buffer.as_str()[ls.span.byte_range.clone()];
                                    let mut inner_lines: Vec<&str> = full_slice.lines().collect();
                                    if !inner_lines.is_empty()
                                        && (inner_lines[0].trim_start().starts_with("```")
                                            || inner_lines[0].trim_start().starts_with("~~~"))
                                    {
                                        inner_lines.remove(0);
                                    }
                                    if !inner_lines.is_empty()
                                        && (inner_lines.last().unwrap().trim_start().starts_with("```")
                                            || inner_lines
                                                .last()
                                                .unwrap()
                                                .trim_start()
                                                .starts_with("~~~"))
                                    {
                                        inner_lines.pop();
                                    }
                                    let mermaid_code = inner_lines.join("\n");
                                    let (tex, mw, mh) = get_or_load_mermaid_texture(
                                        ui.ctx(),
                                        session,
                                        &mermaid_code,
                                    );
                                    let avail_box = Rect::from_min_max(
                                        Pos2::new(x_body + 8.0, sy + 6.0),
                                        Pos2::new(x_right - 8.0, ey - 6.0),
                                    );
                                    let aspect = (mw.max(1) as f32) / (mh.max(1) as f32);
                                    let mut draw_w = avail_box.width().min((mw as f32) * 0.55);
                                    let mut draw_h = draw_w / aspect;
                                    if draw_h > avail_box.height().max(40.0) {
                                        draw_h = avail_box.height().max(40.0);
                                        draw_w = draw_h * aspect;
                                    }
                                    let img_rect = Rect::from_center_size(
                                        avail_box.center(),
                                        Vec2::new(draw_w, draw_h),
                                    );
                                    scroll_painter.image(
                                        tex.id(),
                                        img_rect,
                                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                }
                                if is_collapsed && !info.trim().is_empty() {
                                    scroll_painter.text(
                                        Pos2::new(x_right - 10.0, sy + 14.0),
                                        Align2::RIGHT_CENTER,
                                        info.trim(),
                                        font_ui_bold(11.0),
                                        pal.fg_muted,
                                    );
                                }
                            }
                        }
                        accent_core::AstSpanKind::BlockQuote if is_collapsed => {
                            let probe_top = (ls.span.char_range.start + 3)
                                .min(ls.span.char_range.end);
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                probe_top,
                            ));
                            let r_bot = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.end.saturating_sub(1),
                            ));
                            let sy = output.galley_pos.y + r_top.min.y - 3.5;
                            let ey = output.galley_pos.y + r_bot.max.y - 6.0;
                            if ey > sy {
                                scroll_painter.rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(x_body - 12.0, sy),
                                        Pos2::new(x_body - 8.5, ey),
                                    ),
                                    CornerRadius::same(2),
                                    pal.border_headerbar,
                                );
                            }
                        }
                        accent_core::AstSpanKind::Alert { alert_type } if is_collapsed => {
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let r_bot = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.end.saturating_sub(1),
                            ));
                            let sy = output.galley_pos.y + row_top(r_top) - 3.5;
                            let ey = output.galley_pos.y + r_bot.max.y - 6.0;
                            let (bar_color, label) = match alert_type.to_ascii_lowercase().as_str() {
                                "tip" => (Color32::from_rgb(0x2E, 0xA0, 0x43), "💡  TIP"),
                                "important" => {
                                    (Color32::from_rgb(0x89, 0x57, 0xE5), "❗  IMPORTANT")
                                }
                                "warning" => (pal.accent_orange, "⚠  WARNING"),
                                "caution" => {
                                    (Color32::from_rgb(0xDA, 0x36, 0x33), "🛑  CAUTION")
                                }
                                _ => (pal.accent_link, "ℹ  NOTE"),
                            };
                            if ey > sy {
                                scroll_painter.rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(x_body - 12.0, sy),
                                        Pos2::new(x_body - 8.0, ey),
                                    ),
                                    CornerRadius::same(2),
                                    bar_color,
                                );
                                scroll_painter.text(
                                    Pos2::new(x_body, sy + 9.5),
                                    Align2::LEFT_CENTER,
                                    label,
                                    font_ui_bold(12.5),
                                    bar_color,
                                );
                            }
                        }
                        accent_core::AstSpanKind::ListItem {
                            ordered,
                            task,
                            content_char_start,
                        } if is_collapsed => {
                            if !*ordered || task.is_some() {
                                let probe_char = (*content_char_start + 1)
                                    .min(ls.span.char_range.end)
                                    .max(ls.span.char_range.start);
                                let r_content = output
                                    .galley
                                    .pos_from_ccursor(egui::text::CCursor::new(probe_char));
                                let cy = output.galley_pos.y + r_content.min.y + font_size * 0.22;
                                if let Some(checked) = task {
                                    let cb_rect = Rect::from_center_size(
                                        Pos2::new(x_body - 13.0, cy),
                                        Vec2::new(13.5, 13.5),
                                    );
                                    if *checked {
                                        scroll_painter.rect_filled(
                                            cb_rect,
                                            CornerRadius::same(3),
                                            pal.accent_orange,
                                        );
                                        scroll_painter.line_segment(
                                            [
                                                Pos2::new(cb_rect.left() + 3.0, cb_rect.center().y),
                                                Pos2::new(
                                                    cb_rect.center().x - 0.5,
                                                    cb_rect.bottom() - 3.5,
                                                ),
                                            ],
                                            Stroke::new(1.6, Color32::WHITE),
                                        );
                                        scroll_painter.line_segment(
                                            [
                                                Pos2::new(
                                                    cb_rect.center().x - 0.5,
                                                    cb_rect.bottom() - 3.5,
                                                ),
                                                Pos2::new(
                                                    cb_rect.right() - 2.8,
                                                    cb_rect.top() + 3.2,
                                                ),
                                            ],
                                            Stroke::new(1.6, Color32::WHITE),
                                        );
                                    } else {
                                        scroll_painter.rect_stroke(
                                            cb_rect,
                                            CornerRadius::same(3),
                                            Stroke::new(1.3, pal.fg_muted),
                                            StrokeKind::Middle,
                                        );
                                    }
                                } else {
                                    scroll_painter.circle_filled(
                                        Pos2::new(x_body - 12.0, cy),
                                        3.2,
                                        pal.fg_primary,
                                    );
                                }
                            }
                        }
                        accent_core::AstSpanKind::ThematicBreak if is_collapsed => {
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let cy = output.galley_pos.y + row_center_y(r_top);
                            scroll_painter.line_segment(
                                [Pos2::new(x_body, cy), Pos2::new(x_right, cy)],
                                Stroke::new(1.5, pal.border_headerbar),
                            );
                        }
                        accent_core::AstSpanKind::Table { headers, rows } if is_collapsed => {
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let r_bot = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.end.saturating_sub(1),
                            ));
                            let sy = output.galley_pos.y + row_top(r_top) + 2.0;
                            let ey = (output.galley_pos.y + r_bot.max.y - 2.0).max(sy + 32.0);
                            let total_rows = (1 + rows.len()).max(1);
                            let row_h = (ey - sy) / (total_rows as f32);
                            let cols = headers.len().max(1);
                            let table_w = (x_right - x_body).max(120.0);

                            // Compute proportional column widths from max character length per column
                            let mut col_weights: Vec<f32> = headers
                                .iter()
                                .map(|h| (h.chars().count().max(6) + 4) as f32)
                                .collect();
                            for row in rows {
                                for (c_idx, cell) in row.iter().enumerate() {
                                    if c_idx < col_weights.len() {
                                        let w = (cell.chars().count().max(6) + 4) as f32;
                                        if w > col_weights[c_idx] {
                                            col_weights[c_idx] = w;
                                        }
                                    }
                                }
                            }
                            let weight_sum: f32 = col_weights.iter().sum::<f32>().max(1.0);
                            let col_widths: Vec<f32> = if col_weights.is_empty() {
                                vec![table_w / (cols as f32); cols]
                            } else {
                                col_weights
                                    .iter()
                                    .map(|w| table_w * (*w / weight_sum))
                                    .collect()
                            };

                            let table_rect =
                                Rect::from_min_max(Pos2::new(x_body, sy), Pos2::new(x_right, ey));
                            scroll_painter.rect_filled(
                                table_rect,
                                CornerRadius::same(4),
                                pal.bg_canvas,
                            );

                            for r_idx in 0..total_rows {
                                let ry = sy + (r_idx as f32) * row_h;
                                let row_rect = Rect::from_min_max(
                                    Pos2::new(x_body, ry),
                                    Pos2::new(x_right, ry + row_h),
                                );
                                if r_idx == 0 {
                                    scroll_painter.rect_filled(
                                        row_rect,
                                        CornerRadius::ZERO,
                                        pal.bg_code_block,
                                    );
                                } else if r_idx % 2 == 1 {
                                    scroll_painter.rect_filled(
                                        row_rect,
                                        CornerRadius::ZERO,
                                        pal.bg_code_block.gamma_multiply(0.45),
                                    );
                                }
                                let cells = if r_idx == 0 {
                                    headers.as_slice()
                                } else {
                                    rows[r_idx - 1].as_slice()
                                };
                                let mut cx = x_body;
                                for (c_idx, cell_text) in cells.iter().enumerate() {
                                    let cw = col_widths
                                        .get(c_idx)
                                        .copied()
                                        .unwrap_or(table_w / (cols as f32));
                                    let cell_r = Rect::from_min_max(
                                        Pos2::new(cx, ry),
                                        Pos2::new(cx + cw, ry + row_h),
                                    );
                                    scroll_painter.rect_stroke(
                                        cell_r,
                                        CornerRadius::ZERO,
                                        Stroke::new(1.0, pal.border_headerbar),
                                        StrokeKind::Middle,
                                    );
                                    scroll_painter.with_clip_rect(cell_r.shrink(2.0)).text(
                                        Pos2::new(cell_r.left() + 10.0, cell_r.center().y),
                                        Align2::LEFT_CENTER,
                                        cell_text,
                                        if r_idx == 0 {
                                            font_ui_bold(13.5)
                                        } else {
                                            font_ui_regular(13.5)
                                        },
                                        pal.fg_primary,
                                    );
                                    cx += cw;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                scroll_painter.set(code_bg_shape_idx, egui::Shape::Vec(bg_shapes));

                if output.response.clicked()
                    && session.overlay != ActiveOverlay::None
                    && session.overlay != ActiveOverlay::PreferencesDialog
                {
                    actions.push(Action::SetOverlay(ActiveOverlay::None));
                }

                if output.response.changed() && editable_text != session.buffer.as_str() {
                    let is_deletion = editable_text.len() < session.buffer.len_bytes();
                    if !(session.hemingway_mode && is_deletion) {
                        actions.push(Action::UpdateBufferText(editable_text));
                    }
                }

                // Sync programmatic cursor/selection updates into TextEdit and auto-scroll to keep cursor in view
                if *sync_cursor_to_widget {
                    *sync_cursor_to_widget = false;
                    let sel = session.buffer.selection_char_range();
                    let c_range = egui::text::CCursorRange::two(
                        egui::text::CCursor::new(sel.start),
                        egui::text::CCursor::new(sel.end),
                    );
                    output.state.cursor.set_char_range(Some(c_range));
                    output.state.clone().store(ui.ctx(), output.response.id);
                    let cur_local = output
                        .galley
                        .pos_from_ccursor(egui::text::CCursor::new(session.buffer.cursor()));
                    let cur_screen = cur_local
                        .translate(output.galley_pos.to_vec2())
                        .expand2(Vec2::new(4.0, 16.0));
                    ui.scroll_to_rect(cur_screen, None);
                    if session.overlay == ActiveOverlay::None && !session.search_bar_open {
                        output.response.request_focus();
                    }
                } else if let Some(c_range) = active_cursor_range {
                    let p = c_range.primary.index;
                    let s = c_range.secondary.index;
                    let new_sel_chars = p.min(s)..p.max(s);
                    let cur_sel = session.buffer.selection_char_range();
                    let cursor_moved = p != session.buffer.cursor()
                        || new_sel_chars != cur_sel
                        || post_selection_bytes != pre_selection_bytes;
                    if cursor_moved || output.response.changed() {
                        actions.push(Action::SetSelection(p, s));
                        ui.ctx().data_mut(|d| {
                            d.remove::<(EditorGalleyCacheKey, Arc<egui::Galley>)>(Id::new(
                                "accent_editor_galley_cache",
                            ));
                        });
                        ui.ctx().request_repaint();
                    }
                    if cursor_moved || output.response.changed() {
                        let cur_local = output
                            .galley
                            .pos_from_ccursor(egui::text::CCursor::new(p));
                        let cur_screen = cur_local
                            .translate(output.galley_pos.to_vec2())
                            .expand2(Vec2::new(4.0, 16.0));
                        ui.scroll_to_rect(cur_screen, None);
                    }
                }

                // Draw a crisp, character-proportioned caret centered vertically on the active text glyph
                if !(session.preview_active && session.preview_layout == PreviewLayout::FullWidth) {
                    let caret_char_idx = output
                        .state
                        .cursor
                        .char_range()
                        .map(|r| r.primary.index)
                        .unwrap_or_else(|| session.buffer.cursor());
                    let cur_rect = output
                        .galley
                        .pos_from_ccursor(egui::text::CCursor::new(caret_char_idx));
                    let cur_line = session.buffer.cursor_line();
                    let caret_font_size = if session.view_while_typing {
                        let h_lvl = live_spans.iter().find_map(|ls| match ls.span.kind {
                            accent_core::AstSpanKind::Heading { level }
                                if ls.span.start_line <= cur_line
                                    && cur_line <= ls.span.end_line =>
                            {
                                Some(level)
                            }
                            _ => None,
                        });
                        match h_lvl {
                            Some(1) => font_size * 1.42,
                            Some(2) => font_size * 1.24,
                            Some(3) => font_size * 1.12,
                            Some(_) => font_size * 1.04,
                            None => font_size,
                        }
                    } else {
                        font_size
                    };
                    let cx = (output.galley_pos.x + cur_rect.min.x).max(x_body);
                    let text_center_y =
                        output.galley_pos.y + row_top(cur_rect) + caret_font_size * 0.22;
                    let caret_h = (caret_font_size * 1.02).max(14.0);
                    let cy_top = text_center_y - caret_h * 0.52;
                    let cy_bot = text_center_y + caret_h * 0.48;
                    if cy_bot >= pane_rect.top() && cy_top <= pane_rect.bottom() {
                        scroll_painter.line_segment(
                            [Pos2::new(cx, cy_top), Pos2::new(cx, cy_bot)],
                            Stroke::new(1.5, pal.fg_primary),
                        );
                    }
                }
            });
            ui.add_space(72.0);
        });
}

fn render_preview_pane(
    ui: &mut Ui,
    painter: &Painter,
    pane_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
) {
    painter.rect_filled(pane_rect, CornerRadius::ZERO, pal.bg_preview);

    let blocks = session.buffer.parse_preview_blocks_arc(session.dialect);
    let inner_rect = Rect::from_min_max(
        Pos2::new(pane_rect.left() + 18.0, pane_rect.top() + 16.0),
        Pos2::new(pane_rect.right() - 18.0, pane_rect.bottom() - 8.0),
    );

    let mut preview_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    ScrollArea::vertical()
        .id_salt("accent_rendered_preview_scroll")
        .auto_shrink([false, false])
        .show(&mut preview_ui, |ui| {
            let avail_w = ui.available_width();
            for block in blocks.iter() {
                match block {
                    PreviewBlock::Heading {
                        level,
                        inlines,
                        is_link_heading,
                    } => {
                        ui.add_space(if *level == 1 { 14.0 } else { 8.0 });
                        let h_size = match level {
                            1 => 25.0,
                            2 => 20.0,
                            3 => 16.5,
                            _ => 14.5,
                        };
                        let h_color = if *is_link_heading {
                            pal.accent_link
                        } else {
                            pal.fg_heading
                        };
                        let job = build_preview_inline_job(
                            inlines,
                            h_size,
                            true,
                            h_color,
                            pal,
                            avail_w,
                        );
                        ui.label(job);
                        if *level == 1 {
                            ui.add_space(4.0);
                            let (rule_rect, _) =
                                ui.allocate_exact_size(Vec2::new(avail_w, 2.0), Sense::hover());
                            ui.painter().line_segment(
                                [rule_rect.left_center(), rule_rect.right_center()],
                                Stroke::new(1.0, pal.border_headerbar),
                            );
                        }
                        ui.add_space(6.0);
                    }
                    PreviewBlock::Paragraph { inlines } => {
                        let job = build_preview_inline_job(
                            inlines,
                            14.5,
                            false,
                            pal.fg_primary,
                            pal,
                            avail_w,
                        );
                        ui.label(job);
                        ui.add_space(10.0);
                    }
                    PreviewBlock::List { ordered, items } => {
                        for (idx, item) in items.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if let Some(checked) = item.task {
                                    let (cb_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(14.0, 14.0),
                                        Sense::hover(),
                                    );
                                    if checked {
                                        ui.painter().rect_filled(
                                            cb_rect,
                                            CornerRadius::same(3),
                                            pal.accent_orange,
                                        );
                                        ui.painter().line_segment(
                                            [
                                                Pos2::new(cb_rect.left() + 3.0, cb_rect.center().y),
                                                Pos2::new(
                                                    cb_rect.center().x - 0.5,
                                                    cb_rect.bottom() - 3.5,
                                                ),
                                            ],
                                            Stroke::new(1.6, Color32::WHITE),
                                        );
                                        ui.painter().line_segment(
                                            [
                                                Pos2::new(
                                                    cb_rect.center().x - 0.5,
                                                    cb_rect.bottom() - 3.5,
                                                ),
                                                Pos2::new(
                                                    cb_rect.right() - 2.8,
                                                    cb_rect.top() + 3.2,
                                                ),
                                            ],
                                            Stroke::new(1.6, Color32::WHITE),
                                        );
                                    } else {
                                        ui.painter().rect_stroke(
                                            cb_rect,
                                            CornerRadius::same(3),
                                            Stroke::new(1.2, pal.fg_muted),
                                            StrokeKind::Middle,
                                        );
                                    }
                                } else if *ordered {
                                    ui.label(
                                        egui::RichText::new(format!("{}.", idx + 1))
                                            .font(font_ui_bold(14.0))
                                            .color(pal.fg_primary),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("•")
                                            .font(font_ui_bold(14.0))
                                            .color(pal.fg_primary),
                                    );
                                }
                                let job = build_preview_inline_job(
                                    &item.inlines,
                                    14.5,
                                    false,
                                    pal.fg_primary,
                                    pal,
                                    avail_w - 28.0,
                                );
                                ui.label(job);
                            });
                            ui.add_space(4.0);
                        }
                        ui.add_space(8.0);
                    }
                    PreviewBlock::Image { alt, .. } => {
                        ui.add_space(6.0);
                        let (img_rect, _) =
                            ui.allocate_exact_size(Vec2::new(140.0, 44.0), Sense::hover());
                        let box_r = Rect::from_min_max(
                            Pos2::new(img_rect.left() + 32.0, img_rect.top()),
                            Pos2::new(img_rect.left() + 114.0, img_rect.top() + 20.0),
                        );
                        ui.painter().rect_stroke(
                            box_r,
                            CornerRadius::same(2),
                            Stroke::new(1.2, pal.fg_muted),
                            StrokeKind::Middle,
                        );
                        let badge = Rect::from_center_size(box_r.center(), Vec2::new(13.0, 13.0));
                        ui.painter()
                            .rect_filled(badge, CornerRadius::same(2), pal.accent_link);
                        ui.painter().text(
                            badge.center(),
                            Align2::CENTER_CENTER,
                            "?",
                            font_ui_bold(10.0),
                            Color32::WHITE,
                        );
                        ui.painter().text(
                            Pos2::new(box_r.center().x, box_r.bottom() + 12.0),
                            Align2::CENTER_CENTER,
                            alt,
                            font_ui_regular(14.0),
                            pal.fg_primary,
                        );
                        ui.add_space(8.0);
                    }
                    PreviewBlock::BlockQuote { inlines } => {
                        ui.horizontal(|ui| {
                            let (bar_r, _) =
                                ui.allocate_exact_size(Vec2::new(4.0, 34.0), Sense::hover());
                            ui.painter().rect_filled(
                                bar_r,
                                CornerRadius::same(1),
                                pal.border_headerbar,
                            );
                            ui.add_space(6.0);
                            let job = build_preview_inline_job(
                                inlines,
                                14.0,
                                false,
                                pal.fg_primary,
                                pal,
                                avail_w - 32.0,
                            );
                            ui.add(egui::Label::new(job).wrap());
                        });
                        ui.add_space(10.0);
                    }
                    PreviewBlock::CodeBlock { info, code } => {
                        if info.trim().to_ascii_lowercase().starts_with("mermaid") {
                            let (tex, mw, mh) = get_or_load_mermaid_texture(ui.ctx(), session, code);
                            let aspect = (mw.max(1) as f32) / (mh.max(1) as f32);
                            let max_w = avail_w.min((mw as f32) * 0.55).max(180.0);
                            let draw_h = (max_w / aspect).clamp(90.0, 340.0);
                            let draw_w = (draw_h * aspect).min(avail_w);
                            let box_h = draw_h + 18.0;
                            let (cb_rect, _) =
                                ui.allocate_exact_size(Vec2::new(avail_w, box_h), Sense::hover());
                            ui.painter()
                                .rect_filled(cb_rect, CornerRadius::same(6), pal.bg_code_block);
                            let img_rect =
                                Rect::from_center_size(cb_rect.center(), Vec2::new(draw_w, draw_h));
                            ui.painter().image(
                                tex.id(),
                                img_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            ui.add_space(10.0);
                        } else {
                            let lines_count = code.lines().count().max(1);
                            let box_h = (lines_count as f32) * 20.0 + 22.0;
                            let (cb_rect, _) =
                                ui.allocate_exact_size(Vec2::new(avail_w, box_h), Sense::hover());
                            ui.painter()
                                .rect_filled(cb_rect, CornerRadius::same(4), pal.bg_code_block);
                            ui.painter().text(
                                Pos2::new(cb_rect.left() + 12.0, cb_rect.top() + 10.0),
                                Align2::LEFT_TOP,
                                code,
                                font_editor_regular(13.0),
                                pal.fg_primary,
                            );
                            ui.add_space(10.0);
                        }
                    }
                    PreviewBlock::Table { headers, rows } => {
                        ui.add_space(4.0);
                        let cols = headers.len().max(1);
                        let row_h = 30.0_f32;
                        let total_rows = 1 + rows.len();
                        let table_w = avail_w.max(200.0);
                        let table_h = (total_rows as f32) * row_h;

                        let mut col_weights: Vec<f32> = headers
                            .iter()
                            .map(|h| (h.chars().count().max(6) + 4) as f32)
                            .collect();
                        for row in rows {
                            for (c_idx, cell) in row.iter().enumerate() {
                                if c_idx < col_weights.len() {
                                    let w = (cell.chars().count().max(6) + 4) as f32;
                                    if w > col_weights[c_idx] {
                                        col_weights[c_idx] = w;
                                    }
                                }
                            }
                        }
                        let weight_sum: f32 = col_weights.iter().sum::<f32>().max(1.0);
                        let col_widths: Vec<f32> = if col_weights.is_empty() {
                            vec![table_w / (cols as f32); cols]
                        } else {
                            col_weights
                                .iter()
                                .map(|w| table_w * (*w / weight_sum))
                                .collect()
                        };

                        let (t_rect, _) =
                            ui.allocate_exact_size(Vec2::new(table_w, table_h), Sense::hover());
                        let tp = ui.painter();

                        // Header row + body rows
                        for r_idx in 0..total_rows {
                            let ry = t_rect.top() + (r_idx as f32) * row_h;
                            let r_rect = Rect::from_min_max(
                                Pos2::new(t_rect.left(), ry),
                                Pos2::new(t_rect.right(), ry + row_h),
                            );
                            if r_idx == 0 {
                                tp.rect_filled(r_rect, CornerRadius::ZERO, pal.bg_code_block);
                            } else if r_idx % 2 == 1 {
                                tp.rect_filled(
                                    r_rect,
                                    CornerRadius::ZERO,
                                    pal.bg_code_block.gamma_multiply(0.45),
                                );
                            }
                            let cells = if r_idx == 0 {
                                headers.as_slice()
                            } else {
                                rows[r_idx - 1].as_slice()
                            };
                            let mut cx = t_rect.left();
                            for (c_idx, cell_text) in cells.iter().enumerate() {
                                let cw = col_widths
                                    .get(c_idx)
                                    .copied()
                                    .unwrap_or(table_w / (cols as f32));
                                let cell_r = Rect::from_min_max(
                                    Pos2::new(cx, ry),
                                    Pos2::new(cx + cw, ry + row_h),
                                );
                                tp.rect_stroke(
                                    cell_r,
                                    CornerRadius::ZERO,
                                    Stroke::new(1.0, pal.border_headerbar),
                                    StrokeKind::Middle,
                                );
                                tp.with_clip_rect(cell_r.shrink(2.0)).text(
                                    Pos2::new(cell_r.left() + 8.0, cell_r.center().y),
                                    Align2::LEFT_CENTER,
                                    cell_text,
                                    if r_idx == 0 {
                                        font_ui_bold(13.5)
                                    } else {
                                        font_ui_regular(13.5)
                                    },
                                    pal.fg_primary,
                                );
                                cx += cw;
                            }
                        }
                        ui.add_space(12.0);
                    }
                }
            }
        });
}

fn build_preview_inline_job(
    inlines: &[PreviewInline],
    font_size: f32,
    base_bold: bool,
    base_color: Color32,
    pal: &AccentPalette,
    wrap_width: f32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    for item in inlines {
        let font_id = if item.code {
            font_editor_regular(font_size - 1.0)
        } else if base_bold || item.bold {
            font_ui_bold(font_size)
        } else {
            font_ui_regular(font_size)
        };
        let color = if item.link_url.is_some() {
            pal.accent_link
        } else {
            base_color
        };
        let strikethrough = if item.strikethrough {
            Stroke::new(1.3, color)
        } else {
            Stroke::NONE
        };
        job.append(
            &item.text,
            0.0,
            TextFormat {
                font_id,
                color,
                strikethrough,
                ..Default::default()
            },
        );
    }
    job
}

fn render_bottom_bar(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    // Floating rounded pill bottom bar detached from the bottom of the window
    let bar_rect = Rect::from_min_max(
        Pos2::new(win_rect.left() + 18.0, win_rect.bottom() - 50.0),
        Pos2::new(win_rect.right() - 18.0, win_rect.bottom() - 8.0),
    );
    let shadow_rect = bar_rect.translate(Vec2::new(0.0, 2.5)).expand(1.5);
    painter.rect_filled(
        shadow_rect,
        CornerRadius::same(22),
        Color32::from_rgba_unmultiplied(0, 0, 0, 26),
    );
    painter.rect_filled(bar_rect, CornerRadius::same(21), pal.bg_popover);
    painter.rect_stroke(
        bar_rect,
        CornerRadius::same(21),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Inside,
    );

    let y_mid = bar_rect.center().y;
    let fg = pal.fg_primary;
    let x0 = bar_rect.left();

    if session.toolbar_expanded {
        let icon_xs = [
            (x0 + 26.0, FormatCommand::Bold, "tooltip_bold"),
            (x0 + 62.0, FormatCommand::Italic, "tooltip_italic"),
            (x0 + 98.0, FormatCommand::Strikethrough, "tooltip_strike"),
            (x0 + 134.0, FormatCommand::Heading, "tooltip_heading"),
            (x0 + 170.0, FormatCommand::BulletList, "tooltip_bullet"),
            (x0 + 206.0, FormatCommand::OrderedList, "tooltip_ordered"),
            (x0 + 242.0, FormatCommand::TaskList, "tooltip_task"),
            (x0 + 278.0, FormatCommand::BlockQuote, "tooltip_quote"),
            (x0 + 314.0, FormatCommand::Code, "tooltip_code"),
            (x0 + 350.0, FormatCommand::Link, "tooltip_link"),
            (x0 + 386.0, FormatCommand::Image, "tooltip_image"),
            (x0 + 422.0, FormatCommand::Table, "tooltip_table"),
        ];
        for (idx, &(ix, cmd, tip_key)) in icon_xs.iter().enumerate() {
            let btn_r = Rect::from_center_size(Pos2::new(ix, y_mid), Vec2::new(32.0, 32.0));
            let hit = ui
                .interact(btn_r, ui.id().with(("fmt_toolbar_btn", idx)), Sense::click())
                .on_hover_text(I18n::tr(session.locale, tip_key));
            if hit.hovered() {
                painter.rect_filled(btn_r, CornerRadius::same(8), pal.bg_pill_button);
            }
            draw_toolbar_icon(painter, Pos2::new(ix, y_mid), fg, idx);
            if hit.clicked() {
                actions.push(Action::ApplyFormat(cmd));
            }
        }
    }

    // Collapse '<' / Expand '>' pill button
    let pill_x = if session.toolbar_expanded {
        x0 + 462.0
    } else {
        x0 + 26.0
    };
    let pill_rect = Rect::from_center_size(Pos2::new(pill_x, y_mid), Vec2::new(30.0, 30.0));
    painter.rect_filled(pill_rect, CornerRadius::same(15), pal.bg_pill_button);
    if session.toolbar_expanded {
        draw_chevron_left(painter, pill_rect.center(), fg);
    } else {
        painter.text(
            pill_rect.center(),
            Align2::CENTER_CENTER,
            "›",
            font_ui_bold(18.0),
            fg,
        );
    }

    let pill_hit = ui
        .interact(pill_rect, ui.id().with("toolbar_toggle"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_toolbar_toggle"));
    if pill_hit.clicked() {
        actions.push(Action::ToggleToolbar);
    }

    // Center-Right: Discreet status indicator ("Modificato • Salvataggio automatico attivo")
    let status_text = if let Some(msg) = &session.status_message {
        msg.clone()
    } else if session.is_dirty {
        match session.locale {
            Locale::It => "Modificato • Salvataggio automatico attivo".to_string(),
            Locale::Es => "Modificado • Guardado automático activo".to_string(),
            Locale::En => "Modified • Autosave active".to_string(),
        }
    } else {
        match session.locale {
            Locale::It => "Salvataggio automatico attivo".to_string(),
            Locale::Es => "Guardado automático activo".to_string(),
            Locale::En => "Autosave active".to_string(),
        }
    };

    let status_right_x = bar_rect.right() - 156.0;
    if status_right_x > pill_x + 180.0 {
        painter.text(
            Pos2::new(status_right_x, y_mid),
            Align2::RIGHT_CENTER,
            &status_text,
            font_ui_regular(12.5),
            pal.fg_muted,
        );
    }

    // Right: Word counter pill ("415 parole ▴")
    let stat_text = session.format_active_stat();
    let stat_btn_rect = Rect::from_min_max(
        Pos2::new(bar_rect.right() - 144.0, y_mid - 15.0),
        Pos2::new(bar_rect.right() - 8.0, y_mid + 15.0),
    );
    let stat_hit = ui
        .interact(stat_btn_rect, ui.id().with("stats_menu_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_stats"));
    if session.overlay == ActiveOverlay::StatsMenu || stat_hit.hovered() {
        painter.rect_filled(stat_btn_rect, CornerRadius::same(15), pal.bg_pill_button);
    }
    painter.text(
        Pos2::new(stat_btn_rect.right() - 26.0, y_mid),
        Align2::RIGHT_CENTER,
        &stat_text,
        font_ui_bold(13.5),
        fg,
    );
    draw_chevron_up(painter, Pos2::new(stat_btn_rect.right() - 14.0, y_mid), fg);

    if stat_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::StatsMenu));
    }
}

fn render_popovers(
    _ctx: &Context,
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    match session.overlay {
        ActiveOverlay::None => {}
        ActiveOverlay::OpenMenu => {
            let n_recent = session.recent_files.len().max(1);
            let h = 136.0 + (n_recent as f32) * 38.0;
            let pop_rect = Rect::from_min_max(Pos2::new(48.0, 44.0), Pos2::new(346.0, 44.0 + h));
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );

            // Top action: "+ Nuovo documento"
            let new_row = Rect::from_min_max(
                Pos2::new(pop_rect.left() + 8.0, pop_rect.top() + 8.0),
                Pos2::new(pop_rect.right() - 8.0, pop_rect.top() + 40.0),
            );
            let new_hit = ui.interact(new_row, ui.id().with("open_pop_new_doc"), Sense::click());
            if new_hit.hovered() {
                painter.rect_filled(new_row, CornerRadius::same(6), pal.bg_pill_button);
            }
            painter.text(
                Pos2::new(new_row.left() + 12.0, new_row.center().y),
                Align2::LEFT_CENTER,
                format!("+ {}", I18n::tr(session.locale, "new_doc")),
                font_ui_bold(13.5),
                pal.fg_primary,
            );
            if new_hit.clicked() {
                actions.push(Action::NewDocument);
            }

            painter.line_segment(
                [
                    Pos2::new(pop_rect.left() + 10.0, pop_rect.top() + 44.0),
                    Pos2::new(pop_rect.right() - 10.0, pop_rect.top() + 44.0),
                ],
                Stroke::new(1.0, pal.border_headerbar),
            );

            painter.text(
                Pos2::new(pop_rect.left() + 18.0, pop_rect.top() + 62.0),
                Align2::LEFT_CENTER,
                I18n::tr(session.locale, "recent_docs"),
                font_ui_bold(12.5),
                pal.fg_muted,
            );

            let mut cur_y = pop_rect.top() + 78.0;
            for (idx, path) in session.recent_files.iter().enumerate() {
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, cur_y),
                    Pos2::new(pop_rect.right() - 8.0, cur_y + 34.0),
                );
                let hit = ui.interact(
                    row_r,
                    ui.id().with(("open_recent_file", idx)),
                    Sense::click(),
                );
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }
                let fname = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("documento.md");
                let is_current = session.current_file_path.as_ref() == Some(path);
                painter.text(
                    Pos2::new(row_r.left() + 12.0, row_r.center().y),
                    Align2::LEFT_CENTER,
                    fname,
                    if is_current {
                        font_ui_bold(13.5)
                    } else {
                        font_ui_regular(13.5)
                    },
                    if is_current {
                        pal.accent_orange
                    } else {
                        pal.fg_primary
                    },
                );
                if is_current {
                    let ck = Pos2::new(row_r.right() - 16.0, row_r.center().y);
                    painter.line_segment(
                        [Pos2::new(ck.x - 4.0, ck.y), Pos2::new(ck.x - 1.0, ck.y + 3.0)],
                        Stroke::new(1.8, pal.accent_orange),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(ck.x - 1.0, ck.y + 3.0),
                            Pos2::new(ck.x + 4.5, ck.y - 3.5),
                        ],
                        Stroke::new(1.8, pal.accent_orange),
                    );
                }
                if hit.clicked() {
                    actions.push(Action::OpenFile(path.clone()));
                }
                cur_y += 38.0;
            }

            painter.line_segment(
                [
                    Pos2::new(pop_rect.left() + 10.0, cur_y + 2.0),
                    Pos2::new(pop_rect.right() - 10.0, cur_y + 2.0),
                ],
                Stroke::new(1.0, pal.border_headerbar),
            );

            let browse_row = Rect::from_min_max(
                Pos2::new(pop_rect.left() + 8.0, cur_y + 8.0),
                Pos2::new(pop_rect.right() - 8.0, cur_y + 44.0),
            );
            let h_browse = ui.interact(browse_row, ui.id().with("open_file_btn"), Sense::click());
            if h_browse.hovered() {
                painter.rect_filled(browse_row, CornerRadius::same(6), pal.bg_pill_button);
            }
            painter.text(
                Pos2::new(browse_row.left() + 12.0, browse_row.center().y),
                Align2::LEFT_CENTER,
                I18n::tr(session.locale, "open_file"),
                font_ui_bold(14.0),
                pal.fg_primary,
            );
            if h_browse.clicked() {
                trigger_open_file_dialog(session, actions);
            }
        }
        ActiveOverlay::PreviewMenu => {
            // 5-item Compact Editing Mode dropdown menu:
            // 0: "Modifica"
            // 1: "Anteprima in-line •" (default, with checkmark)
            // 2: "Anteprima split a destra"
            // 3: "Anteprima split sotto"
            // 4: "Solo lettura"
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 352.0, 44.0),
                Pos2::new(win_rect.right() - 36.0, 240.0),
            );
            let shadow_rect = pop_rect.translate(Vec2::new(0.0, 3.0)).expand(1.5);
            painter.rect_filled(
                shadow_rect,
                CornerRadius::same(13),
                Color32::from_rgba_unmultiplied(0, 0, 0, 28),
            );
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );

            let active_mode = active_view_mode_index(session);
            let items: [(&str, &str, Action); 5] = match session.locale {
                Locale::It => [
                    ("Modifica", "Ctrl+Shift+E", Action::SetEditorOnlyRaw),
                    (
                        "Anteprima in-line •",
                        "Ctrl+L",
                        Action::SetEditorLivePreview,
                    ),
                    (
                        "Anteprima split a destra",
                        "Ctrl+Shift+P",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::HalfWidth,
                        },
                    ),
                    (
                        "Anteprima split sotto",
                        "Ctrl+Shift+J",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::HalfHeight,
                        },
                    ),
                    (
                        "Solo lettura",
                        "Ctrl+Alt+P",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::FullWidth,
                        },
                    ),
                ],
                Locale::Es => [
                    ("Editar", "Ctrl+Shift+E", Action::SetEditorOnlyRaw),
                    (
                        "Vista previa en línea •",
                        "Ctrl+L",
                        Action::SetEditorLivePreview,
                    ),
                    (
                        "Vista previa dividida a la derecha",
                        "Ctrl+Shift+P",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::HalfWidth,
                        },
                    ),
                    (
                        "Vista previa dividida abajo",
                        "Ctrl+Shift+J",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::HalfHeight,
                        },
                    ),
                    (
                        "Solo lectura",
                        "Ctrl+Alt+P",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::FullWidth,
                        },
                    ),
                ],
                Locale::En => [
                    ("Edit", "Ctrl+Shift+E", Action::SetEditorOnlyRaw),
                    ("Inline Preview •", "Ctrl+L", Action::SetEditorLivePreview),
                    (
                        "Split Preview Right",
                        "Ctrl+Shift+P",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::HalfWidth,
                        },
                    ),
                    (
                        "Split Preview Bottom",
                        "Ctrl+Shift+J",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::HalfHeight,
                        },
                    ),
                    (
                        "Read Only",
                        "Ctrl+Alt+P",
                        Action::SetPreviewMode {
                            active: true,
                            layout: PreviewLayout::FullWidth,
                        },
                    ),
                ],
            };

            for (idx, (label, shortcut, act)) in items.into_iter().enumerate() {
                let ry = pop_rect.top() + 26.0 + (idx as f32) * 36.0;
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, ry - 16.0),
                    Pos2::new(pop_rect.right() - 8.0, ry + 16.0),
                );
                let hit = ui.interact(
                    row_r,
                    ui.id().with(("compact_view_mode_row", idx)),
                    Sense::click(),
                );
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(7), pal.bg_pill_button);
                }

                let is_selected = active_mode == idx;
                let ic = Pos2::new(pop_rect.left() + 26.0, ry);
                draw_view_mode_icon(painter, ic, pal.fg_primary, pal.accent_orange, idx);

                painter.text(
                    Pos2::new(pop_rect.left() + 46.0, ry),
                    Align2::LEFT_CENTER,
                    label,
                    if is_selected {
                        font_ui_bold(13.5)
                    } else {
                        font_ui_regular(13.5)
                    },
                    pal.fg_primary,
                );

                painter.text(
                    Pos2::new(pop_rect.right() - 34.0, ry),
                    Align2::RIGHT_CENTER,
                    shortcut,
                    font_ui_regular(12.0),
                    pal.fg_muted,
                );

                if is_selected {
                    let ck = Pos2::new(pop_rect.right() - 18.0, ry);
                    painter.line_segment(
                        [Pos2::new(ck.x - 4.5, ck.y), Pos2::new(ck.x - 1.0, ck.y + 3.5)],
                        Stroke::new(2.0, pal.accent_orange),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(ck.x - 1.0, ck.y + 3.5),
                            Pos2::new(ck.x + 5.0, ck.y - 3.5),
                        ],
                        Stroke::new(2.0, pal.accent_orange),
                    );
                }

                if hit.clicked() {
                    actions.push(act);
                    actions.push(Action::SetOverlay(ActiveOverlay::None));
                }
            }
        }
        ActiveOverlay::MainMenu => {
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 302.0, 44.0),
                Pos2::new(win_rect.right() - 6.0, 444.0),
            );
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );

            // Theme selector circles row (System, Light, Sepia, Dark)
            let circle_y = pop_rect.top() + 48.0;
            let xs = [
                pop_rect.left() + 56.0,
                pop_rect.left() + 114.0,
                pop_rect.left() + 172.0,
                pop_rect.left() + 230.0,
            ];
            let modes = [
                (ThemeMode::Sepia, Color32::from_rgb(0xff, 0xff, 0xff)),
                (ThemeMode::Light, Color32::from_rgb(0xff, 0xff, 0xff)),
                (ThemeMode::Sepia, Color32::from_rgb(0xf9, 0xf3, 0xe9)),
                (ThemeMode::Dark, Color32::from_rgb(0x2b, 0x2b, 0x2f)),
            ];
            for (idx, (&cx, &(mode, fill))) in xs.iter().zip(modes.iter()).enumerate() {
                let center = Pos2::new(cx, circle_y);
                painter.circle_filled(center, 21.0, fill);
                if idx == 0 {
                    painter.circle_filled(center, 20.0, Color32::from_rgb(0x2b, 0x2b, 0x2f));
                    let left_half = Rect::from_min_max(
                        Pos2::new(center.x - 21.0, center.y - 21.0),
                        Pos2::new(center.x, center.y + 21.0),
                    );
                    painter.with_clip_rect(left_half).circle_filled(
                        center,
                        20.0,
                        Color32::from_rgb(0xff, 0xff, 0xff),
                    );
                }
                let is_selected = idx > 0 && session.theme == mode;
                let border_stroke = if is_selected {
                    Stroke::new(2.2, pal.accent_orange)
                } else {
                    Stroke::new(1.0, pal.border_headerbar)
                };
                painter.circle_stroke(center, 21.0, border_stroke);
                if is_selected {
                    let badge = Pos2::new(center.x + 13.0, center.y + 13.0);
                    painter.circle_filled(badge, 8.0, pal.accent_orange);
                    painter.line_segment(
                        [
                            Pos2::new(badge.x - 3.2, badge.y - 0.2),
                            Pos2::new(badge.x - 0.6, badge.y + 2.6),
                        ],
                        Stroke::new(1.6, Color32::WHITE),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(badge.x - 0.6, badge.y + 2.6),
                            Pos2::new(badge.x + 3.5, badge.y - 2.4),
                        ],
                        Stroke::new(1.6, Color32::WHITE),
                    );
                }
                let hit = ui.interact(
                    Rect::from_center_size(center, Vec2::new(42.0, 42.0)),
                    ui.id().with(("theme_circle", idx)),
                    Sense::click(),
                );
                if hit.clicked() {
                    actions.push(Action::SetTheme(mode));
                }
            }

            let items = [
                (
                    I18n::tr(session.locale, "hemingway_mode"),
                    "Ctrl+T",
                    108.0,
                    session.hemingway_mode,
                ),
                (
                    I18n::tr(session.locale, "focus_mode"),
                    "Ctrl+D",
                    142.0,
                    session.focus_mode,
                ),
                (
                    I18n::tr(session.locale, "find_replace"),
                    "Ctrl+H",
                    182.0,
                    session.search_bar_open,
                ),
                (
                    I18n::tr(session.locale, "preferences"),
                    "Ctrl+,",
                    222.0,
                    false,
                ),
                (
                    I18n::tr(session.locale, "advanced_export"),
                    "Ctrl+E",
                    256.0,
                    false,
                ),
                (I18n::tr(session.locale, "open_tutorial"), "", 290.0, false),
                (
                    I18n::tr(session.locale, "keyboard_shortcuts"),
                    "Ctrl+?",
                    322.0,
                    false,
                ),
                (I18n::tr(session.locale, "about"), "", 354.0, false),
            ];
            for (idx, (label, shortcut, dy, active)) in items.into_iter().enumerate() {
                let y = pop_rect.top() + dy;
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, y - 14.0),
                    Pos2::new(pop_rect.right() - 8.0, y + 14.0),
                );
                let hit = ui.interact(row_r, ui.id().with(("main_menu_row", idx)), Sense::click());
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }
                painter.text(
                    Pos2::new(pop_rect.left() + 20.0, y),
                    Align2::LEFT_CENTER,
                    label,
                    font_ui_regular(14.0),
                    if active {
                        pal.accent_orange
                    } else {
                        pal.fg_primary
                    },
                );
                if !shortcut.is_empty() {
                    painter.text(
                        Pos2::new(pop_rect.right() - 18.0, y),
                        Align2::RIGHT_CENTER,
                        shortcut,
                        font_ui_regular(13.5),
                        pal.fg_muted,
                    );
                }
                if hit.clicked() {
                    match idx {
                        0 => actions.push(Action::ToggleHemingway),
                        1 => actions.push(Action::ToggleFocusMode),
                        2 => {
                            actions.push(Action::ToggleSearchBar);
                            actions.push(Action::SetOverlay(ActiveOverlay::None));
                        }
                        3 => actions.push(Action::SetOverlay(ActiveOverlay::PreferencesDialog)),
                        4 => actions.push(Action::SetOverlay(ActiveOverlay::ExportDialog)),
                        5 => actions.push(Action::OpenTutorial),
                        6 => actions.push(Action::SetOverlay(ActiveOverlay::ShortcutsDialog)),
                        7 => actions.push(Action::SetOverlay(ActiveOverlay::AboutDialog)),
                        _ => actions.push(Action::SetOverlay(ActiveOverlay::None)),
                    }
                }
            }

            // Language switcher row (IT / EN / ES) at the bottom of MainMenu
            let lang_y = pop_rect.bottom() - 22.0;
            painter.text(
                Pos2::new(pop_rect.left() + 20.0, lang_y),
                Align2::LEFT_CENTER,
                I18n::tr(session.locale, "language"),
                font_ui_regular(13.5),
                pal.fg_muted,
            );
            for (l_idx, (loc, code)) in
                [(Locale::It, "IT"), (Locale::En, "EN"), (Locale::Es, "ES")]
                    .into_iter()
                    .enumerate()
            {
                let lx = pop_rect.right() - 118.0 + (l_idx as f32) * 36.0;
                let lr = Rect::from_center_size(Pos2::new(lx, lang_y), Vec2::new(32.0, 22.0));
                let active = session.locale == loc;
                painter.rect_filled(
                    lr,
                    CornerRadius::same(5),
                    if active {
                        pal.accent_orange
                    } else {
                        pal.bg_pill_button
                    },
                );
                painter.text(
                    lr.center(),
                    Align2::CENTER_CENTER,
                    code,
                    font_ui_bold(11.5),
                    if active {
                        Color32::WHITE
                    } else {
                        pal.fg_primary
                    },
                );
                let l_hit = ui.interact(lr, ui.id().with(("lang_btn", l_idx)), Sense::click());
                if l_hit.clicked() {
                    actions.push(Action::SetLocale(loc));
                }
            }
        }
        ActiveOverlay::LineWidthMenu => {
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 310.0, 44.0),
                Pos2::new(win_rect.right() - 70.0, 232.0),
            );
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );
            painter.text(
                Pos2::new(pop_rect.left() + 18.0, pop_rect.top() + 24.0),
                Align2::LEFT_CENTER,
                I18n::tr(session.locale, "line_width"),
                font_ui_bold(13.5),
                pal.fg_primary,
            );
            let modes = [
                (LineWidthMode::Compact, I18n::tr(session.locale, "compact")),
                (LineWidthMode::Medium, I18n::tr(session.locale, "medium")),
                (LineWidthMode::Wide, I18n::tr(session.locale, "wide")),
                (
                    LineWidthMode::Fluid { margin_pct: 6 },
                    I18n::tr(session.locale, "fluid"),
                ),
            ];
            for (idx, (m, label)) in modes.into_iter().enumerate() {
                let ry = pop_rect.top() + 56.0 + (idx as f32) * 32.0;
                let active = matches!(
                    (session.line_width, m),
                    (LineWidthMode::Compact, LineWidthMode::Compact)
                        | (LineWidthMode::Medium, LineWidthMode::Medium)
                        | (LineWidthMode::Wide, LineWidthMode::Wide)
                        | (LineWidthMode::Fluid { .. }, LineWidthMode::Fluid { .. })
                );
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 10.0, ry - 14.0),
                    Pos2::new(pop_rect.right() - 10.0, ry + 14.0),
                );
                let row_hit = ui.interact(row_r, ui.id().with(("lw_pop_row", idx)), Sense::click());
                if row_hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }
                let c = Pos2::new(pop_rect.left() + 24.0, ry);
                painter.circle_stroke(
                    c,
                    7.0,
                    Stroke::new(
                        1.4,
                        if active {
                            pal.accent_orange
                        } else {
                            pal.fg_muted
                        },
                    ),
                );
                if active {
                    painter.circle_filled(c, 3.8, pal.accent_orange);
                }
                painter.text(
                    Pos2::new(pop_rect.left() + 40.0, ry),
                    Align2::LEFT_CENTER,
                    label,
                    font_ui_regular(13.5),
                    pal.fg_primary,
                );
                if row_hit.clicked() {
                    actions.push(Action::SetLineWidth(m));
                    actions.push(Action::SetOverlay(ActiveOverlay::None));
                }
            }
        }
        ActiveOverlay::SaveMenu => {
            let pop_rect = Rect::from_min_max(Pos2::new(124.0, 44.0), Pos2::new(374.0, 326.0));
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );
            let export_hdr = match session.locale {
                Locale::It => "Esportazione",
                Locale::Es => "Exportación",
                Locale::En => "Export",
            };
            painter.line_segment(
                [
                    Pos2::new(pop_rect.left() + 10.0, pop_rect.top() + 46.0),
                    Pos2::new(pop_rect.right() - 10.0, pop_rect.top() + 46.0),
                ],
                Stroke::new(1.0, pal.border_headerbar),
            );
            painter.text(
                Pos2::new(pop_rect.left() + 18.0, pop_rect.top() + 64.0),
                Align2::LEFT_CENTER,
                export_hdr,
                font_ui_bold(12.0),
                pal.fg_muted,
            );
            painter.line_segment(
                [
                    Pos2::new(pop_rect.left() + 10.0, pop_rect.top() + 240.0),
                    Pos2::new(pop_rect.right() - 10.0, pop_rect.top() + 240.0),
                ],
                Stroke::new(1.0, pal.border_headerbar),
            );
            let rows = [
                (I18n::tr(session.locale, "save_as"), 26.0),
                ("PDF (.pdf)", 92.0),
                ("Word (.docx)", 122.0),
                ("OpenDocument (.odt)", 152.0),
                ("LaTeX (.tex)", 182.0),
                ("EPUB (.epub)", 212.0),
                (I18n::tr(session.locale, "advanced_export"), 260.0),
            ];
            for (idx, (label, dy)) in rows.into_iter().enumerate() {
                let y = pop_rect.top() + dy;
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, y - 14.0),
                    Pos2::new(pop_rect.right() - 8.0, y + 14.0),
                );
                let hit = ui.interact(row_r, ui.id().with(("save_pop_row", idx)), Sense::click());
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }
                painter.text(
                    Pos2::new(pop_rect.left() + 18.0, y),
                    Align2::LEFT_CENTER,
                    label,
                    if idx == 0 {
                        font_ui_bold(13.8)
                    } else {
                        font_ui_regular(13.8)
                    },
                    pal.fg_primary,
                );
                if hit.clicked() {
                    match idx {
                        0 => trigger_save_as_file_dialog(session, actions),
                        1 => actions.push(Action::ExportDocument("pdf".to_string())),
                        2 => actions.push(Action::ExportDocument("docx".to_string())),
                        3 => actions.push(Action::ExportDocument("odt".to_string())),
                        4 => actions.push(Action::ExportDocument("latex".to_string())),
                        5 => actions.push(Action::ExportDocument("epub".to_string())),
                        6 => actions.push(Action::SetOverlay(ActiveOverlay::ExportDialog)),
                        _ => actions.push(Action::SetOverlay(ActiveOverlay::None)),
                    }
                }
            }
        }
        ActiveOverlay::StatsMenu => {
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 266.0, win_rect.bottom() - 232.0),
                Pos2::new(win_rect.right() - 18.0, win_rect.bottom() - 58.0),
            );
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );
            let stats = session.buffer.stats();
            let options = [
                (
                    StatMetric::Characters,
                    format!(
                        "{} {}",
                        accent_session::format_thousands(stats.characters, session.locale),
                        I18n::tr(session.locale, "characters")
                    ),
                ),
                (
                    StatMetric::Words,
                    format!(
                        "{} {}",
                        accent_session::format_thousands(stats.words, session.locale),
                        I18n::tr(session.locale, "words")
                    ),
                ),
                (
                    StatMetric::Sentences,
                    format!(
                        "{} {}",
                        accent_session::format_thousands(stats.sentences, session.locale),
                        I18n::tr(session.locale, "sentences")
                    ),
                ),
                (
                    StatMetric::Paragraphs,
                    format!(
                        "{} {}",
                        accent_session::format_thousands(stats.paragraphs, session.locale),
                        I18n::tr(session.locale, "paragraphs")
                    ),
                ),
                (
                    StatMetric::ReadingTime,
                    format!(
                        "{} {}",
                        stats.format_reading_time(),
                        I18n::tr(session.locale, "reading_time")
                    ),
                ),
            ];
            for (idx, (metric, text)) in options.iter().enumerate() {
                let ry = pop_rect.top() + 24.0 + (idx as f32) * 31.0;
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, ry - 14.0),
                    Pos2::new(pop_rect.right() - 8.0, ry + 14.0),
                );
                let hit = ui.interact(row_r, ui.id().with(("stat_pop_row", idx)), Sense::click());
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }
                let c = Pos2::new(pop_rect.left() + 22.0, ry);
                painter.circle_stroke(c, 7.0, Stroke::new(1.2, pal.fg_muted));
                if session.stat_metric == *metric {
                    painter.circle_filled(c, 3.5, pal.fg_primary);
                }
                painter.text(
                    Pos2::new(pop_rect.left() + 38.0, ry),
                    Align2::LEFT_CENTER,
                    text,
                    font_ui_regular(13.5),
                    pal.fg_primary,
                );
                if hit.clicked() {
                    actions.push(Action::SetStatMetric(*metric));
                    actions.push(Action::SetOverlay(ActiveOverlay::None));
                }
            }
        }
        ActiveOverlay::PreferencesDialog => {
            render_preferences_dialog(ui, painter, win_rect, session, pal, actions);
        }
        ActiveOverlay::OpenFileDialog => {
            render_file_browser_dialog(ui, painter, win_rect, session, pal, actions, false);
        }
        ActiveOverlay::SaveFileDialog => {
            render_file_browser_dialog(ui, painter, win_rect, session, pal, actions, true);
        }
        ActiveOverlay::ExportDialog => {
            render_export_dialog(ui, painter, win_rect, session, pal, actions);
        }
        ActiveOverlay::ShortcutsDialog => {
            render_shortcuts_dialog(ui, painter, win_rect, session, pal, actions);
        }
        ActiveOverlay::AboutDialog => {
            render_about_dialog(ui, painter, win_rect, session, pal, actions);
        }
    }
}

fn render_file_browser_dialog(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
    is_save_mode: bool,
) {
    painter.rect_filled(
        win_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 48),
    );
    let dlg_rect = Rect::from_center_size(win_rect.center(), Vec2::new(660.0, 480.0));
    painter.rect_filled(dlg_rect, CornerRadius::same(14), pal.bg_popover);
    painter.rect_stroke(
        dlg_rect,
        CornerRadius::same(14),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let hdr_y = dlg_rect.top() + 26.0;
    let title = if is_save_mode {
        I18n::tr(session.locale, "save_as")
    } else {
        I18n::tr(session.locale, "open_file")
    };
    painter.text(
        Pos2::new(dlg_rect.center().x, hdr_y),
        Align2::CENTER_CENTER,
        title,
        font_ui_bold(15.0),
        pal.fg_primary,
    );
    let close_pos = Pos2::new(dlg_rect.right() - 26.0, hdr_y);
    draw_window_button(painter, close_pos, pal.bg_pill_button, pal.fg_primary, 'x');
    if ui
        .interact(
            Rect::from_center_size(close_pos, Vec2::new(28.0, 28.0)),
            ui.id().with("file_dlg_close_btn"),
            Sense::click(),
        )
        .clicked()
    {
        actions.push(Action::SetOverlay(ActiveOverlay::None));
    }

    // Current directory bar + parent folder ("..") button
    let dir_bar = Rect::from_min_max(
        Pos2::new(dlg_rect.left() + 24.0, dlg_rect.top() + 52.0),
        Pos2::new(dlg_rect.right() - 24.0, dlg_rect.top() + 84.0),
    );
    painter.rect_filled(dir_bar, CornerRadius::same(6), pal.bg_pill_button);
    let up_btn = Rect::from_min_max(
        Pos2::new(dir_bar.left() + 4.0, dir_bar.top() + 4.0),
        Pos2::new(dir_bar.left() + 86.0, dir_bar.bottom() - 4.0),
    );
    painter.rect_filled(up_btn, CornerRadius::same(5), pal.bg_popover);
    painter.text(
        up_btn.center(),
        Align2::CENTER_CENTER,
        "⬆ ..",
        font_ui_bold(12.5),
        pal.fg_primary,
    );
    if ui
        .interact(up_btn, ui.id().with("file_dlg_up_btn"), Sense::click())
        .clicked()
    {
        if let Some(parent) = session.file_dialog_dir.parent() {
            actions.push(Action::SetFileDialogDir(parent.to_path_buf()));
        }
    }
    painter.text(
        Pos2::new(up_btn.right() + 12.0, dir_bar.center().y),
        Align2::LEFT_CENTER,
        session.file_dialog_dir.display().to_string(),
        font_ui_regular(12.5),
        pal.fg_muted,
    );

    // Directory & Markdown file listing inside a scrollable area
    let list_rect = Rect::from_min_max(
        Pos2::new(dlg_rect.left() + 24.0, dlg_rect.top() + 94.0),
        Pos2::new(dlg_rect.right() - 24.0, dlg_rect.bottom() - 72.0),
    );
    painter.rect_filled(list_rect, CornerRadius::same(8), pal.bg_canvas);
    painter.rect_stroke(
        list_rect,
        CornerRadius::same(8),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let mut dirs = Vec::new();
    let mut files = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&session.file_dialog_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if p.is_dir() {
                dirs.push((name, p));
            } else {
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if matches!(ext.as_str(), "md" | "markdown" | "txt") {
                    files.push((name, p));
                }
            }
        }
    }
    dirs.sort_by(|a, b| a.0.cmp(&b.0));
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut list_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(list_rect.shrink(8.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    ScrollArea::vertical()
        .id_salt("accent_file_browser_scroll")
        .auto_shrink([false, false])
        .show(&mut list_ui, |ui| {
            for (f_idx, (f_name, f_path)) in files.iter().enumerate() {
                let is_sel = session.file_dialog_input == *f_name;
                let (row_r, hit) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 28.0),
                    Sense::click(),
                );
                if is_sel || hit.hovered() {
                    ui.painter()
                        .rect_filled(row_r, CornerRadius::same(5), pal.bg_pill_button);
                }
                ui.painter().text(
                    Pos2::new(row_r.left() + 10.0, row_r.center().y),
                    Align2::LEFT_CENTER,
                    format!("📄  {f_name}"),
                    if is_sel {
                        font_ui_bold(13.5)
                    } else {
                        font_ui_regular(13.5)
                    },
                    if is_sel {
                        pal.accent_orange
                    } else {
                        pal.fg_primary
                    },
                );
                if hit.clicked() {
                    if is_save_mode {
                        actions.push(Action::SetFileDialogInput(f_name.clone()));
                    } else {
                        actions.push(Action::OpenFile(f_path.clone()));
                    }
                }
                let _ = f_idx;
            }
            for (d_idx, (d_name, d_path)) in dirs.iter().enumerate() {
                let (row_r, hit) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 28.0),
                    Sense::click(),
                );
                if hit.hovered() {
                    ui.painter()
                        .rect_filled(row_r, CornerRadius::same(5), pal.bg_pill_button);
                }
                ui.painter().text(
                    Pos2::new(row_r.left() + 10.0, row_r.center().y),
                    Align2::LEFT_CENTER,
                    format!("📁  {d_name}/"),
                    font_ui_bold(13.5),
                    pal.fg_primary,
                );
                if hit.clicked() {
                    actions.push(Action::SetFileDialogDir(d_path.clone()));
                }
                let _ = d_idx;
            }
        });

    // Bottom bar: Filename TextEdit + Cancel / Open(or Save) button
    let bot_rect = Rect::from_min_max(
        Pos2::new(dlg_rect.left() + 24.0, dlg_rect.bottom() - 56.0),
        Pos2::new(dlg_rect.right() - 24.0, dlg_rect.bottom() - 18.0),
    );
    let mut bot_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bot_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let mut fname = session.file_dialog_input.clone();
    let te = bot_ui.add(TextEdit::singleline(&mut fname).desired_width(410.0));
    if te.changed() {
        actions.push(Action::SetFileDialogInput(fname.clone()));
    }
    bot_ui.add_space(12.0);
    if bot_ui.button(I18n::tr(session.locale, "cancel")).clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::None));
    }
    bot_ui.add_space(8.0);
    let confirm_label = if is_save_mode {
        I18n::tr(session.locale, "save")
    } else {
        I18n::tr(session.locale, "open")
    };
    if bot_ui.button(confirm_label).clicked() {
        let candidate = std::path::PathBuf::from(&fname);
        let full_path = if candidate.is_absolute() {
            candidate
        } else {
            session.file_dialog_dir.join(&fname)
        };
        if is_save_mode {
            actions.push(Action::SaveAsFile(full_path));
        } else {
            actions.push(Action::OpenFile(full_path));
        }
    }
}

fn get_or_load_mermaid_texture(
    ctx: &Context,
    session: &Session,
    code: &str,
) -> (egui::TextureHandle, u32, u32) {
    let diagram = session.render_mermaid(code);
    let cache_id = Id::new("accent_mermaid_gpu_textures");
    let key = (diagram.content_hash, diagram.theme);
    if let Some(cached) = ctx.data_mut(|d| {
        d.get_temp::<std::collections::HashMap<
            (u64, accent_session::MermaidTheme),
            (egui::TextureHandle, u32, u32),
        >>(cache_id)
        .and_then(|m| m.get(&key).cloned())
    }) {
        return cached;
    }

    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [diagram.width as usize, diagram.height as usize],
        &diagram.rgba_pixels,
    );
    let tex = ctx.load_texture(
        format!("mermaid_{:016x}_{:?}", diagram.content_hash, diagram.theme),
        color_image,
        egui::TextureOptions::LINEAR,
    );
    let entry = (tex, diagram.width, diagram.height);
    ctx.data_mut(|d| {
        let mut map = d
            .get_temp::<std::collections::HashMap<
                (u64, accent_session::MermaidTheme),
                (egui::TextureHandle, u32, u32),
            >>(cache_id)
            .unwrap_or_default();
        map.insert(key, entry.clone());
        d.insert_temp(cache_id, map);
    });
    entry
}

fn render_export_dialog(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    painter.rect_filled(
        win_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 56),
    );
    let dlg_rect = Rect::from_center_size(win_rect.center(), Vec2::new(666.0, 468.0));
    painter.rect_filled(dlg_rect, CornerRadius::same(14), pal.bg_popover);
    painter.rect_stroke(
        dlg_rect,
        CornerRadius::same(14),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let sidebar_w = 212.0;
    let split_x = dlg_rect.left() + sidebar_w;
    let sidebar_rect = Rect::from_min_max(dlg_rect.left_top(), Pos2::new(split_x, dlg_rect.bottom()));
    let right_rect = Rect::from_min_max(Pos2::new(split_x, dlg_rect.top()), dlg_rect.right_bottom());

    // Vertical separator between Formati sidebar and options pane
    painter.line_segment(
        [
            Pos2::new(split_x, dlg_rect.top() + 1.0),
            Pos2::new(split_x, dlg_rect.bottom() - 1.0),
        ],
        Stroke::new(1.0, pal.border_headerbar),
    );

    let hdr_y = dlg_rect.top() + 26.0;
    let formats_title = match session.locale {
        Locale::It => "Formati",
        Locale::Es => "Formatos",
        Locale::En => "Formats",
    };
    painter.text(
        Pos2::new(sidebar_rect.center().x, hdr_y),
        Align2::CENTER_CENTER,
        formats_title,
        font_ui_bold(14.5),
        pal.fg_primary,
    );

    let export_header = match session.locale {
        Locale::It => format!("Esporta in {}", session.selected_export_format.sidebar_label()),
        Locale::Es => format!("Exportar a {}", session.selected_export_format.sidebar_label()),
        Locale::En => format!("Export to {}", session.selected_export_format.sidebar_label()),
    };
    painter.text(
        Pos2::new(right_rect.center().x - 10.0, hdr_y),
        Align2::CENTER_CENTER,
        export_header,
        font_ui_bold(15.0),
        pal.fg_primary,
    );

    let close_pos = Pos2::new(dlg_rect.right() - 26.0, hdr_y);
    draw_window_button(painter, close_pos, pal.bg_pill_button, pal.fg_primary, 'x');
    if ui
        .interact(
            Rect::from_center_size(close_pos, Vec2::new(28.0, 28.0)),
            ui.id().with("export_dlg_close"),
            Sense::click(),
        )
        .clicked()
    {
        actions.push(Action::SetOverlay(ActiveOverlay::None));
    }

    // Left Sidebar: Format list (matching Apostrophe's Export Dialog)
    let formats = [
        ExportFormat::Pdf,
        ExportFormat::Odt,
        ExportFormat::Docx,
        ExportFormat::Epub,
        ExportFormat::Latex,
        ExportFormat::Html,
    ];
    for (idx, fmt) in formats.into_iter().enumerate() {
        let ry = dlg_rect.top() + 74.0 + (idx as f32) * 44.0;
        let row_r = Rect::from_min_max(
            Pos2::new(sidebar_rect.left() + 8.0, ry - 18.0),
            Pos2::new(sidebar_rect.right() - 8.0, ry + 18.0),
        );
        let hit = ui.interact(row_r, ui.id().with(("exp_fmt_row", idx)), Sense::click());
        let is_selected = session.selected_export_format == fmt;
        if is_selected {
            painter.rect_filled(
                row_r,
                CornerRadius::same(8),
                pal.accent_orange.gamma_multiply(0.16),
            );
            painter.rect_stroke(
                row_r,
                CornerRadius::same(8),
                Stroke::new(1.2, pal.accent_orange),
                StrokeKind::Middle,
            );
        } else if hit.hovered() {
            painter.rect_filled(row_r, CornerRadius::same(8), pal.bg_pill_button);
        }
        painter.text(
            Pos2::new(row_r.left() + 14.0, row_r.center().y),
            Align2::LEFT_CENTER,
            fmt.sidebar_label(),
            if is_selected {
                font_ui_bold(13.5)
            } else {
                font_ui_regular(13.5)
            },
            if is_selected {
                pal.accent_orange
            } else {
                pal.fg_primary
            },
        );
        if hit.clicked() {
            actions.push(Action::SelectExportFormat(fmt));
        }
    }

    // Right Pane Option Cards
    let card_left = right_rect.left() + 26.0;
    let card_right = right_rect.right() - 26.0;

    // Card 1: Destination File Path
    let dest_label = match session.locale {
        Locale::It => "Percorso file di destinazione",
        Locale::Es => "Ruta del archivo de destino",
        Locale::En => "Destination file path",
    };
    painter.text(
        Pos2::new(card_left + 4.0, dlg_rect.top() + 68.0),
        Align2::LEFT_CENTER,
        dest_label,
        font_ui_bold(13.0),
        pal.fg_muted,
    );
    let path_card = Rect::from_min_max(
        Pos2::new(card_left, dlg_rect.top() + 84.0),
        Pos2::new(card_right, dlg_rect.top() + 132.0),
    );
    painter.rect_filled(path_card, CornerRadius::same(10), pal.bg_canvas);
    painter.rect_stroke(
        path_card,
        CornerRadius::same(10),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let mut out_path_str = session.export_output_path.clone();
    let mut path_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(path_card.shrink2(Vec2::new(12.0, 12.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let path_resp = path_ui.add(
        TextEdit::singleline(&mut out_path_str)
            .id(Id::new("accent_export_output_path_input"))
            .frame(false)
            .desired_width(path_card.width() - 28.0)
            .font(font_editor_regular(13.5))
            .text_color(pal.fg_primary),
    );
    if path_resp.changed() {
        actions.push(Action::SetExportOutputPath(out_path_str));
    }

    // Card 2: Page Size & Options (inspired by Apostrophe's Export options card)
    let opts_card = Rect::from_min_max(
        Pos2::new(card_left, dlg_rect.top() + 154.0),
        Pos2::new(card_right, dlg_rect.top() + 266.0),
    );
    painter.rect_filled(opts_card, CornerRadius::same(10), pal.bg_canvas);
    painter.rect_stroke(
        opts_card,
        CornerRadius::same(10),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );
    painter.line_segment(
        [
            Pos2::new(opts_card.left() + 12.0, opts_card.center().y),
            Pos2::new(opts_card.right() - 12.0, opts_card.center().y),
        ],
        Stroke::new(1.0, pal.border_headerbar),
    );

    let row1_y = (opts_card.top() + opts_card.center().y) * 0.5;
    let page_size_label = match session.locale {
        Locale::It => "Formato pagina",
        Locale::Es => "Tamaño de página",
        Locale::En => "Page Size",
    };
    painter.text(
        Pos2::new(opts_card.left() + 16.0, row1_y),
        Align2::LEFT_CENTER,
        page_size_label,
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    let a4_badge = Rect::from_center_size(
        Pos2::new(opts_card.right() - 48.0, row1_y),
        Vec2::new(56.0, 26.0),
    );
    painter.rect_filled(a4_badge, CornerRadius::same(6), pal.bg_pill_button);
    painter.text(
        a4_badge.center(),
        Align2::CENTER_CENTER,
        "A4",
        font_ui_bold(13.0),
        pal.fg_primary,
    );

    let row2_y = (opts_card.center().y + opts_card.bottom()) * 0.5;
    let mermaid_opt_label = match session.locale {
        Locale::It => "Diagrammi Mermaid & stili codice",
        Locale::Es => "Diagramas Mermaid y estilos de código",
        Locale::En => "Mermaid diagrams & code styles",
    };
    painter.text(
        Pos2::new(opts_card.left() + 16.0, row2_y),
        Align2::LEFT_CENTER,
        mermaid_opt_label,
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    let sw_rect = Rect::from_center_size(
        Pos2::new(opts_card.right() - 38.0, row2_y),
        Vec2::new(42.0, 22.0),
    );
    let sw_hit = ui.interact(sw_rect, ui.id().with("export_mermaid_toggle"), Sense::click());
    painter.rect_filled(
        sw_rect,
        CornerRadius::same(11),
        if session.export_syntax_highlight {
            pal.accent_orange
        } else {
            pal.bg_pill_button
        },
    );
    let knob_x = if session.export_syntax_highlight {
        sw_rect.right() - 11.0
    } else {
        sw_rect.left() + 11.0
    };
    painter.circle_filled(Pos2::new(knob_x, row2_y), 8.5, Color32::WHITE);
    if sw_hit.clicked() {
        actions.push(Action::ToggleExportSyntaxHighlight);
    }

    // Format description badge
    let engine_desc = match session.selected_export_format {
        ExportFormat::Pdf => "Motore Rust nativo (PDF 1.4 paginato + XObject PNG Mermaid)",
        ExportFormat::Docx => "Motore Rust nativo (docx-rs OpenXML + relazioni immagini)",
        ExportFormat::Odt => "Motore Rust nativo (ODF 1.2 ZIP + Pictures/mermaid_*.png)",
        ExportFormat::Latex => "Serializzatore AST LaTeX (.tex con escaping rigoroso)",
        ExportFormat::Epub => "Motore Rust nativo (EPUB v3 XHTML + OPF + foglio stile)",
        ExportFormat::Html => "Generatore HTML5 autonomo con CSS tipografico integrato",
    };
    painter.text(
        Pos2::new(right_rect.center().x, dlg_rect.top() + 305.0),
        Align2::CENTER_CENTER,
        engine_desc,
        font_ui_regular(12.5),
        pal.fg_muted,
    );

    // Bottom-centered Orange "Esporta" Button (matching Apostrophe's Export button)
    let exp_btn_rect = Rect::from_center_size(
        Pos2::new(right_rect.center().x, dlg_rect.bottom() - 46.0),
        Vec2::new(136.0, 40.0),
    );
    let exp_hit = ui.interact(
        exp_btn_rect,
        ui.id().with("export_dlg_execute_btn"),
        Sense::click(),
    );
    painter.rect_filled(
        exp_btn_rect,
        CornerRadius::same(20),
        if exp_hit.hovered() {
            pal.accent_orange.gamma_multiply(1.08)
        } else {
            pal.accent_orange
        },
    );
    let export_btn_label = match session.locale {
        Locale::It => "Esporta",
        Locale::Es => "Exportar",
        Locale::En => "Export",
    };
    painter.text(
        exp_btn_rect.center(),
        Align2::CENTER_CENTER,
        export_btn_label,
        font_ui_bold(14.5),
        Color32::WHITE,
    );
    if exp_hit.clicked() || ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
        actions.push(Action::ExecuteExport);
    }
}

fn render_shortcuts_dialog(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    painter.rect_filled(
        win_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 48),
    );
    let dlg_rect = Rect::from_center_size(win_rect.center(), Vec2::new(540.0, 490.0));
    painter.rect_filled(dlg_rect, CornerRadius::same(14), pal.bg_popover);
    painter.rect_stroke(
        dlg_rect,
        CornerRadius::same(14),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );
    let hdr_y = dlg_rect.top() + 26.0;
    painter.text(
        Pos2::new(dlg_rect.center().x, hdr_y),
        Align2::CENTER_CENTER,
        I18n::tr(session.locale, "keyboard_shortcuts"),
        font_ui_bold(15.0),
        pal.fg_primary,
    );
    let close_pos = Pos2::new(dlg_rect.right() - 26.0, hdr_y);
    draw_window_button(painter, close_pos, pal.bg_pill_button, pal.fg_primary, 'x');
    if ui
        .interact(
            Rect::from_center_size(close_pos, Vec2::new(28.0, 28.0)),
            ui.id().with("shortcuts_dlg_close"),
            Sense::click(),
        )
        .clicked()
    {
        actions.push(Action::SetOverlay(ActiveOverlay::None));
    }
    let shortcuts = [
        ("Ctrl+Shift+E", I18n::tr(session.locale, "editor_only_raw")),
        ("Ctrl+L", I18n::tr(session.locale, "view_while_typing")),
        ("Ctrl+Shift+P", I18n::tr(session.locale, "half_width")),
        ("Ctrl+Alt+P", I18n::tr(session.locale, "full_width")),
        ("Ctrl+Shift+J", I18n::tr(session.locale, "half_height")),
        ("Ctrl+Shift+W", I18n::tr(session.locale, "windowed")),
        ("Ctrl+T", I18n::tr(session.locale, "hemingway_mode")),
        ("Ctrl+D", I18n::tr(session.locale, "focus_mode")),
        ("Ctrl+H", I18n::tr(session.locale, "find_replace")),
        ("Ctrl+,", I18n::tr(session.locale, "preferences")),
        ("F10", "Menu principale"),
        ("Esc", "Chiudi menu / finestra di dialogo"),
    ];
    for (idx, (key, label)) in shortcuts.into_iter().enumerate() {
        let ry = dlg_rect.top() + 68.0 + (idx as f32) * 33.0;
        painter.text(
            Pos2::new(dlg_rect.left() + 32.0, ry),
            Align2::LEFT_CENTER,
            label,
            font_ui_regular(14.0),
            pal.fg_primary,
        );
        painter.text(
            Pos2::new(dlg_rect.right() - 32.0, ry),
            Align2::RIGHT_CENTER,
            key,
            font_ui_bold(13.5),
            pal.accent_orange,
        );
    }
}

fn render_about_dialog(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    painter.rect_filled(
        win_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 48),
    );
    let dlg_rect = Rect::from_center_size(win_rect.center(), Vec2::new(440.0, 240.0));
    painter.rect_filled(dlg_rect, CornerRadius::same(14), pal.bg_popover);
    painter.rect_stroke(
        dlg_rect,
        CornerRadius::same(14),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );
    let hdr_y = dlg_rect.top() + 26.0;
    painter.text(
        Pos2::new(dlg_rect.center().x, hdr_y),
        Align2::CENTER_CENTER,
        I18n::tr(session.locale, "about"),
        font_ui_bold(15.0),
        pal.fg_primary,
    );
    let close_pos = Pos2::new(dlg_rect.right() - 26.0, hdr_y);
    draw_window_button(painter, close_pos, pal.bg_pill_button, pal.fg_primary, 'x');
    if ui
        .interact(
            Rect::from_center_size(close_pos, Vec2::new(28.0, 28.0)),
            ui.id().with("about_dlg_close"),
            Sense::click(),
        )
        .clicked()
    {
        actions.push(Action::SetOverlay(ActiveOverlay::None));
    }
    painter.text(
        Pos2::new(dlg_rect.center().x, dlg_rect.top() + 86.0),
        Align2::CENTER_CENTER,
        "Accent Markdown Editor v0.3.0",
        font_ui_bold(17.0),
        pal.fg_primary,
    );
    painter.text(
        Pos2::new(dlg_rect.center().x, dlg_rect.top() + 122.0),
        Align2::CENTER_CENTER,
        "100% Pure Rust • eframe / egui • comrak AST • ropey",
        font_ui_regular(13.5),
        pal.fg_muted,
    );
    painter.text(
        Pos2::new(dlg_rect.center().x, dlg_rect.top() + 154.0),
        Align2::CENTER_CENTER,
        "Clean-room multiplatform Markdown editor & viewer",
        font_ui_regular(13.0),
        pal.fg_muted,
    );
}

fn render_preferences_dialog(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    // Modal backdrop dimming matching Apostrophe's Preferences dialog
    painter.rect_filled(
        win_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 42),
    );

    let dlg_w = 760.0_f32;
    let dlg_h = 540.0_f32;
    let dlg_rect = Rect::from_center_size(win_rect.center(), Vec2::new(dlg_w, dlg_h));

    // Soft drop shadow
    for i in 1..=6 {
        let expand = (i as f32) * 2.2;
        painter.rect_filled(
            dlg_rect.expand(expand),
            CornerRadius::same(16),
            Color32::from_rgba_unmultiplied(0, 0, 0, (18 - i * 2) as u8),
        );
    }

    painter.rect_filled(dlg_rect, CornerRadius::same(14), pal.bg_popover);
    painter.rect_stroke(
        dlg_rect,
        CornerRadius::same(14),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    // Dialog Header: centered title "Preferenze" / "Preferences" + close 'x' button
    let hdr_y = dlg_rect.top() + 26.0;
    painter.text(
        Pos2::new(dlg_rect.center().x, hdr_y),
        Align2::CENTER_CENTER,
        I18n::tr(session.locale, "preferences"),
        font_ui_bold(15.0),
        pal.fg_primary,
    );
    let close_pos = Pos2::new(dlg_rect.right() - 26.0, hdr_y);
    draw_window_button(painter, close_pos, pal.bg_pill_button, pal.fg_primary, 'x');
    let close_hit = ui.interact(
        Rect::from_center_size(close_pos, Vec2::new(28.0, 28.0)),
        ui.id().with("prefs_close_btn"),
        Sense::click(),
    );
    if close_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::None));
    }

    // Inner grouped card container
    let card_rect = Rect::from_min_max(
        Pos2::new(dlg_rect.left() + 34.0, dlg_rect.top() + 58.0),
        Pos2::new(dlg_rect.right() - 34.0, dlg_rect.bottom() - 28.0),
    );
    let card_fill = match session.theme {
        ThemeMode::Light => Color32::from_rgb(0xfb, 0xfb, 0xfb),
        ThemeMode::Sepia => Color32::from_rgb(0xfc, 0xf8, 0xf1),
        ThemeMode::Dark => Color32::from_rgb(0x35, 0x35, 0x3a),
    };
    painter.rect_filled(card_rect, CornerRadius::same(12), card_fill);
    painter.rect_stroke(
        card_rect,
        CornerRadius::same(12),
        Stroke::new(1.0, pal.border_headerbar),
        StrokeKind::Middle,
    );

    let row_h = 75.0_f32;
    for r in 1..6 {
        let ry = card_rect.top() + (r as f32) * row_h;
        painter.line_segment(
            [
                Pos2::new(card_rect.left(), ry),
                Pos2::new(card_rect.right(), ry),
            ],
            Stroke::new(1.0, pal.border_headerbar),
        );
    }

    let left_x = card_rect.left() + 20.0;
    let right_x = card_rect.right() - 20.0;

    // ------------------------------------------------------------------------
    // Row 0: Configurable Line Width (Compact, Medium, Wide, Fluid)
    // ------------------------------------------------------------------------
    let r0_y = card_rect.top() + row_h * 0.5;
    painter.text(
        Pos2::new(left_x, r0_y - 10.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "line_width"),
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    let (_, active_col_px) = session.line_width.compute_column_bounds(win_rect.width());
    painter.text(
        Pos2::new(left_x, r0_y + 11.0),
        Align2::LEFT_CENTER,
        format!(
            "{} ({:.0}px)",
            I18n::tr(session.locale, "line_width_sub"),
            active_col_px
        ),
        font_ui_regular(11.5),
        pal.fg_muted,
    );

    let lw_pills = [
        (LineWidthMode::Compact, "Compact"),
        (LineWidthMode::Medium, "Medium"),
        (LineWidthMode::Wide, "Wide"),
        (LineWidthMode::Fluid { margin_pct: 6 }, "Fluid"),
    ];
    let mut px = right_x - 278.0;
    for (idx, (mode, title)) in lw_pills.into_iter().enumerate() {
        let active = matches!(
            (session.line_width, mode),
            (LineWidthMode::Compact, LineWidthMode::Compact)
                | (LineWidthMode::Medium, LineWidthMode::Medium)
                | (LineWidthMode::Wide, LineWidthMode::Wide)
                | (LineWidthMode::Fluid { .. }, LineWidthMode::Fluid { .. })
        );
        let pill_r = Rect::from_min_max(
            Pos2::new(px, r0_y - 15.0),
            Pos2::new(px + 66.0, r0_y + 15.0),
        );
        painter.rect_filled(
            pill_r,
            CornerRadius::same(7),
            if active {
                pal.accent_orange
            } else {
                pal.bg_pill_button
            },
        );
        painter.text(
            pill_r.center(),
            Align2::CENTER_CENTER,
            title,
            font_ui_bold(12.0),
            if active {
                Color32::WHITE
            } else {
                pal.fg_primary
            },
        );
        let hit = ui.interact(pill_r, ui.id().with(("pref_lw_pill", idx)), Sense::click());
        if hit.clicked() {
            actions.push(Action::SetLineWidth(mode));
        }
        px += 70.0;
    }

    // ------------------------------------------------------------------------
    // Row 1: Interactive Fluid Lateral Margin Slider (2% .. 25%)
    // ------------------------------------------------------------------------
    let r1_y = card_rect.top() + row_h * 1.5;
    let margin_pct = match session.line_width {
        LineWidthMode::Fluid { margin_pct } => margin_pct,
        _ => 6,
    };
    painter.text(
        Pos2::new(left_x, r1_y - 9.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "fluid_margin"),
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    painter.text(
        Pos2::new(left_x, r1_y + 11.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "fluid_margin_sub"),
        font_ui_regular(11.5),
        pal.fg_muted,
    );
    let track_l = right_x - 210.0;
    let track_r = right_x - 46.0;
    let slider_hit_rect = Rect::from_min_max(
        Pos2::new(track_l - 8.0, r1_y - 14.0),
        Pos2::new(track_r + 8.0, r1_y + 14.0),
    );
    let slider_resp = ui.interact(
        slider_hit_rect,
        ui.id().with("pref_fluid_slider"),
        Sense::click_and_drag(),
    );
    if (slider_resp.clicked() || slider_resp.dragged())
        && ui.input(|i| i.pointer.interact_pos()).is_some()
    {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let t = ((pos.x - track_l) / (track_r - track_l)).clamp(0.0, 1.0);
            let new_pct = (2.0 + t * 23.0).round() as u8;
            actions.push(Action::SetLineWidth(LineWidthMode::Fluid {
                margin_pct: new_pct.clamp(2, 25),
            }));
        }
    }
    painter.line_segment(
        [Pos2::new(track_l, r1_y), Pos2::new(track_r, r1_y)],
        Stroke::new(4.0, pal.bg_pill_button),
    );
    let norm = (f32::from(margin_pct.saturating_sub(2)) / 23.0).clamp(0.0, 1.0);
    let knob_x = track_l + norm * (track_r - track_l);
    painter.line_segment(
        [Pos2::new(track_l, r1_y), Pos2::new(knob_x, r1_y)],
        Stroke::new(4.0, pal.accent_orange),
    );
    painter.circle_filled(Pos2::new(knob_x, r1_y), 8.0, pal.accent_orange);
    painter.text(
        Pos2::new(right_x, r1_y),
        Align2::RIGHT_CENTER,
        format!("{margin_pct}%"),
        font_ui_bold(13.0),
        pal.fg_primary,
    );

    // ------------------------------------------------------------------------
    // Row 2: Prose Font Selector (powered by fontdb + interactive ComboBox)
    // ------------------------------------------------------------------------
    let r2_y = card_rect.top() + row_h * 2.5;
    painter.text(
        Pos2::new(left_x, r2_y - 9.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "prose_font"),
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    painter.text(
        Pos2::new(left_x, r2_y + 11.0),
        Align2::LEFT_CENTER,
        format!(
            "{} ({})",
            I18n::tr(session.locale, "prose_font_sub"),
            session.font_catalog.families.len()
        ),
        font_ui_regular(11.5),
        pal.fg_muted,
    );
    let prose_choices = ["Nimbus Mono PS", "Inter", "Literata"];
    let mut pfx = right_x - 332.0;
    for (idx, fam) in prose_choices.into_iter().enumerate() {
        let active = session.typography.prose_font_family == fam;
        let w_pill = if idx == 0 { 98.0 } else { 56.0 };
        let pill_r = Rect::from_min_max(
            Pos2::new(pfx, r2_y - 14.0),
            Pos2::new(pfx + w_pill, r2_y + 14.0),
        );
        painter.rect_filled(
            pill_r,
            CornerRadius::same(6),
            if active {
                pal.accent_orange
            } else {
                pal.bg_pill_button
            },
        );
        painter.text(
            pill_r.center(),
            Align2::CENTER_CENTER,
            fam,
            font_ui_bold(11.0),
            if active {
                Color32::WHITE
            } else {
                pal.fg_primary
            },
        );
        let hit = ui.interact(pill_r, ui.id().with(("pref_prose", idx)), Sense::click());
        if hit.clicked() {
            actions.push(Action::SetProseFont(fam.to_string()));
        }
        pfx += w_pill + 4.0;
    }
    let prose_combo_rect = Rect::from_min_max(
        Pos2::new(pfx + 2.0, r2_y - 14.0),
        Pos2::new(right_x, r2_y + 14.0),
    );
    let mut prose_combo_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(prose_combo_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let mut current_prose = session.typography.prose_font_family.clone();
    egui::ComboBox::from_id_salt("pref_prose_fontdb_combo")
        .width(prose_combo_rect.width() - 4.0)
        .selected_text(&current_prose)
        .show_ui(&mut prose_combo_ui, |ui| {
            for fam in &session.font_catalog.families {
                if ui
                    .selectable_value(&mut current_prose, fam.clone(), fam)
                    .clicked()
                {
                    actions.push(Action::SetProseFont(fam.clone()));
                }
            }
        });

    // ------------------------------------------------------------------------
    // Row 3: Code Font Selector (powered by fontdb + interactive ComboBox)
    // ------------------------------------------------------------------------
    let r3_y = card_rect.top() + row_h * 3.5;
    painter.text(
        Pos2::new(left_x, r3_y - 9.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "code_font"),
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    painter.text(
        Pos2::new(left_x, r3_y + 11.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "code_font_sub"),
        font_ui_regular(11.5),
        pal.fg_muted,
    );
    let code_choices = ["Nimbus Mono PS", "JetBrains Mono"];
    let mut cfx = right_x - 332.0;
    for (idx, fam) in code_choices.into_iter().enumerate() {
        let active = session.typography.code_font_family == fam;
        let w_pill = 102.0;
        let pill_r = Rect::from_min_max(
            Pos2::new(cfx, r3_y - 14.0),
            Pos2::new(cfx + w_pill, r3_y + 14.0),
        );
        painter.rect_filled(
            pill_r,
            CornerRadius::same(6),
            if active {
                pal.accent_orange
            } else {
                pal.bg_pill_button
            },
        );
        painter.text(
            pill_r.center(),
            Align2::CENTER_CENTER,
            fam,
            font_ui_bold(11.0),
            if active {
                Color32::WHITE
            } else {
                pal.fg_primary
            },
        );
        let hit = ui.interact(pill_r, ui.id().with(("pref_code", idx)), Sense::click());
        if hit.clicked() {
            actions.push(Action::SetCodeFont(fam.to_string()));
        }
        cfx += w_pill + 4.0;
    }
    let code_combo_rect = Rect::from_min_max(
        Pos2::new(cfx + 2.0, r3_y - 14.0),
        Pos2::new(right_x, r3_y + 14.0),
    );
    let mut code_combo_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(code_combo_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let mut current_code = session.typography.code_font_family.clone();
    egui::ComboBox::from_id_salt("pref_code_fontdb_combo")
        .width(code_combo_rect.width() - 4.0)
        .selected_text(&current_code)
        .show_ui(&mut code_combo_ui, |ui| {
            for fam in &session.font_catalog.families {
                if ui
                    .selectable_value(&mut current_code, fam.clone(), fam)
                    .clicked()
                {
                    actions.push(Action::SetCodeFont(fam.clone()));
                }
            }
        });

    // ------------------------------------------------------------------------
    // Row 4: Interactive Font Size & Line Height Steppers (`-` / `+`)
    // ------------------------------------------------------------------------
    let r4_y = card_rect.top() + row_h * 4.5;
    painter.text(
        Pos2::new(left_x, r4_y - 9.0),
        Align2::LEFT_CENTER,
        format!(
            "{} & {}",
            I18n::tr(session.locale, "font_size"),
            I18n::tr(session.locale, "line_height")
        ),
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    painter.text(
        Pos2::new(left_x, r4_y + 11.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "size_lh_sub"),
        font_ui_regular(11.5),
        pal.fg_muted,
    );

    // Font size stepper: [-] [17.4 pt] [+]
    let fs_minus = Rect::from_min_max(
        Pos2::new(right_x - 284.0, r4_y - 14.0),
        Pos2::new(right_x - 260.0, r4_y + 14.0),
    );
    let fs_val = Rect::from_min_max(
        Pos2::new(right_x - 257.0, r4_y - 14.0),
        Pos2::new(right_x - 183.0, r4_y + 14.0),
    );
    let fs_plus = Rect::from_min_max(
        Pos2::new(right_x - 180.0, r4_y - 14.0),
        Pos2::new(right_x - 156.0, r4_y + 14.0),
    );
    for (r_box, txt, id_key, delta) in [
        (fs_minus, "−", "fs_dec", -0.5_f32),
        (fs_plus, "+", "fs_inc", 0.5_f32),
    ] {
        painter.rect_filled(r_box, CornerRadius::same(6), pal.bg_pill_button);
        painter.text(
            r_box.center(),
            Align2::CENTER_CENTER,
            txt,
            font_ui_bold(14.0),
            pal.fg_primary,
        );
        if ui.interact(r_box, ui.id().with(id_key), Sense::click()).clicked() {
            actions.push(Action::SetFontSize(session.typography.font_size + delta));
        }
    }
    painter.rect_filled(fs_val, CornerRadius::same(6), pal.bg_pill_button);
    painter.text(
        fs_val.center(),
        Align2::CENTER_CENTER,
        format!("{:.1} pt", session.typography.font_size),
        font_ui_bold(12.0),
        pal.fg_primary,
    );

    // Line height stepper: [-] [1.50×] [+]
    let lh_minus = Rect::from_min_max(
        Pos2::new(right_x - 140.0, r4_y - 14.0),
        Pos2::new(right_x - 116.0, r4_y + 14.0),
    );
    let lh_val = Rect::from_min_max(
        Pos2::new(right_x - 113.0, r4_y - 14.0),
        Pos2::new(right_x - 27.0, r4_y + 14.0),
    );
    let lh_plus = Rect::from_min_max(
        Pos2::new(right_x - 24.0, r4_y - 14.0),
        Pos2::new(right_x, r4_y + 14.0),
    );
    for (r_box, txt, id_key, delta) in [
        (lh_minus, "−", "lh_dec", -0.05_f32),
        (lh_plus, "+", "lh_inc", 0.05_f32),
    ] {
        painter.rect_filled(r_box, CornerRadius::same(6), pal.bg_pill_button);
        painter.text(
            r_box.center(),
            Align2::CENTER_CENTER,
            txt,
            font_ui_bold(14.0),
            pal.fg_primary,
        );
        if ui.interact(r_box, ui.id().with(id_key), Sense::click()).clicked() {
            actions.push(Action::SetLineHeight(session.typography.line_height + delta));
        }
    }
    painter.rect_filled(lh_val, CornerRadius::same(6), pal.bg_pill_button);
    painter.text(
        lh_val.center(),
        Align2::CENTER_CENTER,
        format!("{:.2}×", session.typography.line_height),
        font_ui_bold(12.0),
        pal.fg_primary,
    );

    // ------------------------------------------------------------------------
    // Row 5: Multi-Dialect Markdown AST Switcher (CommonMark, GFM, GLFM)
    // ------------------------------------------------------------------------
    let r5_y = card_rect.top() + row_h * 5.5;
    painter.text(
        Pos2::new(left_x, r5_y - 9.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "input_format"),
        font_ui_regular(14.0),
        pal.fg_primary,
    );
    painter.text(
        Pos2::new(left_x, r5_y + 11.0),
        Align2::LEFT_CENTER,
        I18n::tr(session.locale, "input_format_sub"),
        font_ui_regular(11.5),
        pal.fg_muted,
    );

    let dialects = [
        (MarkdownDialect::CommonMark, "CommonMark"),
        (MarkdownDialect::Gfm, "GFM"),
        (MarkdownDialect::Glfm, "GLFM"),
    ];
    let mut dx = right_x - 224.0;
    for (idx, (d, label)) in dialects.into_iter().enumerate() {
        let active = session.dialect == d;
        let w_pill = if idx == 0 { 94.0 } else { 60.0 };
        let pill_r = Rect::from_min_max(
            Pos2::new(dx, r5_y - 14.0),
            Pos2::new(dx + w_pill, r5_y + 14.0),
        );
        painter.rect_filled(
            pill_r,
            CornerRadius::same(6),
            if active {
                pal.accent_orange
            } else {
                pal.bg_pill_button
            },
        );
        painter.text(
            pill_r.center(),
            Align2::CENTER_CENTER,
            label,
            font_ui_bold(12.0),
            if active {
                Color32::WHITE
            } else {
                pal.fg_primary
            },
        );
        let hit = ui.interact(pill_r, ui.id().with(("pref_dialect", idx)), Sense::click());
        if hit.clicked() {
            actions.push(Action::SetDialect(d));
        }
        dx += w_pill + 5.0;
    }
}
