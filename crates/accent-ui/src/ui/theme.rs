use accent_session::{ColorPalettePreset, ThemeMode};
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
    pub fg_subheading: Color32,
    pub fg_muted: Color32,
    pub fg_focus_dimmed: Color32,
    pub bg_code_block: Color32,
    pub bg_code_inline: Color32,
    pub bg_pill_button: Color32,
    pub bg_popover: Color32,
    pub bg_preview: Color32,
    pub divider_preview: Color32,
    pub accent_orange: Color32,
    pub accent_link: Color32,
    pub markup_emphasis_fg: Color32,
    pub markup_code_fg: Color32,
    pub markup_codeblock_fg: Color32,
    pub markup_list_marker: Color32,
    pub markup_task_done: Color32,
    pub markup_quote_bar: Color32,
    pub markup_quote_fg: Color32,
    pub markup_link_url: Color32,
    pub bg_table_header: Color32,
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

    pub const fn for_preset(preset: ColorPalettePreset, mode: ThemeMode) -> Self {
        match (preset, mode) {
            (ColorPalettePreset::AmberCoral, ThemeMode::Light) => Self::amber_light(),
            (ColorPalettePreset::AmberCoral, ThemeMode::Sepia) => Self::amber_sepia(),
            (ColorPalettePreset::AmberCoral, ThemeMode::Dark) => Self::amber_dark(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::Light) => Self::editorial_light(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::Sepia) => Self::editorial_sepia(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::Dark) => Self::editorial_dark(),
            (ColorPalettePreset::NordicForest, ThemeMode::Light) => Self::nordic_light(),
            (ColorPalettePreset::NordicForest, ThemeMode::Sepia) => Self::nordic_sepia(),
            (ColorPalettePreset::NordicForest, ThemeMode::Dark) => Self::nordic_dark(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::Light) => Self::velvet_light(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::Sepia) => Self::velvet_sepia(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::Dark) => Self::velvet_dark(),
            (ColorPalettePreset::Classic, _) => Self::for_mode(mode),
        }
    }

    /// Amber & Coral (mdfried-inspired) — Light (*Alabaster & Ochre*)
    pub const fn amber_light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xfa, 0xf8, 0xf5),         // #faf8f5
            bg_headerbar: Color32::from_rgb(0xf1, 0xed, 0xe6),      // #f1ede6
            border_headerbar: Color32::from_rgb(0xe0, 0xda, 0xd1),  // #e0dad1
            fg_primary: Color32::from_rgb(0x2c, 0x2e, 0x33),        // #2c2e33
            fg_heading: Color32::from_rgb(0x1e, 0x29, 0x3b),        // #1e293b
            fg_subheading: Color32::from_rgb(0x2b, 0x6c, 0xb0),     // #2b6cb0
            fg_muted: Color32::from_rgb(0x86, 0x8a, 0x92),          // #868a92
            fg_focus_dimmed: Color32::from_rgb(0x9b, 0x9f, 0xa8),   // #9b9fa8
            bg_code_block: Color32::from_rgb(0xef, 0xeb, 0xe4),     // #efebe4
            bg_code_inline: Color32::from_rgb(0xe9, 0xe3, 0xd9),    // #e9e3d9
            bg_pill_button: Color32::from_rgb(0xe6, 0xe0, 0xd6),    // #e6e0d6
            bg_popover: Color32::from_rgb(0xfd, 0xfb, 0xf8),        // #fdfbf8
            bg_preview: Color32::from_rgb(0xf5, 0xf1, 0xea),        // #f5f1ea
            divider_preview: Color32::from_rgb(0xe0, 0xda, 0xd1),   // #e0dad1
            accent_orange: Color32::from_rgb(0xb4, 0x53, 0x09),     // #b45309
            accent_link: Color32::from_rgb(0x25, 0x63, 0xeb),       // #2563eb
            markup_emphasis_fg: Color32::from_rgb(0xb4, 0x53, 0x09),// #b45309 (Warm Ochre/Gold)
            markup_code_fg: Color32::from_rgb(0xc5, 0x30, 0x30),    // #c53030 (Coral Crimson)
            markup_codeblock_fg: Color32::from_rgb(0x25, 0x63, 0xeb),// #2563eb (Cornflower Blue)
            markup_list_marker: Color32::from_rgb(0xb4, 0x53, 0x09),// #b45309
            markup_task_done: Color32::from_rgb(0x2f, 0x85, 0x5a),  // #2f855a
            markup_quote_bar: Color32::from_rgb(0xb4, 0x53, 0x09),  // #b45309
            markup_quote_fg: Color32::from_rgb(0x57, 0x53, 0x4e),   // #57534e
            markup_link_url: Color32::from_rgb(0x5a, 0x82, 0xb8),   // #5a82b8
            bg_table_header: Color32::from_rgb(0xea, 0xe4, 0xdc),   // #eae4dc
            spellcheck_red: Color32::from_rgb(0xc5, 0x30, 0x30),    // #c53030
            scrollbar_thumb: Color32::from_rgb(0xc8, 0xc2, 0xb8),   // #c8c2b8
        }
    }

    /// Amber & Coral (mdfried-inspired) — Sepia (*Parchment & Burnt Amber*)
    pub const fn amber_sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf4, 0xef, 0xe4),         // #f4efe4
            bg_headerbar: Color32::from_rgb(0xeb, 0xe3, 0xd3),      // #ebe3d3
            border_headerbar: Color32::from_rgb(0xda, 0xcf, 0xb8),  // #dacfb8
            fg_primary: Color32::from_rgb(0x38, 0x30, 0x28),        // #383028
            fg_heading: Color32::from_rgb(0x29, 0x21, 0x1a),        // #29211a
            fg_subheading: Color32::from_rgb(0x1d, 0x5b, 0x79),     // #1d5b79
            fg_muted: Color32::from_rgb(0x8a, 0x7f, 0x70),          // #8a7f70
            fg_focus_dimmed: Color32::from_rgb(0x9e, 0x93, 0x84),   // #9e9384
            bg_code_block: Color32::from_rgb(0xe8, 0xe0, 0xcf),     // #e8e0cf
            bg_code_inline: Color32::from_rgb(0xe4, 0xdb, 0xc7),    // #e4dbc7
            bg_pill_button: Color32::from_rgb(0xe0, 0xd6, 0xc3),    // #e0d6c3
            bg_popover: Color32::from_rgb(0xf7, 0xf2, 0xe8),        // #f7f2e8
            bg_preview: Color32::from_rgb(0xef, 0xe7, 0xd8),        // #efe7d8
            divider_preview: Color32::from_rgb(0xda, 0xcf, 0xb8),   // #dacfb8
            accent_orange: Color32::from_rgb(0xa1, 0x48, 0x07),     // #a14807
            accent_link: Color32::from_rgb(0x1d, 0x5b, 0x79),       // #1d5b79
            markup_emphasis_fg: Color32::from_rgb(0xa1, 0x48, 0x07),// #a14807 (Burnt Amber)
            markup_code_fg: Color32::from_rgb(0xb8, 0x32, 0x32),    // #b83232 (Warm Coral Red)
            markup_codeblock_fg: Color32::from_rgb(0x1d, 0x5b, 0x79),// #1d5b79 (Deep Petrol Blue)
            markup_list_marker: Color32::from_rgb(0xa1, 0x48, 0x07),// #a14807
            markup_task_done: Color32::from_rgb(0x27, 0x67, 0x49),  // #276749
            markup_quote_bar: Color32::from_rgb(0xa1, 0x48, 0x07),  // #a14807
            markup_quote_fg: Color32::from_rgb(0x5c, 0x50, 0x42),   // #5c5042
            markup_link_url: Color32::from_rgb(0x52, 0x79, 0x8c),   // #52798c
            bg_table_header: Color32::from_rgb(0xe2, 0xd9, 0xc5),   // #e2d9c5
            spellcheck_red: Color32::from_rgb(0xb8, 0x3b, 0x2a),    // #b83b2a
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xbc, 0xae),   // #c6bcae
        }
    }

    /// Amber & Coral (mdfried-inspired) — Dark (*Charcoal Night*): exact mdfried palette
    pub const fn amber_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x1d, 0x1f, 0x21),         // #1d1f21
            bg_headerbar: Color32::from_rgb(0x17, 0x19, 0x1b),      // #17191b
            border_headerbar: Color32::from_rgb(0x2e, 0x32, 0x37),  // #2e3237
            fg_primary: Color32::from_rgb(0xc5, 0xc8, 0xc6),        // #c5c8c6
            fg_heading: Color32::from_rgb(0xe6, 0xe8, 0xeb),        // #e6e8eb
            fg_subheading: Color32::from_rgb(0x81, 0xa2, 0xbe),     // #81a2be
            fg_muted: Color32::from_rgb(0x69, 0x70, 0x79),          // #697079
            fg_focus_dimmed: Color32::from_rgb(0x56, 0x5c, 0x64),   // #565c64
            bg_code_block: Color32::from_rgb(0x25, 0x28, 0x2d),     // #25282d
            bg_code_inline: Color32::from_rgb(0x2c, 0x2f, 0x35),    // #2c2f35
            bg_pill_button: Color32::from_rgb(0x2a, 0x2d, 0x32),    // #2a2d32
            bg_popover: Color32::from_rgb(0x23, 0x26, 0x2a),        // #23262a
            bg_preview: Color32::from_rgb(0x1a, 0x1c, 0x1e),        // #1a1c1e
            divider_preview: Color32::from_rgb(0x2e, 0x32, 0x37),   // #2e3237
            accent_orange: Color32::from_rgb(0xea, 0xb7, 0x00),     // #eab700
            accent_link: Color32::from_rgb(0x81, 0xa2, 0xbe),       // #81a2be
            markup_emphasis_fg: Color32::from_rgb(0xea, 0xb7, 0x00),// #eab700 (mdfried Warm Gold)
            markup_code_fg: Color32::from_rgb(0xe0, 0x6c, 0x75),    // #e06c75 (mdfried Coral Rose)
            markup_codeblock_fg: Color32::from_rgb(0x81, 0xa2, 0xbe),// #81a2be (mdfried Periwinkle Blue)
            markup_list_marker: Color32::from_rgb(0xea, 0xb7, 0x00),// #eab700 (Gold numbers & pipes)
            markup_task_done: Color32::from_rgb(0x98, 0xc3, 0x79),  // #98c379 (Sage Green)
            markup_quote_bar: Color32::from_rgb(0xea, 0xb7, 0x00),  // #eab700
            markup_quote_fg: Color32::from_rgb(0xb5, 0xbd, 0x68),   // #b5bd68
            markup_link_url: Color32::from_rgb(0x5f, 0x7d, 0x95),   // #5f7d95
            bg_table_header: Color32::from_rgb(0x28, 0x2c, 0x32),   // #282c32
            spellcheck_red: Color32::from_rgb(0xe0, 0x6c, 0x75),    // #e06c75
            scrollbar_thumb: Color32::from_rgb(0x45, 0x4a, 0x52),   // #454a52
        }
    }

    /// Editorial Paper & Slate — Light (*Alabaster Paper*): zero-glare warm alabaster with royal blue accent
    pub const fn editorial_light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xfa, 0xf9, 0xf5),         // #faf9f5
            bg_headerbar: Color32::from_rgb(0xf3, 0xf1, 0xec),      // #f3f1ec
            border_headerbar: Color32::from_rgb(0xe2, 0xdf, 0xd7),  // #e2dfd7
            fg_primary: Color32::from_rgb(0x24, 0x29, 0x2f),        // #24292f
            fg_heading: Color32::from_rgb(0x1a, 0x36, 0x5d),        // #1a365d
            fg_subheading: Color32::from_rgb(0x2c, 0x52, 0x82),     // #2c5282
            fg_muted: Color32::from_rgb(0x82, 0x8a, 0x95),          // #828a95
            fg_focus_dimmed: Color32::from_rgb(0x9a, 0xa0, 0xa6),   // #9aa0a6
            bg_code_block: Color32::from_rgb(0xef, 0xec, 0xe6),     // #efece6
            bg_code_inline: Color32::from_rgb(0xea, 0xe6, 0xdf),    // #eae6df
            bg_pill_button: Color32::from_rgb(0xe8, 0xe5, 0xde),    // #e8e5de
            bg_popover: Color32::from_rgb(0xfc, 0xfb, 0xf9),        // #fcfbf9
            bg_preview: Color32::from_rgb(0xf5, 0xf3, 0xee),        // #f5f3ee
            divider_preview: Color32::from_rgb(0xe2, 0xdf, 0xd7),   // #e2dfd7
            accent_orange: Color32::from_rgb(0x2b, 0x6c, 0xb0),     // #2b6cb0
            accent_link: Color32::from_rgb(0x1d, 0x5b, 0x9e),       // #1d5b9e
            markup_emphasis_fg: Color32::from_rgb(0xb4, 0x53, 0x09),// #b45309
            markup_code_fg: Color32::from_rgb(0xc5, 0x30, 0x30),    // #c53030
            markup_codeblock_fg: Color32::from_rgb(0x1d, 0x5b, 0x9e),// #1d5b9e
            markup_list_marker: Color32::from_rgb(0x2b, 0x6c, 0xb0),// #2b6cb0
            markup_task_done: Color32::from_rgb(0x2f, 0x85, 0x5a),  // #2f855a
            markup_quote_bar: Color32::from_rgb(0x4a, 0x6f, 0xa5),  // #4a6fa5
            markup_quote_fg: Color32::from_rgb(0x4a, 0x55, 0x68),   // #4a5568
            markup_link_url: Color32::from_rgb(0x5a, 0x82, 0xb8),   // #5a82b8
            bg_table_header: Color32::from_rgb(0xe6, 0xeb, 0xf2),   // #e6ebf2
            spellcheck_red: Color32::from_rgb(0xc5, 0x30, 0x30),    // #c53030
            scrollbar_thumb: Color32::from_rgb(0xc8, 0xc4, 0xba),   // #c8c4ba
        }
    }

    /// Editorial Paper & Slate — Sepia (*Warm Book Paper*): natural book page with deep espresso ink & forest teal accent
    pub const fn editorial_sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf4, 0xef, 0xe6),         // #f4efe6
            bg_headerbar: Color32::from_rgb(0xec, 0xe5, 0xd8),      // #ece5d8
            border_headerbar: Color32::from_rgb(0xdd, 0xd4, 0xc3),  // #ddd4c3
            fg_primary: Color32::from_rgb(0x36, 0x2f, 0x28),        // #362f28
            fg_heading: Color32::from_rgb(0x23, 0x4e, 0x52),        // #234e52
            fg_subheading: Color32::from_rgb(0x6b, 0x42, 0x26),     // #6b4226
            fg_muted: Color32::from_rgb(0x8a, 0x7f, 0x70),          // #8a7f70
            fg_focus_dimmed: Color32::from_rgb(0x9e, 0x93, 0x84),   // #9e9384
            bg_code_block: Color32::from_rgb(0xe8, 0xe0, 0xd0),     // #e8e0d0
            bg_code_inline: Color32::from_rgb(0xe3, 0xda, 0xc8),    // #e3dac8
            bg_pill_button: Color32::from_rgb(0xe2, 0xd9, 0xc8),    // #e2d9c8
            bg_popover: Color32::from_rgb(0xf7, 0xf3, 0xeb),        // #f7f3eb
            bg_preview: Color32::from_rgb(0xef, 0xe8, 0xdc),        // #efe8dc
            divider_preview: Color32::from_rgb(0xdd, 0xd4, 0xc3),   // #ddd4c3
            accent_orange: Color32::from_rgb(0x2c, 0x6e, 0x63),     // #2c6e63
            accent_link: Color32::from_rgb(0x23, 0x5e, 0x54),       // #235e54
            markup_emphasis_fg: Color32::from_rgb(0x9c, 0x42, 0x21),// #9c4221
            markup_code_fg: Color32::from_rgb(0xb8, 0x32, 0x32),    // #b83232
            markup_codeblock_fg: Color32::from_rgb(0x23, 0x5e, 0x54),// #235e54
            markup_list_marker: Color32::from_rgb(0x2c, 0x6e, 0x63),// #2c6e63
            markup_task_done: Color32::from_rgb(0x27, 0x67, 0x49),  // #276749
            markup_quote_bar: Color32::from_rgb(0x4e, 0x7a, 0x70),  // #4e7a70
            markup_quote_fg: Color32::from_rgb(0x5c, 0x52, 0x48),   // #5c5248
            markup_link_url: Color32::from_rgb(0x58, 0x85, 0x7c),   // #58857c
            bg_table_header: Color32::from_rgb(0xe2, 0xde, 0xd0),   // #e2ded0
            spellcheck_red: Color32::from_rgb(0xb8, 0x3b, 0x2a),    // #b83b2a
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xbc, 0xae),   // #c6bcae
        }
    }

    /// Editorial Paper & Slate — Dark (*Midnight Slate*): anti-halation silk text on deep slate
    pub const fn editorial_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x19, 0x1c, 0x22),         // #191c22
            bg_headerbar: Color32::from_rgb(0x14, 0x17, 0x1c),      // #14171c
            border_headerbar: Color32::from_rgb(0x28, 0x2d, 0x37),  // #282d37
            fg_primary: Color32::from_rgb(0xdc, 0xe0, 0xe6),        // #dce0e6
            fg_heading: Color32::from_rgb(0x93, 0xc5, 0xfd),        // #93c5fd
            fg_subheading: Color32::from_rgb(0x67, 0xe8, 0xf9),     // #67e8f9
            fg_muted: Color32::from_rgb(0x75, 0x7e, 0x8c),          // #757e8c
            fg_focus_dimmed: Color32::from_rgb(0x5e, 0x66, 0x73),   // #5e6673
            bg_code_block: Color32::from_rgb(0x22, 0x26, 0x30),     // #222630
            bg_code_inline: Color32::from_rgb(0x28, 0x2d, 0x38),    // #282d38
            bg_pill_button: Color32::from_rgb(0x28, 0x2d, 0x37),    // #282d37
            bg_popover: Color32::from_rgb(0x21, 0x25, 0x2d),        // #21252d
            bg_preview: Color32::from_rgb(0x16, 0x19, 0x1e),        // #16191e
            divider_preview: Color32::from_rgb(0x28, 0x2d, 0x37),   // #282d37
            accent_orange: Color32::from_rgb(0x3b, 0x82, 0xf6),     // #3b82f6
            accent_link: Color32::from_rgb(0x60, 0xa5, 0xfa),       // #60a5fa
            markup_emphasis_fg: Color32::from_rgb(0xea, 0xb7, 0x00),// #eab700
            markup_code_fg: Color32::from_rgb(0xf0, 0x71, 0x78),    // #f07178
            markup_codeblock_fg: Color32::from_rgb(0x60, 0xa5, 0xfa),// #60a5fa
            markup_list_marker: Color32::from_rgb(0x60, 0xa5, 0xfa),// #60a5fa
            markup_task_done: Color32::from_rgb(0x48, 0xbb, 0x78),  // #48bb78
            markup_quote_bar: Color32::from_rgb(0x4a, 0x72, 0xb2),  // #4a72b2
            markup_quote_fg: Color32::from_rgb(0xa0, 0xae, 0xc0),   // #a0aec0
            markup_link_url: Color32::from_rgb(0x4b, 0x7a, 0xc7),   // #4b7ac7
            bg_table_header: Color32::from_rgb(0x22, 0x2b, 0x3a),   // #222b3a
            spellcheck_red: Color32::from_rgb(0xe5, 0x53, 0x4b),    // #e5534b
            scrollbar_thumb: Color32::from_rgb(0x45, 0x4c, 0x59),   // #454c59
        }
    }

    /// Nordic Zen & Forest — Light (*Snow Mist*)
    pub const fn nordic_light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf7, 0xf9, 0xfc),         // #f7f9fc
            bg_headerbar: Color32::from_rgb(0xee, 0xf2, 0xf7),      // #eef2f7
            border_headerbar: Color32::from_rgb(0xdc, 0xe3, 0xec),  // #dce3ec
            fg_primary: Color32::from_rgb(0x2e, 0x34, 0x40),        // #2e3440
            fg_heading: Color32::from_rgb(0x2b, 0x4c, 0x7e),        // #2b4c7e
            fg_subheading: Color32::from_rgb(0x3b, 0x69, 0x78),     // #3b6978
            fg_muted: Color32::from_rgb(0x7b, 0x88, 0xa1),          // #7b88a1
            fg_focus_dimmed: Color32::from_rgb(0x90, 0x9c, 0xb3),   // #909cb3
            bg_code_block: Color32::from_rgb(0xe7, 0xed, 0xf4),     // #e7edf4
            bg_code_inline: Color32::from_rgb(0xe1, 0xe8, 0xf1),    // #e1e8f1
            bg_pill_button: Color32::from_rgb(0xe1, 0xe8, 0xf0),    // #e1e8f0
            bg_popover: Color32::from_rgb(0xfa, 0xfc, 0xff),        // #fafcff
            bg_preview: Color32::from_rgb(0xf1, 0xf5, 0xfa),        // #f1f5fa
            divider_preview: Color32::from_rgb(0xdc, 0xe3, 0xec),   // #dce3ec
            accent_orange: Color32::from_rgb(0x4c, 0x78, 0xa8),     // #4c78a8
            accent_link: Color32::from_rgb(0x3b, 0x69, 0x98),       // #3b6998
            markup_emphasis_fg: Color32::from_rgb(0xb4, 0x53, 0x09),// #b45309
            markup_code_fg: Color32::from_rgb(0xa3, 0x48, 0x57),    // #a34857
            markup_codeblock_fg: Color32::from_rgb(0x3b, 0x69, 0x98),// #3b6998
            markup_list_marker: Color32::from_rgb(0x4c, 0x78, 0xa8),// #4c78a8
            markup_task_done: Color32::from_rgb(0x4d, 0x7c, 0x5d),  // #4d7c5d
            markup_quote_bar: Color32::from_rgb(0x68, 0x86, 0xa6),  // #6886a6
            markup_quote_fg: Color32::from_rgb(0x4c, 0x56, 0x6a),   // #4c566a
            markup_link_url: Color32::from_rgb(0x6c, 0x8e, 0xb5),   // #6c8eb5
            bg_table_header: Color32::from_rgb(0xe1, 0xea, 0xf4),   // #e1eaf4
            spellcheck_red: Color32::from_rgb(0xbf, 0x61, 0x6a),    // #bf616a
            scrollbar_thumb: Color32::from_rgb(0xbc, 0xc6, 0xd4),   // #bcc6d4
        }
    }

    /// Nordic Zen & Forest — Sepia (*Everforest Warm*)
    pub const fn nordic_sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf3, 0xea, 0xd3),         // #f3ead3
            bg_headerbar: Color32::from_rgb(0xea, 0xe0, 0xc8),      // #eae0c8
            border_headerbar: Color32::from_rgb(0xdc, 0xd1, 0xb8),  // #dcd1b8
            fg_primary: Color32::from_rgb(0x4b, 0x56, 0x5c),        // #4b565c
            fg_heading: Color32::from_rgb(0x2d, 0x5a, 0x4c),        // #2d5a4c
            fg_subheading: Color32::from_rgb(0x3a, 0x7d, 0x6c),     // #3a7d6c
            fg_muted: Color32::from_rgb(0x85, 0x92, 0x89),          // #859289
            fg_focus_dimmed: Color32::from_rgb(0x9d, 0xa9, 0xa0),   // #9da9a0
            bg_code_block: Color32::from_rgb(0xe6, 0xde, 0xc8),     // #e6dec8
            bg_code_inline: Color32::from_rgb(0xe1, 0xd8, 0xc0),    // #e1d8c0
            bg_pill_button: Color32::from_rgb(0xe0, 0xd7, 0xc0),    // #e0d7c0
            bg_popover: Color32::from_rgb(0xf6, 0xef, 0xe0),        // #f6efe0
            bg_preview: Color32::from_rgb(0xed, 0xe4, 0xce),        // #ede4ce
            divider_preview: Color32::from_rgb(0xdc, 0xd1, 0xb8),   // #dcd1b8
            accent_orange: Color32::from_rgb(0x3a, 0x94, 0x85),     // #3a9485
            accent_link: Color32::from_rgb(0x2d, 0x7d, 0x6f),       // #2d7d6f
            markup_emphasis_fg: Color32::from_rgb(0xa8, 0x50, 0x48),// #a85048
            markup_code_fg: Color32::from_rgb(0xb8, 0x3b, 0x3b),    // #b83b3b
            markup_codeblock_fg: Color32::from_rgb(0x2d, 0x7d, 0x6f),// #2d7d6f
            markup_list_marker: Color32::from_rgb(0x3a, 0x94, 0x85),// #3a9485
            markup_task_done: Color32::from_rgb(0x56, 0x82, 0x59),  // #568259
            markup_quote_bar: Color32::from_rgb(0x68, 0x8f, 0x80),  // #688f80
            markup_quote_fg: Color32::from_rgb(0x5c, 0x6a, 0x67),   // #5c6a67
            markup_link_url: Color32::from_rgb(0x5c, 0x96, 0x8b),   // #5c968b
            bg_table_header: Color32::from_rgb(0xe0, 0xde, 0xc6),   // #e0dec6
            spellcheck_red: Color32::from_rgb(0xe6, 0x7e, 0x80),    // #e67e80
            scrollbar_thumb: Color32::from_rgb(0xc5, 0xb9, 0xa5),   // #c5b9a5
        }
    }

    /// Nordic Zen & Forest — Dark (*Nordic Night*)
    pub const fn nordic_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x21, 0x25, 0x2e),         // #21252e
            bg_headerbar: Color32::from_rgb(0x1b, 0x1f, 0x27),      // #1b1f27
            border_headerbar: Color32::from_rgb(0x2e, 0x34, 0x40),  // #2e3440
            fg_primary: Color32::from_rgb(0xd8, 0xde, 0xe9),        // #d8dee9
            fg_heading: Color32::from_rgb(0x88, 0xc0, 0xd0),        // #88c0d0
            fg_subheading: Color32::from_rgb(0x81, 0xa1, 0xc1),     // #81a1c1
            fg_muted: Color32::from_rgb(0x7b, 0x88, 0xa1),          // #7b88a1
            fg_focus_dimmed: Color32::from_rgb(0x61, 0x6e, 0x88),   // #616e88
            bg_code_block: Color32::from_rgb(0x29, 0x2e, 0x3b),     // #292e3b
            bg_code_inline: Color32::from_rgb(0x2e, 0x34, 0x42),    // #2e3442
            bg_pill_button: Color32::from_rgb(0x32, 0x38, 0x46),    // #323846
            bg_popover: Color32::from_rgb(0x28, 0x2d, 0x38),        // #282d38
            bg_preview: Color32::from_rgb(0x1d, 0x21, 0x29),        // #1d2129
            divider_preview: Color32::from_rgb(0x2e, 0x34, 0x40),   // #2e3440
            accent_orange: Color32::from_rgb(0x5e, 0x81, 0xac),     // #5e81ac
            accent_link: Color32::from_rgb(0x88, 0xc0, 0xd0),       // #88c0d0
            markup_emphasis_fg: Color32::from_rgb(0xeb, 0xcb, 0x8b),// #ebcb8b
            markup_code_fg: Color32::from_rgb(0xe0, 0x6c, 0x75),    // #e06c75
            markup_codeblock_fg: Color32::from_rgb(0x88, 0xc0, 0xd0),// #88c0d0
            markup_list_marker: Color32::from_rgb(0xeb, 0xcb, 0x8b),// #ebcb8b
            markup_task_done: Color32::from_rgb(0xa3, 0xbe, 0x8c),  // #a3be8c
            markup_quote_bar: Color32::from_rgb(0x5e, 0x81, 0xac),  // #5e81ac
            markup_quote_fg: Color32::from_rgb(0xae, 0xb7, 0xc6),   // #aeb7c6
            markup_link_url: Color32::from_rgb(0x61, 0x8c, 0xa1),   // #618ca1
            bg_table_header: Color32::from_rgb(0x2c, 0x35, 0x45),   // #2c3545
            spellcheck_red: Color32::from_rgb(0xbf, 0x61, 0x6a),    // #bf616a
            scrollbar_thumb: Color32::from_rgb(0x4c, 0x56, 0x6a),   // #4c566a
        }
    }

    /// Rosé Pine & Solarized Modern — Light (*Dawn Cream*)
    pub const fn velvet_light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xfa, 0xf4, 0xed),         // #faf4ed
            bg_headerbar: Color32::from_rgb(0xf2, 0xe9, 0xe1),      // #f2e9e1
            border_headerbar: Color32::from_rgb(0xe4, 0xd8, 0xce),  // #e4d8ce
            fg_primary: Color32::from_rgb(0x46, 0x42, 0x61),        // #464261
            fg_heading: Color32::from_rgb(0x28, 0x69, 0x83),        // #286983
            fg_subheading: Color32::from_rgb(0x6e, 0x56, 0x90),     // #6e5690
            fg_muted: Color32::from_rgb(0x79, 0x75, 0x93),          // #797593
            fg_focus_dimmed: Color32::from_rgb(0x98, 0x93, 0xa5),   // #9893a5
            bg_code_block: Color32::from_rgb(0xef, 0xe6, 0xdd),     // #efe6dd
            bg_code_inline: Color32::from_rgb(0xec, 0xe1, 0xd7),    // #ece1d7
            bg_pill_button: Color32::from_rgb(0xea, 0xe0, 0xd6),    // #eae0d6
            bg_popover: Color32::from_rgb(0xfd, 0xf9, 0xf5),        // #fdf9f5
            bg_preview: Color32::from_rgb(0xf5, 0xec, 0xe4),        // #f5ece4
            divider_preview: Color32::from_rgb(0xe4, 0xd8, 0xce),   // #e4d8ce
            accent_orange: Color32::from_rgb(0x28, 0x69, 0x83),     // #286983
            accent_link: Color32::from_rgb(0x28, 0x69, 0x83),       // #286983
            markup_emphasis_fg: Color32::from_rgb(0xc0, 0x6c, 0x30),// #c06c30
            markup_code_fg: Color32::from_rgb(0xb4, 0x63, 0x7a),    // #b4637a
            markup_codeblock_fg: Color32::from_rgb(0x28, 0x69, 0x83),// #286983
            markup_list_marker: Color32::from_rgb(0xd7, 0x82, 0x7e),// #d7827e
            markup_task_done: Color32::from_rgb(0x3e, 0x8f, 0xb0),  // #3e8fb0
            markup_quote_bar: Color32::from_rgb(0x90, 0x7a, 0xa9),  // #907aa9
            markup_quote_fg: Color32::from_rgb(0x57, 0x52, 0x79),   // #575279
            markup_link_url: Color32::from_rgb(0x5d, 0x8b, 0x9e),   // #5d8b9e
            bg_table_header: Color32::from_rgb(0xec, 0xe2, 0xda),   // #ece2da
            spellcheck_red: Color32::from_rgb(0xb4, 0x63, 0x7a),    // #b4637a
            scrollbar_thumb: Color32::from_rgb(0xce, 0xca, 0xcd),   // #cecacd
        }
    }

    /// Rosé Pine & Solarized Modern — Sepia (*Warm Sand*)
    pub const fn velvet_sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf5, 0xec, 0xdc),         // #f5ecdc
            bg_headerbar: Color32::from_rgb(0xed, 0xe2, 0xcf),      // #ede2cf
            border_headerbar: Color32::from_rgb(0xdd, 0xd0, 0xba),  // #ddd0ba
            fg_primary: Color32::from_rgb(0x45, 0x3e, 0x38),        // #453e38
            fg_heading: Color32::from_rgb(0x7a, 0x3e, 0x32),        // #7a3e32
            fg_subheading: Color32::from_rgb(0x28, 0x69, 0x83),     // #286983
            fg_muted: Color32::from_rgb(0x87, 0x7b, 0x70),          // #877b70
            fg_focus_dimmed: Color32::from_rgb(0x9c, 0x90, 0x85),   // #9c9085
            bg_code_block: Color32::from_rgb(0xe8, 0xdc, 0xc8),     // #e8dcc8
            bg_code_inline: Color32::from_rgb(0xe5, 0xd8, 0xc3),    // #e5d8c3
            bg_pill_button: Color32::from_rgb(0xe2, 0xd5, 0xc0),    // #e2d5c0
            bg_popover: Color32::from_rgb(0xf8, 0xf1, 0xe4),        // #f8f1e4
            bg_preview: Color32::from_rgb(0xef, 0xe5, 0xd4),        // #efe5d4
            divider_preview: Color32::from_rgb(0xdd, 0xd0, 0xba),   // #ddd0ba
            accent_orange: Color32::from_rgb(0x9b, 0x5b, 0x4a),     // #9b5b4a
            accent_link: Color32::from_rgb(0x28, 0x69, 0x83),       // #286983
            markup_emphasis_fg: Color32::from_rgb(0xa1, 0x48, 0x07),// #a14807
            markup_code_fg: Color32::from_rgb(0xb4, 0x4c, 0x5c),    // #b44c5c
            markup_codeblock_fg: Color32::from_rgb(0x28, 0x69, 0x83),// #286983
            markup_list_marker: Color32::from_rgb(0xb4, 0x63, 0x7a),// #b4637a
            markup_task_done: Color32::from_rgb(0x3b, 0x7a, 0x57),  // #3b7a57
            markup_quote_bar: Color32::from_rgb(0x9b, 0x5b, 0x4a),  // #9b5b4a
            markup_quote_fg: Color32::from_rgb(0x64, 0x59, 0x50),   // #645950
            markup_link_url: Color32::from_rgb(0x5d, 0x8b, 0x9e),   // #5d8b9e
            bg_table_header: Color32::from_rgb(0xe6, 0xd8, 0xc3),   // #e6d8c3
            spellcheck_red: Color32::from_rgb(0xb4, 0x63, 0x7a),    // #b4637a
            scrollbar_thumb: Color32::from_rgb(0xc8, 0xb9, 0xa6),   // #c8b9a6
        }
    }

    /// Rosé Pine & Solarized Modern — Dark (*Velvet Dusk*)
    pub const fn velvet_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x1f, 0x1d, 0x29),         // #1f1d29
            bg_headerbar: Color32::from_rgb(0x19, 0x17, 0x22),      // #191722
            border_headerbar: Color32::from_rgb(0x2d, 0x2a, 0x3b),  // #2d2a3b
            fg_primary: Color32::from_rgb(0xe0, 0xde, 0xf4),        // #e0def4
            fg_heading: Color32::from_rgb(0xc4, 0xa7, 0xe7),        // #c4a7e7
            fg_subheading: Color32::from_rgb(0x9c, 0xcf, 0xd8),     // #9ccfd8
            fg_muted: Color32::from_rgb(0x82, 0x7e, 0x9c),          // #827e9c
            fg_focus_dimmed: Color32::from_rgb(0x6e, 0x6a, 0x86),   // #6e6a86
            bg_code_block: Color32::from_rgb(0x26, 0x23, 0x34),     // #262334
            bg_code_inline: Color32::from_rgb(0x2c, 0x28, 0x3b),    // #2c283b
            bg_pill_button: Color32::from_rgb(0x31, 0x2e, 0x42),    // #312e42
            bg_popover: Color32::from_rgb(0x26, 0x23, 0x33),        // #262333
            bg_preview: Color32::from_rgb(0x1b, 0x19, 0x24),        // #1b1924
            divider_preview: Color32::from_rgb(0x2d, 0x2a, 0x3b),   // #2d2a3b
            accent_orange: Color32::from_rgb(0x3e, 0x8f, 0xb0),     // #3e8fb0
            accent_link: Color32::from_rgb(0x9c, 0xcf, 0xd8),       // #9ccfd8
            markup_emphasis_fg: Color32::from_rgb(0xf6, 0xc1, 0x77),// #f6c177
            markup_code_fg: Color32::from_rgb(0xeb, 0x6f, 0x92),    // #eb6f92
            markup_codeblock_fg: Color32::from_rgb(0x9c, 0xcf, 0xd8),// #9ccfd8
            markup_list_marker: Color32::from_rgb(0xf6, 0xc1, 0x77),// #f6c177
            markup_task_done: Color32::from_rgb(0x3e, 0x8f, 0xb0),  // #3e8fb0
            markup_quote_bar: Color32::from_rgb(0x90, 0x7a, 0xa9),  // #907aa9
            markup_quote_fg: Color32::from_rgb(0xb8, 0xb4, 0xd0),   // #b8b4d0
            markup_link_url: Color32::from_rgb(0x6f, 0xa0, 0xab),   // #6fa0ab
            bg_table_header: Color32::from_rgb(0x2c, 0x28, 0x3e),   // #2c283e
            spellcheck_red: Color32::from_rgb(0xeb, 0x6f, 0x92),    // #eb6f92
            scrollbar_thumb: Color32::from_rgb(0x52, 0x4f, 0x67),   // #524f67
        }
    }

    /// Classic — Light
    pub const fn light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xff, 0xff, 0xff),         // #ffffff
            bg_headerbar: Color32::from_rgb(0xff, 0xff, 0xff),      // #ffffff
            border_headerbar: Color32::from_rgb(0xe4, 0xe4, 0xe4),  // #e4e4e4
            fg_primary: Color32::from_rgb(0x2e, 0x34, 0x36),        // #2e3436
            fg_heading: Color32::from_rgb(0x1a, 0x4b, 0x84),        // #1a4b84
            fg_subheading: Color32::from_rgb(0x2b, 0x6c, 0xb0),     // #2b6cb0
            fg_muted: Color32::from_rgb(0x9a, 0x9a, 0x9a),          // #9a9a9a
            fg_focus_dimmed: Color32::from_rgb(0x88, 0x8a, 0x85),   // #888a85
            bg_code_block: Color32::from_rgb(0xe0, 0xe0, 0xe0),     // #e0e0e0
            bg_code_inline: Color32::from_rgb(0xeb, 0xeb, 0xeb),    // #ebebeb
            bg_pill_button: Color32::from_rgb(0xf1, 0xf1, 0xf1),    // #f1f1f1
            bg_popover: Color32::from_rgb(0xff, 0xff, 0xff),        // #ffffff
            bg_preview: Color32::from_rgb(0xfa, 0xfa, 0xfa),        // #fafafa
            divider_preview: Color32::from_rgb(0xe4, 0xe4, 0xe4),   // #e4e4e4
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),     // #db632c
            accent_link: Color32::from_rgb(0x1c, 0x71, 0xd8),       // #1c71d8
            markup_emphasis_fg: Color32::from_rgb(0xb4, 0x53, 0x09),// #b45309
            markup_code_fg: Color32::from_rgb(0xc5, 0x30, 0x30),    // #c53030
            markup_codeblock_fg: Color32::from_rgb(0x1c, 0x71, 0xd8),// #1c71d8
            markup_list_marker: Color32::from_rgb(0xdb, 0x63, 0x2c),// #db632c
            markup_task_done: Color32::from_rgb(0x26, 0xa2, 0x69),  // #26a269
            markup_quote_bar: Color32::from_rgb(0xdb, 0x63, 0x2c),  // #db632c
            markup_quote_fg: Color32::from_rgb(0x55, 0x57, 0x53),   // #555753
            markup_link_url: Color32::from_rgb(0x62, 0x98, 0xd8),   // #6298d8
            bg_table_header: Color32::from_rgb(0xf0, 0xed, 0xe9),   // #f0ede9
            spellcheck_red: Color32::from_rgb(0xb8, 0x42, 0x42),    // #b84242
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xc6, 0xc6),   // #c6c6c6
        }
    }

    /// Classic — Sepia
    pub const fn sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf9, 0xf3, 0xe9),         // #f9f3e9
            bg_headerbar: Color32::from_rgb(0xf9, 0xf3, 0xe9),      // #f9f3e9
            border_headerbar: Color32::from_rgb(0xde, 0xd9, 0xd0),  // #ded9d0
            fg_primary: Color32::from_rgb(0x4f, 0x39, 0x15),        // #4f3915
            fg_heading: Color32::from_rgb(0x7a, 0x33, 0x10),        // #7a3310
            fg_subheading: Color32::from_rgb(0x00, 0x66, 0x66),     // #006666
            fg_muted: Color32::from_rgb(0x9e, 0x90, 0x78),          // #9e9078
            fg_focus_dimmed: Color32::from_rgb(0x80, 0x80, 0x80),   // #808080
            bg_code_block: Color32::from_rgb(0xdf, 0xd7, 0xc9),     // #dfd7c9
            bg_code_inline: Color32::from_rgb(0xe5, 0xdd, 0xd0),    // #e5ddd0
            bg_pill_button: Color32::from_rgb(0xec, 0xe5, 0xda),    // #ece5da
            bg_popover: Color32::from_rgb(0xf9, 0xf3, 0xe9),        // #f9f3e9
            bg_preview: Color32::from_rgb(0xf2, 0xeb, 0xe0),        // #f2ebe0
            divider_preview: Color32::from_rgb(0xda, 0xd5, 0xcb),   // #dad5cb
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),     // #db632c
            accent_link: Color32::from_rgb(0x00, 0x80, 0x80),       // #008080
            markup_emphasis_fg: Color32::from_rgb(0xa1, 0x48, 0x07),// #a14807
            markup_code_fg: Color32::from_rgb(0xb8, 0x32, 0x32),    // #b83232
            markup_codeblock_fg: Color32::from_rgb(0x00, 0x80, 0x80),// #008080
            markup_list_marker: Color32::from_rgb(0xdb, 0x63, 0x2c),// #db632c
            markup_task_done: Color32::from_rgb(0x2e, 0x7d, 0x32),  // #2e7d32
            markup_quote_bar: Color32::from_rgb(0xb5, 0x65, 0x3d),  // #b5653d
            markup_quote_fg: Color32::from_rgb(0x6e, 0x56, 0x30),   // #6e5630
            markup_link_url: Color32::from_rgb(0x4d, 0x99, 0x99),   // #4d9999
            bg_table_header: Color32::from_rgb(0xe8, 0xdf, 0xd1),   // #e8dfd1
            spellcheck_red: Color32::from_rgb(0xa8, 0x3c, 0x32),    // #a83c32
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xc0, 0xb6),   // #c6c0b6
        }
    }

    /// Classic — Dark
    pub const fn dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2b, 0x2b, 0x2f),         // #2b2b2f
            bg_headerbar: Color32::from_rgb(0x2e, 0x2e, 0x32),      // #2e2e32
            border_headerbar: Color32::from_rgb(0x1e, 0x1e, 0x23),  // #1e1e23
            fg_primary: Color32::from_rgb(0xee, 0xee, 0xee),        // #eeeeee
            fg_heading: Color32::from_rgb(0x8c, 0xc0, 0xf8),        // #8cc0f8
            fg_subheading: Color32::from_rgb(0xf5, 0x9e, 0x6c),     // #f59e6c
            fg_muted: Color32::from_rgb(0x6e, 0x6e, 0x74),          // #6e6e74
            fg_focus_dimmed: Color32::from_rgb(0x80, 0x80, 0x80),   // #808080
            bg_code_block: Color32::from_rgb(0x3c, 0x3c, 0x42),     // #3c3c42
            bg_code_inline: Color32::from_rgb(0x3a, 0x3a, 0x40),    // #3a3a40
            bg_pill_button: Color32::from_rgb(0x3a, 0x3a, 0x3d),    // #3a3a3d
            bg_popover: Color32::from_rgb(0x36, 0x36, 0x3a),        // #36363a
            bg_preview: Color32::from_rgb(0x26, 0x26, 0x2a),        // #26262a
            divider_preview: Color32::from_rgb(0x16, 0x16, 0x1a),   // #16161a
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),     // #db632c
            accent_link: Color32::from_rgb(0x78, 0xae, 0xed),       // #78aeed
            markup_emphasis_fg: Color32::from_rgb(0xea, 0xb7, 0x00),// #eab700
            markup_code_fg: Color32::from_rgb(0xe0, 0x6c, 0x75),    // #e06c75
            markup_codeblock_fg: Color32::from_rgb(0x78, 0xae, 0xed),// #78aeed
            markup_list_marker: Color32::from_rgb(0xea, 0xb7, 0x00),// #eab700
            markup_task_done: Color32::from_rgb(0x57, 0xe3, 0x89),  // #57e389
            markup_quote_bar: Color32::from_rgb(0xdb, 0x63, 0x2c),  // #db632c
            markup_quote_fg: Color32::from_rgb(0xb8, 0xb8, 0xbe),   // #b8b8be
            markup_link_url: Color32::from_rgb(0x59, 0x82, 0xb3),   // #5982b3
            bg_table_header: Color32::from_rgb(0x38, 0x38, 0x42),   // #383842
            spellcheck_red: Color32::from_rgb(0xb6, 0x36, 0x3c),    // #b6363c
            scrollbar_thumb: Color32::from_rgb(0x6c, 0x6c, 0x70),   // #6c6c70
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

    let has_sys_emoji = try_load_font(
        &mut defs,
        "sys_emoji_vec",
        &[
            "/usr/share/fonts/noto/NotoEmoji-Regular.ttf",
            "/usr/share/fonts/TTF/NotoEmoji-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
            "C:\\Windows\\Fonts\\seguiemj.ttf",
        ],
    );
    let has_sys_symbols = try_load_font(
        &mut defs,
        "sys_symbols_vec",
        &[
            "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
            "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/gnu-free/FreeSans.otf",
            "C:\\Windows\\Fonts\\seguisym.ttf",
        ],
    );

    if has_sys_emoji {
        if let Some(mono) = defs.families.get_mut(&FontFamily::Monospace) {
            mono.push("sys_emoji_vec".to_string());
        }
        if let Some(prop) = defs.families.get_mut(&FontFamily::Proportional) {
            prop.push("sys_emoji_vec".to_string());
        }
    }
    if has_sys_symbols {
        if let Some(mono) = defs.families.get_mut(&FontFamily::Monospace) {
            mono.push("sys_symbols_vec".to_string());
        }
        if let Some(prop) = defs.families.get_mut(&FontFamily::Proportional) {
            prop.push("sys_symbols_vec".to_string());
        }
    }

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
    let emoji_slot_vec = if !reg_vec.is_empty() {
        reg_vec.clone()
    } else {
        fallback_mono.clone()
    };
    defs.families
        .insert(FontFamily::Name("EmojiSlot".into()), emoji_slot_vec);
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

