pub mod theme;

use accent_core::{MarkdownDialect, PreviewBlock, PreviewInline};
use accent_session::{
    Action, ActiveOverlay, FormatCommand, I18n, LineWidthMode, Locale, PreviewLayout, Session,
    StatMetric, ThemeMode,
};
use egui::{
    epaint::StrokeKind, text::LayoutJob, Align2, Color32, Context, CornerRadius, CursorIcon, FontId,
    Id, Margin, Order, Painter, Pos2, Rect, ResizeDirection, ScrollArea, Sense, Stroke, TextEdit,
    TextFormat, Ui, Vec2, ViewportCommand, ViewportId,
};
use std::sync::Arc;
use theme::{
    apply_theme, draw_chevron_down, draw_chevron_left, draw_chevron_up, draw_hamburger_icon,
    draw_new_doc_icon, draw_preview_book_icon, draw_search_icon, draw_toolbar_icon,
    draw_window_button, font_editor_regular, font_ui_bold, font_ui_regular, resolve_code_font,
    resolve_prose_font, AccentPalette,
};

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
            actions.push(Action::SaveDocument);
        }
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S) {
            actions.push(Action::SetOverlay(ActiveOverlay::SaveFileDialog));
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
            actions.push(Action::NewDocument);
        }
        if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
            actions.push(Action::SetOverlay(ActiveOverlay::OpenFileDialog));
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
        && !session.search_bar_open;

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

            // 2. Render Top Headerbar (47px + 1px bottom border) unless auto-hidden while typing
            if !chrome_hidden {
                render_headerbar(ctx, ui, &painter, rect, session, &pal, actions);
            }

            // 3. Render Find/Replace Bar if open
            if session.search_bar_open {
                render_search_bar(ui, &painter, rect, session, &pal, actions);
            }

            // 4. Render Bottom Overlay Bar unless auto-hidden while typing
            if !chrome_hidden {
                render_bottom_bar(ui, &painter, rect, session, &pal, actions);
            }

            // 5. Outer 1px window frame border
            painter.rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Inside,
            );

            // 6. Invisible 6px CSD window resize borders along all 8 edges/corners
            handle_csd_window_resize(ctx, ui, rect);
        });

    // 7. Render Active Popover or Modal Dialog in Foreground Layer so clicks are never stolen by TextEdit,
    // while anchoring the Area at the exact popover top-left so headerbar buttons remain clickable!
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

    // 8. Separate OS Window Preview via Multi-Viewport API when PreviewLayout::Windowed is active
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
            Rect::from_min_max(Pos2::new(72.0, 44.0), Pos2::new(320.0, 274.0))
        }
        ActiveOverlay::PreviewMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 418.0, 44.0),
            Pos2::new(win_rect.right() - 44.0, 324.0),
        ),
        ActiveOverlay::MainMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 302.0, 44.0),
            Pos2::new(win_rect.right() - 6.0, 412.0),
        ),
        ActiveOverlay::LineWidthMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 390.0, 44.0),
            Pos2::new(win_rect.right() - 150.0, 232.0),
        ),
        ActiveOverlay::StatsMenu => Rect::from_min_max(
            Pos2::new(win_rect.right() - 260.0, win_rect.bottom() - 220.0),
            Pos2::new(win_rect.right() - 12.0, win_rect.bottom() - 48.0),
        ),
        ActiveOverlay::PreferencesDialog
        | ActiveOverlay::ExportDialog
        | ActiveOverlay::OpenFileDialog
        | ActiveOverlay::SaveFileDialog
        | ActiveOverlay::ShortcutsDialog
        | ActiveOverlay::AboutDialog => {
            Rect::from_min_max(Pos2::new(win_rect.left(), 48.0), win_rect.right_bottom())
        }
    }
}

