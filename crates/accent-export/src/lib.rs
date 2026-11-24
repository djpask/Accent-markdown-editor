use accent_core::{
    DocumentBuffer, MarkdownConfig, PreviewBlock, PreviewInline, TableAlignment,
};
use accent_mermaid::{MermaidCache, MermaidTheme};
use docx_rs::{
    AlignmentType, BreakType, Docx, Hyperlink, HyperlinkType, Paragraph, Pic, Run, RunFonts, Table,
    TableCell, TableCellBorder, TableCellBorderPosition, TableRow, WidthType,
};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExportFormat {
    #[default]
    Pdf,
    Docx,
    Odt,
    Latex,
    Epub,
    Html,
}

impl ExportFormat {
    pub fn from_code(code: &str) -> Self {
        match code.trim().to_ascii_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "docx" | "word" => Self::Docx,
            "odt" | "opendocument" => Self::Odt,
            "tex" | "latex" => Self::Latex,
            "epub" => Self::Epub,
            "html" | "htm" => Self::Html,
            _ => Self::Pdf,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Odt => "odt",
            Self::Latex => "tex",
            Self::Epub => "epub",
            Self::Html => "html",
        }
    }

    pub fn extension(self) -> &'static str {
        self.code()
    }

    pub fn sidebar_label(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Docx => "Microsoft Word (docx)",
            Self::Odt => "LibreOffice Text Document",
            Self::Latex => "LaTeX (tex)",
            Self::Epub => "EPUB v3",
            Self::Html => "HTML",
        }
    }
}

// ============================================================================
// 0. LOCAL RELATIVE IMAGE RESOLUTION (§6.7), SVG SANITIZATION & BASE64 DATA URI
// ============================================================================

pub const ALLOWED_LOCAL_IMAGE_EXTENSIONS: [&str; 9] = [
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLocalImageFile {
    pub canonical_path: PathBuf,
    pub ext_lower: String,
    pub mime: &'static str,
}

pub fn mime_for_image_ext(ext_lower: &str) -> Option<&'static str> {
    match ext_lower {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "svg" => Some("image/svg+xml"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn percent_decode_utf8(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Encodes `file_path` using the exact `encodeURIComponent` rules expected by Tauri v2's `asset:` protocol
/// (`asset://localhost/<encoded>` on Linux/macOS, `http://asset.localhost/<encoded>` on Windows).
pub fn convert_canonical_path_to_tauri_asset_url(canonical_path: &Path) -> String {
    let path_str = canonical_path.to_string_lossy();
    let mut encoded = String::with_capacity(path_str.len() + 32);
    for &b in path_str.as_bytes() {
        let unescaped = b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
            );
        if unescaped {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{b:02X}"));
        }
    }
    if cfg!(target_os = "windows") {
        format!("http://asset.localhost/{encoded}")
    } else {
        format!("asset://localhost/{encoded}")
    }
}

fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(s) = path.to_str() {
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                if !stripped.starts_with(r"UNC\") {
                    return PathBuf::from(stripped);
                }
            }
        }
    }
    path
}

fn has_uri_scheme(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return false;
    }
    for (idx, &b) in bytes.iter().enumerate().skip(1) {
        if b == b':' {
            // Single letter followed by `:\` or `:/` is a Windows drive path, which is also non-relative
            return idx >= 1;
        }
        if !(b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.') {
            return false;
        }
    }
    false
}

/// Resolves a Markdown/HTML local relative image destination (`diagram.png`, `./assets/img.jpg`,
/// `../img/logo.png`, including spaces and percent-encoding) against `base_dir`, canonicalizing
/// the path on disk and enforcing the allowed image extension whitelist.
pub fn resolve_local_image_file(
    raw_url: &str,
    base_dir: Option<&Path>,
) -> Option<ResolvedLocalImageFile> {
    let base = base_dir?;
    let unescaped_attr = html_unescape_basic(raw_url);
    let mut trimmed = unescaped_attr.trim();
    if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() >= 2 {
        trimmed = trimmed[1..trimmed.len() - 1].trim();
    }
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || has_uri_scheme(trimmed)
        || trimmed.chars().any(|c| c.is_control())
    {
        return None;
    }

    // Strip optional `?query` and `#fragment` suffix
    let without_frag = trimmed.split('#').next().unwrap_or(trimmed);
    let clean_rel = without_frag.split('?').next().unwrap_or(without_frag).trim();
    if clean_rel.is_empty() {
        return None;
    }

    let decoded = percent_decode_utf8(clean_rel);
    if decoded.starts_with('/')
        || decoded.starts_with('\\')
        || has_uri_scheme(&decoded)
        || decoded.chars().any(|c| c.is_control())
    {
        return None;
    }

    let rel_decoded = Path::new(&decoded);
    if !rel_decoded.is_relative() {
        return None;
    }

    let candidate_decoded = base.join(rel_decoded);
    let canonical = std::fs::canonicalize(&candidate_decoded)
        .or_else(|_| {
            let rel_raw = Path::new(clean_rel);
            if rel_raw.is_relative() {
                std::fs::canonicalize(base.join(rel_raw))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "not relative",
                ))
            }
        })
        .ok()?;

    let canonical = strip_windows_verbatim_prefix(canonical);
    if !canonical.is_file() {
        return None;
    }

    let ext_lower = canonical
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    let mime = mime_for_image_ext(&ext_lower)?;

    Some(ResolvedLocalImageFile {
        canonical_path: canonical,
        ext_lower,
        mime,
    })
}

/// Strips dangerous elements (`<script>`, `<foreignObject>`, `<iframe>`, `<object>`, `<embed>`)
/// together with their inner payloads, plus inline `on*` event handlers and `javascript:` URIs
/// from SVG markup so untrusted `.svg` assets cannot execute scripts.
pub fn sanitize_svg_markup(svg_raw: &str) -> String {
    let mut cleaned = svg_raw.to_string();
    for forbidden_tag in ["script", "foreignobject", "iframe", "object", "embed"] {
        loop {
            let lower = cleaned.to_ascii_lowercase();
            let open_pat = format!("<{forbidden_tag}");
            let Some(start) = lower.find(&open_pat) else {
                break;
            };
            let after_name = start + open_pat.len();
            if after_name < lower.len() {
                let next_b = lower.as_bytes()[after_name];
                if !(next_b.is_ascii_whitespace() || next_b == b'>' || next_b == b'/') {
                    // Prefix of a benign tag (e.g. `<feComposite` vs `<embed`): advance safely
                    let mut rebuilt = String::with_capacity(cleaned.len());
                    rebuilt.push_str(&cleaned[..after_name]);
                    rebuilt.push_str(&sanitize_svg_markup(&cleaned[after_name..]));
                    return rebuilt;
                }
            }
            let close_pat = format!("</{forbidden_tag}");
            if let Some(rel_gt) = lower[start..].find('>') {
                let tag_end = start + rel_gt;
                let self_closing = tag_end > start && lower.as_bytes()[tag_end - 1] == b'/';
                if self_closing {
                    cleaned.replace_range(start..=tag_end, "");
                    continue;
                }
                if let Some(rel_close) = lower[tag_end + 1..].find(&close_pat) {
                    let close_pos = tag_end + 1 + rel_close;
                    if let Some(close_gt) = lower[close_pos..].find('>') {
                        cleaned.replace_range(start..=(close_pos + close_gt), "");
                        continue;
                    }
                }
                cleaned.replace_range(start..=tag_end, "");
            } else {
                cleaned.truncate(start);
                break;
            }
        }
    }
    accent_core::sanitize_html_for_webview(&cleaned)
}