pub fn apply_theme(ctx: &Context, preset: ColorPalettePreset, mode: ThemeMode) -> AccentPalette {
    let pal = AccentPalette::for_preset(preset, mode);
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

/// Draws the 5 high-legibility editing/preview mode icons:
/// 0 = Modifica (Raw source editor)
/// 1 = Anteprima in-line • (Caret-aware live preview)
/// 2 = Anteprima split a destra (Half-width split)
/// 3 = Anteprima split sotto (Half-height split)
/// 4 = Solo lettura (Full-width read-only book/reader)
pub fn draw_view_mode_icon(
    painter: &Painter,
    center: Pos2,
    color: Color32,
    accent_orange: Color32,
    mode_idx: usize,
) {
    let s = Stroke::new(1.5, color);
    let r = Rect::from_center_size(center, Vec2::new(15.0, 12.5));
    match mode_idx {
        // 0: Modifica (Raw source editor: page outline with diagonal pen/stylus)
        0 => {
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(r.left() + 3.0, center.y - 2.2),
                    Pos2::new(r.left() + 7.5, center.y - 2.2),
                ],
                Stroke::new(1.3, color),
            );
            painter.line_segment(
                [
                    Pos2::new(r.left() + 3.0, center.y + 1.8),
                    Pos2::new(r.left() + 6.5, center.y + 1.8),
                ],
                Stroke::new(1.3, color),
            );
            // Crisp diagonal stylus
            painter.line_segment(
                [
                    Pos2::new(center.x + 0.5, center.y + 3.0),
                    Pos2::new(r.right() - 2.2, r.top() + 2.5),
                ],
                Stroke::new(1.8, color),
            );
        }
        // 1: Anteprima in-line • (Live preview: typographic lines + orange live dot)
        1 => {
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(r.left() + 2.8, center.y - 2.5),
                    Pos2::new(r.right() - 5.5, center.y - 2.5),
                ],
                Stroke::new(1.6, color),
            );
            painter.line_segment(
                [
                    Pos2::new(r.left() + 2.8, center.y + 0.5),
                    Pos2::new(r.right() - 2.8, center.y + 0.5),
                ],
                Stroke::new(1.2, color),
            );
            painter.line_segment(
                [
                    Pos2::new(r.left() + 2.8, center.y + 3.2),
                    Pos2::new(r.right() - 4.2, center.y + 3.2),
                ],
                Stroke::new(1.2, color),
            );
            painter.circle_filled(Pos2::new(r.right() - 2.8, r.top() + 2.8), 2.3, accent_orange);
        }
        // 2: Anteprima split a destra (Vertical split with shaded right pane)
        2 => {
            let right_half = Rect::from_min_max(Pos2::new(center.x, r.top()), r.right_bottom());
            painter.rect_filled(
                right_half,
                CornerRadius::same(2),
                color.gamma_multiply(0.24),
            );
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [Pos2::new(center.x, r.top()), Pos2::new(center.x, r.bottom())],
                s,
            );
        }
        // 3: Anteprima split sotto (Horizontal split with shaded bottom pane)
        3 => {
            let bot_half = Rect::from_min_max(Pos2::new(r.left(), center.y), r.right_bottom());
            painter.rect_filled(bot_half, CornerRadius::same(2), color.gamma_multiply(0.24));
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [Pos2::new(r.left(), center.y), Pos2::new(r.right(), center.y)],
                s,
            );
        }
        // 4: Solo lettura (Open book reader icon)
        _ => {
            draw_preview_book_icon(painter, center, color);
        }
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

