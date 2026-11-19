use accent_session::{ColorPalettePreset, ThemeMode};
use serde::{Deserialize, Serialize};

/// Pure-Rust RGBA8 color value used for theme palette definitions and CSS token generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color32(pub u8, pub u8, pub u8, pub u8);

impl Color32 {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b, 255)
    }

    pub const fn r(self) -> u8 {
        self.0
    }

    pub const fn g(self) -> u8 {
        self.1
    }

    pub const fn b(self) -> u8 {
        self.2
    }

    pub const fn a(self) -> u8 {
        self.3
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteCssTokens {
    pub bg_canvas: String,
    pub bg_headerbar: String,
    pub border_headerbar: String,
    pub fg_primary: String,
    pub fg_heading: String,
    pub fg_subheading: String,
    pub fg_muted: String,
    pub bg_code_block: String,
    pub bg_code_inline: String,
    pub bg_pill_button: String,
    pub bg_popover: String,
    pub bg_preview: String,
    pub divider_preview: String,
    pub accent_orange: String,
    pub accent_link: String,
    pub markup_bold_fg: String,
    pub markup_emphasis_fg: String,
    pub markup_code_fg: String,
    pub markup_codeblock_fg: String,
    pub markup_list_marker: String,
    pub markup_task_done: String,
    pub markup_quote_bar: String,
    pub markup_quote_fg: String,
    pub markup_link_url: String,
    pub bg_table_header: String,
    pub spellcheck_red: String,
    pub scrollbar_thumb: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccentPalette {
    pub bg_canvas: Color32,
    pub bg_headerbar: Color32,
    pub border_headerbar: Color32,
    pub fg_primary: Color32,
    pub fg_heading: Color32,
    pub fg_subheading: Color32,
    pub fg_muted: Color32,
    pub bg_code_block: Color32,
    pub bg_code_inline: Color32,
    pub bg_pill_button: Color32,
    pub bg_popover: Color32,
    pub bg_preview: Color32,
    pub divider_preview: Color32,
    pub accent_orange: Color32,
    pub accent_link: Color32,
    pub markup_bold_fg: Color32,
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
            ThemeMode::SemiDark => Self::semi_dark(),
            ThemeMode::Dark => Self::dark(),
        }
    }

    pub const fn for_preset(preset: ColorPalettePreset, mode: ThemeMode) -> Self {
        match (preset, mode) {
            (ColorPalettePreset::AmberCoral, ThemeMode::Light) => Self::amber_light(),
            (ColorPalettePreset::AmberCoral, ThemeMode::Sepia) => Self::amber_sepia(),
            (ColorPalettePreset::AmberCoral, ThemeMode::SemiDark) => Self::amber_semi_dark(),
            (ColorPalettePreset::AmberCoral, ThemeMode::Dark) => Self::amber_dark(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::Light) => Self::editorial_light(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::Sepia) => Self::editorial_sepia(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::SemiDark) => Self::editorial_semi_dark(),
            (ColorPalettePreset::EditorialSlate, ThemeMode::Dark) => Self::editorial_dark(),
            (ColorPalettePreset::NordicForest, ThemeMode::Light) => Self::nordic_light(),
            (ColorPalettePreset::NordicForest, ThemeMode::Sepia) => Self::nordic_sepia(),
            (ColorPalettePreset::NordicForest, ThemeMode::SemiDark) => Self::nordic_semi_dark(),
            (ColorPalettePreset::NordicForest, ThemeMode::Dark) => Self::nordic_dark(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::Light) => Self::velvet_light(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::Sepia) => Self::velvet_sepia(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::SemiDark) => Self::velvet_semi_dark(),
            (ColorPalettePreset::VelvetDawn, ThemeMode::Dark) => Self::velvet_dark(),
            (ColorPalettePreset::Classic, _) => Self::for_mode(mode),
        }
    }

    pub fn to_css_tokens(&self) -> PaletteCssTokens {
        PaletteCssTokens {
            bg_canvas: self.bg_canvas.to_hex(),
            bg_headerbar: self.bg_headerbar.to_hex(),
            border_headerbar: self.border_headerbar.to_hex(),
            fg_primary: self.fg_primary.to_hex(),
            fg_heading: self.fg_heading.to_hex(),
            fg_subheading: self.fg_subheading.to_hex(),
            fg_muted: self.fg_muted.to_hex(),
            bg_code_block: self.bg_code_block.to_hex(),
            bg_code_inline: self.bg_code_inline.to_hex(),
            bg_pill_button: self.bg_pill_button.to_hex(),
            bg_popover: self.bg_popover.to_hex(),
            bg_preview: self.bg_preview.to_hex(),
            divider_preview: self.divider_preview.to_hex(),
            accent_orange: self.accent_orange.to_hex(),
            accent_link: self.accent_link.to_hex(),
            markup_bold_fg: self.markup_bold_fg.to_hex(),
            markup_emphasis_fg: self.markup_emphasis_fg.to_hex(),
            markup_code_fg: self.markup_code_fg.to_hex(),
            markup_codeblock_fg: self.markup_codeblock_fg.to_hex(),
            markup_list_marker: self.markup_list_marker.to_hex(),
            markup_task_done: self.markup_task_done.to_hex(),
            markup_quote_bar: self.markup_quote_bar.to_hex(),
            markup_quote_fg: self.markup_quote_fg.to_hex(),
            markup_link_url: self.markup_link_url.to_hex(),
            bg_table_header: self.bg_table_header.to_hex(),
            spellcheck_red: self.spellcheck_red.to_hex(),
            scrollbar_thumb: self.scrollbar_thumb.to_hex(),
        }
    }

    /// Amber & Coral (mdfried-inspired) — Light (*Alabaster & Ochre*)
    pub const fn amber_light() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xfa, 0xf8, 0xf5),         // #faf8f5
            bg_headerbar: Color32::from_rgb(0xf1, 0xed, 0xe6),      // #f1ede6
            border_headerbar: Color32::from_rgb(0xe0, 0xda, 0xd1),  // #e0dad1
            fg_primary: Color32::from_rgb(0x22, 0x25, 0x2a),        // #22252a
            fg_heading: Color32::from_rgb(0x18, 0x22, 0x32),        // #182232
            fg_subheading: Color32::from_rgb(0x1d, 0x5b, 0x79),     // #1d5b79
            fg_muted: Color32::from_rgb(0x78, 0x7d, 0x85),          // #787d85
            bg_code_block: Color32::from_rgb(0xef, 0xec, 0xe6),     // #efece6
            bg_code_inline: Color32::from_rgb(0xee, 0xf0, 0xf4),    // #eef0f4
            bg_pill_button: Color32::from_rgb(0xe6, 0xe0, 0xd6),    // #e6e0d6
            bg_popover: Color32::from_rgb(0xfd, 0xfb, 0xf8),        // #fdfbf8
            bg_preview: Color32::from_rgb(0xf5, 0xf1, 0xea),        // #f5f1ea
            divider_preview: Color32::from_rgb(0xe0, 0xda, 0xd1),   // #e0dad1
            accent_orange: Color32::from_rgb(0x9a, 0x47, 0x08),     // #9a4708
            accent_link: Color32::from_rgb(0x1d, 0x5b, 0x79),       // #1d5b79
            markup_bold_fg: Color32::from_rgb(0x9a, 0x47, 0x08),    // #9a4708
            markup_emphasis_fg: Color32::from_rgb(0xa6, 0x3a, 0x50),// #a63a50
            markup_code_fg: Color32::from_rgb(0x1e, 0x56, 0x70),    // #1e5670
            markup_codeblock_fg: Color32::from_rgb(0x1d, 0x5b, 0x79),// #1d5b79
            markup_list_marker: Color32::from_rgb(0x9a, 0x47, 0x08),// #9a4708
            markup_task_done: Color32::from_rgb(0x2f, 0x7a, 0x4d),  // #2f7a4d
            markup_quote_bar: Color32::from_rgb(0x9a, 0x47, 0x08),  // #9a4708
            markup_quote_fg: Color32::from_rgb(0x52, 0x4b, 0x42),   // #524b42
            markup_link_url: Color32::from_rgb(0x4a, 0x74, 0x91),   // #4a7491
            bg_table_header: Color32::from_rgb(0xea, 0xe4, 0xdc),   // #eae4dc
            spellcheck_red: Color32::from_rgb(0xb8, 0x32, 0x32),    // #b83232
            scrollbar_thumb: Color32::from_rgb(0xc8, 0xc2, 0xb8),   // #c8c2b8
        }
    }

    /// Amber & Coral (mdfried-inspired) — Sepia (*Parchment & Burnt Amber*)
    pub const fn amber_sepia() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0xf4, 0xef, 0xe4),         // #f4efe4
            bg_headerbar: Color32::from_rgb(0xeb, 0xe3, 0xd3),      // #ebe3d3
            border_headerbar: Color32::from_rgb(0xda, 0xcf, 0xb8),  // #dacfb8
            fg_primary: Color32::from_rgb(0x2f, 0x27, 0x1f),        // #2f271f
            fg_heading: Color32::from_rgb(0x21, 0x1a, 0x13),        // #211a13
            fg_subheading: Color32::from_rgb(0x1d, 0x5b, 0x79),     // #1d5b79
            fg_muted: Color32::from_rgb(0x7d, 0x72, 0x63),          // #7d7263
            bg_code_block: Color32::from_rgb(0xe8, 0xe0, 0xcf),     // #e8e0cf
            bg_code_inline: Color32::from_rgb(0xea, 0xe4, 0xd7),    // #eae4d7
            bg_pill_button: Color32::from_rgb(0xe0, 0xd6, 0xc3),    // #e0d6c3
            bg_popover: Color32::from_rgb(0xf7, 0xf2, 0xe8),        // #f7f2e8
            bg_preview: Color32::from_rgb(0xef, 0xe7, 0xd8),        // #efe7d8
            divider_preview: Color32::from_rgb(0xda, 0xcf, 0xb8),   // #dacfb8
            accent_orange: Color32::from_rgb(0x8f, 0x3f, 0x06),     // #8f3f06
            accent_link: Color32::from_rgb(0x1d, 0x5b, 0x79),       // #1d5b79
            markup_bold_fg: Color32::from_rgb(0x8f, 0x3f, 0x06),    // #8f3f06
            markup_emphasis_fg: Color32::from_rgb(0x9b, 0x37, 0x4b),// #9b374b
            markup_code_fg: Color32::from_rgb(0x1e, 0x56, 0x70),    // #1e5670
            markup_codeblock_fg: Color32::from_rgb(0x1d, 0x5b, 0x79),// #1d5b79
            markup_list_marker: Color32::from_rgb(0x8f, 0x3f, 0x06),// #8f3f06
            markup_task_done: Color32::from_rgb(0x2f, 0x7a, 0x4d),  // #2f7a4d
            markup_quote_bar: Color32::from_rgb(0x8f, 0x3f, 0x06),  // #8f3f06
            markup_quote_fg: Color32::from_rgb(0x52, 0x46, 0x38),   // #524638
            markup_link_url: Color32::from_rgb(0x46, 0x6e, 0x82),   // #466e82
            bg_table_header: Color32::from_rgb(0xe2, 0xd9, 0xc5),   // #e2d9c5
            spellcheck_red: Color32::from_rgb(0xb8, 0x3b, 0x2a),    // #b83b2a
            scrollbar_thumb: Color32::from_rgb(0xc6, 0xbc, 0xae),   // #c6bcae
        }
    }

    /// Amber & Coral (mdfried-inspired) — Semi-Dark (*Soft Anthracite & Warm Gold*)
    pub const fn amber_semi_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2b, 0x2e, 0x33),         // #2b2e33
            bg_headerbar: Color32::from_rgb(0x23, 0x26, 0x2a),      // #23262a
            border_headerbar: Color32::from_rgb(0x3c, 0x40, 0x47),  // #3c4047
            fg_primary: Color32::from_rgb(0xec, 0xee, 0xf2),        // #eceef2
            fg_heading: Color32::from_rgb(0xf5, 0xf7, 0xfa),        // #f5f7fa
            fg_subheading: Color32::from_rgb(0x81, 0xa2, 0xbe),     // #81a2be
            fg_muted: Color32::from_rgb(0x8e, 0x95, 0xa0),          // #8e95a0
            bg_code_block: Color32::from_rgb(0x33, 0x37, 0x3e),     // #33373e
            bg_code_inline: Color32::from_rgb(0x38, 0x3d, 0x45),    // #383d45
            bg_pill_button: Color32::from_rgb(0x39, 0x3e, 0x46),    // #393e46
            bg_popover: Color32::from_rgb(0x31, 0x35, 0x3b),        // #31353b
            bg_preview: Color32::from_rgb(0x27, 0x2a, 0x2e),        // #272a2e
            divider_preview: Color32::from_rgb(0x3c, 0x40, 0x47),   // #3c4047
            accent_orange: Color32::from_rgb(0xf2, 0xc1, 0x2e),     // #f2c12e
            accent_link: Color32::from_rgb(0x81, 0xa2, 0xbe),       // #81a2be
            markup_bold_fg: Color32::from_rgb(0xf2, 0xc1, 0x2e),    // #f2c12e
            markup_emphasis_fg: Color32::from_rgb(0xeb, 0x85, 0x7e),// #eb857e
            markup_code_fg: Color32::from_rgb(0xf0, 0x85, 0x8d),    // #f0858d
            markup_codeblock_fg: Color32::from_rgb(0x81, 0xa2, 0xbe),// #81a2be
            markup_list_marker: Color32::from_rgb(0xf2, 0xc1, 0x2e),// #f2c12e
            markup_task_done: Color32::from_rgb(0x98, 0xc3, 0x79),  // #98c379
            markup_quote_bar: Color32::from_rgb(0xe0, 0x7a, 0x38),  // #e07a38
            markup_quote_fg: Color32::from_rgb(0xd6, 0xcf, 0xc4),   // #d6cfc4
            markup_link_url: Color32::from_rgb(0x73, 0x91, 0xa8),   // #7391a8
            bg_table_header: Color32::from_rgb(0x37, 0x3b, 0x42),   // #373b42
            spellcheck_red: Color32::from_rgb(0xeb, 0x85, 0x7e),    // #eb857e
            scrollbar_thumb: Color32::from_rgb(0x55, 0x5b, 0x64),   // #555b64
        }
    }

    /// Amber & Coral (mdfried-inspired) — Dark (*Charcoal Night*): exact mdfried palette
    pub const fn amber_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x18, 0x1a, 0x1d),         // #181a1d
            bg_headerbar: Color32::from_rgb(0x13, 0x15, 0x18),      // #131518
            border_headerbar: Color32::from_rgb(0x28, 0x2c, 0x32),  // #282c32
            fg_primary: Color32::from_rgb(0xe6, 0xe8, 0xeb),        // #e6e8eb
            fg_heading: Color32::from_rgb(0xf0, 0xf2, 0xf5),        // #f0f2f5
            fg_subheading: Color32::from_rgb(0x81, 0xa2, 0xbe),     // #81a2be
            fg_muted: Color32::from_rgb(0x74, 0x7c, 0x87),          // #747c87
            bg_code_block: Color32::from_rgb(0x22, 0x25, 0x2a),     // #22252a
            bg_code_inline: Color32::from_rgb(0x26, 0x2a, 0x30),    // #262a30
            bg_pill_button: Color32::from_rgb(0x27, 0x2b, 0x31),    // #272b31
            bg_popover: Color32::from_rgb(0x1e, 0x21, 0x26),        // #1e2126
            bg_preview: Color32::from_rgb(0x15, 0x17, 0x1a),        // #15171a
            divider_preview: Color32::from_rgb(0x28, 0x2c, 0x32),   // #282c32
            accent_orange: Color32::from_rgb(0xea, 0xb7, 0x00),     // #eab700
            accent_link: Color32::from_rgb(0x81, 0xa2, 0xbe),       // #81a2be
            markup_bold_fg: Color32::from_rgb(0xea, 0xb7, 0x00),    // #eab700
            markup_emphasis_fg: Color32::from_rgb(0xe5, 0x7a, 0x74),// #e57a74
            markup_code_fg: Color32::from_rgb(0xeb, 0x7a, 0x82),    // #eb7a82
            markup_codeblock_fg: Color32::from_rgb(0x81, 0xa2, 0xbe),// #81a2be
            markup_list_marker: Color32::from_rgb(0xea, 0xb7, 0x00),// #eab700
            markup_task_done: Color32::from_rgb(0x98, 0xc3, 0x79),  // #98c379
            markup_quote_bar: Color32::from_rgb(0xe0, 0x7a, 0x38),  // #e07a38
            markup_quote_fg: Color32::from_rgb(0xce, 0xc6, 0xba),   // #cec6ba
            markup_link_url: Color32::from_rgb(0x66, 0x85, 0x9e),   // #66859e
            bg_table_header: Color32::from_rgb(0x24, 0x28, 0x2e),   // #24282e
            spellcheck_red: Color32::from_rgb(0xe5, 0x7a, 0x74),    // #e57a74
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
            bg_code_block: Color32::from_rgb(0xef, 0xec, 0xe6),     // #efece6
            bg_code_inline: Color32::from_rgb(0xea, 0xe6, 0xdf),    // #eae6df
            bg_pill_button: Color32::from_rgb(0xe8, 0xe5, 0xde),    // #e8e5de
            bg_popover: Color32::from_rgb(0xfc, 0xfb, 0xf9),        // #fcfbf9
            bg_preview: Color32::from_rgb(0xf5, 0xf3, 0xee),        // #f5f3ee
            divider_preview: Color32::from_rgb(0xe2, 0xdf, 0xd7),   // #e2dfd7
            accent_orange: Color32::from_rgb(0x2b, 0x6c, 0xb0),     // #2b6cb0
            accent_link: Color32::from_rgb(0x1d, 0x5b, 0x9e),       // #1d5b9e
            markup_bold_fg: Color32::from_rgb(0x1a, 0x36, 0x5d),    // #1a365d
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
            bg_code_block: Color32::from_rgb(0xe8, 0xe0, 0xd0),     // #e8e0d0
            bg_code_inline: Color32::from_rgb(0xe3, 0xda, 0xc8),    // #e3dac8
            bg_pill_button: Color32::from_rgb(0xe2, 0xd9, 0xc8),    // #e2d9c8
            bg_popover: Color32::from_rgb(0xf7, 0xf3, 0xeb),        // #f7f3eb
            bg_preview: Color32::from_rgb(0xef, 0xe8, 0xdc),        // #efe8dc
            divider_preview: Color32::from_rgb(0xdd, 0xd4, 0xc3),   // #ddd4c3
            accent_orange: Color32::from_rgb(0x2c, 0x6e, 0x63),     // #2c6e63
            accent_link: Color32::from_rgb(0x23, 0x5e, 0x54),       // #235e54
            markup_bold_fg: Color32::from_rgb(0x23, 0x4e, 0x52),    // #234e52
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

    /// Editorial Paper & Slate — Semi-Dark (*Anthracite Slate*)
    pub const fn editorial_semi_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2c, 0x30, 0x38),         // #2c3038
            bg_headerbar: Color32::from_rgb(0x25, 0x29, 0x31),      // #252931
            border_headerbar: Color32::from_rgb(0x3c, 0x42, 0x4e),  // #3c424e
            fg_primary: Color32::from_rgb(0xe1, 0xe5, 0xec),        // #e1e5ec
            fg_heading: Color32::from_rgb(0xa3, 0xcf, 0xfe),        // #a3cffe
            fg_subheading: Color32::from_rgb(0x7e, 0xec, 0xfa),     // #7eecfa
            fg_muted: Color32::from_rgb(0x8e, 0x97, 0xa6),          // #8e97a6
            bg_code_block: Color32::from_rgb(0x35, 0x3a, 0x44),     // #353a44
            bg_code_inline: Color32::from_rgb(0x3b, 0x41, 0x4c),    // #3b414c
            bg_pill_button: Color32::from_rgb(0x3a, 0x40, 0x4b),    // #3a404b
            bg_popover: Color32::from_rgb(0x33, 0x38, 0x42),        // #333842
            bg_preview: Color32::from_rgb(0x28, 0x2c, 0x34),        // #282c34
            divider_preview: Color32::from_rgb(0x3c, 0x42, 0x4e),   // #3c424e
            accent_orange: Color32::from_rgb(0x4b, 0x8d, 0xf8),     // #4b8df8
            accent_link: Color32::from_rgb(0x6e, 0xb0, 0xfb),       // #6eb0fb
            markup_bold_fg: Color32::from_rgb(0xa3, 0xcf, 0xfe),    // #a3cffe
            markup_emphasis_fg: Color32::from_rgb(0xec, 0xbe, 0x24),// #ecbe24
            markup_code_fg: Color32::from_rgb(0xf2, 0x7d, 0x84),    // #f27d84
            markup_codeblock_fg: Color32::from_rgb(0x6e, 0xb0, 0xfb),// #6eb0fb
            markup_list_marker: Color32::from_rgb(0x6e, 0xb0, 0xfb),// #6eb0fb
            markup_task_done: Color32::from_rgb(0x54, 0xc4, 0x84),  // #54c484
            markup_quote_bar: Color32::from_rgb(0x5c, 0x85, 0xc4),  // #5c85c4
            markup_quote_fg: Color32::from_rgb(0xb2, 0xbf, 0xd0),   // #b2bfd0
            markup_link_url: Color32::from_rgb(0x62, 0x8e, 0xd4),   // #628ed4
            bg_table_header: Color32::from_rgb(0x35, 0x3e, 0x4d),   // #353e4d
            spellcheck_red: Color32::from_rgb(0xe8, 0x62, 0x5a),    // #e8625a
            scrollbar_thumb: Color32::from_rgb(0x55, 0x5d, 0x6c),   // #555d6c
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
            bg_code_block: Color32::from_rgb(0x22, 0x26, 0x30),     // #222630
            bg_code_inline: Color32::from_rgb(0x28, 0x2d, 0x38),    // #282d38
            bg_pill_button: Color32::from_rgb(0x28, 0x2d, 0x37),    // #282d37
            bg_popover: Color32::from_rgb(0x21, 0x25, 0x2d),        // #21252d
            bg_preview: Color32::from_rgb(0x16, 0x19, 0x1e),        // #16191e
            divider_preview: Color32::from_rgb(0x28, 0x2d, 0x37),   // #282d37
            accent_orange: Color32::from_rgb(0x3b, 0x82, 0xf6),     // #3b82f6
            accent_link: Color32::from_rgb(0x60, 0xa5, 0xfa),       // #60a5fa
            markup_bold_fg: Color32::from_rgb(0x93, 0xc5, 0xfd),    // #93c5fd
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
            bg_code_block: Color32::from_rgb(0xe7, 0xed, 0xf4),     // #e7edf4
            bg_code_inline: Color32::from_rgb(0xe1, 0xe8, 0xf1),    // #e1e8f1
            bg_pill_button: Color32::from_rgb(0xe1, 0xe8, 0xf0),    // #e1e8f0
            bg_popover: Color32::from_rgb(0xfa, 0xfc, 0xff),        // #fafcff
            bg_preview: Color32::from_rgb(0xf1, 0xf5, 0xfa),        // #f1f5fa
            divider_preview: Color32::from_rgb(0xdc, 0xe3, 0xec),   // #dce3ec
            accent_orange: Color32::from_rgb(0x4c, 0x78, 0xa8),     // #4c78a8
            accent_link: Color32::from_rgb(0x3b, 0x69, 0x98),       // #3b6998
            markup_bold_fg: Color32::from_rgb(0x2b, 0x4c, 0x7e),    // #2b4c7e
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
            bg_code_block: Color32::from_rgb(0xe6, 0xde, 0xc8),     // #e6dec8
            bg_code_inline: Color32::from_rgb(0xe1, 0xd8, 0xc0),    // #e1d8c0
            bg_pill_button: Color32::from_rgb(0xe0, 0xd7, 0xc0),    // #e0d7c0
            bg_popover: Color32::from_rgb(0xf6, 0xef, 0xe0),        // #f6efe0
            bg_preview: Color32::from_rgb(0xed, 0xe4, 0xce),        // #ede4ce
            divider_preview: Color32::from_rgb(0xdc, 0xd1, 0xb8),   // #dcd1b8
            accent_orange: Color32::from_rgb(0x3a, 0x94, 0x85),     // #3a9485
            accent_link: Color32::from_rgb(0x2d, 0x7d, 0x6f),       // #2d7d6f
            markup_bold_fg: Color32::from_rgb(0x2d, 0x5a, 0x4c),    // #2d5a4c
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

    /// Nordic Zen & Forest — Semi-Dark (*Polar Anthracite*)
    pub const fn nordic_semi_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2e, 0x34, 0x40),         // #2e3440
            bg_headerbar: Color32::from_rgb(0x27, 0x2c, 0x36),      // #272c36
            border_headerbar: Color32::from_rgb(0x3b, 0x42, 0x52),  // #3b4252
            fg_primary: Color32::from_rgb(0xe5, 0xe9, 0xf0),        // #e5e9f0
            fg_heading: Color32::from_rgb(0x8f, 0xbc, 0xbb),        // #8fbcbb
            fg_subheading: Color32::from_rgb(0x88, 0xc0, 0xd0),     // #88c0d0
            fg_muted: Color32::from_rgb(0x90, 0x9c, 0xb3),          // #909cb3
            bg_code_block: Color32::from_rgb(0x37, 0x3e, 0x4c),     // #373e4c
            bg_code_inline: Color32::from_rgb(0x3b, 0x42, 0x52),    // #3b4252
            bg_pill_button: Color32::from_rgb(0x3f, 0x47, 0x58),    // #3f4758
            bg_popover: Color32::from_rgb(0x35, 0x3c, 0x4a),        // #353c4a
            bg_preview: Color32::from_rgb(0x2a, 0x30, 0x3b),        // #2a303b
            divider_preview: Color32::from_rgb(0x3b, 0x42, 0x52),   // #3b4252
            accent_orange: Color32::from_rgb(0x68, 0x8b, 0xb5),     // #688bb5
            accent_link: Color32::from_rgb(0x88, 0xc0, 0xd0),       // #88c0d0
            markup_bold_fg: Color32::from_rgb(0x8f, 0xbc, 0xbb),    // #8fbcbb
            markup_emphasis_fg: Color32::from_rgb(0xeb, 0xcb, 0x8b),// #ebcb8b
            markup_code_fg: Color32::from_rgb(0xe4, 0x7a, 0x82),    // #e47a82
            markup_codeblock_fg: Color32::from_rgb(0x88, 0xc0, 0xd0),// #88c0d0
            markup_list_marker: Color32::from_rgb(0xeb, 0xcb, 0x8b),// #ebcb8b
            markup_task_done: Color32::from_rgb(0xa3, 0xbe, 0x8c),  // #a3be8c
            markup_quote_bar: Color32::from_rgb(0x68, 0x8b, 0xb5),  // #688bb5
            markup_quote_fg: Color32::from_rgb(0xb8, 0xc2, 0xd1),   // #b8c2d1
            markup_link_url: Color32::from_rgb(0x6e, 0x99, 0xae),   // #6e99ae
            bg_table_header: Color32::from_rgb(0x39, 0x42, 0x53),   // #394253
            spellcheck_red: Color32::from_rgb(0xbf, 0x61, 0x6a),    // #bf616a
            scrollbar_thumb: Color32::from_rgb(0x59, 0x64, 0x7a),   // #59647a
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
            bg_code_block: Color32::from_rgb(0x29, 0x2e, 0x3b),     // #292e3b
            bg_code_inline: Color32::from_rgb(0x2e, 0x34, 0x42),    // #2e3442
            bg_pill_button: Color32::from_rgb(0x32, 0x38, 0x46),    // #323846
            bg_popover: Color32::from_rgb(0x28, 0x2d, 0x38),        // #282d38
            bg_preview: Color32::from_rgb(0x1d, 0x21, 0x29),        // #1d2129
            divider_preview: Color32::from_rgb(0x2e, 0x34, 0x40),   // #2e3440
            accent_orange: Color32::from_rgb(0x5e, 0x81, 0xac),     // #5e81ac
            accent_link: Color32::from_rgb(0x88, 0xc0, 0xd0),       // #88c0d0
            markup_bold_fg: Color32::from_rgb(0x88, 0xc0, 0xd0),    // #88c0d0
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
            bg_code_block: Color32::from_rgb(0xef, 0xe6, 0xdd),     // #efe6dd
            bg_code_inline: Color32::from_rgb(0xec, 0xe1, 0xd7),    // #ece1d7
            bg_pill_button: Color32::from_rgb(0xea, 0xe0, 0xd6),    // #eae0d6
            bg_popover: Color32::from_rgb(0xfd, 0xf9, 0xf5),        // #fdf9f5
            bg_preview: Color32::from_rgb(0xf5, 0xec, 0xe4),        // #f5ece4
            divider_preview: Color32::from_rgb(0xe4, 0xd8, 0xce),   // #e4d8ce
            accent_orange: Color32::from_rgb(0x28, 0x69, 0x83),     // #286983
            accent_link: Color32::from_rgb(0x28, 0x69, 0x83),       // #286983
            markup_bold_fg: Color32::from_rgb(0x28, 0x69, 0x83),    // #286983
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
            bg_code_block: Color32::from_rgb(0xe8, 0xdc, 0xc8),     // #e8dcc8
            bg_code_inline: Color32::from_rgb(0xe5, 0xd8, 0xc3),    // #e5d8c3
            bg_pill_button: Color32::from_rgb(0xe2, 0xd5, 0xc0),    // #e2d5c0
            bg_popover: Color32::from_rgb(0xf8, 0xf1, 0xe4),        // #f8f1e4
            bg_preview: Color32::from_rgb(0xef, 0xe5, 0xd4),        // #efe5d4
            divider_preview: Color32::from_rgb(0xdd, 0xd0, 0xba),   // #ddd0ba
            accent_orange: Color32::from_rgb(0x9b, 0x5b, 0x4a),     // #9b5b4a
            accent_link: Color32::from_rgb(0x28, 0x69, 0x83),       // #286983
            markup_bold_fg: Color32::from_rgb(0x7a, 0x3e, 0x32),    // #7a3e32
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

    /// Rosé Pine & Solarized Modern — Semi-Dark (*Moon Anthracite*)
    pub const fn velvet_semi_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2f, 0x2c, 0x3c),         // #2f2c3c
            bg_headerbar: Color32::from_rgb(0x28, 0x25, 0x34),      // #282534
            border_headerbar: Color32::from_rgb(0x3f, 0x3b, 0x50),  // #3f3b50
            fg_primary: Color32::from_rgb(0xe6, 0xe4, 0xf6),        // #e6e4f6
            fg_heading: Color32::from_rgb(0xcd, 0xb4, 0xec),        // #cdb4ec
            fg_subheading: Color32::from_rgb(0xa6, 0xd5, 0xdd),     // #a6d5dd
            fg_muted: Color32::from_rgb(0x96, 0x92, 0xb0),          // #9692b0
            bg_code_block: Color32::from_rgb(0x37, 0x34, 0x47),     // #373447
            bg_code_inline: Color32::from_rgb(0x3d, 0x39, 0x4e),    // #3d394e
            bg_pill_button: Color32::from_rgb(0x41, 0x3d, 0x54),    // #413d54
            bg_popover: Color32::from_rgb(0x36, 0x32, 0x46),        // #363246
            bg_preview: Color32::from_rgb(0x2b, 0x28, 0x37),        // #2b2837
            divider_preview: Color32::from_rgb(0x3f, 0x3b, 0x50),   // #3f3b50
            accent_orange: Color32::from_rgb(0x4b, 0x9c, 0xbd),     // #4b9cbd
            accent_link: Color32::from_rgb(0xa6, 0xd5, 0xdd),       // #a6d5dd
            markup_bold_fg: Color32::from_rgb(0xcd, 0xb4, 0xec),    // #cdb4ec
            markup_emphasis_fg: Color32::from_rgb(0xf6, 0xc1, 0x77),// #f6c177
            markup_code_fg: Color32::from_rgb(0xee, 0x7c, 0x9c),    // #ee7c9c
            markup_codeblock_fg: Color32::from_rgb(0xa6, 0xd5, 0xdd),// #a6d5dd
            markup_list_marker: Color32::from_rgb(0xf6, 0xc1, 0x77),// #f6c177
            markup_task_done: Color32::from_rgb(0x4b, 0x9c, 0xbd),  // #4b9cbd
            markup_quote_bar: Color32::from_rgb(0x9c, 0x86, 0xb5),  // #9c86b5
            markup_quote_fg: Color32::from_rgb(0xc2, 0xbe, 0xd8),   // #c2bed8
            markup_link_url: Color32::from_rgb(0x7c, 0xad, 0xb8),   // #7cadb8
            bg_table_header: Color32::from_rgb(0x3c, 0x38, 0x50),   // #3c3850
            spellcheck_red: Color32::from_rgb(0xeb, 0x6f, 0x92),    // #eb6f92
            scrollbar_thumb: Color32::from_rgb(0x5e, 0x5a, 0x75),   // #5e5a75
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
            bg_code_block: Color32::from_rgb(0x26, 0x23, 0x34),     // #262334
            bg_code_inline: Color32::from_rgb(0x2c, 0x28, 0x3b),    // #2c283b
            bg_pill_button: Color32::from_rgb(0x31, 0x2e, 0x42),    // #312e42
            bg_popover: Color32::from_rgb(0x26, 0x23, 0x33),        // #262333
            bg_preview: Color32::from_rgb(0x1b, 0x19, 0x24),        // #1b1924
            divider_preview: Color32::from_rgb(0x2d, 0x2a, 0x3b),   // #2d2a3b
            accent_orange: Color32::from_rgb(0x3e, 0x8f, 0xb0),     // #3e8fb0
            accent_link: Color32::from_rgb(0x9c, 0xcf, 0xd8),       // #9ccfd8
            markup_bold_fg: Color32::from_rgb(0xc4, 0xa7, 0xe7),    // #c4a7e7
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
            bg_code_block: Color32::from_rgb(0xe0, 0xe0, 0xe0),     // #e0e0e0
            bg_code_inline: Color32::from_rgb(0xeb, 0xeb, 0xeb),    // #ebebeb
            bg_pill_button: Color32::from_rgb(0xf1, 0xf1, 0xf1),    // #f1f1f1
            bg_popover: Color32::from_rgb(0xff, 0xff, 0xff),        // #ffffff
            bg_preview: Color32::from_rgb(0xfa, 0xfa, 0xfa),        // #fafafa
            divider_preview: Color32::from_rgb(0xe4, 0xe4, 0xe4),   // #e4e4e4
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),     // #db632c
            accent_link: Color32::from_rgb(0x1c, 0x71, 0xd8),       // #1c71d8
            markup_bold_fg: Color32::from_rgb(0x2e, 0x34, 0x36),    // #2e3436
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
            bg_code_block: Color32::from_rgb(0xdf, 0xd7, 0xc9),     // #dfd7c9
            bg_code_inline: Color32::from_rgb(0xe5, 0xdd, 0xd0),    // #e5ddd0
            bg_pill_button: Color32::from_rgb(0xec, 0xe5, 0xda),    // #ece5da
            bg_popover: Color32::from_rgb(0xf9, 0xf3, 0xe9),        // #f9f3e9
            bg_preview: Color32::from_rgb(0xf2, 0xeb, 0xe0),        // #f2ebe0
            divider_preview: Color32::from_rgb(0xda, 0xd5, 0xcb),   // #dad5cb
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),     // #db632c
            accent_link: Color32::from_rgb(0x00, 0x80, 0x80),       // #008080
            markup_bold_fg: Color32::from_rgb(0x7a, 0x33, 0x10),    // #7a3310
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

    /// Classic — Semi-Dark (*Soft Anthracite*)
    pub const fn semi_dark() -> Self {
        Self {
            bg_canvas: Color32::from_rgb(0x2e, 0x31, 0x36),         // #2e3136
            bg_headerbar: Color32::from_rgb(0x28, 0x2b, 0x30),      // #282b30
            border_headerbar: Color32::from_rgb(0x3c, 0x40, 0x47),  // #3c4047
            fg_primary: Color32::from_rgb(0xe6, 0xe8, 0xeb),        // #e6e8eb
            fg_heading: Color32::from_rgb(0x96, 0xc5, 0xf8),        // #96c5f8
            fg_subheading: Color32::from_rgb(0xf5, 0xa6, 0x76),     // #f5a676
            fg_muted: Color32::from_rgb(0x8a, 0x90, 0x99),          // #8a9099
            bg_code_block: Color32::from_rgb(0x38, 0x3c, 0x42),     // #383c42
            bg_code_inline: Color32::from_rgb(0x3d, 0x41, 0x48),    // #3d4148
            bg_pill_button: Color32::from_rgb(0x3c, 0x40, 0x47),    // #3c4047
            bg_popover: Color32::from_rgb(0x35, 0x39, 0x40),        // #353940
            bg_preview: Color32::from_rgb(0x2a, 0x2d, 0x32),        // #2a2d32
            divider_preview: Color32::from_rgb(0x3c, 0x40, 0x47),   // #3c4047
            accent_orange: Color32::from_rgb(0xe0, 0x6c, 0x35),     // #e06c35
            accent_link: Color32::from_rgb(0x82, 0xb4, 0xf0),       // #82b4f0
            markup_bold_fg: Color32::from_rgb(0xe6, 0xe8, 0xeb),    // #e6e8eb
            markup_emphasis_fg: Color32::from_rgb(0xec, 0xbe, 0x24),// #ecbe24
            markup_code_fg: Color32::from_rgb(0xe6, 0x78, 0x80),    // #e67880
            markup_codeblock_fg: Color32::from_rgb(0x82, 0xb4, 0xf0),// #82b4f0
            markup_list_marker: Color32::from_rgb(0xec, 0xbe, 0x24),// #ecbe24
            markup_task_done: Color32::from_rgb(0x62, 0xe0, 0x92),  // #62e092
            markup_quote_bar: Color32::from_rgb(0xe0, 0x6c, 0x35),  // #e06c35
            markup_quote_fg: Color32::from_rgb(0xc0, 0xc4, 0xcc),   // #c0c4cc
            markup_link_url: Color32::from_rgb(0x68, 0x91, 0xc2),   // #6891c2
            bg_table_header: Color32::from_rgb(0x39, 0x3d, 0x45),   // #393d45
            spellcheck_red: Color32::from_rgb(0xc4, 0x45, 0x4b),    // #c4454b
            scrollbar_thumb: Color32::from_rgb(0x5c, 0x61, 0x6a),   // #5c616a
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
            bg_code_block: Color32::from_rgb(0x3c, 0x3c, 0x42),     // #3c3c42
            bg_code_inline: Color32::from_rgb(0x3a, 0x3a, 0x40),    // #3a3a40
            bg_pill_button: Color32::from_rgb(0x3a, 0x3a, 0x3d),    // #3a3a3d
            bg_popover: Color32::from_rgb(0x36, 0x36, 0x3a),        // #36363a
            bg_preview: Color32::from_rgb(0x26, 0x26, 0x2a),        // #26262a
            divider_preview: Color32::from_rgb(0x16, 0x16, 0x1a),   // #16161a
            accent_orange: Color32::from_rgb(0xdb, 0x63, 0x2c),     // #db632c
            accent_link: Color32::from_rgb(0x78, 0xae, 0xed),       // #78aeed
            markup_bold_fg: Color32::from_rgb(0xee, 0xee, 0xee),    // #eeeeee
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

        let semi_dark = AccentPalette::semi_dark();
        assert_eq!(semi_dark.bg_canvas, Color32::from_rgb(0x2e, 0x31, 0x36));
        assert_eq!(semi_dark.fg_primary, Color32::from_rgb(0xe6, 0xe8, 0xeb));

        let dark = AccentPalette::dark();
        assert_eq!(dark.bg_canvas, Color32::from_rgb(0x2b, 0x2b, 0x2f));
        assert_eq!(dark.bg_headerbar, Color32::from_rgb(0x2e, 0x2e, 0x32));
        assert_eq!(dark.fg_primary, Color32::from_rgb(0xee, 0xee, 0xee));
        assert_eq!(dark.bg_code_block, Color32::from_rgb(0x3c, 0x3c, 0x42));
    }

    #[test]
    fn test_css_tokens_serialization_for_all_20_preset_mode_pairs() {
        let presets = [
            ColorPalettePreset::AmberCoral,
            ColorPalettePreset::EditorialSlate,
            ColorPalettePreset::NordicForest,
            ColorPalettePreset::VelvetDawn,
            ColorPalettePreset::Classic,
        ];
        let modes = [
            ThemeMode::Light,
            ThemeMode::Sepia,
            ThemeMode::SemiDark,
            ThemeMode::Dark,
        ];
        for preset in presets {
            for mode in modes {
                let pal = AccentPalette::for_preset(preset, mode);
                let css = pal.to_css_tokens();
                assert!(css.bg_canvas.starts_with('#') && css.bg_canvas.len() == 7);
                assert!(css.fg_primary.starts_with('#') && css.fg_primary.len() == 7);
                assert!(css.accent_orange.starts_with('#') && css.accent_orange.len() == 7);
            }
        }
    }
}