fn handle_csd_window_resize(ctx: &Context, ui: &mut Ui, win_rect: Rect) {
    let b = 5.0_f32;
    let l = win_rect.left();
    let r = win_rect.right();
    let t = win_rect.top();
    let bot = win_rect.bottom();

    let zones = [
        (
            Rect::from_min_max(Pos2::new(l, t), Pos2::new(l + b * 2.0, t + b * 2.0)),
            ResizeDirection::NorthWest,
            CursorIcon::ResizeNorthWest,
        ),
        (
            Rect::from_min_max(Pos2::new(r - b * 2.0, t), Pos2::new(r, t + b * 2.0)),
            ResizeDirection::NorthEast,
            CursorIcon::ResizeNorthEast,
        ),
        (
            Rect::from_min_max(Pos2::new(l, bot - b * 2.0), Pos2::new(l + b * 2.0, bot)),
            ResizeDirection::SouthWest,
            CursorIcon::ResizeSouthWest,
        ),
        (
            Rect::from_min_max(Pos2::new(r - b * 2.0, bot - b * 2.0), Pos2::new(r, bot)),
            ResizeDirection::SouthEast,
            CursorIcon::ResizeSouthEast,
        ),
        (
            Rect::from_min_max(Pos2::new(l + b * 2.0, t), Pos2::new(r - b * 2.0, t + b)),
            ResizeDirection::North,
            CursorIcon::ResizeNorth,
        ),
        (
            Rect::from_min_max(
                Pos2::new(l + b * 2.0, bot - b),
                Pos2::new(r - b * 2.0, bot),
            ),
            ResizeDirection::South,
            CursorIcon::ResizeSouth,
        ),
        (
            Rect::from_min_max(Pos2::new(l, t + b * 2.0), Pos2::new(l + b, bot - b * 2.0)),
            ResizeDirection::West,
            CursorIcon::ResizeWest,
        ),
        (
            Rect::from_min_max(Pos2::new(r - b, t + b * 2.0), Pos2::new(r, bot - b * 2.0)),
            ResizeDirection::East,
            CursorIcon::ResizeEast,
        ),
    ];

    for (idx, (z_rect, dir, icon)) in zones.into_iter().enumerate() {
        let resp = ui.interact(z_rect, ui.id().with(("csd_resize", idx)), Sense::drag());
        if resp.hovered() {
            ctx.set_cursor_icon(icon);
        }
        if resp.drag_started() {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}

fn render_headerbar(
    ctx: &Context,
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    let hb_height = 47.0;
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

    let y_mid = win_rect.top() + 24.0;
    let fg = pal.fg_primary;
    let w = win_rect.right();

    // Define interactive button hit-rects first so background drag doesn't conflict with buttons
    let new_doc_rect = Rect::from_center_size(Pos2::new(28.0, y_mid), Vec2::new(32.0, 32.0));
    let open_rect = Rect::from_min_max(Pos2::new(54.0, 7.0), Pos2::new(116.0, 41.0));
    let save_btn_rect = Rect::from_min_max(Pos2::new(128.0, 7.0), Pos2::new(185.0, 41.0));
    let save_menu_rect = Rect::from_min_max(Pos2::new(185.0, 7.0), Pos2::new(214.0, 41.0));

    // When split preview is open, the preview button widens to show both the book icon and layout dropdown
    let preview_pill_rect = if session.preview_active {
        Rect::from_min_max(Pos2::new(w - 224.0, 7.0), Pos2::new(w - 158.0, 41.0))
    } else {
        Rect::from_center_size(Pos2::new(w - 182.0, y_mid), Vec2::new(34.0, 34.0))
    };
    let search_x = if session.preview_active {
        w - 246.0
    } else {
        w - 222.0
    };
    let search_rect = Rect::from_center_size(Pos2::new(search_x, y_mid), Vec2::new(32.0, 32.0));
    let lw_x = search_x - 48.0;
    let lw_rect = Rect::from_center_size(Pos2::new(lw_x, y_mid), Vec2::new(52.0, 28.0));

    let menu_center = Pos2::new(w - 142.0, y_mid);
    let menu_rect = Rect::from_center_size(menu_center, Vec2::new(32.0, 32.0));
    let min_center = Pos2::new(w - 102.0, y_mid);
    let min_rect = Rect::from_center_size(min_center, Vec2::new(26.0, 26.0));
    let max_center = Pos2::new(w - 65.0, y_mid);
    let max_rect = Rect::from_center_size(max_center, Vec2::new(26.0, 26.0));
    let close_center = Pos2::new(w - 28.0, y_mid);
    let close_rect = Rect::from_center_size(close_center, Vec2::new(26.0, 26.0));

    // Left: New doc icon (x=28)
    let new_hit = ui
        .interact(new_doc_rect, ui.id().with("new_doc_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_new_doc"));
    if new_hit.hovered() {
        painter.rect_filled(new_doc_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    draw_new_doc_icon(painter, Pos2::new(28.0, y_mid), fg);
    if new_hit.clicked() {
        actions.push(Action::NewDocument);
    }

    // Left: "Apri v" (x=62..114)
    let open_hit = ui
        .interact(open_rect, ui.id().with("open_menu_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_open"));
    if session.overlay == ActiveOverlay::OpenMenu || open_hit.hovered() {
        painter.rect_filled(open_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    let open_label = I18n::tr(session.locale, "open");
    painter.text(
        Pos2::new(62.0, y_mid),
        Align2::LEFT_CENTER,
        open_label,
        font_ui_bold(14.5),
        fg,
    );
    draw_chevron_down(painter, Pos2::new(103.0, y_mid + 1.0), fg);
    if open_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::OpenMenu));
    }

    // Left: "Salva | v" (x=136..210)
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
    if session.overlay == ActiveOverlay::SaveMenu
        || save_hit.hovered()
        || save_menu_hit.hovered()
    {
        let active_r = Rect::from_min_max(Pos2::new(128.0, 7.0), Pos2::new(212.0, 41.0));
        painter.rect_filled(active_r, CornerRadius::same(6), pal.bg_pill_button);
    }
    let save_label = I18n::tr(session.locale, "save");
    painter.text(
        Pos2::new(136.0, y_mid),
        Align2::LEFT_CENTER,
        save_label,
        font_ui_bold(14.5),
        fg,
    );
    painter.line_segment(
        [Pos2::new(185.5, 13.0), Pos2::new(185.5, 35.0)],
        Stroke::new(1.0, pal.border_headerbar),
    );
    draw_chevron_down(painter, Pos2::new(198.0, y_mid + 1.0), fg);
    if save_hit.clicked() {
        actions.push(Action::SaveDocument);
    }
    if save_menu_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::SaveMenu));
    }

    // Center: Document title (and CSD drag / double-click maximize region across unused headerbar space)
    let drag_rect = Rect::from_min_max(Pos2::new(216.0, 4.0), Pos2::new(lw_rect.left() - 6.0, 44.0));
    let drag_resp = ui.interact(
        drag_rect,
        ui.id().with("hb_csd_drag_zone"),
        Sense::click_and_drag(),
    );
    let drag_state_id = Id::new("accent_csd_drag_state");
    if drag_resp.drag_started() {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        let anchor = ctx.input(|i| i.pointer.press_origin().or_else(|| i.pointer.interact_pos()));
        let cur_outer = ctx
            .input(|i| i.viewport().outer_rect.map(|r| r.min))
            .unwrap_or(Pos2::ZERO);
        ctx.data_mut(|d| d.insert_temp(drag_state_id, (cur_outer, anchor)));
    }
    if drag_resp.drag_started() || drag_resp.dragged() {
        let cur_local = ctx.input(|i| i.pointer.interact_pos());
        let state: Option<(Pos2, Option<Pos2>)> = ctx.data_mut(|d| d.get_temp(drag_state_id));
        if let (Some((mut win_pos, Some(anchor))), Some(cur)) = (state, cur_local) {
            let delta = cur - anchor;
            if delta.length_sq() > 0.5 {
                win_pos += delta;
                ctx.data_mut(|d| d.insert_temp(drag_state_id, (win_pos, Some(anchor))));
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(win_pos));
            }
        }
    }
    if drag_resp.double_clicked() {
        let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
    }

    let title_display = if session.is_dirty {
        format!("• {}", session.document_title)
    } else {
        session.document_title.clone()
    };
    painter.text(
        Pos2::new(win_rect.center().x, y_mid),
        Align2::CENTER_CENTER,
        &title_display,
        font_ui_bold(14.5),
        fg,
    );

    // Always-visible Line-Width selector pill (`80ch`, `110ch`, `140ch`, `Fluid`)
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

    // Search icon button
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

    // Preview / Mode button (Book icon + layout selector dropdown when active)
    let prev_hit = ui
        .interact(
            preview_pill_rect,
            ui.id().with("preview_btn"),
            Sense::click(),
        )
        .on_hover_text(I18n::tr(session.locale, "tooltip_preview"));
    if session.preview_active
        || session.view_while_typing
        || session.overlay == ActiveOverlay::PreviewMenu
        || prev_hit.hovered()
    {
        painter.rect_filled(preview_pill_rect, CornerRadius::same(6), pal.bg_pill_button);
    }
    if session.preview_active {
        draw_preview_book_icon(painter, Pos2::new(preview_pill_rect.left() + 16.0, y_mid), fg);
        let icon_r = Rect::from_center_size(
            Pos2::new(preview_pill_rect.left() + 40.0, y_mid),
            Vec2::new(13.0, 11.0),
        );
        painter.rect_stroke(
            icon_r,
            CornerRadius::same(2),
            Stroke::new(1.4, fg),
            StrokeKind::Middle,
        );
        painter.line_segment(
            [
                Pos2::new(icon_r.center().x, icon_r.top()),
                Pos2::new(icon_r.center().x, icon_r.bottom()),
            ],
            Stroke::new(1.4, fg),
        );
        draw_chevron_down(
            painter,
            Pos2::new(preview_pill_rect.right() - 10.0, y_mid + 1.0),
            fg,
        );
    } else {
        draw_preview_book_icon(painter, preview_pill_rect.center(), fg);
    }
    if prev_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::PreviewMenu));
    }

    // Hamburger Main Menu button
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

    // Real CSD Window Controls: Minimize ('-'), Maximize ('□'), Close ('×')
    let min_hit = ui
        .interact(min_rect, ui.id().with("win_min_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_minimize"));
    draw_window_button(painter, min_center, pal.bg_pill_button, fg, '-');
    if min_hit.clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
    }

    let max_hit = ui
        .interact(max_rect, ui.id().with("win_max_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_maximize"));
    draw_window_button(painter, max_center, pal.bg_pill_button, fg, '□');
    if max_hit.clicked() {
        let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
    }

    let close_hit = ui
        .interact(close_rect, ui.id().with("win_close_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_close"));
    draw_window_button(painter, close_center, pal.bg_pill_button, fg, 'x');
    if close_hit.clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn render_search_bar(
    ui: &mut Ui,
    painter: &Painter,
    win_rect: Rect,
    session: &Session,
    pal: &AccentPalette,
    actions: &mut Vec<Action>,
) {
    let bar_rect = Rect::from_min_max(
        Pos2::new(win_rect.left(), 48.0),
        Pos2::new(win_rect.right(), 88.0),
    );
    painter.rect_filled(bar_rect, CornerRadius::ZERO, pal.bg_headerbar);
    painter.line_segment(
        [
            Pos2::new(bar_rect.left(), bar_rect.bottom()),
            Pos2::new(bar_rect.right(), bar_rect.bottom()),
        ],
        Stroke::new(1.0, pal.border_headerbar),
    );

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bar_rect.shrink2(Vec2::new(24.0, 6.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let mut q = session.search_query.clone();
    let q_resp = child.add(
        TextEdit::singleline(&mut q)
            .hint_text(I18n::tr(session.locale, "find_placeholder"))
            .desired_width(210.0),
    );
    if q_resp.changed() {
        actions.push(Action::SetSearchQuery(q));
    }
    child.add_space(4.0);
    if child.button("▲").clicked() {
        actions.push(Action::FindPrev);
    }
    if child.button("▼").clicked() {
        actions.push(Action::FindNext);
    }
    if !session.search_query.is_empty() {
        let count = session.buffer.text().matches(&session.search_query).count();
        child.label(
            egui::RichText::new(format!("({count})"))
                .font(font_ui_regular(12.0))
                .color(pal.fg_muted),
        );
    }
    child.add_space(10.0);
    let mut r = session.replace_text.clone();
    let r_resp = child.add(
        TextEdit::singleline(&mut r)
            .hint_text(I18n::tr(session.locale, "replace_placeholder"))
            .desired_width(190.0),
    );
    if r_resp.changed() {
        actions.push(Action::SetReplaceText(r));
    }
    child.add_space(8.0);
    if child
        .button(I18n::tr(session.locale, "replace_btn"))
        .clicked()
    {
        actions.push(Action::ReplaceNext);
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
        && !session.search_bar_open;
    let top_y = if chrome_hidden {
        8.0
    } else if session.search_bar_open {
        88.0
    } else {
        48.0
    };
    let bot_y = if chrome_hidden {
        win_rect.bottom() - 8.0
    } else {
        win_rect.bottom() - 44.0
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
    painter: &Painter,
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
    let prose_reg_font = resolve_prose_font(
        &session.typography.prose_font_family,
        font_size,
        false,
        false,
    );
    let char_w = painter
        .layout_no_wrap("MMMMMMMMMM".to_string(), prose_reg_font, Color32::WHITE)
        .rect
        .width()
        / 10.0;

    // Reserve horizontal space to the left of `x_body` inside the TextEdit so outdented `#` hashes
    // (`#### `, `### `, `# `) are laid out natively inside the editable `TextEdit` widget!
    let max_outdent = (5.0 * char_w).min(x_body_rel - 8.0).max(0.0);
    let te_left = x_body - max_outdent;
    let base_indent = x_body - te_left;

    // Genuine interactive vertical `egui::ScrollArea` spanning the full editor pane!
    let mut pane_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(pane_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    let mut editable_text = session.buffer.text();
    let dialect = session.dialect;
    let view_while_typing = session.view_while_typing;
    let focus_mode = session.focus_mode;
    let prose_family = session.typography.prose_font_family.clone();
    let code_family = session.typography.code_font_family.clone();
    let selection_bytes = session.buffer.selection_byte_range();
    let active_cursor_line = if focus_mode {
        session.buffer.cursor_line()
    } else {
        0
    };
    let pal_copy = *pal;

    let mut layouter = move |ui: &Ui, text_str: &str, wrap_width: f32| -> Arc<egui::Galley> {
        let temp_buf;
        let buf_ref = if text_str == session.buffer.text() {
            &session.buffer
        } else {
            temp_buf = accent_core::DocumentBuffer::from_str_content(text_str);
            &temp_buf
        };

        let segments =
            buf_ref.build_editor_segments(dialect, view_while_typing, selection_bytes.clone());
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width.max(col_width + base_indent + 16.0);

        for seg in &segments {
            let leading_space = if seg.is_line_start {
                if seg.is_heading_hashes {
                    if seg.collapsed_delimiter {
                        base_indent
                    } else {
                        let lvl = seg.heading_level.unwrap_or(1);
                        (base_indent - (f32::from(lvl) + 1.0) * char_w).max(0.0)
                    }
                } else if seg.blockquote {
                    base_indent + 2.0 * char_w
                } else if seg.list_item && seg.collapsed_delimiter {
                    base_indent + 24.0
                } else if seg.code_block {
                    base_indent + 5.0
                } else {
                    base_indent
                }
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
                job.append(
                    &seg.text,
                    leading_space,
                    TextFormat {
                        font_id: FontId::new(0.01, egui::FontFamily::Monospace),
                        color: Color32::TRANSPARENT,
                        line_height: Some(seg_line_step),
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

        ui.fonts(|f| f.layout_job(job))
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

                // Populate block/inline visual overlays on `scroll_painter`
                let live_spans = session.buffer.compute_live_spans(
                    session.dialect,
                    session.view_while_typing,
                    session.buffer.selection_byte_range(),
                );
                let mut bg_shapes = Vec::new();
                for ls in &live_spans {
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
                            let sy = output.galley_pos.y + r_top.min.y - 4.0;
                            let ey = output.galley_pos.y + r_bot.max.y + 4.0;
                            if ey > sy {
                                bg_shapes.push(egui::Shape::rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(x_body, sy),
                                        Pos2::new(x_right, ey),
                                    ),
                                    CornerRadius::same(4),
                                    pal.bg_code_block,
                                ));
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
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let r_bot = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.end.saturating_sub(1),
                            ));
                            let sy = output.galley_pos.y + r_top.min.y + 2.0;
                            let ey = output.galley_pos.y + r_bot.max.y - 2.0;
                            if ey > sy {
                                scroll_painter.rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(x_body + 4.0, sy),
                                        Pos2::new(x_body + 8.0, ey),
                                    ),
                                    CornerRadius::same(2),
                                    pal.border_headerbar,
                                );
                            }
                        }
                        accent_core::AstSpanKind::ListItem { task } if is_collapsed => {
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let cy = output.galley_pos.y + r_top.min.y + 10.5;
                            if let Some(checked) = task {
                                let cb_rect = Rect::from_center_size(
                                    Pos2::new(x_body + 10.0, cy),
                                    Vec2::new(14.0, 14.0),
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
                                            Pos2::new(cb_rect.right() - 2.8, cb_rect.top() + 3.2),
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
                                    Pos2::new(x_body + 10.0, cy),
                                    3.2,
                                    pal.fg_primary,
                                );
                            }
                        }
                        accent_core::AstSpanKind::ThematicBreak if is_collapsed => {
                            let r_top = output.galley.pos_from_ccursor(egui::text::CCursor::new(
                                ls.span.char_range.start,
                            ));
                            let cy = output.galley_pos.y + r_top.min.y + 10.5;
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
                            let sy = output.galley_pos.y + r_top.min.y + 2.0;
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

                if output.response.changed() && editable_text != session.buffer.text() {
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
                    output.state.store(ui.ctx(), output.response.id);
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
                } else if let Some(c_range) = output.state.cursor.char_range() {
                    let p = c_range.primary.index;
                    let s = c_range.secondary.index;
                    let cur_sel = session.buffer.selection_char_range();
                    let cursor_moved = p != session.buffer.cursor();
                    if cursor_moved
                        || (p.min(s)..p.max(s)) != cur_sel
                        || output.response.changed()
                    {
                        actions.push(Action::SetSelection(p, s));
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

                // Ensure a crisp caret line is always visible in headless screenshot mode when unfocused
                if !output.response.has_focus() && !session.preview_active {
                    let cur_rect = output
                        .galley
                        .pos_from_ccursor(egui::text::CCursor::new(session.buffer.cursor()));
                    let cx = (output.galley_pos.x + cur_rect.min.x).max(x_body);
                    let cy = output.galley_pos.y + cur_rect.min.y;
                    if cy >= pane_rect.top() && cy < pane_rect.bottom() {
                        scroll_painter.line_segment(
                            [Pos2::new(cx, cy + 2.0), Pos2::new(cx, cy + 20.0)],
                            Stroke::new(1.3, pal.fg_primary),
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

    let blocks = session.buffer.parse_preview_blocks(session.dialect);
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
            for block in &blocks {
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
                    PreviewBlock::CodeBlock { code, .. } => {
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
                    PreviewBlock::Table { headers, rows } => {
                        ui.add_space(4.0);
                        let cols = headers.len().max(1);
                        let col_w = 96.0_f32;
                        let row_h = 30.0_f32;
                        let total_rows = 1 + rows.len();
                        let table_w = (cols as f32) * col_w + 24.0;
                        let table_h = (total_rows as f32) * row_h;
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
                            if r_idx % 2 == 1 {
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
                                let cw = if c_idx + 1 == cols {
                                    t_rect.right() - cx
                                } else {
                                    88.0
                                };
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
                                tp.text(
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
    let bot_h = 44.0;
    let bot_top = win_rect.bottom() - bot_h;

    // Soft top fade over the bottom bar matching Apostrophe's bottom bar overlay
    for i in 0..18 {
        let t = (i as f32) / 18.0;
        let y = bot_top - 18.0 + (i as f32);
        let c = Color32::from_rgba_unmultiplied(
            pal.bg_canvas.r(),
            pal.bg_canvas.g(),
            pal.bg_canvas.b(),
            (t * t * 235.0) as u8,
        );
        painter.line_segment(
            [Pos2::new(win_rect.left(), y), Pos2::new(win_rect.right(), y)],
            Stroke::new(1.2, c),
        );
    }

    let bar_rect = Rect::from_min_max(
        Pos2::new(win_rect.left(), bot_top),
        win_rect.right_bottom(),
    );
    painter.rect_filled(bar_rect, CornerRadius::ZERO, pal.bg_canvas);

    let y_mid = win_rect.bottom() - 24.0;
    let fg = pal.fg_primary;

    if session.toolbar_expanded {
        let icon_xs = [
            (28.0_f32, FormatCommand::Bold, "tooltip_bold"),
            (68.0, FormatCommand::Italic, "tooltip_italic"),
            (108.0, FormatCommand::Strikethrough, "tooltip_strike"),
            (148.0, FormatCommand::Heading, "tooltip_heading"),
            (189.0, FormatCommand::BulletList, "tooltip_bullet"),
            (229.0, FormatCommand::OrderedList, "tooltip_ordered"),
            (269.0, FormatCommand::TaskList, "tooltip_task"),
            (310.0, FormatCommand::BlockQuote, "tooltip_quote"),
            (349.0, FormatCommand::Code, "tooltip_code"),
            (389.0, FormatCommand::Link, "tooltip_link"),
            (429.0, FormatCommand::Image, "tooltip_image"),
            (469.0, FormatCommand::Table, "tooltip_table"),
        ];
        for (idx, &(ix, cmd, tip_key)) in icon_xs.iter().enumerate() {
            let btn_r = Rect::from_center_size(Pos2::new(ix, y_mid), Vec2::new(32.0, 32.0));
            let hit = ui
                .interact(btn_r, ui.id().with(("fmt_toolbar_btn", idx)), Sense::click())
                .on_hover_text(I18n::tr(session.locale, tip_key));
            if hit.hovered() {
                painter.rect_filled(btn_r, CornerRadius::same(6), pal.bg_pill_button);
            }
            draw_toolbar_icon(painter, Pos2::new(ix, y_mid), fg, idx);
            if hit.clicked() {
                actions.push(Action::ApplyFormat(cmd));
            }
        }
    }

    // Collapse '<' / Expand '>' pill button at x=508 (or x=28 if collapsed)
    let pill_x = if session.toolbar_expanded { 508.0 } else { 28.0 };
    let pill_rect = Rect::from_center_size(Pos2::new(pill_x, y_mid), Vec2::new(34.0, 34.0));
    painter.rect_filled(pill_rect, CornerRadius::same(8), pal.bg_pill_button);
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

    if let Some(msg) = &session.status_message {
        painter.text(
            Pos2::new(win_rect.center().x + 40.0, y_mid),
            Align2::CENTER_CENTER,
            msg,
            font_ui_regular(12.5),
            pal.fg_muted,
        );
    }

    // Bottom-right word counter
    let stat_text = session.format_active_stat();
    let stat_btn_rect = Rect::from_min_max(
        Pos2::new(win_rect.right() - 160.0, win_rect.bottom() - 42.0),
        Pos2::new(win_rect.right() - 12.0, win_rect.bottom() - 6.0),
    );
    let stat_hit = ui
        .interact(stat_btn_rect, ui.id().with("stats_menu_btn"), Sense::click())
        .on_hover_text(I18n::tr(session.locale, "tooltip_stats"));
    if session.overlay == ActiveOverlay::StatsMenu || stat_hit.hovered() {
        painter.rect_filled(stat_btn_rect, CornerRadius::same(8), pal.bg_pill_button);
    }
    painter.text(
        Pos2::new(win_rect.right() - 38.0, y_mid),
        Align2::RIGHT_CENTER,
        &stat_text,
        font_ui_bold(14.0),
        fg,
    );
    draw_chevron_up(painter, Pos2::new(win_rect.right() - 25.0, y_mid), fg);

    if stat_hit.clicked() {
        actions.push(Action::SetOverlay(ActiveOverlay::StatsMenu));
    }
}

fn render_popovers(
    ctx: &Context,
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
            let pop_rect = Rect::from_min_max(Pos2::new(18.0, 44.0), Pos2::new(316.0, 44.0 + h));
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
                actions.push(Action::SetOverlay(ActiveOverlay::OpenFileDialog));
            }
        }
        ActiveOverlay::PreviewMenu => {
            // Unambiguous, mutually-exclusive Editing & Viewing Mode Menu:
            // Exactly ONE row out of the 6 modes has a checkmark `✓` at any time,
            // and every row shows its keyboard shortcut right-aligned!
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 418.0, 44.0),
                Pos2::new(win_rect.right() - 44.0, 324.0),
            );
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );

            // Section 1 Header: "Modalità editor"
            painter.text(
                Pos2::new(pop_rect.left() + 18.0, pop_rect.top() + 20.0),
                Align2::LEFT_CENTER,
                I18n::tr(session.locale, "editor_modes_hdr"),
                font_ui_bold(12.0),
                pal.fg_muted,
            );

            let editor_modes = [
                (
                    I18n::tr(session.locale, "editor_only_raw"),
                    "Ctrl+Shift+E",
                    !session.preview_active && !session.view_while_typing,
                    Action::SetEditorOnlyRaw,
                ),
                (
                    I18n::tr(session.locale, "view_while_typing"),
                    "Ctrl+L",
                    !session.preview_active && session.view_while_typing,
                    Action::SetEditorLivePreview,
                ),
            ];

            for (idx, (label, shortcut, is_selected, act)) in editor_modes.into_iter().enumerate() {
                let ry = pop_rect.top() + 46.0 + (idx as f32) * 34.0;
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, ry - 15.0),
                    Pos2::new(pop_rect.right() - 8.0, ry + 15.0),
                );
                let hit = ui.interact(
                    row_r,
                    ui.id().with(("editor_mode_row", idx)),
                    Sense::click(),
                );
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }

                // Icon at left
                let ic = Pos2::new(pop_rect.left() + 26.0, ry);
                let ir = Rect::from_center_size(ic, Vec2::new(13.0, 11.0));
                painter.rect_stroke(
                    ir,
                    CornerRadius::same(2),
                    Stroke::new(1.4, pal.fg_primary),
                    StrokeKind::Middle,
                );
                if idx == 1 {
                    painter.circle_filled(ic, 2.2, pal.accent_orange);
                }

                painter.text(
                    Pos2::new(pop_rect.left() + 44.0, ry),
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
                    Pos2::new(pop_rect.right() - 36.0, ry),
                    Align2::RIGHT_CENTER,
                    shortcut,
                    font_ui_regular(12.5),
                    pal.fg_muted,
                );

                if is_selected {
                    let ck = Pos2::new(pop_rect.right() - 20.0, ry);
                    painter.line_segment(
                        [Pos2::new(ck.x - 4.5, ck.y), Pos2::new(ck.x - 1.0, ck.y + 3.5)],
                        Stroke::new(1.8, pal.accent_orange),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(ck.x - 1.0, ck.y + 3.5),
                            Pos2::new(ck.x + 5.0, ck.y - 3.5),
                        ],
                        Stroke::new(1.8, pal.accent_orange),
                    );
                }

                if hit.clicked() {
                    actions.push(act);
                    actions.push(Action::SetOverlay(ActiveOverlay::None));
                }
            }

            // Divider between Editor Modes and Preview Panel layouts
            let sep_y = pop_rect.top() + 104.0;
            painter.line_segment(
                [
                    Pos2::new(pop_rect.left() + 12.0, sep_y),
                    Pos2::new(pop_rect.right() - 12.0, sep_y),
                ],
                Stroke::new(1.0, pal.border_headerbar),
            );

            // Section 2 Header: "Pannello di anteprima"
            painter.text(
                Pos2::new(pop_rect.left() + 18.0, pop_rect.top() + 124.0),
                Align2::LEFT_CENTER,
                I18n::tr(session.locale, "preview_panel_hdr"),
                font_ui_bold(12.0),
                pal.fg_muted,
            );

            let layouts = [
                (
                    PreviewLayout::HalfWidth,
                    I18n::tr(session.locale, "half_width"),
                    "Ctrl+Shift+P",
                ),
                (
                    PreviewLayout::FullWidth,
                    I18n::tr(session.locale, "full_width"),
                    "Ctrl+Alt+P",
                ),
                (
                    PreviewLayout::HalfHeight,
                    I18n::tr(session.locale, "half_height"),
                    "Ctrl+Shift+J",
                ),
                (
                    PreviewLayout::Windowed,
                    I18n::tr(session.locale, "windowed"),
                    "Ctrl+Shift+W",
                ),
            ];

            for (idx, (layout, label, shortcut)) in layouts.into_iter().enumerate() {
                let ry = pop_rect.top() + 154.0 + (idx as f32) * 34.0;
                let row_r = Rect::from_min_max(
                    Pos2::new(pop_rect.left() + 8.0, ry - 15.0),
                    Pos2::new(pop_rect.right() - 8.0, ry + 15.0),
                );
                let hit = ui.interact(row_r, ui.id().with(("prev_pop_row", idx)), Sense::click());
                if hit.hovered() {
                    painter.rect_filled(row_r, CornerRadius::same(6), pal.bg_pill_button);
                }

                // Draw layout icon at left
                let ic = Pos2::new(pop_rect.left() + 26.0, ry);
                let ir = Rect::from_center_size(ic, Vec2::new(13.0, 11.0));
                painter.rect_stroke(
                    ir,
                    CornerRadius::same(2),
                    Stroke::new(1.4, pal.fg_primary),
                    StrokeKind::Middle,
                );
                match layout {
                    PreviewLayout::HalfWidth => {
                        painter.line_segment(
                            [Pos2::new(ic.x, ir.top()), Pos2::new(ic.x, ir.bottom())],
                            Stroke::new(1.4, pal.fg_primary),
                        );
                    }
                    PreviewLayout::HalfHeight => {
                        painter.line_segment(
                            [Pos2::new(ir.left(), ic.y), Pos2::new(ir.right(), ic.y)],
                            Stroke::new(1.4, pal.fg_primary),
                        );
                    }
                    PreviewLayout::FullWidth => {
                        painter.rect_filled(
                            ir.shrink(2.0),
                            CornerRadius::same(1),
                            pal.fg_primary.gamma_multiply(0.35),
                        );
                    }
                    PreviewLayout::Windowed => {
                        painter.rect_stroke(
                            ir.translate(Vec2::new(2.5, -2.0)),
                            CornerRadius::same(1),
                            Stroke::new(1.2, pal.fg_primary),
                            StrokeKind::Middle,
                        );
                    }
                }

                let is_selected = session.preview_active && session.preview_layout == layout;
                painter.text(
                    Pos2::new(pop_rect.left() + 44.0, ry),
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
                    Pos2::new(pop_rect.right() - 36.0, ry),
                    Align2::RIGHT_CENTER,
                    shortcut,
                    font_ui_regular(12.5),
                    pal.fg_muted,
                );

                if is_selected {
                    let ck = Pos2::new(pop_rect.right() - 20.0, ry);
                    painter.line_segment(
                        [Pos2::new(ck.x - 4.5, ck.y), Pos2::new(ck.x - 1.0, ck.y + 3.5)],
                        Stroke::new(1.8, pal.accent_orange),
                    );
                    painter.line_segment(
                        [
                            Pos2::new(ck.x - 1.0, ck.y + 3.5),
                            Pos2::new(ck.x + 5.0, ck.y - 3.5),
                        ],
                        Stroke::new(1.8, pal.accent_orange),
                    );
                }

                if hit.clicked() {
                    actions.push(Action::SetPreviewMode {
                        active: true,
                        layout,
                    });
                    actions.push(Action::SetOverlay(ActiveOverlay::None));
                }
            }
        }
        ActiveOverlay::MainMenu => {
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 302.0, 44.0),
                Pos2::new(win_rect.right() - 6.0, 412.0),
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
                (I18n::tr(session.locale, "open_tutorial"), "", 260.0, false),
                (
                    I18n::tr(session.locale, "keyboard_shortcuts"),
                    "Ctrl+?",
                    292.0,
                    false,
                ),
                (I18n::tr(session.locale, "about"), "", 324.0, false),
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
                        4 => actions.push(Action::OpenTutorial),
                        5 => actions.push(Action::SetOverlay(ActiveOverlay::ShortcutsDialog)),
                        6 => actions.push(Action::SetOverlay(ActiveOverlay::AboutDialog)),
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
                Pos2::new(win_rect.right() - 390.0, 44.0),
                Pos2::new(win_rect.right() - 150.0, 232.0),
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
            let pop_rect = Rect::from_min_max(Pos2::new(72.0, 44.0), Pos2::new(320.0, 274.0));
            painter.rect_filled(pop_rect, CornerRadius::same(12), pal.bg_popover);
            painter.rect_stroke(
                pop_rect,
                CornerRadius::same(12),
                Stroke::new(1.0, pal.border_headerbar),
                StrokeKind::Middle,
            );
            let rows = [
                (I18n::tr(session.locale, "save_as"), 32.0),
                ("PDF", 72.0),
                ("HTML", 104.0),
                ("ODT", 136.0),
                (I18n::tr(session.locale, "advanced_export"), 176.0),
                (I18n::tr(session.locale, "copy_html"), 212.0),
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
                    font_ui_regular(14.0),
                    pal.fg_primary,
                );
                if hit.clicked() {
                    match idx {
                        0 => actions.push(Action::SetOverlay(ActiveOverlay::SaveFileDialog)),
                        1 => actions.push(Action::ExportDocument("pdf".to_string())),
                        2 => actions.push(Action::ExportDocument("html".to_string())),
                        3 => actions.push(Action::ExportDocument("odt".to_string())),
                        4 => actions.push(Action::SetOverlay(ActiveOverlay::ExportDialog)),
                        5 => {
                            let html = session.buffer.render_html(session.dialect);
                            ctx.copy_text(html);
                            actions.push(Action::SetOverlay(ActiveOverlay::None));
                        }
                        _ => actions.push(Action::SetOverlay(ActiveOverlay::None)),
                    }
                }
            }
        }
        ActiveOverlay::StatsMenu => {
            let pop_rect = Rect::from_min_max(
                Pos2::new(win_rect.right() - 260.0, win_rect.bottom() - 220.0),
                Pos2::new(win_rect.right() - 12.0, win_rect.bottom() - 48.0),
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
        Color32::from_rgba_unmultiplied(0, 0, 0, 48),
    );
    let dlg_rect = Rect::from_center_size(win_rect.center(), Vec2::new(480.0, 290.0));
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
        I18n::tr(session.locale, "advanced_export"),
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

    for (idx, (fmt_code, desc)) in [
        ("html", "HTML5 Standalone (.html)"),
        ("pdf", "Documento PDF (.pdf)"),
        ("docx", "Microsoft Word (.docx)"),
        ("epub", "E-Book EPUB (.epub)"),
    ]
    .into_iter()
    .enumerate()
    {
        let ry = dlg_rect.top() + 78.0 + (idx as f32) * 46.0;
        let row_r = Rect::from_min_max(
            Pos2::new(dlg_rect.left() + 24.0, ry - 18.0),
            Pos2::new(dlg_rect.right() - 24.0, ry + 18.0),
        );
        let hit = ui.interact(row_r, ui.id().with(("exp_fmt_btn", idx)), Sense::click());
        painter.rect_filled(
            row_r,
            CornerRadius::same(8),
            if hit.hovered() {
                pal.accent_orange
            } else {
                pal.bg_pill_button
            },
        );
        painter.text(
            row_r.center(),
            Align2::CENTER_CENTER,
            desc,
            font_ui_bold(13.5),
            if hit.hovered() {
                Color32::WHITE
            } else {
                pal.fg_primary
            },
        );
        if hit.clicked() {
            actions.push(Action::ExportDocument(fmt_code.to_string()));
        }
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