/// Draws the 12 bottom formatting toolbar icons (`0..12`) with crisp custom vector geometry
/// inspired by Apostrophe's typographic `'a'` series (bold `'a'`, italic `'a'`, strikethrough `'a'`)
/// and precision vector glyphs.
pub fn draw_toolbar_icon(painter: &Painter, center: Pos2, fg: Color32, idx: usize) {
    let s = Stroke::new(1.6, fg);
    match idx {
        // 0: Bold typographic 'a' (heavy stem + sculpted double-story arch & bowl)
        0 => {
            let b_stroke = Stroke::new(2.35, fg);
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.8, center.y - 4.2),
                    Pos2::new(center.x + 1.8, center.y - 4.8),
                ],
                b_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 1.8, center.y - 4.8),
                    Pos2::new(center.x + 3.6, center.y - 2.6),
                ],
                b_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 3.6, center.y - 3.0),
                    Pos2::new(center.x + 3.6, center.y + 4.8),
                ],
                Stroke::new(2.6, fg),
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 3.6, center.y + 4.6),
                    Pos2::new(center.x + 5.4, center.y + 4.6),
                ],
                Stroke::new(2.1, fg),
            );
            let bowl_r = Rect::from_min_max(
                Pos2::new(center.x - 4.4, center.y - 0.4),
                Pos2::new(center.x + 3.4, center.y + 4.8),
            );
            painter.rect_stroke(bowl_r, CornerRadius::same(3), b_stroke, StrokeKind::Middle);
        }
        // 1: Italic typographic 'a' (slanted cursive single-story 'a' with hook)
        1 => {
            let i_stroke = Stroke::new(1.75, fg);
            let bowl = Rect::from_min_max(
                Pos2::new(center.x - 4.2, center.y - 3.6),
                Pos2::new(center.x + 2.2, center.y + 4.6),
            );
            painter.rect_stroke(bowl, CornerRadius::same(4), i_stroke, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(center.x + 3.6, center.y - 4.2),
                    Pos2::new(center.x + 1.6, center.y + 4.2),
                ],
                Stroke::new(1.9, fg),
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 1.6, center.y + 4.2),
                    Pos2::new(center.x + 4.6, center.y + 3.4),
                ],
                Stroke::new(1.6, fg),
            );
        }
        // 2: Strikethrough typographic 'a' (clean 'a' crossed by crisp horizontal bar)
        2 => {
            let r_stroke = Stroke::new(1.65, fg);
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.4, center.y - 4.2),
                    Pos2::new(center.x + 2.2, center.y - 4.6),
                ],
                r_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 2.8, center.y - 3.6),
                    Pos2::new(center.x + 2.8, center.y + 4.8),
                ],
                r_stroke,
            );
            let bowl_r = Rect::from_min_max(
                Pos2::new(center.x - 4.0, center.y - 0.2),
                Pos2::new(center.x + 2.8, center.y + 4.8),
            );
            painter.rect_stroke(bowl_r, CornerRadius::same(3), r_stroke, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(center.x - 7.0, center.y + 0.2),
                    Pos2::new(center.x + 7.0, center.y + 0.2),
                ],
                Stroke::new(1.85, fg),
            );
        }
        // 3: Heading 'H1' (vector H + subscript 1)
        3 => {
            let hs = Stroke::new(1.85, fg);
            painter.line_segment(
                [
                    Pos2::new(center.x - 6.0, center.y - 5.0),
                    Pos2::new(center.x - 6.0, center.y + 4.8),
                ],
                hs,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 0.5, center.y - 5.0),
                    Pos2::new(center.x + 0.5, center.y + 4.8),
                ],
                hs,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 6.0, center.y - 0.2),
                    Pos2::new(center.x + 0.5, center.y - 0.2),
                ],
                hs,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 3.5, center.y - 1.2),
                    Pos2::new(center.x + 5.5, center.y - 2.6),
                ],
                Stroke::new(1.5, fg),
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 5.5, center.y - 2.6),
                    Pos2::new(center.x + 5.5, center.y + 4.8),
                ],
                Stroke::new(1.7, fg),
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 3.6, center.y + 4.8),
                    Pos2::new(center.x + 7.2, center.y + 4.8),
                ],
                Stroke::new(1.5, fg),
            );
        }
        // 4: Unordered bullet list
        4 => {
            for dy in [-4.5_f32, 0.0, 4.5] {
                painter.circle_filled(Pos2::new(center.x - 5.5, center.y + dy), 1.5, fg);
                painter.line_segment(
                    [
                        Pos2::new(center.x - 2.0, center.y + dy),
                        Pos2::new(center.x + 6.5, center.y + dy),
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
                    font_ui_bold(7.2),
                    fg,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x - 1.5, center.y + dy),
                        Pos2::new(center.x + 6.5, center.y + dy),
                    ],
                    s,
                );
            }
        }
        // 6: Task list checkbox [v]
        6 => {
            let r = Rect::from_center_size(center, Vec2::new(13.0, 13.0));
            painter.rect_stroke(r, CornerRadius::same(3), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(center.x - 3.4, center.y + 0.2),
                    Pos2::new(center.x - 0.8, center.y + 2.8),
                ],
                Stroke::new(1.8, fg),
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 0.8, center.y + 2.8),
                    Pos2::new(center.x + 3.8, center.y - 2.6),
                ],
                Stroke::new(1.8, fg),
            );
        }
        // 7: Blockquote (sculptural double curved quotation glyphs)
        7 => {
            for dx in [-3.2_f32, 3.0] {
                let qc = Pos2::new(center.x + dx, center.y - 1.8);
                painter.circle_filled(qc, 2.2, fg);
                painter.line_segment(
                    [
                        Pos2::new(qc.x + 1.2, qc.y + 0.5),
                        Pos2::new(qc.x - 1.2, qc.y + 5.2),
                    ],
                    Stroke::new(1.8, fg),
                );
            }
        }
        // 8: Code </>
        8 => {
            painter.line_segment(
                [
                    Pos2::new(center.x - 2.5, center.y - 4.2),
                    Pos2::new(center.x - 6.5, center.y),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - 6.5, center.y),
                    Pos2::new(center.x - 2.5, center.y + 4.2),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 2.5, center.y - 4.2),
                    Pos2::new(center.x + 6.5, center.y),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 6.5, center.y),
                    Pos2::new(center.x + 2.5, center.y + 4.2),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 1.2, center.y - 5.0),
                    Pos2::new(center.x - 1.2, center.y + 5.0),
                ],
                Stroke::new(1.3, fg),
            );
        }
        // 9: Link icon (interlocking chain capsules)
        9 => {
            let l_rect =
                Rect::from_center_size(Pos2::new(center.x - 2.6, center.y), Vec2::new(7.5, 6.5));
            let r_rect =
                Rect::from_center_size(Pos2::new(center.x + 2.6, center.y), Vec2::new(7.5, 6.5));
            painter.rect_stroke(l_rect, CornerRadius::same(3), s, StrokeKind::Middle);
            painter.rect_stroke(r_rect, CornerRadius::same(3), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(center.x - 2.2, center.y),
                    Pos2::new(center.x + 2.2, center.y),
                ],
                Stroke::new(1.8, fg),
            );
        }
        // 10: Image frame icon
        10 => {
            let r = Rect::from_center_size(center, Vec2::new(13.5, 11.5));
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.circle_filled(Pos2::new(r.left() + 3.6, r.top() + 3.6), 1.4, fg);
            painter.line_segment(
                [
                    Pos2::new(r.left() + 2.0, r.bottom() - 2.0),
                    Pos2::new(center.x + 0.5, center.y + 0.2),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + 0.5, center.y + 0.2),
                    Pos2::new(r.right() - 2.0, r.bottom() - 2.0),
                ],
                s,
            );
        }
        // 11: Table grid icon
        _ => {
            let r = Rect::from_center_size(center, Vec2::new(13.0, 12.0));
            let hdr_r = Rect::from_min_max(r.left_top(), Pos2::new(r.right(), r.top() + 4.0));
            painter.rect_filled(hdr_r, CornerRadius::same(2), fg.gamma_multiply(0.24));
            painter.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(r.left(), r.top() + 4.2),
                    Pos2::new(r.right(), r.top() + 4.2),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(r.left(), r.top() + 8.0),
                    Pos2::new(r.right(), r.top() + 8.0),
                ],
                s,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x, r.top() + 4.2),
                    Pos2::new(center.x, r.bottom()),
                ],
                s,
            );
        }
    }
}