/// Standard RFC 4648 Base64 encoder (no external crate needed).
pub fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut chunks = input.chunks_exact(3);
    for chunk in &mut chunks {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        out.push(TABLE[(n & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    if rem.len() == 1 {
        let n = u32::from(rem[0]) << 16;
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem.len() == 2 {
        let n = (u32::from(rem[0]) << 16) | (u32::from(rem[1]) << 8);
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}

/// Reads a resolved local image file and encodes it as a safe `data:image/...;base64,...` URI
/// (sanitizing `.svg` files before encoding).
pub fn load_local_image_as_data_uri(resolved: &ResolvedLocalImageFile) -> Option<String> {
    if resolved.ext_lower == "svg" {
        let raw_svg = std::fs::read_to_string(&resolved.canonical_path).ok()?;
        let clean_svg = sanitize_svg_markup(&raw_svg);
        Some(format!(
            "data:image/svg+xml;base64,{}",
            base64_encode(clean_svg.as_bytes())
        ))
    } else {
        let bytes = std::fs::read(&resolved.canonical_path).ok()?;
        Some(format!(
            "data:{};base64,{}",
            resolved.mime,
            base64_encode(&bytes)
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlImageRewriteMode {
    /// WebView Preview Pane: converts valid raster files to Tauri `asset:` URLs, sanitizes `.svg`
    /// into `data:image/svg+xml;base64,...`, and replaces missing/unsaved/remote images with a clean
    /// `.preview-image-fallback` badge so the offline WebView never shows raw broken-image icons.
    WebViewAsset,
    /// Standalone HTML Export (`ExportFormat::Html`): embeds all resolved local images (raster & sanitized SVG)
    /// as self-contained `data:image/...;base64,...` URIs so the `.html` file works in any external browser.
    StandaloneDataUri,
}

fn extract_html_attr(tag_inner: &str, target_attr: &str) -> Option<String> {
    let bytes = tag_inner.as_bytes();
    let mut p = 0;
    while p < bytes.len() && !bytes[p].is_ascii_whitespace() && bytes[p] != b'/' {
        p += 1;
    }
    while p < bytes.len() {
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if p >= bytes.len() || bytes[p] == b'/' {
            break;
        }
        let name_s = p;
        while p < bytes.len()
            && !bytes[p].is_ascii_whitespace()
            && bytes[p] != b'='
            && bytes[p] != b'/'
        {
            p += 1;
        }
        let name = &tag_inner[name_s..p];
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if p < bytes.len() && bytes[p] == b'=' {
            p += 1;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                p += 1;
            }
            let val = if p < bytes.len() && (bytes[p] == b'"' || bytes[p] == b'\'') {
                let q = bytes[p];
                p += 1;
                let vs = p;
                while p < bytes.len() && bytes[p] != q {
                    p += 1;
                }
                let slice = &tag_inner[vs..p];
                if p < bytes.len() && bytes[p] == q {
                    p += 1;
                }
                slice
            } else {
                let vs = p;
                while p < bytes.len() && !bytes[p].is_ascii_whitespace() && bytes[p] != b'/' {
                    p += 1;
                }
                &tag_inner[vs..p]
            };
            if name.eq_ignore_ascii_case(target_attr) {
                return Some(html_unescape_basic(val));
            }
        } else if name.is_empty() {
            p += 1;
        }
    }
    None
}

fn replace_img_src_in_tag(tag_inner: &str, new_src: &str) -> String {
    let alt = extract_html_attr(tag_inner, "alt").unwrap_or_default();
    let title = extract_html_attr(tag_inner, "title");
    let width = extract_html_attr(tag_inner, "width");
    let height = extract_html_attr(tag_inner, "height");
    let mut out = String::from("<img src=\"");
    out.push_str(&xml_escape(new_src));
    out.push_str("\" alt=\"");
    out.push_str(&xml_escape(&alt));
    out.push('"');
    if let Some(t) = title.filter(|s| !s.is_empty()) {
        out.push_str(" title=\"");
        out.push_str(&xml_escape(&t));
        out.push('"');
    }
    if let Some(w) = width.filter(|s| !s.is_empty()) {
        out.push_str(" width=\"");
        out.push_str(&xml_escape(&w));
        out.push('"');
    }
    if let Some(h) = height.filter(|s| !s.is_empty()) {
        out.push_str(" height=\"");
        out.push_str(&xml_escape(&h));
        out.push('"');
    }
    out.push_str(" />");
    out
}

fn build_html_image_fallback_badge(alt: &str, raw_src: &str, title: Option<&str>) -> String {
    let label = if !alt.trim().is_empty() {
        alt.trim()
    } else if !raw_src.trim().is_empty() {
        raw_src.trim()
    } else {
        "Image"
    };
    let tooltip = match title.filter(|t| !t.trim().is_empty()) {
        Some(t) if !raw_src.trim().is_empty() => format!("{t} ({})", raw_src.trim()),
        Some(t) => t.to_string(),
        None => raw_src.trim().to_string(),
    };
    let esc_label = xml_escape(label);
    let esc_tooltip = xml_escape(&tooltip);
    format!(
        "<span class=\"preview-image-fallback\" data-testid=\"preview-image-fallback\" role=\"img\" aria-label=\"{esc_label}\" title=\"{esc_tooltip}\"><span class=\"preview-image-fallback-icon\" aria-hidden=\"true\">🖼</span><span class=\"preview-image-fallback-alt\">{esc_label}</span></span>"
    )
}

fn is_safe_inline_raster_data_uri(src: &str) -> bool {
    let lower = src.trim().to_ascii_lowercase();
    lower.starts_with("data:image/png;base64,")
        || lower.starts_with("data:image/jpeg;base64,")
        || lower.starts_with("data:image/gif;base64,")
        || lower.starts_with("data:image/webp;base64,")
        || lower.starts_with("data:image/bmp;base64,")
        || lower.starts_with("data:image/x-icon;base64,")
        || lower.starts_with("data:image/avif;base64,")
}

/// Rewrites `<img ...>` tags in sanitized HTML outside `<code>`/`<pre>` blocks according to `mode`.
pub fn rewrite_html_images_with_callback<F>(
    html: &str,
    base_dir: Option<&Path>,
    mode: HtmlImageRewriteMode,
    mut on_allowed_asset_file: F,
) -> String
where
    F: FnMut(&Path),
{
    let mut out = String::with_capacity(html.len() + 512);
    let bytes = html.as_bytes();
    let mut i = 0;
    let mut code_depth = 0usize;

    while i < bytes.len() {
        let Some(rel_lt) = html[i..].find('<') else {
            out.push_str(&html[i..]);
            break;
        };
        let lt = i + rel_lt;
        if lt > i {
            out.push_str(&html[i..lt]);
        }

        let tail = &html[lt..];
        let tb = tail.as_bytes();
        let mut quote: Option<u8> = None;
        let mut gt_rel: Option<usize> = None;
        for (k, &b) in tb.iter().enumerate().skip(1) {
            if let Some(q) = quote {
                if b == q {
                    quote = None;
                }
            } else if b == b'"' || b == b'\'' {
                quote = Some(b);
            } else if b == b'>' {
                gt_rel = Some(k);
                break;
            }
        }

        let Some(gt) = gt_rel else {
            out.push_str(&tail[..1]);
            i = lt + 1;
            continue;
        };

        let full_tag = &tail[..=gt];
        let inner = &tail[1..gt];
        let inner_trim = inner.trim();

        if inner_trim
            .get(..4)
            .is_some_and(|p| p.eq_ignore_ascii_case("code"))
            && inner_trim
                .as_bytes()
                .get(4)
                .is_none_or(|b| b.is_ascii_whitespace() || *b == b'/')
            && !inner_trim.ends_with('/')
        {
            code_depth += 1;
            out.push_str(full_tag);
            i = lt + gt + 1;
            continue;
        }
        if inner_trim
            .get(..5)
            .is_some_and(|p| p.eq_ignore_ascii_case("/code"))
        {
            code_depth = code_depth.saturating_sub(1);
            out.push_str(full_tag);
            i = lt + gt + 1;
            continue;
        }

        let is_img = code_depth == 0
            && inner_trim
                .get(..3)
                .is_some_and(|p| p.eq_ignore_ascii_case("img"))
            && inner_trim
                .as_bytes()
                .get(3)
                .is_none_or(|b| b.is_ascii_whitespace() || *b == b'/');

        if is_img {
            let raw_src = extract_html_attr(inner_trim, "src").unwrap_or_default();
            let alt = extract_html_attr(inner_trim, "alt").unwrap_or_default();
            let title = extract_html_attr(inner_trim, "title");

            if is_safe_inline_raster_data_uri(&raw_src) {
                out.push_str(full_tag);
            } else if let Some(resolved) = resolve_local_image_file(&raw_src, base_dir) {
                match mode {
                    HtmlImageRewriteMode::WebViewAsset => {
                        if resolved.ext_lower == "svg" {
                            if let Some(data_uri) = load_local_image_as_data_uri(&resolved) {
                                out.push_str(&replace_img_src_in_tag(inner_trim, &data_uri));
                            } else {
                                out.push_str(&build_html_image_fallback_badge(
                                    &alt,
                                    &raw_src,
                                    title.as_deref(),
                                ));
                            }
                        } else {
                            on_allowed_asset_file(&resolved.canonical_path);
                            let asset_url =
                                convert_canonical_path_to_tauri_asset_url(&resolved.canonical_path);
                            out.push_str(&replace_img_src_in_tag(inner_trim, &asset_url));
                        }
                    }
                    HtmlImageRewriteMode::StandaloneDataUri => {
                        if let Some(data_uri) = load_local_image_as_data_uri(&resolved) {
                            out.push_str(&replace_img_src_in_tag(inner_trim, &data_uri));
                        } else {
                            out.push_str(&build_html_image_fallback_badge(
                                &alt,
                                &raw_src,
                                title.as_deref(),
                            ));
                        }
                    }
                }
            } else {
                let lower_src = raw_src.trim().to_ascii_lowercase();
                let is_remote_http =
                    lower_src.starts_with("https://") || lower_src.starts_with("http://");
                if mode == HtmlImageRewriteMode::StandaloneDataUri && is_remote_http {
                    out.push_str(full_tag);
                } else {
                    out.push_str(&build_html_image_fallback_badge(
                        &alt,
                        &raw_src,
                        title.as_deref(),
                    ));
                }
            }
            i = lt + gt + 1;
            continue;
        }

        out.push_str(full_tag);
        i = lt + gt + 1;
    }

    out
}

#[derive(Debug, Clone)]
struct DecodedRasterImage {
    width: u32,
    height: u32,
    /// Alpha-composited over white paper (`#FFFFFF`) for clean PDF `DeviceRGB` rendering.
    rgb_bytes: Vec<u8>,
    /// Valid PNG stream for DOCX `Pic::new` and ODT/EPUB raster fallback if needed.
    png_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct LoadedExportImage {
    ext_lower: String,
    mime: &'static str,
    /// Sanitized UTF-8 bytes if `.svg`, otherwise original file bytes.
    raw_bytes: Vec<u8>,
    raster: Option<DecodedRasterImage>,
}

fn rgba_to_white_composited_rgb(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgb = Vec::with_capacity((width as usize) * (height as usize) * 3);
    for px in rgba.chunks_exact(4) {
        let a = u16::from(px[3]);
        let inv = 255u16.saturating_sub(a);
        let r = ((u16::from(px[0]) * a + 255 * inv + 127) / 255) as u8;
        let g = ((u16::from(px[1]) * a + 255 * inv + 127) / 255) as u8;
        let b = ((u16::from(px[2]) * a + 255 * inv + 127) / 255) as u8;
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    rgb
}

fn load_local_image_for_export(raw_url: &str, base_dir: Option<&Path>) -> Option<LoadedExportImage> {
    let resolved = resolve_local_image_file(raw_url, base_dir)?;
    if resolved.ext_lower == "svg" {
        let raw_svg = std::fs::read_to_string(&resolved.canonical_path).ok()?;
        let clean_svg = sanitize_svg_markup(&raw_svg);
        let raster = accent_mermaid::rasterize_svg(&clean_svg, 1.0).map(
            |(width, height, rgba_pixels, png_bytes)| DecodedRasterImage {
                width,
                height,
                rgb_bytes: rgba_to_white_composited_rgb(&rgba_pixels, width, height),
                png_bytes,
            },
        );
        return Some(LoadedExportImage {
            ext_lower: resolved.ext_lower,
            mime: resolved.mime,
            raw_bytes: clean_svg.into_bytes(),
            raster,
        });
    }

    let raw_bytes = std::fs::read(&resolved.canonical_path).ok()?;
    let raster = image::load_from_memory(&raw_bytes).ok().and_then(|dyn_img| {
        let rgba = dyn_img.to_rgba8();
        let (width, height) = rgba.dimensions();
        if width == 0 || height == 0 {
            return None;
        }
        let rgb_bytes = rgba_to_white_composited_rgb(rgba.as_raw(), width, height);
        let png_bytes = if resolved.ext_lower == "png" {
            raw_bytes.clone()
        } else {
            let mut png_cursor = Cursor::new(Vec::new());
            dyn_img
                .write_to(&mut png_cursor, image::ImageFormat::Png)
                .ok()?;
            png_cursor.into_inner()
        };
        Some(DecodedRasterImage {
            width,
            height,
            rgb_bytes,
            png_bytes,
        })
    });

    Some(LoadedExportImage {
        ext_lower: resolved.ext_lower,
        mime: resolved.mime,
        raw_bytes,
        raster,
    })
}

fn format_missing_image_fallback(alt: &str, url: &str) -> String {
    let clean_alt = alt.trim();
    let clean_url = url.trim();
    if !clean_alt.is_empty() && !clean_url.is_empty() {
        format!("[Immagine: {clean_alt} ({clean_url})]")
    } else if !clean_alt.is_empty() {
        format!("[Immagine: {clean_alt}]")
    } else {
        format!("[Immagine: {clean_url}]")
    }
}

fn html_unescape_basic(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Unified HTML preview/copy/export renderer built on `accent-core`'s `comrak` + GFM `§6.11` pipeline.
/// Renders fenced `mermaid` blocks into themed SVGs ONLY when `config.mermaid_enabled()` is `true`.
pub fn render_markdown_to_html_with_mermaid<F>(
    markdown: &str,
    config: MarkdownConfig,
    mut render_mermaid_svg: F,
) -> String
where
    F: FnMut(&str) -> String,
{
    let raw_html = accent_core::render_webview_html(markdown, config);
    let prefix = "<pre><code class=\"language-";
    let close_tag = "</code></pre>";
    let mut out = String::with_capacity(raw_html.len() + 1024);
    let mut rest = raw_html.as_str();

    while let Some(start) = rest.find(prefix) {
        out.push_str(&rest[..start]);
        let after_prefix = &rest[start + prefix.len()..];
        if let Some(quote_end) = after_prefix.find('"') {
            let lang = &after_prefix[..quote_end];
            let after_quote = &after_prefix[quote_end + 1..];
            if let Some(gt_rel) = after_quote.find('>') {
                let extra_code_attrs = &after_quote[..gt_rel];
                let after_open = &after_quote[gt_rel + 1..];

                if lang.eq_ignore_ascii_case("mermaid") && config.mermaid_enabled() {
                    if let Some(end) = after_open.find(close_tag) {
                        let escaped_code = &after_open[..end];
                        let code = html_unescape_basic(escaped_code);
                        let svg = render_mermaid_svg(&code);
                        out.push_str(
                            "<div class=\"accent-mermaid-block\" data-testid=\"mermaid-svg-block\">",
                        );
                        out.push_str(&svg);
                        out.push_str("</div>");
                        rest = &after_open[end + close_tag.len()..];
                        continue;
                    }
                } else if extra_code_attrs.contains("data-math-style") {
                    out.push_str("<pre><code class=\"language-");
                    out.push_str(lang);
                    out.push('"');
                    out.push_str(extra_code_attrs);
                    out.push('>');
                    rest = after_open;
                    continue;
                } else {
                    out.push_str("<pre data-code-lang=\"");
                    out.push_str(lang);
                    out.push_str("\"><code class=\"language-");
                    out.push_str(lang);
                    out.push('"');
                    out.push_str(extra_code_attrs);
                    out.push('>');
                    rest = after_open;
                    continue;
                }
            }
        }
        out.push_str(prefix);
        rest = after_prefix;
    }

    out.push_str(rest);
    out
}

/// Renders Markdown for Accent's WebView Preview Pane, resolving local relative images against `base_dir`
/// via `asset:` (or sanitized `data:image/svg+xml;base64,...` for `.svg`) and invoking `on_allowed_asset_file`
/// for each canonicalized local raster file.
pub fn render_markdown_to_webview_preview_html<FM, FA>(
    markdown: &str,
    config: MarkdownConfig,
    base_dir: Option<&Path>,
    render_mermaid_svg: FM,
    on_allowed_asset_file: FA,
) -> String
where
    FM: FnMut(&str) -> String,
    FA: FnMut(&Path),
{
    let html_with_mermaid = render_markdown_to_html_with_mermaid(markdown, config, render_mermaid_svg);
    rewrite_html_images_with_callback(
        &html_with_mermaid,
        base_dir,
        HtmlImageRewriteMode::WebViewAsset,
        on_allowed_asset_file,
    )
}

pub fn render_markdown_to_html(
    markdown: &str,
    config: MarkdownConfig,
    theme: MermaidTheme,
) -> String {
    let mut cache = MermaidCache::new();
    render_markdown_to_html_with_mermaid(markdown, config, |code| {
        cache.get_or_render_screen_themed(code, theme).svg.clone()
    })
}

/// Exports Markdown to a self-contained HTML5 document using the unified `comrak` pipeline,
/// embedding local relative images from `base_dir` as `data:image/...;base64,...` URIs.
pub fn export_markdown_to_standalone_html(
    markdown: &str,
    config: MarkdownConfig,
    title: &str,
) -> String {
    export_markdown_to_standalone_html_with_base_dir(markdown, config, title, None)
}

pub fn export_markdown_to_standalone_html_with_base_dir(
    markdown: &str,
    config: MarkdownConfig,
    title: &str,
    base_dir: Option<&Path>,
) -> String {
    let raw_body = render_markdown_to_html(markdown, config, MermaidTheme::Light);
    let body = rewrite_html_images_with_callback(
        &raw_body,
        base_dir,
        HtmlImageRewriteMode::StandaloneDataUri,
        |_| {},
    );
    let esc_title = xml_escape(title);
    format!(
        "<!DOCTYPE html>\n\
<html lang=\"it\">\n\
<head>\n\
  <meta charset=\"utf-8\"/>\n\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>\n\
  <title>{esc_title}</title>\n\
  <style>\n\
    body {{ font-family: system-ui, -apple-system, sans-serif; line-height: 1.6; max-width: 52rem; margin: 2rem auto; padding: 0 1.25rem; color: #2e3436; background: #ffffff; }}\n\
    .preview-prose {{ position: relative; isolation: isolate; contain: layout style paint; overflow-wrap: break-word; }}\n\
    h1, h2, h3, h4, h5, h6 {{ color: #1a202c; line-height: 1.3; }}\n\
    img {{ max-width: 100%; height: auto; border-radius: 6px; }}\n\
    pre {{ background: #f4f1ea; padding: 0.85rem 1rem; border-radius: 6px; overflow-x: auto; }}\n\
    code {{ font-family: monospace; background: #f4f1ea; padding: 0.15em 0.35em; border-radius: 4px; }}\n\
    pre code {{ background: none; padding: 0; }}\n\
    blockquote {{ margin: 1em 0; padding: 0.25em 0 0.25em 1em; border-left: 3px solid #c85a17; color: #4a5568; }}\n\
    hr {{ border: 0; border-top: 1px solid #cbd5e0; margin: 1.5em 0; }}\n\
    table {{ border-collapse: collapse; width: 100%; margin: 1em 0; }}\n\
    th, td {{ border: 1px solid #cbd5e0; padding: 0.45em 0.7em; }}\n\
    th:not([align]), td:not([align]) {{ text-align: left; }}\n\
    [align=\"center\"] {{ text-align: center; }}\n\
    [align=\"right\"] {{ text-align: right; }}\n\
    th {{ background: #edf2f7; font-weight: 700; }}\n\
    kbd {{ font-family: monospace; font-size: 0.88em; padding: 0.1em 0.4em; border: 1px solid #cbd5e0; border-radius: 4px; background: #f7fafc; }}\n\
    .markdown-alert {{ margin: 1em 0; padding: 0.65em 1em; border-left: 4px solid #3182ce; border-radius: 6px; background: #f7fafc; }}\n\
    .markdown-alert-title {{ margin: 0 0 0.35em 0; font-weight: 700; }}\n\
    .markdown-alert-tip {{ border-left-color: #2f855a; }}\n\
    .markdown-alert-important {{ border-left-color: #6b46c1; }}\n\
    .markdown-alert-warning {{ border-left-color: #c05621; }}\n\
    .markdown-alert-caution {{ border-left-color: #c53030; }}\n\
    .preview-image-fallback {{ display: inline-flex; align-items: center; gap: 0.4em; padding: 0.2em 0.65em; border-radius: 6px; border: 1px dashed #cbd5e0; background: #f7fafc; color: #4a5568; font-size: 0.9em; }}\n\
    .accent-mermaid-block {{ display: flex; justify-content: center; margin: 1em 0; overflow-x: auto; }}\n\
    .accent-mermaid-block svg {{ max-width: 100%; height: auto; }}\n\
  </style>\n\
</head>\n\
<body>\n\
<article class=\"preview-prose\">\n\
{body}</article>\n\
</body>\n\
</html>\n"
    )
}

/// Top-level dispatcher that exports a `DocumentBuffer` to the target `ExportFormat`,
/// resolving relative local images against `base_dir` (derived from `current_file_path`).
pub fn export_document_bytes(
    buffer: &DocumentBuffer,
    config: MarkdownConfig,
    title: &str,
    format: ExportFormat,
    base_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    if format == ExportFormat::Html {
        return Ok(export_markdown_to_standalone_html_with_base_dir(
            buffer.as_str(),
            config,
            title,
            base_dir,
        )
        .into_bytes());
    }
    let blocks = buffer.parse_preview_blocks(config);
    let allow_mermaid = config.mermaid_enabled();
    match format {
        ExportFormat::Pdf => Ok(export_to_pdf_with_options(
            &blocks,
            title,
            allow_mermaid,
            base_dir,
        )),
        ExportFormat::Docx => {
            export_to_docx_with_options(&blocks, title, allow_mermaid, base_dir)
        }
        ExportFormat::Odt => export_to_odt_with_options(&blocks, title, allow_mermaid, base_dir),
        ExportFormat::Latex => Ok(export_to_latex_with_options(&blocks, title, base_dir).into_bytes()),
        ExportFormat::Epub => {
            export_to_epub_with_options(&blocks, title, allow_mermaid, base_dir)
        }
        ExportFormat::Html => unreachable!(),
    }
}

// ============================================================================
// 1. PURE-RUST LaTeX (.tex) AST SERIALIZER WITH STRICT SPECIAL-CHAR ESCAPING
// ============================================================================

/// Strictly escapes all LaTeX special characters (`\`, `{`, `}`, `%`, `&`, `$`, `#`, `_`, `~`, `^`) in normal text.
pub fn escape_latex_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str(r"\textbackslash{}"),
            '{' => out.push_str(r"\{"),
            '}' => out.push_str(r"\}"),
            '%' => out.push_str(r"\%"),
            '&' => out.push_str(r"\&"),
            '$' => out.push_str(r"\$"),
            '#' => out.push_str(r"\#"),
            '_' => out.push_str(r"\_"),
            '~' => out.push_str(r"\textasciitilde{}"),
            '^' => out.push_str(r"\textasciicircum{}"),
            _ => out.push(ch),
        }
    }
    out
}

fn clean_latex_graphics_path(path: &Path) -> String {
    let forward = path.to_string_lossy().replace('\\', "/");
    let mut safe = String::with_capacity(forward.len() + 8);
    for ch in forward.chars() {
        match ch {
            '{' | '}' | '%' | '#' | '$' | '&' => {}
            _ => safe.push(ch),
        }
    }
    if safe.contains(' ') {
        format!("\"{safe}\"")
    } else {
        safe
    }
}

fn escape_latex_href(url: &str) -> String {
    let mut out = String::with_capacity(url.len() + 8);
    for ch in url.chars() {
        match ch {
            '%' => out.push_str(r"\%"),
            '#' => out.push_str(r"\#"),
            '{' => out.push_str("%7B"),
            '}' => out.push_str("%7D"),
            '\\' => out.push('/'),
            _ => out.push(ch),
        }
    }
    out
}

fn serialize_inlines_to_latex(
    inlines: &[PreviewInline],
    base_dir: Option<&Path>,
    in_table_cell: bool,
) -> String {
    let mut out = String::new();
    for span in inlines {
        match span {
            PreviewInline::Text {
                text,
                bold,
                italic,
                strikethrough,
                code,
                link_url,
                ..
            } => {
                let mut chunk = escape_latex_text(text);
                if *code {
                    chunk = format!(r"\texttt{{{chunk}}}");
                }
                if *bold {
                    chunk = format!(r"\textbf{{{chunk}}}");
                }
                if *italic {
                    chunk = format!(r"\textit{{{chunk}}}");
                }
                if *strikethrough {
                    chunk = format!(r"\sout{{{chunk}}}");
                }
                if let Some(ref url) = link_url {
                    let safe_url = escape_latex_href(url);
                    chunk = format!(r"\href{{{safe_url}}}{{{chunk}}}");
                }
                out.push_str(&chunk);
            }
            PreviewInline::LineBreak => {
                if in_table_cell {
                    out.push_str(r" \newline ");
                } else {
                    out.push_str("\\\\\n");
                }
            }
            PreviewInline::Image { alt, url, .. } => {
                if let Some(resolved) = resolve_local_image_file(url, base_dir) {
                    let clean_p = clean_latex_graphics_path(&resolved.canonical_path);
                    out.push_str(&format!(r"\includegraphics[height=1.4em]{{{clean_p}}}"));
                } else {
                    let fb = escape_latex_text(&format_missing_image_fallback(alt, url));
                    out.push_str(&format!(r"\textit{{{fb}}}"));
                }
            }
            PreviewInline::Math {
                literal,
                display_math,
            } => {
                if *display_math && !in_table_cell {
                    out.push_str(&format!(r"\[{literal}\]"));
                } else {
                    out.push_str(&format!("${literal}$"));
                }
            }
        }
    }
    out
}

fn render_latex_blocks(blocks: &[PreviewBlock], base_dir: Option<&Path>, tex: &mut String) {
    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let cmd = match level {
                    1 => "section",
                    2 => "subsection",
                    3 => "subsubsection",
                    4 => "paragraph",
                    _ => "subparagraph",
                };
                let inner = serialize_inlines_to_latex(inlines, base_dir, false);
                tex.push_str(&format!("\\{cmd}{{{inner}}}\n\n"));
            }
            PreviewBlock::Paragraph { inlines } => {
                let inner = serialize_inlines_to_latex(inlines, base_dir, false);
                tex.push_str(&format!("{inner}\n\n"));
            }
            PreviewBlock::ThematicBreak => {
                tex.push_str("\\par\\noindent\\rule{\\textwidth}{0.4pt}\\par\\medskip\n\n");
            }
            PreviewBlock::BlockQuote { alert, children } => {
                tex.push_str("\\begin{quote}\n");
                if let Some(kind) = alert {
                    tex.push_str(&format!("\\textbf{{[{}]}}\\par\n", kind.label()));
                }
                render_latex_blocks(children, base_dir, tex);
                tex.push_str("\\end{quote}\n\n");
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("mermaid") {
                    tex.push_str("% Mermaid Diagram Block\n");
                }
                tex.push_str("\\begin{lstlisting}\n");
                tex.push_str(code);
                tex.push_str("\n\\end{lstlisting}\n\n");
            }
            PreviewBlock::List {
                ordered,
                start,
                items,
                ..
            } => {
                let env = if *ordered { "enumerate" } else { "itemize" };
                tex.push_str(&format!("\\begin{{{env}}}\n"));
                if *ordered && *start != 1 {
                    let prev_num = (*start as isize) - 1;
                    tex.push_str(&format!("  \\setcounter{{enumi}}{{{prev_num}}}\n"));
                }
                for item in items {
                    let prefix = match item.checked {
                        Some(true) => "$\\boxtimes$ ",
                        Some(false) => "$\\square$ ",
                        None => "",
                    };
                    let inner = serialize_inlines_to_latex(&item.inlines, base_dir, false);
                    tex.push_str(&format!("  \\item {prefix}{inner}\n"));
                    if !item.children.is_empty() {
                        render_latex_blocks(&item.children, base_dir, tex);
                    }
                }
                tex.push_str(&format!("\\end{{{env}}}\n\n"));
            }
            PreviewBlock::Table {
                alignments,
                headers,
                rows,
            } => {
                let cols = headers
                    .len()
                    .max(rows.iter().map(Vec::len).max().unwrap_or(0))
                    .max(1);
                let mut col_spec = String::with_capacity(cols);
                for c_idx in 0..cols {
                    let ch = match alignments.get(c_idx).copied().unwrap_or_default() {
                        TableAlignment::None | TableAlignment::Left => 'l',
                        TableAlignment::Center => 'c',
                        TableAlignment::Right => 'r',
                    };
                    col_spec.push(ch);
                }
                tex.push_str(&format!("\\begin{{tabular}}{{{col_spec}}}\n\\toprule\n"));
                if !headers.is_empty() {
                    let mut h_cells = Vec::with_capacity(cols);
                    for c_idx in 0..cols {
                        let cell_tex = headers
                            .get(c_idx)
                            .map(|inl| serialize_inlines_to_latex(inl, base_dir, true))
                            .unwrap_or_default();
                        h_cells.push(format!("\\textbf{{{cell_tex}}}"));
                    }
                    tex.push_str(&format!("{} \\\\\n\\midrule\n", h_cells.join(" & ")));
                }
                for row in rows {
                    let mut r_cells = Vec::with_capacity(cols);
                    for c_idx in 0..cols {
                        let cell_tex = row
                            .get(c_idx)
                            .map(|inl| serialize_inlines_to_latex(inl, base_dir, true))
                            .unwrap_or_default();
                        r_cells.push(cell_tex);
                    }
                    tex.push_str(&format!("{} \\\\\n", r_cells.join(" & ")));
                }
                tex.push_str("\\bottomrule\n\\end{tabular}\n\n");
            }
            PreviewBlock::Image { alt, url, .. } => {
                let esc_alt = escape_latex_text(alt);
                if let Some(resolved) = resolve_local_image_file(url, base_dir) {
                    let clean_p = clean_latex_graphics_path(&resolved.canonical_path);
                    tex.push_str("\\begin{figure}[htbp]\n\\centering\n");
                    tex.push_str(&format!(
                        "\\includegraphics[width=0.8\\textwidth]{{{clean_p}}}\n"
                    ));
                    if !alt.trim().is_empty() {
                        tex.push_str(&format!("\\caption{{{esc_alt}}}\n"));
                    }
                    tex.push_str("\\end{figure}\n\n");
                } else {
                    let fb = escape_latex_text(&format_missing_image_fallback(alt, url));
                    tex.push_str(&format!("\\textit{{{fb}}}\n\n"));
                }
            }
            PreviewBlock::FootnoteDefinition { name, children } => {
                let esc_name = escape_latex_text(name);
                tex.push_str(&format!("\\paragraph{{Nota [{esc_name}]:}}\n"));
                render_latex_blocks(children, base_dir, tex);
            }
        }
    }
}

/// Serializes the AST `PreviewBlock`s into a complete, self-contained LaTeX `.tex` document.
pub fn export_to_latex(blocks: &[PreviewBlock], title: &str) -> String {
    export_to_latex_with_options(blocks, title, None)
}

pub fn export_to_latex_with_options(
    blocks: &[PreviewBlock],
    title: &str,
    base_dir: Option<&Path>,
) -> String {
    let mut tex = String::with_capacity(4096);
    let escaped_title = escape_latex_text(title);
    tex.push_str("\\documentclass[11pt,a4paper]{article}\n");
    tex.push_str("\\usepackage[utf8]{inputenc}\n");
    tex.push_str("\\usepackage[T1]{fontenc}\n");
    tex.push_str("\\usepackage{hyperref}\n");
    tex.push_str("\\usepackage{listings}\n");
    tex.push_str("\\usepackage{graphicx}\n");
    tex.push_str("\\usepackage[normalem]{ulem}\n");
    tex.push_str("\\usepackage{amsmath,amssymb}\n");
    tex.push_str("\\usepackage{booktabs}\n\n");
    tex.push_str(&format!("\\title{{{escaped_title}}}\n"));
    tex.push_str("\\date{\\today}\n\n");
    tex.push_str("\\begin{document}\n");
    tex.push_str("\\maketitle\n\n");

    render_latex_blocks(blocks, base_dir, &mut tex);

    tex.push_str("\\end{document}\n");
    tex
}

// ============================================================================
// 2. PURE-RUST WORD (.docx) EXPORTER VIA `docx-rs`
// ============================================================================

fn docx_emu_dimensions(width_px: u32, height_px: u32, max_width_emu: u32) -> (u32, u32) {
    let raw_w = width_px.max(1).saturating_mul(9525);
    let raw_h = height_px.max(1).saturating_mul(9525);
    if raw_w <= max_width_emu {
        (raw_w, raw_h)
    } else {
        let scale = (max_width_emu as f64) / (raw_w as f64);
        let scaled_h = ((raw_h as f64) * scale).round().max(9525.0) as u32;
        (max_width_emu, scaled_h)
    }
}

fn docx_alignment_for_table(align: TableAlignment) -> AlignmentType {
    match align {
        TableAlignment::None | TableAlignment::Left => AlignmentType::Left,
        TableAlignment::Center => AlignmentType::Center,
        TableAlignment::Right => AlignmentType::Right,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_docx_paragraph_from_inlines(
    inlines: &[PreviewInline],
    base_size_half_pt: Option<usize>,
    force_bold: bool,
    force_italic: bool,
    left_indent_twips: Option<i32>,
    alignment: Option<AlignmentType>,
    base_dir: Option<&Path>,
) -> Paragraph {
    let mut p = Paragraph::new();
    if let Some(ind) = left_indent_twips.filter(|&v| v > 0) {
        p = p.indent(Some(ind), None, None, None);
    }
    if let Some(al) = alignment {
        p = p.align(al);
    }

    for span in inlines {
        match span {
            PreviewInline::Text {
                text,
                bold,
                italic,
                strikethrough,
                code,
                link_url,
                ..
            } => {
                let mut r = Run::new().add_text(text);
                if force_bold || *bold {
                    r = r.bold();
                }
                if force_italic || *italic {
                    r = r.italic();
                }
                if *strikethrough {
                    r = r.strike();
                }
                if *code {
                    r = r.fonts(RunFonts::new().ascii("Courier New").hi_ansi("Courier New"));
                }
                if let Some(sz) = base_size_half_pt {
                    r = r.size(sz);
                }
                if let Some(ref url) = link_url {
                    r = r.color("25589E").underline("single");
                    let link = if let Some(anchor) = url.strip_prefix('#') {
                        Hyperlink::new(anchor, HyperlinkType::Anchor).add_run(r)
                    } else {
                        Hyperlink::new(url, HyperlinkType::External).add_run(r)
                    };
                    p = p.add_hyperlink(link);
                } else {
                    p = p.add_run(r);
                }
            }
            PreviewInline::LineBreak => {
                p = p.add_run(Run::new().add_break(BreakType::TextWrapping));
            }
            PreviewInline::Image { alt, url, .. } => {
                if let Some(img) = load_local_image_for_export(url, base_dir) {
                    if let Some(raster) = img.raster {
                        let (w_emu, h_emu) =
                            docx_emu_dimensions(raster.width, raster.height, 4_200_000);
                        let pic = Pic::new(&raster.png_bytes).size(w_emu, h_emu);
                        p = p.add_run(Run::new().add_image(pic));
                        continue;
                    }
                }
                let mut r = Run::new()
                    .add_text(format_missing_image_fallback(alt, url))
                    .italic();
                if let Some(sz) = base_size_half_pt {
                    r = r.size(sz);
                }
                p = p.add_run(r);
            }
            PreviewInline::Math { literal, .. } => {
                let mut r = Run::new()
                    .add_text(format!("${literal}$"))
                    .italic()
                    .fonts(RunFonts::new().ascii("Courier New").hi_ansi("Courier New"));
                if let Some(sz) = base_size_half_pt {
                    r = r.size(sz);
                }
                p = p.add_run(r);
            }
        }
    }
    p
}

enum DocxElement {
    Para(Paragraph),
    Tbl(Table),
}

fn append_docx_blocks(
    blocks: &[PreviewBlock],
    allow_mermaid: bool,
    base_dir: Option<&Path>,
    mermaid_cache: &mut MermaidCache,
    indent_level: usize,
    in_quote: bool,
    out: &mut Vec<DocxElement>,
) {
    let indent_twips = (indent_level as i32) * 360;
    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let sz = match level {
                    1 => 36,
                    2 => 30,
                    3 => 26,
                    _ => 24,
                };
                out.push(DocxElement::Para(build_docx_paragraph_from_inlines(
                    inlines,
                    Some(sz),
                    true,
                    in_quote,
                    Some(indent_twips),
                    None,
                    base_dir,
                )));
            }
            PreviewBlock::Paragraph { inlines } => {
                out.push(DocxElement::Para(build_docx_paragraph_from_inlines(
                    inlines,
                    Some(22),
                    false,
                    in_quote,
                    Some(indent_twips),
                    None,
                    base_dir,
                )));
            }
            PreviewBlock::ThematicBreak => {
                let hr_cell = TableCell::new()
                    .width(9000, WidthType::Dxa)
                    .clear_all_border()
                    .set_border(
                        TableCellBorder::new(TableCellBorderPosition::Bottom)
                            .size(6)
                            .color("CBD5E0"),
                    )
                    .add_paragraph(Paragraph::new());
                out.push(DocxElement::Tbl(Table::new(vec![TableRow::new(vec![
                    hr_cell,
                ])])));
            }
            PreviewBlock::BlockQuote { alert, children } => {
                if let Some(kind) = alert {
                    let alert_p = Paragraph::new()
                        .indent(Some(indent_twips + 360), None, None, None)
                        .add_run(
                            Run::new()
                                .add_text(format!("[{}]", kind.label()))
                                .bold()
                                .size(22),
                        );
                    out.push(DocxElement::Para(alert_p));
                }
                append_docx_blocks(
                    children,
                    allow_mermaid,
                    base_dir,
                    mermaid_cache,
                    indent_level + 1,
                    true,
                    out,
                );
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if allow_mermaid && lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if !rendered.png_bytes.is_empty() {
                        let (w_emu, h_emu) =
                            docx_emu_dimensions(rendered.width, rendered.height, 5_400_000);
                        let pic = Pic::new(&rendered.png_bytes).size(w_emu, h_emu);
                        let mut p = Paragraph::new();
                        if indent_twips > 0 {
                            p = p.indent(Some(indent_twips), None, None, None);
                        }
                        out.push(DocxElement::Para(p.add_run(Run::new().add_image(pic))));
                        continue;
                    }
                }
                for line in code.lines() {
                    let r = Run::new()
                        .add_text(line)
                        .fonts(RunFonts::new().ascii("Courier New").hi_ansi("Courier New"))
                        .size(19);
                    let mut p = Paragraph::new().add_run(r);
                    if indent_twips > 0 {
                        p = p.indent(Some(indent_twips), None, None, None);
                    }
                    out.push(DocxElement::Para(p));
                }
            }
            PreviewBlock::List {
                ordered,
                start,
                items,
                ..
            } => {
                for (idx, item) in items.iter().enumerate() {
                    let num = start.saturating_add(idx);
                    let bullet = if indent_level % 2 == 0 { "• " } else { "◦ " };
                    let prefix = match item.checked {
                        Some(true) if *ordered => format!("{num}. ☑ "),
                        Some(false) if *ordered => format!("{num}. ☐ "),
                        Some(true) => "☑ ".to_string(),
                        Some(false) => "☐ ".to_string(),
                        None if *ordered => format!("{num}. "),
                        None => bullet.to_string(),
                    };
                    let mut inlines_with_prefix = Vec::with_capacity(item.inlines.len() + 1);
                    inlines_with_prefix.push(PreviewInline::plain(prefix));
                    inlines_with_prefix.extend(item.inlines.iter().cloned());
                    out.push(DocxElement::Para(build_docx_paragraph_from_inlines(
                        &inlines_with_prefix,
                        Some(22),
                        false,
                        in_quote,
                        Some(indent_twips + 240),
                        None,
                        base_dir,
                    )));
                    if !item.children.is_empty() {
                        append_docx_blocks(
                            &item.children,
                            allow_mermaid,
                            base_dir,
                            mermaid_cache,
                            indent_level + 1,
                            in_quote,
                            out,
                        );
                    }
                }
            }
            PreviewBlock::Table {
                alignments,
                headers,
                rows,
            } => {
                let cols = headers
                    .len()
                    .max(rows.iter().map(Vec::len).max().unwrap_or(0))
                    .max(1);
                let mut t_rows = Vec::new();
                if !headers.is_empty() {
                    let mut cells = Vec::with_capacity(cols);
                    for c_idx in 0..cols {
                        let align = docx_alignment_for_table(
                            alignments.get(c_idx).copied().unwrap_or_default(),
                        );
                        let empty = Vec::new();
                        let inl = headers.get(c_idx).unwrap_or(&empty);
                        let p = build_docx_paragraph_from_inlines(
                            inl,
                            Some(21),
                            true,
                            false,
                            None,
                            Some(align),
                            base_dir,
                        );
                        cells.push(TableCell::new().add_paragraph(p));
                    }
                    t_rows.push(TableRow::new(cells));
                }
                for row in rows {
                    let mut cells = Vec::with_capacity(cols);
                    for c_idx in 0..cols {
                        let align = docx_alignment_for_table(
                            alignments.get(c_idx).copied().unwrap_or_default(),
                        );
                        let empty = Vec::new();
                        let inl = row.get(c_idx).unwrap_or(&empty);
                        let p = build_docx_paragraph_from_inlines(
                            inl,
                            Some(21),
                            false,
                            false,
                            None,
                            Some(align),
                            base_dir,
                        );
                        cells.push(TableCell::new().add_paragraph(p));
                    }
                    t_rows.push(TableRow::new(cells));
                }
                if !t_rows.is_empty() {
                    out.push(DocxElement::Tbl(Table::new(t_rows)));
                }
            }
            PreviewBlock::Image { alt, url, .. } => {
                if let Some(img) = load_local_image_for_export(url, base_dir) {
                    if let Some(raster) = img.raster {
                        let (w_emu, h_emu) =
                            docx_emu_dimensions(raster.width, raster.height, 5_400_000);
                        let pic = Pic::new(&raster.png_bytes).size(w_emu, h_emu);
                        let mut p = Paragraph::new();
                        if indent_twips > 0 {
                            p = p.indent(Some(indent_twips), None, None, None);
                        }
                        out.push(DocxElement::Para(p.add_run(Run::new().add_image(pic))));
                        continue;
                    }
                }
                let p = Paragraph::new().add_run(
                    Run::new()
                        .add_text(format_missing_image_fallback(alt, url))
                        .italic()
                        .size(21),
                );
                out.push(DocxElement::Para(p));
            }
            PreviewBlock::FootnoteDefinition { name, children } => {
                out.push(DocxElement::Para(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text(format!("[{name}]:"))
                            .bold()
                            .size(20),
                    ),
                ));
                append_docx_blocks(
                    children,
                    allow_mermaid,
                    base_dir,
                    mermaid_cache,
                    indent_level + 1,
                    in_quote,
                    out,
                );
            }
        }
    }
}

/// Generates a structured OpenXML `.docx` document using `docx-rs`, embedding Mermaid and local images.
pub fn export_to_docx(blocks: &[PreviewBlock], title: &str) -> Result<Vec<u8>, String> {
    export_to_docx_with_options(blocks, title, true, None)
}

pub fn export_to_docx_with_options(
    blocks: &[PreviewBlock],
    _title: &str,
    allow_mermaid: bool,
    base_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let mut docx = Docx::new();
    let mut mermaid_cache = MermaidCache::new();
    let mut elements = Vec::new();
    append_docx_blocks(
        blocks,
        allow_mermaid,
        base_dir,
        &mut mermaid_cache,
        0,
        false,
        &mut elements,
    );
    for el in elements {
        match el {
            DocxElement::Para(p) => docx = docx.add_paragraph(p),
            DocxElement::Tbl(t) => docx = docx.add_table(t),
        }
    }

    let mut cursor = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut cursor)
        .map_err(|e| format!("DOCX pack error: {e}"))?;
    Ok(cursor.into_inner())
}

// ============================================================================
// 3. PURE-RUST OPENDOCUMENT TEXT (.odt) ODF 1.2 EXPORTER VIA `zip`
// ============================================================================

fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

struct OdtMediaCollector {
    next_id: usize,
    entries: Vec<(String, &'static str, Vec<u8>)>,
}

impl OdtMediaCollector {
    fn new() -> Self {
        Self {
            next_id: 1,
            entries: Vec::new(),
        }
    }

    fn embed_bytes(&mut self, ext: &str, mime: &'static str, bytes: Vec<u8>) -> String {
        let id = self.next_id;
        self.next_id += 1;
        let path = format!("Pictures/image_{id}.{ext}");
        self.entries.push((path.clone(), mime, bytes));
        path
    }
}

fn odt_size_cm(width_px: u32, height_px: u32, max_w_cm: f32) -> (f32, f32) {
    // 96 DPI: 1 inch = 2.54 cm -> px * 2.54 / 96.0
    let w_cm = (width_px.max(1) as f32) * (2.54 / 96.0);
    let h_cm = (height_px.max(1) as f32) * (2.54 / 96.0);
    if w_cm <= max_w_cm {
        (w_cm.max(0.5), h_cm.max(0.5))
    } else {
        let ratio = max_w_cm / w_cm;
        (max_w_cm, (h_cm * ratio).max(0.5))
    }
}

fn serialize_inlines_to_odf(
    inlines: &[PreviewInline],
    base_dir: Option<&Path>,
    media: &mut OdtMediaCollector,
) -> String {
    let mut out = String::new();
    for span in inlines {
        match span {
            PreviewInline::Text {
                text,
                bold,
                italic,
                strikethrough,
                code,
                link_url,
                link_title,
            } => {
                let esc = xml_escape(text);
                let styled = if *code {
                    format!(r#"<text:span text:style-name="TCode">{esc}</text:span>"#)
                } else if *bold && *italic {
                    format!(r#"<text:span text:style-name="TBoldItalic">{esc}</text:span>"#)
                } else if *bold && *strikethrough {
                    format!(r#"<text:span text:style-name="TBoldStrike">{esc}</text:span>"#)
                } else if *bold {
                    format!(r#"<text:span text:style-name="TBold">{esc}</text:span>"#)
                } else if *italic {
                    format!(r#"<text:span text:style-name="TItalic">{esc}</text:span>"#)
                } else if *strikethrough {
                    format!(r#"<text:span text:style-name="TStrike">{esc}</text:span>"#)
                } else {
                    esc
                };
                if let Some(ref url) = link_url {
                    let esc_url = xml_escape(url);
                    let title_attr = link_title
                        .as_deref()
                        .filter(|t| !t.is_empty())
                        .map(|t| format!(r#" office:title="{}""#, xml_escape(t)))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        r#"<text:a xlink:type="simple" xlink:href="{esc_url}"{title_attr}>{styled}</text:a>"#
                    ));
                } else {
                    out.push_str(&styled);
                }
            }
            PreviewInline::LineBreak => {
                out.push_str("<text:line-break/>");
            }
            PreviewInline::Image { alt, url, .. } => {
                if let Some(img) = load_local_image_for_export(url, base_dir) {
                    let (w_cm, h_cm) = img
                        .raster
                        .as_ref()
                        .map(|r| odt_size_cm(r.width, r.height, 10.0))
                        .unwrap_or((6.0, 4.0));
                    let pic_path = media.embed_bytes(&img.ext_lower, img.mime, img.raw_bytes);
                    let esc_alt = xml_escape(alt);
                    out.push_str(&format!(
                        r#"<draw:frame text:anchor-type="as-char" svg:width="{w_cm:.2}cm" svg:height="{h_cm:.2}cm"><draw:image xlink:href="{pic_path}" xlink:type="simple" xlink:show="embed" xlink:actuate="onLoad"/><svg:title>{esc_alt}</svg:title></draw:frame>"#
                    ));
                } else {
                    out.push_str(&xml_escape(&format_missing_image_fallback(alt, url)));
                }
            }
            PreviewInline::Math { literal, .. } => {
                let esc = xml_escape(&format!("${literal}$"));
                out.push_str(&format!(
                    r#"<text:span text:style-name="TCode">{esc}</text:span>"#
                ));
            }
        }
    }
    out
}

fn odt_cell_para_style(align: TableAlignment, is_header: bool) -> &'static str {
    match (is_header, align) {
        (true, TableAlignment::Center) => "PCellHeaderCenter",
        (true, TableAlignment::Right) => "PCellHeaderRight",
        (true, _) => "PCellHeaderLeft",
        (false, TableAlignment::Center) => "PCellCenter",
        (false, TableAlignment::Right) => "PCellRight",
        (false, _) => "PCellLeft",
    }
}

#[allow(clippy::too_many_arguments)]
fn render_odt_blocks(
    blocks: &[PreviewBlock],
    allow_mermaid: bool,
    base_dir: Option<&Path>,
    mermaid_cache: &mut MermaidCache,
    media: &mut OdtMediaCollector,
    table_seq: &mut usize,
    in_quote: bool,
    body_xml: &mut String,
) {
    let default_p_style = if in_quote {
        "Quotations"
    } else {
        "Text_20_body"
    };

    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let lv = (*level).clamp(1, 6);
                let style_lv = lv.min(3);
                let inner = serialize_inlines_to_odf(inlines, base_dir, media);
                body_xml.push_str(&format!(
                    r#"<text:h text:style-name="Heading_20_{style_lv}" text:outline-level="{lv}">{inner}</text:h>"#
                ));
            }
            PreviewBlock::Paragraph { inlines } => {
                let inner = serialize_inlines_to_odf(inlines, base_dir, media);
                body_xml.push_str(&format!(
                    r#"<text:p text:style-name="{default_p_style}">{inner}</text:p>"#
                ));
            }
            PreviewBlock::ThematicBreak => {
                body_xml.push_str(r#"<text:p text:style-name="PHorizontalRule"/>"#);
            }
            PreviewBlock::BlockQuote { alert, children } => {
                if let Some(kind) = alert {
                    body_xml.push_str(&format!(
                        r#"<text:p text:style-name="Quotations"><text:span text:style-name="TBold">[{}]</text:span></text:p>"#,
                        xml_escape(kind.label())
                    ));
                }
                render_odt_blocks(
                    children,
                    allow_mermaid,
                    base_dir,
                    mermaid_cache,
                    media,
                    table_seq,
                    true,
                    body_xml,
                );
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if allow_mermaid && lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if !rendered.png_bytes.is_empty() {
                        let (w_cm, h_cm) = odt_size_cm(rendered.width, rendered.height, 15.0);
                        let pic_path =
                            media.embed_bytes("png", "image/png", rendered.png_bytes.clone());
                        body_xml.push_str(&format!(
                            r#"<text:p text:style-name="{default_p_style}"><draw:frame text:anchor-type="as-char" svg:width="{w_cm:.2}cm" svg:height="{h_cm:.2}cm"><draw:image xlink:href="{pic_path}" xlink:type="simple" xlink:show="embed" xlink:actuate="onLoad"/></draw:frame></text:p>"#
                        ));
                        continue;
                    }
                }
                for line in code.lines() {
                    body_xml.push_str(&format!(
                        r#"<text:p text:style-name="Preformatted_20_Text">{}</text:p>"#,
                        xml_escape(line)
                    ));
                }
            }
            PreviewBlock::List {
                ordered,
                start,
                items,
                ..
            } => {
                body_xml.push_str("<text:list>");
                for (idx, item) in items.iter().enumerate() {
                    let start_attr = if *ordered && idx == 0 && *start > 1 {
                        format!(r#" text:start-value="{start}""#)
                    } else {
                        String::new()
                    };
                    let num_prefix = if *ordered {
                        format!("{}. ", start.saturating_add(idx))
                    } else {
                        String::new()
                    };
                    let task_prefix = match item.checked {
                        Some(true) => "☑ ",
                        Some(false) => "☐ ",
                        None => "",
                    };
                    body_xml.push_str(&format!("<text:list-item{start_attr}>"));
                    if !item.inlines.is_empty() || item.children.is_empty() {
                        let inner = serialize_inlines_to_odf(&item.inlines, base_dir, media);
                        body_xml.push_str(&format!(
                            r#"<text:p text:style-name="{default_p_style}">{num_prefix}{task_prefix}{inner}</text:p>"#
                        ));
                    }
                    if !item.children.is_empty() {
                        render_odt_blocks(
                            &item.children,
                            allow_mermaid,
                            base_dir,
                            mermaid_cache,
                            media,
                            table_seq,
                            in_quote,
                            body_xml,
                        );
                    }
                    body_xml.push_str("</text:list-item>");
                }
                body_xml.push_str("</text:list>");
            }
            PreviewBlock::Table {
                alignments,
                headers,
                rows,
            } => {
                let cols = headers
                    .len()
                    .max(rows.iter().map(Vec::len).max().unwrap_or(0))
                    .max(1);
                let t_id = *table_seq;
                *table_seq += 1;
                body_xml.push_str(&format!(
                    r#"<table:table table:name="Table_{t_id}"><table:table-column table:number-columns-repeated="{cols}"/>"#
                ));
                if !headers.is_empty() {
                    body_xml.push_str("<table:table-row>");
                    for c_idx in 0..cols {
                        let align = alignments.get(c_idx).copied().unwrap_or_default();
                        let p_style = odt_cell_para_style(align, true);
                        let empty = Vec::new();
                        let inner = serialize_inlines_to_odf(
                            headers.get(c_idx).unwrap_or(&empty),
                            base_dir,
                            media,
                        );
                        body_xml.push_str(&format!(
                            r#"<table:table-cell office:value-type="string"><text:p text:style-name="{p_style}">{inner}</text:p></table:table-cell>"#
                        ));
                    }
                    body_xml.push_str("</table:table-row>");
                }
                for row in rows {
                    body_xml.push_str("<table:table-row>");
                    for c_idx in 0..cols {
                        let align = alignments.get(c_idx).copied().unwrap_or_default();
                        let p_style = odt_cell_para_style(align, false);
                        let empty = Vec::new();
                        let inner = serialize_inlines_to_odf(
                            row.get(c_idx).unwrap_or(&empty),
                            base_dir,
                            media,
                        );
                        body_xml.push_str(&format!(
                            r#"<table:table-cell office:value-type="string"><text:p text:style-name="{p_style}">{inner}</text:p></table:table-cell>"#
                        ));
                    }
                    body_xml.push_str("</table:table-row>");
                }
                body_xml.push_str("</table:table>");
            }
            PreviewBlock::Image { alt, url, .. } => {
                if let Some(img) = load_local_image_for_export(url, base_dir) {
                    let (w_cm, h_cm) = img
                        .raster
                        .as_ref()
                        .map(|r| odt_size_cm(r.width, r.height, 14.5))
                        .unwrap_or((10.0, 6.0));
                    let pic_path = media.embed_bytes(&img.ext_lower, img.mime, img.raw_bytes);
                    let esc_alt = xml_escape(alt);
                    body_xml.push_str(&format!(
                        r#"<text:p text:style-name="{default_p_style}"><draw:frame text:anchor-type="as-char" svg:width="{w_cm:.2}cm" svg:height="{h_cm:.2}cm"><draw:image xlink:href="{pic_path}" xlink:type="simple" xlink:show="embed" xlink:actuate="onLoad"/><svg:title>{esc_alt}</svg:title></draw:frame></text:p>"#
                    ));
                } else {
                    body_xml.push_str(&format!(
                        r#"<text:p text:style-name="{default_p_style}">{}</text:p>"#,
                        xml_escape(&format_missing_image_fallback(alt, url))
                    ));
                }
            }
            PreviewBlock::FootnoteDefinition { name, children } => {
                body_xml.push_str(&format!(
                    r#"<text:p text:style-name="{default_p_style}"><text:span text:style-name="TBold">[{}]:</text:span></text:p>"#,
                    xml_escape(name)
                ));
                render_odt_blocks(
                    children,
                    allow_mermaid,
                    base_dir,
                    mermaid_cache,
                    media,
                    table_seq,
                    in_quote,
                    body_xml,
                );
            }
        }
    }
}

/// Builds a compliant ODF 1.2 `.odt` ZIP archive with an uncompressed `mimetype` header as the first entry.
pub fn export_to_odt(blocks: &[PreviewBlock], title: &str) -> Result<Vec<u8>, String> {
    export_to_odt_with_options(blocks, title, true, None)
}

pub fn export_to_odt_with_options(
    blocks: &[PreviewBlock],
    title: &str,
    allow_mermaid: bool,
    base_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);

    // 1. First entry MUST be uncompressed "mimetype" (CompressionMethod::Stored)
    let stored_opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    zip.start_file("mimetype", stored_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")
        .map_err(|e| e.to_string())?;

    let deflate_opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut mermaid_cache = MermaidCache::new();
    let mut media = OdtMediaCollector::new();
    let mut table_seq = 1usize;
    let mut body_xml = String::with_capacity(4096);

    render_odt_blocks(
        blocks,
        allow_mermaid,
        base_dir,
        &mut mermaid_cache,
        &mut media,
        &mut table_seq,
        false,
        &mut body_xml,
    );

    for (pic_path, _mime, bytes) in &media.entries {
        zip.start_file(pic_path, deflate_opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
    }

    let styles_xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.2">
  <office:styles>
    <style:style style:name="Text_20_body" style:family="paragraph"><style:paragraph-properties fo:margin-bottom="0.25cm"/></style:style>
    <style:style style:name="Heading_20_1" style:family="paragraph"><style:text-properties fo:font-size="20pt" fo:font-weight="bold"/></style:style>
    <style:style style:name="Heading_20_2" style:family="paragraph"><style:text-properties fo:font-size="16pt" fo:font-weight="bold"/></style:style>
    <style:style style:name="Heading_20_3" style:family="paragraph"><style:text-properties fo:font-size="13pt" fo:font-weight="bold"/></style:style>
    <style:style style:name="Quotations" style:family="paragraph"><style:paragraph-properties fo:margin-left="0.8cm" fo:padding-left="0.3cm" fo:border-left="2.2pt solid #C85A17" fo:margin-bottom="0.2cm"/><style:text-properties fo:font-style="italic"/></style:style>
    <style:style style:name="Preformatted_20_Text" style:family="paragraph"><style:text-properties style:font-name="monospace" fo:font-size="10pt"/></style:style>
  </office:styles>
</office:document-styles>"##;

    let content_xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" office:version="1.2">
  <office:automatic-styles>
    <style:style style:name="TBold" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
    <style:style style:name="TItalic" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
    <style:style style:name="TBoldItalic" style:family="text"><style:text-properties fo:font-weight="bold" fo:font-style="italic"/></style:style>
    <style:style style:name="TStrike" style:family="text"><style:text-properties style:text-line-through-style="solid"/></style:style>
    <style:style style:name="TBoldStrike" style:family="text"><style:text-properties fo:font-weight="bold" style:text-line-through-style="solid"/></style:style>
    <style:style style:name="TCode" style:family="text"><style:text-properties style:font-name="monospace" fo:background-color="#F0EFEA"/></style:style>
    <style:style style:name="PHorizontalRule" style:family="paragraph"><style:paragraph-properties fo:border-bottom="0.75pt solid #CBD5E0" fo:padding-bottom="0.15cm" fo:margin-bottom="0.3cm"/></style:style>
    <style:style style:name="PCellLeft" style:family="paragraph"><style:paragraph-properties fo:text-align="left"/></style:style>
    <style:style style:name="PCellCenter" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/></style:style>
    <style:style style:name="PCellRight" style:family="paragraph"><style:paragraph-properties fo:text-align="right"/></style:style>
    <style:style style:name="PCellHeaderLeft" style:family="paragraph"><style:paragraph-properties fo:text-align="left"/><style:text-properties fo:font-weight="bold"/></style:style>
    <style:style style:name="PCellHeaderCenter" style:family="paragraph"><style:paragraph-properties fo:text-align="center"/><style:text-properties fo:font-weight="bold"/></style:style>
    <style:style style:name="PCellHeaderRight" style:family="paragraph"><style:paragraph-properties fo:text-align="right"/><style:text-properties fo:font-weight="bold"/></style:style>
  </office:automatic-styles>
  <office:body>
    <office:text>
      {body_xml}
    </office:text>
  </office:body>
</office:document-content>"##
    );

    let meta_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" office:version="1.2">
  <office:meta><dc:title>{}</dc:title></office:meta>
</office:document-meta>"#,
        xml_escape(title)
    );

    let mut manifest_xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
  <manifest:file-entry manifest:full-path="/" manifest:version="1.2" manifest:media-type="application/vnd.oasis.opendocument.text"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
  <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>"#,
    );
    for (pic_path, mime, _) in &media.entries {
        manifest_xml.push_str(&format!(
            "\n  <manifest:file-entry manifest:full-path=\"{pic_path}\" manifest:media-type=\"{mime}\"/>"
        ));
    }
    manifest_xml.push_str("\n</manifest:manifest>");

    zip.start_file("styles.xml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(styles_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("content.xml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(content_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("meta.xml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(meta_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("META-INF/manifest.xml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(manifest_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

// ============================================================================
// 4. PURE-RUST EPUB 3 (.epub) EXPORTER VIA `zip`
// ============================================================================

struct EpubMediaCollector {
    next_id: usize,
    /// `(id, rel_href, mime, bytes)` under `OEBPS/`
    entries: Vec<(String, String, &'static str, Vec<u8>)>,
}

impl EpubMediaCollector {
    fn new() -> Self {
        Self {
            next_id: 1,
            entries: Vec::new(),
        }
    }

    fn embed_bytes(&mut self, prefix: &str, ext: &str, mime: &'static str, bytes: Vec<u8>) -> String {
        let id = self.next_id;
        self.next_id += 1;
        let item_id = format!("{prefix}_{id}");
        let rel_href = format!("images/{item_id}.{ext}");
        self.entries.push((item_id, rel_href.clone(), mime, bytes));
        rel_href
    }
}

fn serialize_inlines_to_xhtml(
    inlines: &[PreviewInline],
    base_dir: Option<&Path>,
    media: &mut EpubMediaCollector,
) -> String {
    let mut out = String::new();
    for span in inlines {
        match span {
            PreviewInline::Text {
                text,
                bold,
                italic,
                strikethrough,
                code,
                link_url,
                link_title,
            } => {
                let mut chunk = xml_escape(text);
                if *code {
                    chunk = format!("<code>{chunk}</code>");
                }
                if *bold {
                    chunk = format!("<strong>{chunk}</strong>");
                }
                if *italic {
                    chunk = format!("<em>{chunk}</em>");
                }
                if *strikethrough {
                    chunk = format!("<del>{chunk}</del>");
                }
                if let Some(ref url) = link_url {
                    let title_attr = link_title
                        .as_deref()
                        .filter(|t| !t.is_empty())
                        .map(|t| format!(r#" title="{}""#, xml_escape(t)))
                        .unwrap_or_default();
                    chunk = format!(r#"<a href="{}"{title_attr}>{chunk}</a>"#, xml_escape(url));
                }
                out.push_str(&chunk);
            }
            PreviewInline::LineBreak => {
                out.push_str("<br />");
            }
            PreviewInline::Image { alt, url, title } => {
                if let Some(img) = load_local_image_for_export(url, base_dir) {
                    let rel_img = media.embed_bytes("img", &img.ext_lower, img.mime, img.raw_bytes);
                    let title_attr = if title.is_empty() {
                        String::new()
                    } else {
                        format!(r#" title="{}""#, xml_escape(title))
                    };
                    out.push_str(&format!(
                        r#"<img src="{rel_img}" alt="{}"{title_attr}/>"#,
                        xml_escape(alt)
                    ));
                } else {
                    out.push_str(&format!(
                        r#"<span class="img-fallback">{}</span>"#,
                        xml_escape(&format_missing_image_fallback(alt, url))
                    ));
                }
            }
            PreviewInline::Math { literal, .. } => {
                out.push_str(&format!(
                    r#"<code class="math">${}$</code>"#,
                    xml_escape(literal)
                ));
            }
        }
    }
    out
}

fn render_epub_blocks(
    blocks: &[PreviewBlock],
    allow_mermaid: bool,
    base_dir: Option<&Path>,
    mermaid_cache: &mut MermaidCache,
    media: &mut EpubMediaCollector,
    out: &mut String,
) {
    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let lv = (*level).clamp(1, 6);
                out.push_str(&format!(
                    "<h{lv}>{}</h{lv}>\n",
                    serialize_inlines_to_xhtml(inlines, base_dir, media)
                ));
            }
            PreviewBlock::Paragraph { inlines } => {
                out.push_str(&format!(
                    "<p>{}</p>\n",
                    serialize_inlines_to_xhtml(inlines, base_dir, media)
                ));
            }
            PreviewBlock::ThematicBreak => {
                out.push_str("<hr />\n");
            }
            PreviewBlock::BlockQuote { alert, children } => {
                if let Some(kind) = alert {
                    out.push_str(&format!(
                        "<blockquote class=\"alert alert-{}\">\n  <p class=\"alert-title\"><strong>{}</strong></p>\n",
                        kind.label().to_ascii_lowercase(),
                        xml_escape(kind.label())
                    ));
                } else {
                    out.push_str("<blockquote>\n");
                }
                render_epub_blocks(
                    children,
                    allow_mermaid,
                    base_dir,
                    mermaid_cache,
                    media,
                    out,
                );
                out.push_str("</blockquote>\n");
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if allow_mermaid && lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if !rendered.png_bytes.is_empty() {
                        let rel_img = media.embed_bytes(
                            "mermaid",
                            "png",
                            "image/png",
                            rendered.png_bytes.clone(),
                        );
                        out.push_str(&format!(
                            "<div class=\"mermaid\"><img src=\"{rel_img}\" alt=\"Mermaid Diagram\"/></div>\n"
                        ));
                        continue;
                    }
                }
                let cls_attr = if lang.is_empty() {
                    String::new()
                } else {
                    format!(r#" class="language-{}""#, xml_escape(lang))
                };
                out.push_str(&format!(
                    "<pre><code{cls_attr}>{}</code></pre>\n",
                    xml_escape(code)
                ));
            }
            PreviewBlock::List {
                ordered,
                start,
                items,
                ..
            } => {
                if *ordered {
                    if *start != 1 {
                        out.push_str(&format!("<ol start=\"{start}\">\n"));
                    } else {
                        out.push_str("<ol>\n");
                    }
                } else {
                    out.push_str("<ul>\n");
                }
                for item in items {
                    let prefix = match item.checked {
                        Some(true) => "☑ ",
                        Some(false) => "☐ ",
                        None => "",
                    };
                    let lead = serialize_inlines_to_xhtml(&item.inlines, base_dir, media);
                    if item.children.is_empty() {
                        out.push_str(&format!("  <li>{prefix}{lead}</li>\n"));
                    } else {
                        out.push_str(&format!("  <li>{prefix}{lead}\n"));
                        render_epub_blocks(
                            &item.children,
                            allow_mermaid,
                            base_dir,
                            mermaid_cache,
                            media,
                            out,
                        );
                        out.push_str("  </li>\n");
                    }
                }
                out.push_str(if *ordered { "</ol>\n" } else { "</ul>\n" });
            }
            PreviewBlock::Table {
                alignments,
                headers,
                rows,
            } => {
                let cols = headers
                    .len()
                    .max(rows.iter().map(Vec::len).max().unwrap_or(0))
                    .max(1);
                out.push_str("<table>\n");
                if !headers.is_empty() {
                    out.push_str("  <thead><tr>");
                    for c_idx in 0..cols {
                        let style = match alignments.get(c_idx).copied().unwrap_or_default() {
                            TableAlignment::None => "",
                            TableAlignment::Left => r#" style="text-align: left;""#,
                            TableAlignment::Center => r#" style="text-align: center;""#,
                            TableAlignment::Right => r#" style="text-align: right;""#,
                        };
                        let empty = Vec::new();
                        let inner = serialize_inlines_to_xhtml(
                            headers.get(c_idx).unwrap_or(&empty),
                            base_dir,
                            media,
                        );
                        out.push_str(&format!("<th{style}>{inner}</th>"));
                    }
                    out.push_str("</tr></thead>\n");
                }
                out.push_str("  <tbody>\n");
                for row in rows {
                    out.push_str("    <tr>");
                    for c_idx in 0..cols {
                        let style = match alignments.get(c_idx).copied().unwrap_or_default() {
                            TableAlignment::None => "",
                            TableAlignment::Left => r#" style="text-align: left;""#,
                            TableAlignment::Center => r#" style="text-align: center;""#,
                            TableAlignment::Right => r#" style="text-align: right;""#,
                        };
                        let empty = Vec::new();
                        let inner = serialize_inlines_to_xhtml(
                            row.get(c_idx).unwrap_or(&empty),
                            base_dir,
                            media,
                        );
                        out.push_str(&format!("<td{style}>{inner}</td>"));
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("  </tbody>\n</table>\n");
            }
            PreviewBlock::Image {
                alt,
                url,
                title,
            } => {
                if let Some(img) = load_local_image_for_export(url, base_dir) {
                    let rel_img = media.embed_bytes("img", &img.ext_lower, img.mime, img.raw_bytes);
                    let title_attr = if title.is_empty() {
                        String::new()
                    } else {
                        format!(r#" title="{}""#, xml_escape(title))
                    };
                    out.push_str(&format!(
                        "<figure><img src=\"{rel_img}\" alt=\"{}\"{title_attr}/></figure>\n",
                        xml_escape(alt)
                    ));
                } else {
                    out.push_str(&format!(
                        "<p class=\"img-fallback\">{}</p>\n",
                        xml_escape(&format_missing_image_fallback(alt, url))
                    ));
                }
            }
            PreviewBlock::FootnoteDefinition { name, children } => {
                out.push_str(&format!(
                    "<aside class=\"footnote\"><p><strong>[{}]:</strong></p>\n",
                    xml_escape(name)
                ));
                render_epub_blocks(
                    children,
                    allow_mermaid,
                    base_dir,
                    mermaid_cache,
                    media,
                    out,
                );
                out.push_str("</aside>\n");
            }
        }
    }
}

/// Generates a compliant EPUB 3 archive containing `mimetype`, `META-INF/container.xml`, `OEBPS/content.opf`, `OEBPS/style.css`, and `OEBPS/chapter1.xhtml`.
pub fn export_to_epub(blocks: &[PreviewBlock], title: &str) -> Result<Vec<u8>, String> {
    export_to_epub_with_options(blocks, title, true, None)
}

pub fn export_to_epub_with_options(
    blocks: &[PreviewBlock],
    title: &str,
    allow_mermaid: bool,
    base_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);

    let stored_opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    zip.start_file("mimetype", stored_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(b"application/epub+zip")
        .map_err(|e| e.to_string())?;

    let deflate_opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let container_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
    zip.start_file("META-INF/container.xml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(container_xml.as_bytes())
        .map_err(|e| e.to_string())?;

    let css = "body { font-family: serif; line-height: 1.6; margin: 1.5em; color: #2e3436; }\n\
               h1, h2, h3, h4, h5, h6 { font-family: sans-serif; color: #1a202c; }\n\
               img { max-width: 100%; height: auto; }\n\
               hr { border: 0; border-top: 1px solid #cbd5e0; margin: 1.4em 0; }\n\
               pre, code { font-family: monospace; background: #f4f1ea; padding: 0.2em 0.4em; border-radius: 4px; }\n\
               pre code { background: none; padding: 0; }\n\
               blockquote { border-left: 3px solid #c85a17; padding-left: 1em; margin: 1em 0; }\n\
               table { border-collapse: collapse; width: 100%; margin: 1em 0; }\n\
               th, td { border: 1px solid #cbd5e0; padding: 0.4em 0.6em; text-align: left; }\n\
               th { background: #edf2f7; font-weight: bold; }\n\
               .img-fallback { color: #4a5568; font-style: italic; }\n";
    zip.start_file("OEBPS/style.css", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(css.as_bytes()).map_err(|e| e.to_string())?;

    let mut mermaid_cache = MermaidCache::new();
    let mut media = EpubMediaCollector::new();
    let mut chapter_body = String::with_capacity(4096);

    render_epub_blocks(
        blocks,
        allow_mermaid,
        base_dir,
        &mut mermaid_cache,
        &mut media,
        &mut chapter_body,
    );

    let mut image_manifest_items = String::new();
    for (item_id, rel_href, mime, bytes) in &media.entries {
        let zip_path = format!("OEBPS/{rel_href}");
        zip.start_file(&zip_path, deflate_opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
        image_manifest_items.push_str(&format!(
            "\n    <item id=\"{item_id}\" href=\"{rel_href}\" media-type=\"{mime}\"/>"
        ));
    }

    let esc_title = xml_escape(title);
    let chapter_xhtml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="it" lang="it">
<head>
  <meta charset="utf-8"/>
  <title>{esc_title}</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
{chapter_body}</body>
</html>"#
    );
    zip.start_file("OEBPS/chapter1.xhtml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(chapter_xhtml.as_bytes())
        .map_err(|e| e.to_string())?;

    let nav_xhtml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="it" lang="it">
<head><meta charset="utf-8"/><title>{esc_title}</title></head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>Indice</h1>
    <ol><li><a href="chapter1.xhtml">{esc_title}</a></li></ol>
  </nav>
</body>
</html>"#
    );
    zip.start_file("OEBPS/nav.xhtml", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(nav_xhtml.as_bytes())
        .map_err(|e| e.to_string())?;

    let content_opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">urn:uuid:accent-epub-export</dc:identifier>
    <dc:title>{esc_title}</dc:title>
    <dc:language>it</dc:language>
    <meta property="dcterms:modified">2026-10-29T18:00:00Z</meta>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="style" href="style.css" media-type="text/css"/>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>{image_manifest_items}
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#
    );
    zip.start_file("OEBPS/content.opf", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(content_opf.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

// ============================================================================
// 5. PURE-RUST PAGINATED PDF 1.4 (.pdf) EXPORTER WITH RICH AST & LOCAL IMAGES
// ============================================================================

fn pdf_win_ansi_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for ch in text.chars() {
        let code = ch as u32;
        if ch == '(' || ch == ')' || ch == '\\' {
            out.push('\\');
            out.push(ch);
        } else if (0x20..=0x7E).contains(&code) {
            out.push(ch);
        } else if (0xA0..=0xFF).contains(&code) {
            out.push_str(&format!("\\{:03o}", code));
        } else {
            match ch {
                '—' | '–' => out.push('-'),
                '•' => out.push_str("\\267"),
                '“' | '”' | '«' | '»' => out.push('"'),
                '’' | '‘' => out.push('\''),
                '☑' => out.push_str("[x]"),
                '☐' => out.push_str("[ ]"),
                _ => out.push('?'),
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
struct PdfStyledRun {
    text: String,
    bold: bool,
    italic: bool,
    strikethrough: bool,
    code: bool,
    link_url: Option<String>,
}

impl PdfStyledRun {
    fn font_tag(&self) -> &'static str {
        if self.code {
            "/F4"
        } else if self.bold && self.italic {
            "/F5"
        } else if self.bold {
            "/F2"
        } else if self.italic {
            "/F3"
        } else {
            "/F1"
        }
    }

    fn estimate_width_pt(&self, font_size: f32) -> f32 {
        let factor = if self.code {
            0.60
        } else if self.bold {
            0.55
        } else {
            0.51
        };
        (self.text.chars().count() as f32) * font_size * factor
    }
}

struct PdfImageXObject {
    name: String,
    width: u32,
    height: u32,
    rgb_bytes: Vec<u8>,
}

struct PdfLinkAnnot {
    rect: [f32; 4],
    uri: String,
}

struct PdfPageBuilder {
    page_w: f32,
    page_h: f32,
    margin_x: f32,
    top_y: f32,
    bottom_y: f32,
    pages_ops: Vec<String>,
    pages_annots: Vec<Vec<PdfLinkAnnot>>,
    cur_ops: String,
    cur_annots: Vec<PdfLinkAnnot>,
    cur_y: f32,
    embedded_images: Vec<PdfImageXObject>,
}

impl PdfPageBuilder {
    fn new() -> Self {
        let page_w = 595.28_f32;
        let page_h = 841.89_f32;
        let margin_x = 54.0_f32;
        let top_y = page_h - 56.0;
        let bottom_y = 56.0_f32;
        Self {
            page_w,
            page_h,
            margin_x,
            top_y,
            bottom_y,
            pages_ops: Vec::new(),
            pages_annots: Vec::new(),
            cur_ops: String::with_capacity(4096),
            cur_annots: Vec::new(),
            cur_y: top_y,
            embedded_images: Vec::new(),
        }
    }

    fn ensure_space(&mut self, needed: f32) {
        if self.cur_y - needed < self.bottom_y {
            self.pages_ops.push(std::mem::take(&mut self.cur_ops));
            self.pages_annots.push(std::mem::take(&mut self.cur_annots));
            self.cur_y = self.top_y;
        }
    }

    fn register_image(&mut self, width: u32, height: u32, rgb_bytes: Vec<u8>) -> String {
        let idx = self.embedded_images.len() + 1;
        let name = format!("Im{idx}");
        self.embedded_images.push(PdfImageXObject {
            name: name.clone(),
            width,
            height,
            rgb_bytes,
        });
        name
    }

    fn draw_block_image(
        &mut self,
        width: u32,
        height: u32,
        rgb_bytes: Vec<u8>,
        left_x: f32,
        max_w: f32,
    ) {
        if width == 0 || height == 0 || rgb_bytes.is_empty() {
            return;
        }
        let img_name = self.register_image(width, height, rgb_bytes);
        let max_h = (self.top_y - self.bottom_y - 24.0).min(380.0);
        let scale_w = max_w / (width as f32);
        let scale_h = max_h / (height as f32);
        let scale = scale_w.min(scale_h).min(0.75);
        let disp_w = (width as f32) * scale;
        let disp_h = (height as f32) * scale;

        self.ensure_space(disp_h + 14.0);
        let img_bottom_y = self.cur_y - disp_h;
        self.cur_ops.push_str(&format!(
            "q {disp_w:.2} 0 0 {disp_h:.2} {left_x:.2} {img_bottom_y:.2} cm /{img_name} Do Q\n"
        ));
        self.cur_y = img_bottom_y - 10.0;
    }

    /// Splits `inlines` into wrapped lines of `PdfStyledRun`s, honoring explicit `PreviewInline::LineBreak`s
    /// and extracting any inline local images to render alongside the paragraph.
    fn layout_inlines(
        &self,
        inlines: &[PreviewInline],
        max_width_pt: f32,
        font_size: f32,
        force_bold: bool,
        force_italic: bool,
        base_dir: Option<&Path>,
    ) -> (Vec<Vec<PdfStyledRun>>, Vec<DecodedRasterImage>) {
        let mut lines: Vec<Vec<PdfStyledRun>> = vec![Vec::new()];
        let mut cur_line_w = 0.0_f32;
        let mut inline_rasters = Vec::new();

        for item in inlines {
            match item {
                PreviewInline::LineBreak => {
                    lines.push(Vec::new());
                    cur_line_w = 0.0;
                }
                PreviewInline::Image { alt, url, .. } => {
                    if let Some(loaded) = load_local_image_for_export(url, base_dir) {
                        if let Some(raster) = loaded.raster {
                            inline_rasters.push(raster);
                            if !alt.trim().is_empty() {
                                let run = PdfStyledRun {
                                    text: format!("[{}]", alt.trim()),
                                    bold: force_bold,
                                    italic: true,
                                    strikethrough: false,
                                    code: false,
                                    link_url: None,
                                };
                                cur_line_w += run.estimate_width_pt(font_size);
                                lines.last_mut().unwrap().push(run);
                            }
                            continue;
                        }
                    }
                    let fb = PdfStyledRun {
                        text: format_missing_image_fallback(alt, url),
                        bold: force_bold,
                        italic: true,
                        strikethrough: false,
                        code: false,
                        link_url: None,
                    };
                    cur_line_w += fb.estimate_width_pt(font_size);
                    lines.last_mut().unwrap().push(fb);
                }
                PreviewInline::Math { literal, .. } => {
                    let run = PdfStyledRun {
                        text: format!("${literal}$"),
                        bold: force_bold,
                        italic: true,
                        strikethrough: false,
                        code: true,
                        link_url: None,
                    };
                    let w = run.estimate_width_pt(font_size);
                    if cur_line_w > 0.0 && cur_line_w + w > max_width_pt {
                        lines.push(Vec::new());
                        cur_line_w = 0.0;
                    }
                    cur_line_w += w;
                    lines.last_mut().unwrap().push(run);
                }
                PreviewInline::Text {
                    text,
                    bold,
                    italic,
                    strikethrough,
                    code,
                    link_url,
                    ..
                } => {
                    // Tokenize preserving spaces so word-wrapping respects styled spans
                    let mut tokens: Vec<&str> = Vec::new();
                    let mut start = 0usize;
                    let mut in_ws: Option<bool> = None;
                    for (idx, ch) in text.char_indices() {
                        let ws = ch.is_whitespace();
                        if let Some(prev_ws) = in_ws {
                            if prev_ws != ws {
                                tokens.push(&text[start..idx]);
                                start = idx;
                                in_ws = Some(ws);
                            }
                        } else {
                            in_ws = Some(ws);
                        }
                    }
                    if start < text.len() {
                        tokens.push(&text[start..]);
                    }

                    for tok in tokens {
                        let is_ws = tok.chars().all(char::is_whitespace);
                        if is_ws && cur_line_w == 0.0 {
                            continue;
                        }
                        let candidate = PdfStyledRun {
                            text: if is_ws {
                                " ".to_string()
                            } else {
                                tok.to_string()
                            },
                            bold: force_bold || *bold,
                            italic: force_italic || *italic,
                            strikethrough: *strikethrough,
                            code: *code,
                            link_url: link_url.clone(),
                        };
                        let w = candidate.estimate_width_pt(font_size);
                        if !is_ws && cur_line_w > 0.0 && cur_line_w + w > max_width_pt {
                            lines.push(Vec::new());
                            cur_line_w = 0.0;
                        }
                        let cur_runs = lines.last_mut().unwrap();
                        if let Some(prev) = cur_runs.last_mut() {
                            if prev.bold == candidate.bold
                                && prev.italic == candidate.italic
                                && prev.strikethrough == candidate.strikethrough
                                && prev.code == candidate.code
                                && prev.link_url == candidate.link_url
                            {
                                prev.text.push_str(&candidate.text);
                                cur_line_w += w;
                                continue;
                            }
                        }
                        cur_line_w += w;
                        cur_runs.push(candidate);
                    }
                }
            }
        }

        if lines.is_empty() {
            lines.push(Vec::new());
        }
        (lines, inline_rasters)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_run_line(
        &mut self,
        runs: &[PdfStyledRun],
        box_left_x: f32,
        box_width: f32,
        baseline_y: f32,
        font_size: f32,
        align: TableAlignment,
        default_rgb: (f32, f32, f32),
    ) {
        let total_w: f32 = runs.iter().map(|r| r.estimate_width_pt(font_size)).sum();
        let mut x = match align {
            TableAlignment::None | TableAlignment::Left => box_left_x,
            TableAlignment::Center => box_left_x + ((box_width - total_w) * 0.5).max(0.0),
            TableAlignment::Right => box_left_x + (box_width - total_w).max(0.0),
        };

        for run in runs {
            let run_w = run.estimate_width_pt(font_size);
            let (r_c, g_c, b_c) = if run.link_url.is_some() {
                (0.14, 0.35, 0.62)
            } else if run.code {
                (0.55, 0.16, 0.28)
            } else {
                default_rgb
            };
            let f_tag = run.font_tag();
            self.cur_ops.push_str(&format!(
                "BT {f_tag} {font_size:.1} Tf {r_c:.2} {g_c:.2} {b_c:.2} rg {x:.2} {baseline_y:.2} Td ({}) Tj ET\n",
                pdf_win_ansi_escape(&run.text)
            ));
            if run.strikethrough && run_w > 1.0 {
                let strike_y = baseline_y + font_size * 0.30;
                self.cur_ops.push_str(&format!(
                    "{r_c:.2} {g_c:.2} {b_c:.2} RG 0.6 w {x:.2} {strike_y:.2} m {:.2} {strike_y:.2} l S\n",
                    x + run_w
                ));
            }
            if let Some(ref url) = run.link_url {
                let ul_y = baseline_y - 1.5;
                self.cur_ops.push_str(&format!(
                    "0.14 0.35 0.62 RG 0.5 w {x:.2} {ul_y:.2} m {:.2} {ul_y:.2} l S\n",
                    x + run_w
                ));
                if url.starts_with("https://")
                    || url.starts_with("http://")
                    || url.starts_with("mailto:")
                {
                    self.cur_annots.push(PdfLinkAnnot {
                        rect: [x, baseline_y - 2.5, x + run_w.max(4.0), baseline_y + font_size],
                        uri: url.clone(),
                    });
                }
            }
            x += run_w;
        }
    }

    fn render_blocks(
        &mut self,
        blocks: &[PreviewBlock],
        allow_mermaid: bool,
        base_dir: Option<&Path>,
        mermaid_cache: &mut MermaidCache,
        left_indent: f32,
        quote_depth: usize,
    ) {
        let left_x = self.margin_x + left_indent;
        let avail_w = (self.page_w - self.margin_x - left_x).max(120.0);
        let default_rgb = if quote_depth > 0 {
            (0.35, 0.30, 0.24)
        } else {
            (0.18, 0.20, 0.22)
        };

        for block in blocks {
            match block {
                PreviewBlock::Heading { level, inlines, .. } => {
                    let (font_size, leading) = match level {
                        1 => (20.0_f32, 25.0_f32),
                        2 => (16.0_f32, 21.0_f32),
                        3 => (13.5_f32, 18.0_f32),
                        _ => (12.0_f32, 16.0_f32),
                    };
                    self.ensure_space(leading + 12.0);
                    self.cur_y -= 6.0;
                    let (lines, inline_imgs) =
                        self.layout_inlines(inlines, avail_w, font_size, true, false, base_dir);
                    for line_runs in &lines {
                        self.ensure_space(leading);
                        self.draw_run_line(
                            line_runs,
                            left_x,
                            avail_w,
                            self.cur_y,
                            font_size,
                            TableAlignment::Left,
                            (0.14, 0.16, 0.20),
                        );
                        self.cur_y -= leading;
                    }
                    if *level == 1 {
                        let rule_y = self.cur_y + 8.0;
                        self.cur_ops.push_str(&format!(
                            "0.80 0.76 0.70 RG 0.8 w {left_x:.1} {rule_y:.1} m {:.1} {rule_y:.1} l S\n",
                            self.page_w - self.margin_x
                        ));
                        self.cur_y -= 6.0;
                    }
                    for r in inline_imgs {
                        self.draw_block_image(r.width, r.height, r.rgb_bytes, left_x, avail_w * 0.5);
                    }
                }
                PreviewBlock::Paragraph { inlines } => {
                    let (lines, inline_imgs) = self.layout_inlines(
                        inlines,
                        avail_w,
                        11.0,
                        false,
                        quote_depth > 0,
                        base_dir,
                    );
                    for line_runs in &lines {
                        self.ensure_space(15.5);
                        if quote_depth > 0 {
                            let bar_x = left_x - 8.0;
                            self.cur_ops.push_str(&format!(
                                "0.78 0.35 0.09 RG 2.0 w {bar_x:.1} {:.1} m {bar_x:.1} {:.1} l S\n",
                                self.cur_y + 10.0,
                                self.cur_y - 4.5
                            ));
                        }
                        self.draw_run_line(
                            line_runs,
                            left_x,
                            avail_w,
                            self.cur_y,
                            11.0,
                            TableAlignment::Left,
                            default_rgb,
                        );
                        self.cur_y -= 15.5;
                    }
                    for r in inline_imgs {
                        self.draw_block_image(r.width, r.height, r.rgb_bytes, left_x, avail_w * 0.65);
                    }
                    self.cur_y -= 6.0;
                }
                PreviewBlock::ThematicBreak => {
                    self.ensure_space(16.0);
                    let rule_y = self.cur_y - 4.0;
                    self.cur_ops.push_str(&format!(
                        "0.76 0.78 0.82 RG 0.9 w {left_x:.1} {rule_y:.1} m {:.1} {rule_y:.1} l S\n",
                        self.page_w - self.margin_x
                    ));
                    self.cur_y -= 14.0;
                }
                PreviewBlock::BlockQuote { alert, children } => {
                    if let Some(kind) = alert {
                        self.ensure_space(16.0);
                        let bar_x = left_x + 4.0;
                        self.cur_ops.push_str(&format!(
                            "0.78 0.35 0.09 RG 2.4 w {bar_x:.1} {:.1} m {bar_x:.1} {:.1} l S\n",
                            self.cur_y + 10.0,
                            self.cur_y - 4.0
                        ));
                        self.cur_ops.push_str(&format!(
                            "BT /F2 11 Tf 0.78 0.35 0.09 rg {:.1} {:.1} Td ([{}]) Tj ET\n",
                            left_x + 14.0,
                            self.cur_y,
                            pdf_win_ansi_escape(kind.label())
                        ));
                        self.cur_y -= 16.0;
                    }
                    self.render_blocks(
                        children,
                        allow_mermaid,
                        base_dir,
                        mermaid_cache,
                        left_indent + 14.0,
                        quote_depth + 1,
                    );
                }
                PreviewBlock::CodeBlock { info, code } => {
                    let lang = info.split_whitespace().next().unwrap_or("");
                    if allow_mermaid && lang.eq_ignore_ascii_case("mermaid") {
                        let rendered =
                            mermaid_cache.get_or_render_screen_themed(code, MermaidTheme::Light);
                        if rendered.width > 0
                            && rendered.height > 0
                            && !rendered.rgba_pixels.is_empty()
                        {
                            let rgb = rgba_to_white_composited_rgb(
                                &rendered.rgba_pixels,
                                rendered.width,
                                rendered.height,
                            );
                            self.draw_block_image(
                                rendered.width,
                                rendered.height,
                                rgb,
                                left_x,
                                avail_w,
                            );
                            continue;
                        }
                    }
                    let code_lines: Vec<&str> = code.lines().collect();
                    let box_h = (code_lines.len().max(1) as f32) * 13.5 + 12.0;
                    self.ensure_space(box_h + 8.0);
                    let box_bottom = self.cur_y - box_h + 10.0;
                    self.cur_ops.push_str(&format!(
                        "0.95 0.94 0.91 rg {left_x:.1} {box_bottom:.1} {avail_w:.1} {box_h:.1} re f\n"
                    ));
                    for cline in code_lines {
                        self.cur_ops.push_str(&format!(
                            "BT /F4 9.5 Tf 0.20 0.20 0.22 rg {:.1} {:.1} Td ({}) Tj ET\n",
                            left_x + 8.0,
                            self.cur_y,
                            pdf_win_ansi_escape(cline)
                        ));
                        self.cur_y -= 13.5;
                    }
                    self.cur_y -= 10.0;
                }
                PreviewBlock::List {
                    ordered,
                    start,
                    items,
                    ..
                } => {
                    for (idx, item) in items.iter().enumerate() {
                        let num = start.saturating_add(idx);
                        let prefix = match item.checked {
                            Some(true) if *ordered => format!("{num}. [x] "),
                            Some(false) if *ordered => format!("{num}. [ ] "),
                            Some(true) => "[x] ".to_string(),
                            Some(false) => "[ ] ".to_string(),
                            None if *ordered => format!("{num}. "),
                            None => "• ".to_string(),
                        };
                        let mut item_inlines = Vec::with_capacity(item.inlines.len() + 1);
                        item_inlines.push(PreviewInline::plain(prefix));
                        item_inlines.extend(item.inlines.iter().cloned());

                        let item_x = left_x + 10.0;
                        let item_w = (avail_w - 10.0).max(100.0);
                        let (lines, inline_imgs) = self.layout_inlines(
                            &item_inlines,
                            item_w,
                            11.0,
                            false,
                            quote_depth > 0,
                            base_dir,
                        );
                        for line_runs in &lines {
                            self.ensure_space(15.0);
                            if quote_depth > 0 {
                                let bar_x = left_x - 8.0;
                                self.cur_ops.push_str(&format!(
                                    "0.78 0.35 0.09 RG 2.0 w {bar_x:.1} {:.1} m {bar_x:.1} {:.1} l S\n",
                                    self.cur_y + 10.0,
                                    self.cur_y - 4.5
                                ));
                            }
                            self.draw_run_line(
                                line_runs,
                                item_x,
                                item_w,
                                self.cur_y,
                                11.0,
                                TableAlignment::Left,
                                default_rgb,
                            );
                            self.cur_y -= 15.0;
                        }
                        for r in inline_imgs {
                            self.draw_block_image(r.width, r.height, r.rgb_bytes, item_x, item_w * 0.65);
                        }
                        if !item.children.is_empty() {
                            self.render_blocks(
                                &item.children,
                                allow_mermaid,
                                base_dir,
                                mermaid_cache,
                                left_indent + 16.0,
                                quote_depth,
                            );
                        }
                    }
                    self.cur_y -= 4.0;
                }
                PreviewBlock::Table {
                    alignments,
                    headers,
                    rows,
                } => {
                    let cols = headers
                        .len()
                        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
                        .max(1);
                    let col_w = avail_w / (cols as f32);
                    let cell_pad_x = 5.0_f32;
                    let inner_col_w = (col_w - cell_pad_x * 2.0).max(24.0);

                    let draw_table_row = |builder: &mut Self,
                                          cells: &[Vec<PreviewInline>],
                                          is_header: bool| {
                        let mut cell_layouts: Vec<Vec<Vec<PdfStyledRun>>> = Vec::with_capacity(cols);
                        let mut max_lines = 1usize;
                        for c_idx in 0..cols {
                            let empty = Vec::new();
                            let inl = cells.get(c_idx).unwrap_or(&empty);
                            let (lines, _) = builder.layout_inlines(
                                inl,
                                inner_col_w,
                                9.8,
                                is_header,
                                false,
                                base_dir,
                            );
                            max_lines = max_lines.max(lines.len());
                            cell_layouts.push(lines);
                        }
                        let row_h = (max_lines as f32) * 13.0 + 7.0;
                        builder.ensure_space(row_h + 4.0);
                        let row_top = builder.cur_y + 10.0;
                        let row_bot = row_top - row_h;

                        if is_header {
                            builder.cur_ops.push_str(&format!(
                                "0.93 0.95 0.97 rg {left_x:.2} {row_bot:.2} {avail_w:.2} {row_h:.2} re f\n"
                            ));
                        }
                        builder.cur_ops.push_str(&format!(
                            "0.78 0.82 0.86 RG 0.6 w {left_x:.2} {row_bot:.2} {avail_w:.2} {row_h:.2} re S\n"
                        ));
                        for c_idx in 1..cols {
                            let vx = left_x + (c_idx as f32) * col_w;
                            builder.cur_ops.push_str(&format!(
                                "0.78 0.82 0.86 RG 0.5 w {vx:.2} {row_top:.2} m {vx:.2} {row_bot:.2} l S\n"
                            ));
                        }

                        for (c_idx, lines) in cell_layouts.iter().enumerate() {
                            let align = alignments.get(c_idx).copied().unwrap_or_default();
                            let cell_x = left_x + (c_idx as f32) * col_w + cell_pad_x;
                            let mut line_y = row_top - 12.0;
                            for l_runs in lines {
                                builder.draw_run_line(
                                    l_runs,
                                    cell_x,
                                    inner_col_w,
                                    line_y,
                                    9.8,
                                    align,
                                    (0.16, 0.18, 0.22),
                                );
                                line_y -= 13.0;
                            }
                        }
                        builder.cur_y = row_bot - 10.0;
                    };

                    if !headers.is_empty() {
                        draw_table_row(self, headers, true);
                    }
                    for row in rows {
                        draw_table_row(self, row, false);
                    }
                    self.cur_y -= 6.0;
                }
                PreviewBlock::Image { alt, url, .. } => {
                    if let Some(img) = load_local_image_for_export(url, base_dir) {
                        if let Some(raster) = img.raster {
                            self.draw_block_image(
                                raster.width,
                                raster.height,
                                raster.rgb_bytes,
                                left_x,
                                avail_w,
                            );
                            continue;
                        }
                    }
                    self.ensure_space(16.0);
                    self.cur_ops.push_str(&format!(
                        "BT /F3 10.5 Tf 0.30 0.35 0.45 rg {left_x:.1} {:.1} Td ({}) Tj ET\n",
                        self.cur_y,
                        pdf_win_ansi_escape(&format_missing_image_fallback(alt, url))
                    ));
                    self.cur_y -= 16.0;
                }
                PreviewBlock::FootnoteDefinition { name, children } => {
                    self.ensure_space(15.0);
                    self.cur_ops.push_str(&format!(
                        "BT /F2 9.5 Tf 0.30 0.32 0.36 rg {left_x:.1} {:.1} Td ([{}]:) Tj ET\n",
                        self.cur_y,
                        pdf_win_ansi_escape(name)
                    ));
                    self.cur_y -= 14.0;
                    self.render_blocks(
                        children,
                        allow_mermaid,
                        base_dir,
                        mermaid_cache,
                        left_indent + 12.0,
                        quote_depth,
                    );
                }
            }
        }
    }
}

/// Generates a paginated PDF 1.4 binary document (`%PDF-1.4`) from the AST `PreviewBlock`s,
/// preserving styled table cells, column alignments, breaks, nested lists/quotes, clickable links, and local images.
pub fn export_to_pdf(blocks: &[PreviewBlock], title: &str) -> Vec<u8> {
    export_to_pdf_with_options(blocks, title, true, None)
}

pub fn export_to_pdf_with_options(
    blocks: &[PreviewBlock],
    title: &str,
    allow_mermaid: bool,
    base_dir: Option<&Path>,
) -> Vec<u8> {
    let mut builder = PdfPageBuilder::new();
    let mut mermaid_cache = MermaidCache::new();

    if !title.is_empty() {
        builder.cur_ops.push_str(&format!(
            "BT /F2 10 Tf 0.45 0.40 0.32 rg {:.1} {:.1} Td ({}) Tj ET\n",
            builder.margin_x,
            builder.cur_y,
            pdf_win_ansi_escape(title)
        ));
        builder.cur_y -= 18.0;
    }

    builder.render_blocks(blocks, allow_mermaid, base_dir, &mut mermaid_cache, 0.0, 0);

    if !builder.cur_ops.is_empty() || builder.pages_ops.is_empty() {
        builder.pages_ops.push(builder.cur_ops);
        builder.pages_annots.push(builder.cur_annots);
    }

    // Object numbering:
    // 1: Catalog
    // 2: Pages
    // 3..=7: Fonts F1..F5 (Helvetica, Bold, Oblique, Courier, BoldOblique)
    // 8..8+N_img: Image XObjects
    // Following: Page + Content (+ Link Annotation objects)
    let mut objects: Vec<Vec<u8>> = Vec::new();
    let img_start_obj = 8usize;
    let total_annots: usize = builder.pages_annots.iter().map(Vec::len).sum();
    let pages_start_obj = img_start_obj + builder.embedded_images.len();
    let annots_start_obj = pages_start_obj + builder.pages_ops.len() * 2;

    // Obj 1: Catalog
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    // Obj 2: Pages
    let mut kids_refs = String::new();
    for i in 0..builder.pages_ops.len() {
        let page_obj_id = pages_start_obj + i * 2;
        kids_refs.push_str(&format!("{page_obj_id} 0 R "));
    }
    objects.push(
        format!(
            "<< /Type /Pages /Kids [ {}] /Count {} >>",
            kids_refs,
            builder.pages_ops.len()
        )
        .into_bytes(),
    );

    // Obj 3..7: Built-in Type1 Fonts
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>".to_vec(),
    );
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-BoldOblique /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );

    // Image XObjects
    let mut xobj_dict = String::new();
    for (idx, img) in builder.embedded_images.iter().enumerate() {
        let obj_id = img_start_obj + idx;
        xobj_dict.push_str(&format!("/{} {} 0 R ", img.name, obj_id));
        let compressed_rgb = {
            let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
            let _ = enc.write_all(&img.rgb_bytes);
            enc.finish().unwrap_or_default()
        };
        let use_flate = !compressed_rgb.is_empty() && compressed_rgb.len() < img.rgb_bytes.len();
        let (filter_clause, payload): (&str, &[u8]) = if use_flate {
            (" /Filter /FlateDecode", &compressed_rgb)
        } else {
            ("", &img.rgb_bytes)
        };
        let mut obj_bytes = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8{filter_clause} /Length {} >>\nstream\n",
            img.width,
            img.height,
            payload.len()
        )
        .into_bytes();
        obj_bytes.extend_from_slice(payload);
        obj_bytes.extend_from_slice(b"\nendstream");
        objects.push(obj_bytes);
    }

    // Page & Content objects
    let mut annot_cursor = 0usize;
    let mut annot_objects: Vec<Vec<u8>> = Vec::with_capacity(total_annots);
    for (i, content_str) in builder.pages_ops.iter().enumerate() {
        let content_obj_id = pages_start_obj + i * 2 + 1;
        let page_annots = &builder.pages_annots[i];
        let annots_clause = if page_annots.is_empty() {
            String::new()
        } else {
            let mut refs = String::from(" /Annots [ ");
            for ann in page_annots {
                let ann_obj_id = annots_start_obj + annot_cursor;
                annot_cursor += 1;
                refs.push_str(&format!("{ann_obj_id} 0 R "));
                let [x1, y1, x2, y2] = ann.rect;
                annot_objects.push(
                    format!(
                        "<< /Type /Annot /Subtype /Link /Rect [{x1:.2} {y1:.2} {x2:.2} {y2:.2}] /Border [0 0 0] /A << /S /URI /URI ({}) >> >>",
                        pdf_win_ansi_escape(&ann.uri)
                    )
                    .into_bytes(),
                );
            }
            refs.push(']');
            refs
        };
        let page_dict = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R /F5 7 0 R >> /XObject << {xobj_dict}>> >> /Contents {content_obj_id} 0 R{annots_clause} >>",
            builder.page_w, builder.page_h
        );
        objects.push(page_dict.into_bytes());

        let content_bytes = content_str.as_bytes();
        let mut stream_obj = format!("<< /Length {} >>\nstream\n", content_bytes.len()).into_bytes();
        stream_obj.extend_from_slice(content_bytes);
        stream_obj.extend_from_slice(b"endstream");
        objects.push(stream_obj);
    }
    objects.extend(annot_objects);

    let mut pdf = Vec::with_capacity(16384);
    pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets = Vec::with_capacity(objects.len());
    for (idx, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        let obj_num = idx + 1;
        pdf.extend_from_slice(format!("{obj_num} 0 obj\n").as_bytes());
        pdf.extend_from_slice(body);
        pdf.extend_from_slice(b"\nendobj\n");
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets {
        pdf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    pdf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use zip::ZipArchive;

    const RICH_TEST_MARKDOWN: &str = "\
# Documento Completo di Test

Questo paragrafo contiene **grassetto**, *corsivo*, ~~barrato~~, `codice_inline` e caratteri speciali: 100% & $50_1.

## 1. Tabella e Lista

- [x] Esportazione PDF nativa
- [ ] Esportazione Word e ODT

| Formato | Libreria | Stato |
|:---|:---:|---:|
| **PDF** | `Pure Rust` PDF 1.4 | [OK](https://accent.dev) |
| *ODT* | ODF 1.2 + `zip` | ~~bozza~~ OK |

---

## 2. Diagramma Mermaid

```mermaid
flowchart LR
  A[Markdown AST] --> B{Esportatore}
  B -->|PDF| C[Documento PDF]
  B -->|ODT| D[Archivio ODF]
```
";

    #[test]
    fn test_render_markdown_to_html_unified_comrak_and_mermaid_gating() {
        let md = "\
# Title

all you can ~~eat~~ drink at https://accent-editor.org!

<kbd>Ctrl+S</kbd>
<script>alert(1)</script>

```mermaid
flowchart LR
  A[Input] --> B[Output]
```
";
        let html_gfm = render_markdown_to_html(md, MarkdownConfig::gfm(), MermaidTheme::Light);
        assert!(html_gfm.contains("<h1>Title</h1>"));
        assert!(html_gfm.contains("<del>eat</del>"));
        assert!(
            html_gfm.contains("<a href=\"https://accent-editor.org\">https://accent-editor.org</a>!")
        );
        assert!(html_gfm.contains("<kbd>Ctrl+S</kbd>"));
        assert!(!html_gfm.contains("<script>") && html_gfm.contains("&lt;script>"));
        assert!(html_gfm.contains("class=\"accent-mermaid-block\""));
        assert!(html_gfm.contains("<svg"));

        let mut gfm_no_mermaid = MarkdownConfig::gfm();
        gfm_no_mermaid.extensions.mermaid = false;
        let html_no_mermaid = render_markdown_to_html(md, gfm_no_mermaid, MermaidTheme::Light);
        assert!(!html_no_mermaid.contains("accent-mermaid-block"));
        assert!(html_no_mermaid.contains("data-code-lang=\"mermaid\""));

        let html_cm = render_markdown_to_html(md, MarkdownConfig::commonmark(), MermaidTheme::Light);
        assert!(!html_cm.contains("<del>"));
        assert!(!html_cm.contains("<a href=\"https://accent-editor.org\">"));
        assert!(!html_cm.contains("accent-mermaid-block"));
        assert!(!html_cm.contains("<script>") && html_cm.contains("&lt;script>"));
    }

    #[test]
    fn test_all_exporters_pdf_docx_odt_latex_epub_and_standalone_html() {
        let buf = DocumentBuffer::from_str_content(RICH_TEST_MARKDOWN);
        let mut cfg = MarkdownConfig::gfm();
        cfg.extensions.math = false;

        // 1. PDF (.pdf)
        let pdf_bytes = export_document_bytes(&buf, cfg, "Test Export", ExportFormat::Pdf, None)
            .expect("PDF export failed");
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
        assert!(pdf_bytes.len() > 2000);
        assert!(pdf_bytes.ends_with(b"%%EOF\n"));
        let pdf_ascii = String::from_utf8_lossy(&pdf_bytes);
        assert!(
            pdf_ascii.contains("/URI (https://accent.dev)"),
            "PDF must include clickable URI annotation for links inside table cells"
        );

        // 2. Word (.docx)
        let docx_bytes = export_document_bytes(&buf, cfg, "Test Export", ExportFormat::Docx, None)
            .expect("DOCX export failed");
        let mut docx_zip = ZipArchive::new(Cursor::new(&docx_bytes)).expect("valid DOCX zip");
        let mut doc_xml = String::new();
        docx_zip
            .by_name("word/document.xml")
            .expect("word/document.xml must exist in .docx")
            .read_to_string(&mut doc_xml)
            .expect("read word/document.xml");
        assert!(doc_xml.contains("Documento Completo di Test"));
        assert!(doc_xml.contains("Esportazione PDF nativa"));
        assert!(doc_xml.contains(r#"w:val="center""#));
        assert!(doc_xml.contains(r#"w:val="right""#));

        // 3. OpenDocument Text (.odt)
        let odt_bytes = export_document_bytes(&buf, cfg, "Test Export", ExportFormat::Odt, None)
            .expect("ODT export failed");
        let mut odt_zip = ZipArchive::new(Cursor::new(&odt_bytes)).expect("valid ODT zip");
        {
            let first_entry = odt_zip.by_index(0).expect("first entry in ODT");
            assert_eq!(first_entry.name(), "mimetype");
            assert_eq!(first_entry.compression(), CompressionMethod::Stored);
        }
        let mut odt_content = String::new();
        odt_zip
            .by_name("content.xml")
            .expect("content.xml must exist in .odt")
            .read_to_string(&mut odt_content)
            .expect("read content.xml");
        assert!(odt_content.contains("<text:h"));
        assert!(odt_content.contains("<table:table"));
        assert!(odt_content.contains("PCellCenter"));
        assert!(odt_content.contains("PCellRight"));
        assert!(odt_content.contains("PHorizontalRule"));
        assert!(odt_content.contains("Pictures/image_"));

        // 4. LaTeX (.tex)
        let tex_bytes = export_document_bytes(&buf, cfg, "Test Export", ExportFormat::Latex, None)
            .expect("LaTeX export failed");
        let tex_str = String::from_utf8(tex_bytes).expect("valid UTF-8 .tex");
        assert!(tex_str.contains("\\documentclass[11pt,a4paper]{article}"));
        assert!(tex_str.contains("\\begin{tabular}{lcr}"));
        assert!(tex_str.contains("\\textbf{PDF}"));
        assert!(tex_str.contains("\\href{https://accent.dev}{OK}"));
        assert!(tex_str.contains("\\rule{\\textwidth}{0.4pt}"));
        assert!(tex_str.contains(r"100\% \& \$50\_1"));

        // 5. EPUB (.epub)
        let epub_bytes = export_document_bytes(&buf, cfg, "Test Export", ExportFormat::Epub, None)
            .expect("EPUB export failed");
        let mut epub_zip = ZipArchive::new(Cursor::new(&epub_bytes)).expect("valid EPUB zip");
        {
            let first = epub_zip.by_index(0).expect("first entry in EPUB");
            assert_eq!(first.name(), "mimetype");
            assert_eq!(first.compression(), CompressionMethod::Stored);
        }
        let mut ch1 = String::new();
        epub_zip
            .by_name("OEBPS/chapter1.xhtml")
            .expect("OEBPS/chapter1.xhtml must exist")
            .read_to_string(&mut ch1)
            .expect("read chapter1.xhtml");
        assert!(ch1.contains("text-align: center;"));
        assert!(ch1.contains("text-align: right;"));
        assert!(ch1.contains("<strong>PDF</strong>"));
        assert!(ch1.contains("<hr />"));
        assert!(ch1.contains("images/mermaid_"));

        // 6. Standalone HTML (.html)
        let html_bytes = export_document_bytes(&buf, cfg, "Test Export", ExportFormat::Html, None)
            .expect("HTML export failed");
        let html_str = String::from_utf8(html_bytes).expect("valid UTF-8 .html");
        assert!(html_str.contains("<!DOCTYPE html>"));
        assert!(html_str.contains("<article class=\"preview-prose\">"));
        assert!(html_str.contains("<del>barrato</del>"));
        assert!(html_str.contains("class=\"accent-mermaid-block\""));
    }

    #[test]
    fn test_phase_5_4_local_image_resolution_svg_sanitization_and_all_exporters() {
        let tmp_root = std::env::temp_dir().join(format!(
            "accent_phase_5_4_export_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp_root);
        let docs_dir = tmp_root.join("docs");
        let assets_dir = docs_dir.join("assets");
        let shared_dir = tmp_root.join("shared");
        std::fs::create_dir_all(&assets_dir).expect("create assets_dir");
        std::fs::create_dir_all(&shared_dir).expect("create shared_dir");

        // Create a real valid PNG image via resvg
        let sample_svg_clean = r##"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="48" viewBox="0 0 96 48"><rect width="96" height="48" rx="8" fill="#25589E"/><circle cx="28" cy="24" r="12" fill="#F2C12E"/></svg>"##;
        let (_w, _h, _rgba, png_bytes) =
            accent_mermaid::rasterize_svg(sample_svg_clean, 1.0).expect("rasterize sample SVG");
        std::fs::write(assets_dir.join("diagram.png"), &png_bytes).expect("write diagram.png");
        std::fs::write(assets_dir.join("schema con spazi.png"), &png_bytes)
            .expect("write spaced png");
        std::fs::write(shared_dir.join("parent_logo.png"), &png_bytes)
            .expect("write parent_logo.png");
        std::fs::write(assets_dir.join("notes.txt"), b"not an image").expect("write txt");

        // Create an SVG containing script/event payloads to verify sanitization
        let dirty_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32" viewBox="0 0 64 32" onload="evil()"><script>alert('xss')</script><foreignObject><div>bad</div></foreignObject><rect onclick="steal()" width="64" height="32" fill="#1E824C"/></svg>"##;
        std::fs::write(assets_dir.join("badge.svg"), dirty_svg).expect("write badge.svg");

        // 1. Verify path resolution & security checks
        assert!(resolve_local_image_file("./assets/diagram.png", Some(&docs_dir)).is_some());
        assert!(
            resolve_local_image_file("./assets/schema%20con%20spazi.png", Some(&docs_dir))
                .is_some()
        );
        assert!(
            resolve_local_image_file("<./assets/schema con spazi.png>", Some(&docs_dir)).is_some()
        );
        assert!(resolve_local_image_file("../shared/parent_logo.png", Some(&docs_dir)).is_some());
        assert!(resolve_local_image_file("./assets/diagram.png", None).is_none());
        assert!(resolve_local_image_file("./assets/missing.png", Some(&docs_dir)).is_none());
        assert!(resolve_local_image_file("./assets/notes.txt", Some(&docs_dir)).is_none());
        assert!(resolve_local_image_file("https://example.com/diagram.png", Some(&docs_dir)).is_none());
        assert!(resolve_local_image_file("/etc/passwd", Some(&docs_dir)).is_none());

        let sanitized_svg = sanitize_svg_markup(dirty_svg);
        assert!(!sanitized_svg.to_ascii_lowercase().contains("<script"));
        assert!(!sanitized_svg.to_ascii_lowercase().contains("<foreignobject"));
        assert!(!sanitized_svg.to_ascii_lowercase().contains("onload="));
        assert!(!sanitized_svg.to_ascii_lowercase().contains("onclick="));
        assert!(sanitized_svg.contains("fill=\"#1E824C\""));

        // 2. Verify WebView Preview HTML rewriter (asset: for PNG, sanitized data: for SVG, fallback for missing/remote/unsaved)
        let md_doc = "\
# Report Architettura 5.4

Paragrafo con break\\\nSeconda riga e badge inline ![Badge SVG](./assets/badge.svg \"SVG Sicuro\") più [Sito](https://accent.dev \"Portale\").

![Diagramma Locale](./assets/diagram.png \"Schema PNG\")

![Con Spazi](./assets/schema%20con%20spazi.png)

![Logo Padre](../shared/parent_logo.png)

![Non Trovata](./assets/non_esiste.png)

![Remota](https://example.com/remote.png)

---

> [!NOTE]
> Nota multiparagrafo.
>
> Secondo paragrafo nella citazione:
> - Punto interno 1
> - Punto interno 2

4. [x] Fase quattro completata
   - Sotto-punto annidato 1
   - Sotto-punto annidato 2
5. [ ] Fase cinque aperta

| Componente | Allineamento Centro | Allineamento Destra |
|:---|:---:|---:|
| **Core** `L1` | * Corsivo * e ~~vecchio~~ | [Spec](https://accent.dev)<br>Riga 2 |
";

        let mut allowed_files = Vec::new();
        let preview_html = render_markdown_to_webview_preview_html(
            md_doc,
            MarkdownConfig::gfm(),
            Some(&docs_dir),
            |_| String::new(),
            |p| allowed_files.push(p.to_path_buf()),
        );
        assert_eq!(allowed_files.len(), 3, "Three local PNGs must be registered in asset scope");
        assert!(
            preview_html.contains("asset://localhost/")
                || preview_html.contains("http://asset.localhost/"),
            "Raster local images in Preview Pane must use Tauri asset: URL"
        );
        assert!(
            preview_html.contains("data:image/svg+xml;base64,"),
            "SVG local images in Preview Pane must be sanitized data URIs"
        );
        assert!(
            preview_html.contains("data-testid=\"preview-image-fallback\""),
            "Missing and remote images in Preview Pane must render clean fallback badge"
        );
        assert!(
            !preview_html.contains("src=\"https://example.com/remote.png\""),
            "Preview Pane must never emit remote https:// img requests"
        );

        let preview_unsaved = render_markdown_to_webview_preview_html(
            "![Diagramma Locale](./assets/diagram.png)",
            MarkdownConfig::gfm(),
            None,
            |_| String::new(),
            |_| {},
        );
        assert!(preview_unsaved.contains("data-testid=\"preview-image-fallback\""));

        // 3. Export across all 6 formats with Some(&docs_dir)
        let buf = DocumentBuffer::from_str_content(md_doc);
        let cfg = MarkdownConfig::gfm();

        // 3a. Standalone HTML: data:image/png;base64 & data:image/svg+xml;base64, zero asset://localhost
        let html_out = String::from_utf8(
            export_document_bytes(&buf, cfg, "Export 5.4", ExportFormat::Html, Some(&docs_dir))
                .expect("HTML export"),
        )
        .unwrap();
        assert!(html_out.contains("data:image/png;base64,"));
        assert!(html_out.contains("data:image/svg+xml;base64,"));
        assert!(!html_out.contains("asset://localhost"));
        assert!(!html_out.contains("[Immagine: Diagramma Locale"));

        // 3b. PDF: embedded Image XObjects, no [Immagine: Diagramma Locale] placeholder, keeps fallback only for missing/remote
        let pdf_out =
            export_document_bytes(&buf, cfg, "Export 5.4", ExportFormat::Pdf, Some(&docs_dir))
                .expect("PDF export");
        let pdf_lossy = String::from_utf8_lossy(&pdf_out);
        assert!(pdf_out.starts_with(b"%PDF-1.4"));
        assert!(pdf_lossy.contains("/Subtype /Image"));
        assert!(pdf_lossy.contains("/Im1 Do"));
        assert!(!pdf_lossy.contains("Immagine: Diagramma Locale"));
        assert!(!pdf_lossy.contains("Immagine: Badge SVG"));
        assert!(pdf_lossy.contains("Immagine: Non Trovata"));

        // 3c. DOCX: word/media/ entries + <a:blip>, alignments, hyperlinks, no [Immagine: Diagramma Locale]
        let docx_out =
            export_document_bytes(&buf, cfg, "Export 5.4", ExportFormat::Docx, Some(&docs_dir))
                .expect("DOCX export");
        let mut docx_zip = ZipArchive::new(Cursor::new(&docx_out)).expect("DOCX zip");
        let has_word_media = (0..docx_zip.len()).any(|idx| {
            docx_zip
                .by_index(idx)
                .map(|f| f.name().starts_with("word/media/"))
                .unwrap_or(false)
        });
        assert!(has_word_media, "DOCX archive must embed local images in word/media/");
        let mut docx_xml = String::new();
        docx_zip
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut docx_xml)
            .unwrap();
        assert!(docx_xml.contains("a:blip"));
        assert!(!docx_xml.contains("[Immagine: Diagramma Locale"));
        assert!(!docx_xml.contains("[Immagine: Badge SVG"));
        assert!(docx_xml.contains("[Immagine: Non Trovata"));
        assert!(docx_xml.contains("4. "));

        // 3d. ODT: Pictures/ entries + <draw:image>, alignments, start-value="4", <text:line-break/>
        let odt_out =
            export_document_bytes(&buf, cfg, "Export 5.4", ExportFormat::Odt, Some(&docs_dir))
                .expect("ODT export");
        let mut odt_zip = ZipArchive::new(Cursor::new(&odt_out)).expect("ODT zip");
        let has_odt_pics = (0..odt_zip.len()).any(|idx| {
            odt_zip
                .by_index(idx)
                .map(|f| f.name().starts_with("Pictures/image_"))
                .unwrap_or(false)
        });
        assert!(has_odt_pics, "ODT archive must embed local images in Pictures/");
        let mut odt_xml = String::new();
        odt_zip
            .by_name("content.xml")
            .unwrap()
            .read_to_string(&mut odt_xml)
            .unwrap();
        assert!(odt_xml.contains("<draw:image"));
        assert!(odt_xml.contains("<text:line-break/>"));
        assert!(odt_xml.contains("text:start-value=\"4\""));
        assert!(!odt_xml.contains("[Immagine: Diagramma Locale"));
        assert!(odt_xml.contains("[Immagine: Non Trovata"));

        // 3e. EPUB: OEBPS/images/ + manifest items + <img src="images/..."/>, <ol start="4">
        let epub_out =
            export_document_bytes(&buf, cfg, "Export 5.4", ExportFormat::Epub, Some(&docs_dir))
                .expect("EPUB export");
        let mut epub_zip = ZipArchive::new(Cursor::new(&epub_out)).expect("EPUB zip");
        let mut opf_xml = String::new();
        epub_zip
            .by_name("OEBPS/content.opf")
            .unwrap()
            .read_to_string(&mut opf_xml)
            .unwrap();
        assert!(opf_xml.contains("media-type=\"image/png\""));
        assert!(opf_xml.contains("media-type=\"image/svg+xml\""));
        let mut ch_xml = String::new();
        epub_zip
            .by_name("OEBPS/chapter1.xhtml")
            .unwrap()
            .read_to_string(&mut ch_xml)
            .unwrap();
        assert!(ch_xml.contains("<img src=\"images/img_"));
        assert!(ch_xml.contains("<ol start=\"4\">"));
        assert!(!ch_xml.contains("[Immagine: Diagramma Locale"));
        assert!(ch_xml.contains("[Immagine: Non Trovata"));

        // 3f. LaTeX: active \includegraphics (not commented out), lcr tabular, enumi counter
        let tex_out = String::from_utf8(
            export_document_bytes(&buf, cfg, "Export 5.4", ExportFormat::Latex, Some(&docs_dir))
                .expect("LaTeX export"),
        )
        .unwrap();
        assert!(tex_out.contains("\\includegraphics[width=0.8\\textwidth]{"));
        assert!(tex_out.contains("\\includegraphics[height=1.4em]{"));
        assert!(tex_out.contains("\\begin{tabular}{lcr}"));
        assert!(tex_out.contains("\\setcounter{enumi}{3}"));
        assert!(!tex_out.contains("[Immagine: Diagramma Locale"));
        assert!(tex_out.contains("[Immagine: Non Trovata"));

        let _ = std::fs::remove_dir_all(&tmp_root);
    }
}