/// Returns `true` if `ch` is a Unicode color emoji codepoint that should be rendered
/// with full-color RGBA bitmap/vector graphics rather than `ab_glyph`'s monochrome outline.
pub fn is_color_emoji_char(ch: char) -> bool {
    let u = ch as u32;
    if (0x1F3FB..=0x1F3FF).contains(&u) {
        return false;
    }
    matches!(
        u,
        0x1F000..=0x1FAFF
            | 0x2600..=0x27BF
            | 0x231A..=0x231B
            | 0x2328
            | 0x23CF
            | 0x23E9..=0x23F3
            | 0x23F8..=0x23FA
            | 0x25AA..=0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2B05..=0x2B07
            | 0x2B1B..=0x2B1C
            | 0x2B50
            | 0x2B55
            | 0x203C
            | 0x2049
            | 0x2139
            | 0x2934..=0x2935
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
    )
}

/// Returns `true` if `ch` is an invisible emoji variation selector, ZWJ, or skin-tone combiner
/// that should occupy zero visual width while preserving 1:1 character indices in `LayoutJob`.
pub fn is_emoji_invisible_combiner(ch: char) -> bool {
    matches!(
        ch as u32,
        0xFE0E | 0xFE0F | 0x200D | 0x20E3 | 0x1F3FB..=0x1F3FF
    )
}

fn system_color_emoji_font_bytes() -> Option<&'static [u8]> {
    static BYTES: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
    BYTES
        .get_or_init(|| {
            let paths = [
                "/usr/share/fonts/noto/NotoColorEmoji.ttf",
                "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
                "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
                "/usr/share/fonts/noto-color-emoji/NotoColorEmoji.ttf",
            ];
            for p in paths {
                if let Ok(data) = fs::read(p) {
                    return Some(data);
                }
            }
            None
        })
        .as_deref()
}

fn synthesize_fallback_emoji_image(ch: char) -> Option<egui::ColorImage> {
    let size = 64usize;
    let mut pixels = vec![Color32::TRANSPARENT; size * size];
    let center = (size as f32 - 1.0) * 0.5;

    // Colored circles: 🔴 🟠 🟡 🟢 🔵 🟣 🟤 ⚫ ⚪
    let circle_colors: Option<([u8; 3], [u8; 3])> = match ch {
        '🔴' => Some(([0xEF, 0x44, 0x44], [0xB9, 0x1C, 0x1C])),
        '🟠' => Some(([0xF9, 0x73, 0x16], [0xC2, 0x41, 0x0C])),
        '🟡' => Some(([0xFA, 0xCC, 0x15], [0xCA, 0x8A, 0x04])),
        '🟢' => Some(([0x22, 0xC5, 0x5E], [0x15, 0x80, 0x3D])),
        '🔵' => Some(([0x3B, 0x82, 0xF6], [0x1D, 0x4E, 0xD8])),
        '🟣' => Some(([0xA8, 0x55, 0xF7], [0x7E, 0x22, 0xCE])),
        '🟤' => Some(([0xA1, 0x62, 0x07], [0x71, 0x3F, 0x12])),
        '⚫' => Some(([0x37, 0x41, 0x51], [0x11, 0x18, 0x27])),
        '⚪' => Some(([0xF8, 0xFA, 0xFC], [0x94, 0xA3, 0xB8])),
        _ => None,
    };

    if let Some((fill, border)) = circle_colors {
        let radius = 26.5_f32;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= radius + 0.8 {
                    let edge_alpha = ((radius + 0.8 - dist).clamp(0.0, 1.0) * 255.0) as u8;
                    let is_rim = dist >= radius - 2.4;
                    let base = if is_rim { border } else { fill };
                    pixels[y * size + x] =
                        Color32::from_rgba_unmultiplied(base[0], base[1], base[2], edge_alpha);
                }
            }
        }
        return Some(egui::ColorImage {
            size: [size, size],
            pixels,
        });
    }

    // Colored squares: 🟥 🟧 🟨 🟩 🟦 🟪 🟫 ⬛ ⬜
    let square_colors: Option<([u8; 3], [u8; 3])> = match ch {
        '🟥' => Some(([0xEF, 0x44, 0x44], [0xB9, 0x1C, 0x1C])),
        '🟧' => Some(([0xF9, 0x73, 0x16], [0xC2, 0x41, 0x0C])),
        '🟨' => Some(([0xFA, 0xCC, 0x15], [0xCA, 0x8A, 0x04])),
        '🟩' => Some(([0x22, 0xC5, 0x5E], [0x15, 0x80, 0x3D])),
        '🟦' => Some(([0x3B, 0x82, 0xF6], [0x1D, 0x4E, 0xD8])),
        '🟪' => Some(([0xA8, 0x55, 0xF7], [0x7E, 0x22, 0xCE])),
        '🟫' => Some(([0xA1, 0x62, 0x07], [0x71, 0x3F, 0x12])),
        '⬛' | '◼' | '▪' => Some(([0x37, 0x41, 0x51], [0x11, 0x18, 0x27])),
        '⬜' | '◻' | '▫' => Some(([0xF8, 0xFA, 0xFC], [0x94, 0xA3, 0xB8])),
        '✅' => Some(([0x22, 0xC5, 0x5E], [0x15, 0x80, 0x3D])),
        '❌' => Some(([0xEF, 0x44, 0x44], [0xB9, 0x1C, 0x1C])),
        '⚠' | '⭐' | '✨' => Some(([0xFA, 0xCC, 0x15], [0xCA, 0x8A, 0x04])),
        _ => None,
    };

    if let Some((fill, border)) = square_colors {
        let half = 24.0_f32;
        for y in 0..size {
            for x in 0..size {
                let dx = (x as f32 - center).abs();
                let dy = (y as f32 - center).abs();
                if dx <= half && dy <= half {
                    let is_rim = dx >= half - 2.5 || dy >= half - 2.5;
                    let base = if is_rim { border } else { fill };
                    pixels[y * size + x] = Color32::from_rgb(base[0], base[1], base[2]);
                }
            }
        }
        return Some(egui::ColorImage {
            size: [size, size],
            pixels,
        });
    }

    None
}

/// Extracts a full-color RGBA emoji image from the system `NotoColorEmoji.ttf` (`CBDT`/`CBLC`
/// bitmap tables via `ttf-parser`) or falls back to the built-in procedural RGBA renderer.
pub fn load_color_emoji_image(ch: char) -> Option<egui::ColorImage> {
    if let Some(font_bytes) = system_color_emoji_font_bytes() {
        if let Ok(face) = ttf_parser::Face::parse(font_bytes, 0) {
            if let Some(glyph_id) = face.glyph_index(ch) {
                if let Some(raster) = face.glyph_raster_image(glyph_id, 128) {
                    if let Ok(dyn_img) = image::load_from_memory(raster.data) {
                        let rgba = dyn_img.to_rgba8();
                        let w = rgba.width() as usize;
                        let h = rgba.height() as usize;
                        if w > 0 && h > 0 {
                            return Some(egui::ColorImage::from_rgba_unmultiplied(
                                [w, h],
                                rgba.as_raw(),
                            ));
                        }
                    }
                }
            }
        }
    }
    synthesize_fallback_emoji_image(ch)
}

/// Retrieves or uploads a cached `egui::TextureHandle` for a color emoji character `ch`.
pub fn get_or_load_color_emoji_texture(ctx: &Context, ch: char) -> Option<egui::TextureHandle> {
    let cache_id = egui::Id::new(("accent_color_emoji_tex_v1", ch as u32));
    if let Some(cached) = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(cache_id)) {
        return cached;
    }
    let loaded = load_color_emoji_image(ch).map(|color_img| {
        ctx.load_texture(
            format!("accent_emoji_{:X}", ch as u32),
            color_img,
            egui::TextureOptions::LINEAR,
        )
    });
    ctx.data_mut(|d| d.insert_temp(cache_id, loaded.clone()));
    loaded
}

/// Appends `text` to `job`, replacing color emoji codepoints with transparent fixed-advance
/// slots so `ab_glyph` never paints monochrome `◍` or `.notdef` `□` tofu while preserving
/// exact 1:1 `CCursor` character indices for `paint_galley_color_emojis`.
pub fn append_text_with_color_emojis(
    job: &mut egui::text::LayoutJob,
    text: &str,
    mut leading_space: f32,
    base_format: egui::TextFormat,
) {
    if !text
        .chars()
        .any(|c| is_color_emoji_char(c) || is_emoji_invisible_combiner(c))
    {
        job.append(text, leading_space, base_format);
        return;
    }

    let base_size = base_format.font_id.size;
    let mut normal_start = 0usize;
    for (byte_idx, ch) in text.char_indices() {
        let ch_len = ch.len_utf8();
        if is_color_emoji_char(ch) {
            if normal_start < byte_idx {
                job.append(
                    &text[normal_start..byte_idx],
                    leading_space,
                    base_format.clone(),
                );
                leading_space = 0.0;
            }
            let mut emoji_fmt = base_format.clone();
            emoji_fmt.color = Color32::TRANSPARENT;
            emoji_fmt.font_id = FontId::new(base_size, FontFamily::Name("EmojiSlot".into()));
            emoji_fmt.extra_letter_spacing = base_size * 0.42;
            job.append(&text[byte_idx..byte_idx + ch_len], leading_space, emoji_fmt);
            leading_space = 0.0;
            normal_start = byte_idx + ch_len;
        } else if is_emoji_invisible_combiner(ch) {
            if normal_start < byte_idx {
                job.append(
                    &text[normal_start..byte_idx],
                    leading_space,
                    base_format.clone(),
                );
                leading_space = 0.0;
            }
            let mut comb_fmt = base_format.clone();
            comb_fmt.color = Color32::TRANSPARENT;
            comb_fmt.font_id = FontId::new(0.01, FontFamily::Name("EmojiSlot".into()));
            job.append(&text[byte_idx..byte_idx + ch_len], leading_space, comb_fmt);
            leading_space = 0.0;
            normal_start = byte_idx + ch_len;
        }
    }
    if normal_start < text.len() {
        job.append(&text[normal_start..], leading_space, base_format);
    }
}

/// Paints full-color RGBA emoji textures over the transparent emoji character slots in `galley`.
pub fn paint_galley_color_emojis(
    ctx: &Context,
    painter: &Painter,
    galley: &egui::Galley,
    galley_pos: Pos2,
    text: &str,
    min_char_idx: usize,
    max_char_idx: usize,
    default_font_size: f32,
) {
    let mut char_idx = 0usize;
    for ch in text.chars() {
        if char_idx > max_char_idx {
            break;
        }
        if char_idx >= min_char_idx && is_color_emoji_char(ch) {
            let r0 = galley.pos_from_ccursor(egui::text::CCursor::new(char_idx));
            let r1 = galley.pos_from_ccursor(egui::text::CCursor::new(char_idx + 1));
            let slot_w = (r1.min.x - r0.min.x).abs();
            // Skip collapsed delimiter slots (where font_size is 0.01pt and slot_w < 2.0px)
            if slot_w >= 4.0 {
                if let Some(tex) = get_or_load_color_emoji_texture(ctx, ch) {
                    let row_h = r0.height().max(default_font_size);
                    let emoji_h = (slot_w * 0.98)
                        .clamp(default_font_size * 0.88, (row_h * 0.92).max(default_font_size * 1.05));
                    let tex_size = tex.size_vec2();
                    let aspect = if tex_size.y > 0.0 {
                        (tex_size.x / tex_size.y).clamp(0.85, 1.15)
                    } else {
                        1.0
                    };
                    let emoji_w = emoji_h * aspect;
                    let cx = galley_pos.x + r0.min.x + slot_w * 0.5;
                    let top_y = r0.min.y.min(r0.max.y - emoji_h * 1.10);
                    let cy = galley_pos.y + (top_y + r0.max.y) * 0.5;
                    let rect = Rect::from_center_size(Pos2::new(cx, cy), Vec2::new(emoji_w, emoji_h));
                    painter.image(
                        tex.id(),
                        rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            }
        }
        char_idx += 1;
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

    #[test]
    fn test_color_emoji_detection_and_rasterization() {
        for ch in ['🔴', '🟠', '🟡', '🟢', '🔵', '🟣', '🟤', '⚫', '⚪', '🦀', '🚀', '✨', '🔥', '💡', '✅', '⚠'] {
            assert!(is_color_emoji_char(ch), "Expected {ch} to be detected as color emoji");
            let img = load_color_emoji_image(ch)
                .unwrap_or_else(|| panic!("Expected non-empty color image for {ch}"));
            assert!(img.size[0] >= 32 && img.size[1] >= 32);
            let non_transparent = img.pixels.iter().filter(|p| p.a() > 0).count();
            assert!(
                non_transparent > 100,
                "Expected visible colored pixels for {ch}, got {non_transparent}"
            );
        }

        let sample = "(filtro opzionale per pallino 🔴/🟡/🟢/⚪)";
        let mut job = egui::text::LayoutJob::default();
        append_text_with_color_emojis(
            &mut job,
            sample,
            0.0,
            egui::TextFormat {
                font_id: FontId::new(17.0, FontFamily::Monospace),
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        assert_eq!(job.text, sample);
        // Verify all 4 emoji circles in the sample are given transparent slots so monochrome/tofu never shows
        let transparent_sections = job
            .sections
            .iter()
            .filter(|s| s.format.color == Color32::TRANSPARENT)
            .count();
        assert_eq!(transparent_sections, 4);
    }
}
