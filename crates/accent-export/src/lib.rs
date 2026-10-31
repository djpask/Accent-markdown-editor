use accent_core::{DocumentBuffer, MarkdownDialect, PreviewBlock, PreviewInline};
use accent_mermaid::{MermaidCache, MermaidTheme};
use docx_rs::{Docx, Paragraph, Pic, Run, RunFonts, Table, TableCell, TableRow};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
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

pub fn render_markdown_to_html(markdown: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

/// Top-level dispatcher that exports a `DocumentBuffer` (or Markdown string) to the target `ExportFormat`.
pub fn export_document_bytes(
    buffer: &DocumentBuffer,
    dialect: MarkdownDialect,
    title: &str,
    format: ExportFormat,
) -> Result<Vec<u8>, String> {
    let blocks = buffer.parse_preview_blocks(dialect);
    match format {
        ExportFormat::Pdf => Ok(export_to_pdf(&blocks, title)),
        ExportFormat::Docx => export_to_docx(&blocks, title),
        ExportFormat::Odt => export_to_odt(&blocks, title),
        ExportFormat::Latex => Ok(export_to_latex(&blocks, title).into_bytes()),
        ExportFormat::Epub => export_to_epub(&blocks, title),
        ExportFormat::Html => Ok(export_to_standalone_html(&blocks, title).into_bytes()),
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

fn serialize_inlines_to_latex(inlines: &[PreviewInline]) -> String {
    let mut out = String::new();
    for span in inlines {
        // Preserve native inline/display math expressions ($...$ / $$...$$)
        if span.code && span.italic && span.text.starts_with('$') && span.text.ends_with('$') {
            out.push_str(&span.text);
            continue;
        }
        let mut chunk = escape_latex_text(&span.text);
        if span.code {
            chunk = format!(r"\texttt{{{chunk}}}");
        }
        if span.bold {
            chunk = format!(r"\textbf{{{chunk}}}");
        }
        if span.italic {
            chunk = format!(r"\textit{{{chunk}}}");
        }
        if span.strikethrough {
            chunk = format!(r"\sout{{{chunk}}}");
        }
        if let Some(ref url) = span.link_url {
            let safe_url = url.replace('%', r"\%").replace('#', r"\#");
            chunk = format!(r"\href{{{safe_url}}}{{{chunk}}}");
        }
        out.push_str(&chunk);
    }
    out
}

/// Serializes the AST `PreviewBlock`s into a complete, self-contained LaTeX `.tex` document.
pub fn export_to_latex(blocks: &[PreviewBlock], title: &str) -> String {
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
                let inner = serialize_inlines_to_latex(inlines);
                tex.push_str(&format!("\\{cmd}{{{inner}}}\n\n"));
            }
            PreviewBlock::Paragraph { inlines } => {
                let inner = serialize_inlines_to_latex(inlines);
                tex.push_str(&format!("{inner}\n\n"));
            }
            PreviewBlock::BlockQuote { inlines } => {
                let inner = serialize_inlines_to_latex(inlines);
                tex.push_str("\\begin{quote}\n");
                tex.push_str(&inner);
                tex.push_str("\n\\end{quote}\n\n");
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
            PreviewBlock::List { ordered, items } => {
                let env = if *ordered { "enumerate" } else { "itemize" };
                tex.push_str(&format!("\\begin{{{env}}}\n"));
                for item in items {
                    let prefix = match item.task {
                        Some(true) => "$\\boxtimes$ ",
                        Some(false) => "$\\square$ ",
                        None => "",
                    };
                    let inner = serialize_inlines_to_latex(&item.inlines);
                    tex.push_str(&format!("  \\item {prefix}{inner}\n"));
                }
                tex.push_str(&format!("\\end{{{env}}}\n\n"));
            }
            PreviewBlock::Table { headers, rows } => {
                let cols = headers.len().max(1);
                let col_spec = "l".repeat(cols);
                tex.push_str(&format!("\\begin{{tabular}}{{{col_spec}}}\n\\toprule\n"));
                if !headers.is_empty() {
                    let h_cells: Vec<String> = headers
                        .iter()
                        .map(|h| format!("\\textbf{{{}}}", escape_latex_text(h)))
                        .collect();
                    tex.push_str(&format!("{} \\\\\n\\midrule\n", h_cells.join(" & ")));
                }
                for row in rows {
                    let r_cells: Vec<String> = row.iter().map(|c| escape_latex_text(c)).collect();
                    tex.push_str(&format!("{} \\\\\n", r_cells.join(" & ")));
                }
                tex.push_str("\\bottomrule\n\\end{tabular}\n\n");
            }
            PreviewBlock::Image { alt, url } => {
                let esc_alt = escape_latex_text(alt);
                tex.push_str(&format!(
                    "\\begin{{figure}}[h]\n\\centering\n% \\includegraphics[width=0.8\\textwidth]{{{}}}\n\\caption{{{esc_alt}}}\n\\end{{figure}}\n\n",
                    escape_latex_text(url)
                ));
            }
        }
    }

    tex.push_str("\\end{document}\n");
    tex
}

// ============================================================================
// 2. PURE-RUST WORD (.docx) EXPORTER VIA `docx-rs`
// ============================================================================

fn build_docx_paragraph_from_inlines(inlines: &[PreviewInline], base_size_half_pt: Option<usize>, force_bold: bool) -> Paragraph {
    let mut p = Paragraph::new();
    for span in inlines {
        let mut r = Run::new().add_text(&span.text);
        if force_bold || span.bold {
            r = r.bold();
        }
        if span.italic {
            r = r.italic();
        }
        if span.strikethrough {
            r = r.strike();
        }
        if span.code {
            r = r.fonts(RunFonts::new().ascii("Courier New").hi_ansi("Courier New"));
        }
        if let Some(sz) = base_size_half_pt {
            r = r.size(sz);
        }
        p = p.add_run(r);
    }
    p
}

/// Generates a structured OpenXML `.docx` document using `docx-rs`, including embedded Mermaid PNG images.
pub fn export_to_docx(blocks: &[PreviewBlock], _title: &str) -> Result<Vec<u8>, String> {
    let mut docx = Docx::new();
    let mut mermaid_cache = MermaidCache::new();

    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let sz = match level {
                    1 => 36,
                    2 => 30,
                    3 => 26,
                    _ => 24,
                };
                let p = build_docx_paragraph_from_inlines(inlines, Some(sz), true);
                docx = docx.add_paragraph(p);
            }
            PreviewBlock::Paragraph { inlines } => {
                let p = build_docx_paragraph_from_inlines(inlines, Some(22), false);
                docx = docx.add_paragraph(p);
            }
            PreviewBlock::BlockQuote { inlines } => {
                let mut p = Paragraph::new();
                for span in inlines {
                    let mut r = Run::new().add_text(&span.text).italic();
                    if span.bold {
                        r = r.bold();
                    }
                    p = p.add_run(r);
                }
                docx = docx.add_paragraph(p);
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if !rendered.png_bytes.is_empty() {
                        let pic = Pic::new(&rendered.png_bytes).size(
                            (rendered.width as u32).saturating_mul(6500),
                            (rendered.height as u32).saturating_mul(6500),
                        );
                        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic)));
                        continue;
                    }
                }
                for line in code.lines() {
                    let r = Run::new()
                        .add_text(line)
                        .fonts(RunFonts::new().ascii("Courier New").hi_ansi("Courier New"))
                        .size(19);
                    docx = docx.add_paragraph(Paragraph::new().add_run(r));
                }
            }
            PreviewBlock::List { ordered, items } => {
                for (idx, item) in items.iter().enumerate() {
                    let prefix = match item.task {
                        Some(true) => "☑ ".to_string(),
                        Some(false) => "☐ ".to_string(),
                        None if *ordered => format!("{}. ", idx + 1),
                        None => "• ".to_string(),
                    };
                    let mut inlines_with_prefix = Vec::with_capacity(item.inlines.len() + 1);
                    inlines_with_prefix.push(PreviewInline {
                        text: prefix,
                        bold: false,
                        italic: false,
                        strikethrough: false,
                        code: false,
                        link_url: None,
                    });
                    inlines_with_prefix.extend(item.inlines.iter().cloned());
                    let p = build_docx_paragraph_from_inlines(&inlines_with_prefix, Some(22), false);
                    docx = docx.add_paragraph(p);
                }
            }
            PreviewBlock::Table { headers, rows } => {
                let mut t_rows = Vec::new();
                if !headers.is_empty() {
                    let cells: Vec<TableCell> = headers
                        .iter()
                        .map(|h| {
                            TableCell::new()
                                .add_paragraph(Paragraph::new().add_run(Run::new().add_text(h).bold()))
                        })
                        .collect();
                    t_rows.push(TableRow::new(cells));
                }
                for r in rows {
                    let cells: Vec<TableCell> = r
                        .iter()
                        .map(|c| {
                            TableCell::new()
                                .add_paragraph(Paragraph::new().add_run(Run::new().add_text(c)))
                        })
                        .collect();
                    t_rows.push(TableRow::new(cells));
                }
                if !t_rows.is_empty() {
                    docx = docx.add_table(Table::new(t_rows));
                }
            }
            PreviewBlock::Image { alt, url } => {
                let p = Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!("[Immagine: {alt} ({url})]"))
                        .italic(),
                );
                docx = docx.add_paragraph(p);
            }
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

fn serialize_inlines_to_odf(inlines: &[PreviewInline]) -> String {
    let mut out = String::new();
    for span in inlines {
        let esc = xml_escape(&span.text);
        if span.code {
            out.push_str(&format!(r#"<text:span text:style-name="TCode">{esc}</text:span>"#));
        } else if span.bold && span.italic {
            out.push_str(&format!(r#"<text:span text:style-name="TBoldItalic">{esc}</text:span>"#));
        } else if span.bold {
            out.push_str(&format!(r#"<text:span text:style-name="TBold">{esc}</text:span>"#));
        } else if span.italic {
            out.push_str(&format!(r#"<text:span text:style-name="TItalic">{esc}</text:span>"#));
        } else if span.strikethrough {
            out.push_str(&format!(r#"<text:span text:style-name="TStrike">{esc}</text:span>"#));
        } else {
            out.push_str(&esc);
        }
    }
    out
}

/// Builds a compliant ODF 1.2 `.odt` ZIP archive with an uncompressed `mimetype` header as the first entry.
pub fn export_to_odt(blocks: &[PreviewBlock], title: &str) -> Result<Vec<u8>, String> {
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
    let mut picture_entries: Vec<String> = Vec::new();
    let mut body_xml = String::with_capacity(4096);

    for (b_idx, block) in blocks.iter().enumerate() {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let inner = serialize_inlines_to_odf(inlines);
                body_xml.push_str(&format!(
                    r#"<text:h text:style-name="Heading_20_{level}" text:outline-level="{level}">{inner}</text:h>"#
                ));
            }
            PreviewBlock::Paragraph { inlines } => {
                let inner = serialize_inlines_to_odf(inlines);
                body_xml.push_str(&format!(
                    r#"<text:p text:style-name="Text_20_body">{inner}</text:p>"#
                ));
            }
            PreviewBlock::BlockQuote { inlines } => {
                let inner = serialize_inlines_to_odf(inlines);
                body_xml.push_str(&format!(
                    r#"<text:p text:style-name="Quotations">{inner}</text:p>"#
                ));
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if !rendered.png_bytes.is_empty() {
                        let pic_path = format!("Pictures/mermaid_{b_idx}.png");
                        zip.start_file(&pic_path, deflate_opts)
                            .map_err(|e| e.to_string())?;
                        zip.write_all(&rendered.png_bytes)
                            .map_err(|e| e.to_string())?;
                        picture_entries.push(pic_path.clone());
                        body_xml.push_str(&format!(
                            r#"<text:p text:style-name="Text_20_body"><draw:frame draw:name="Mermaid_{b_idx}" text:anchor-type="as-char" svg:width="14cm" svg:height="6cm"><draw:image xlink:href="{pic_path}" xlink:type="simple" xlink:show="embed" xlink:actuate="onLoad"/></draw:frame></text:p>"#
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
            PreviewBlock::List { items, .. } => {
                body_xml.push_str("<text:list>");
                for item in items {
                    let prefix = match item.task {
                        Some(true) => "☑ ",
                        Some(false) => "☐ ",
                        None => "",
                    };
                    let inner = serialize_inlines_to_odf(&item.inlines);
                    body_xml.push_str(&format!(
                        r#"<text:list-item><text:p text:style-name="Text_20_body">{prefix}{inner}</text:p></text:list-item>"#
                    ));
                }
                body_xml.push_str("</text:list>");
            }
            PreviewBlock::Table { headers, rows } => {
                let cols = headers.len().max(1);
                body_xml.push_str(&format!(
                    r#"<table:table table:name="Table_{b_idx}"><table:table-column table:number-columns-repeated="{cols}"/>"#
                ));
                if !headers.is_empty() {
                    body_xml.push_str("<table:table-row>");
                    for h in headers {
                        body_xml.push_str(&format!(
                            r#"<table:table-cell office:value-type="string"><text:p text:style-name="TBold">{}</text:p></table:table-cell>"#,
                            xml_escape(h)
                        ));
                    }
                    body_xml.push_str("</table:table-row>");
                }
                for row in rows {
                    body_xml.push_str("<table:table-row>");
                    for cell in row {
                        body_xml.push_str(&format!(
                            r#"<table:table-cell office:value-type="string"><text:p text:style-name="Text_20_body">{}</text:p></table:table-cell>"#,
                            xml_escape(cell)
                        ));
                    }
                    body_xml.push_str("</table:table-row>");
                }
                body_xml.push_str("</table:table>");
            }
            PreviewBlock::Image { alt, url } => {
                body_xml.push_str(&format!(
                    r#"<text:p text:style-name="Text_20_body">[{} ({})]</text:p>"#,
                    xml_escape(alt),
                    xml_escape(url)
                ));
            }
        }
    }

    let styles_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.2">
  <office:styles>
    <style:style style:name="Text_20_body" style:family="paragraph"><style:paragraph-properties fo:margin-bottom="0.25cm"/></style:style>
    <style:style style:name="Heading_20_1" style:family="paragraph"><style:text-properties fo:font-size="20pt" fo:font-weight="bold"/></style:style>
    <style:style style:name="Heading_20_2" style:family="paragraph"><style:text-properties fo:font-size="16pt" fo:font-weight="bold"/></style:style>
    <style:style style:name="Heading_20_3" style:family="paragraph"><style:text-properties fo:font-size="13pt" fo:font-weight="bold"/></style:style>
    <style:style style:name="Quotations" style:family="paragraph"><style:paragraph-properties fo:margin-left="1cm"/><style:text-properties fo:font-style="italic"/></style:style>
    <style:style style:name="Preformatted_20_Text" style:family="paragraph"><style:text-properties style:font-name="monospace" fo:font-size="10pt"/></style:style>
  </office:styles>
</office:document-styles>"#;

    let content_xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" office:version="1.2">
  <office:automatic-styles>
    <style:style style:name="TBold" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
    <style:style style:name="TItalic" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
    <style:style style:name="TBoldItalic" style:family="text"><style:text-properties fo:font-weight="bold" fo:font-style="italic"/></style:style>
    <style:style style:name="TStrike" style:family="text"><style:text-properties style:text-line-through-style="solid"/></style:style>
    <style:style style:name="TCode" style:family="text"><style:text-properties style:font-name="monospace" fo:background-color="#F0EFEA"/></style:style>
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
    for pic in &picture_entries {
        manifest_xml.push_str(&format!(
            "\n  <manifest:file-entry manifest:full-path=\"{pic}\" manifest:media-type=\"image/png\"/>"
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

fn serialize_inlines_to_xhtml(inlines: &[PreviewInline]) -> String {
    let mut out = String::new();
    for span in inlines {
        let mut chunk = xml_escape(&span.text);
        if span.code {
            chunk = format!("<code>{chunk}</code>");
        }
        if span.bold {
            chunk = format!("<strong>{chunk}</strong>");
        }
        if span.italic {
            chunk = format!("<em>{chunk}</em>");
        }
        if span.strikethrough {
            chunk = format!("<del>{chunk}</del>");
        }
        if let Some(ref url) = span.link_url {
            chunk = format!(r#"<a href="{}">{chunk}</a>"#, xml_escape(url));
        }
        out.push_str(&chunk);
    }
    out
}

pub fn export_to_standalone_html(blocks: &[PreviewBlock], title: &str) -> String {
    let mut body = String::with_capacity(4096);
    let mut mermaid_cache = MermaidCache::new();
    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let lv = (*level).clamp(1, 6);
                body.push_str(&format!(
                    "<h{lv}>{}</h{lv}>\n",
                    serialize_inlines_to_xhtml(inlines)
                ));
            }
            PreviewBlock::Paragraph { inlines } => {
                body.push_str(&format!("<p>{}</p>\n", serialize_inlines_to_xhtml(inlines)));
            }
            PreviewBlock::BlockQuote { inlines } => {
                body.push_str(&format!(
                    "<blockquote><p>{}</p></blockquote>\n",
                    serialize_inlines_to_xhtml(inlines)
                ));
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    body.push_str(&format!(
                        "<div class=\"mermaid-diagram\">{}</div>\n",
                        rendered.svg
                    ));
                } else {
                    body.push_str(&format!(
                        "<pre><code>{}</code></pre>\n",
                        xml_escape(code)
                    ));
                }
            }
            PreviewBlock::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                body.push_str(&format!("<{tag}>\n"));
                for item in items {
                    let prefix = match item.task {
                        Some(true) => "☑ ",
                        Some(false) => "☐ ",
                        None => "",
                    };
                    body.push_str(&format!(
                        "  <li>{prefix}{}</li>\n",
                        serialize_inlines_to_xhtml(&item.inlines)
                    ));
                }
                body.push_str(&format!("</{tag}>\n"));
            }
            PreviewBlock::Table { headers, rows } => {
                body.push_str("<table>\n");
                if !headers.is_empty() {
                    body.push_str("  <thead><tr>");
                    for h in headers {
                        body.push_str(&format!("<th>{}</th>", xml_escape(h)));
                    }
                    body.push_str("</tr></thead>\n");
                }
                body.push_str("  <tbody>\n");
                for row in rows {
                    body.push_str("    <tr>");
                    for c in row {
                        body.push_str(&format!("<td>{}</td>", xml_escape(c)));
                    }
                    body.push_str("</tr>\n");
                }
                body.push_str("  </tbody>\n</table>\n");
            }
            PreviewBlock::Image { alt, url } => {
                body.push_str(&format!(
                    r#"<figure><img src="{}" alt="{}"/><figcaption>{}</figcaption></figure>"#,
                    xml_escape(url),
                    xml_escape(alt),
                    xml_escape(alt)
                ));
            }
        }
    }
    format!(
        "<!DOCTYPE html>\n<html lang=\"it\">\n<head><meta charset=\"utf-8\"/><title>{}</title></head>\n<body>\n{body}</body>\n</html>\n",
        xml_escape(title)
    )
}

/// Generates a compliant EPUB 3 archive containing `mimetype`, `META-INF/container.xml`, `OEBPS/content.opf`, `OEBPS/style.css`, and `OEBPS/chapter1.xhtml`.
pub fn export_to_epub(blocks: &[PreviewBlock], title: &str) -> Result<Vec<u8>, String> {
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
               h1, h2, h3 { font-family: sans-serif; color: #1a202c; }\n\
               pre, code { font-family: monospace; background: #f4f1ea; padding: 0.2em 0.4em; border-radius: 4px; }\n\
               blockquote { border-left: 3px solid #c85a17; padding-left: 1em; font-style: italic; }\n\
               table { border-collapse: collapse; width: 100%; margin: 1em 0; }\n\
               th, td { border: 1px solid #cbd5e0; padding: 0.4em 0.6em; text-align: left; }\n";
    zip.start_file("OEBPS/style.css", deflate_opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(css.as_bytes()).map_err(|e| e.to_string())?;

    let mut mermaid_cache = MermaidCache::new();
    let mut image_manifest_items = String::new();
    let mut chapter_body = String::with_capacity(4096);

    for (b_idx, block) in blocks.iter().enumerate() {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let lv = (*level).clamp(1, 6);
                chapter_body.push_str(&format!(
                    "<h{lv}>{}</h{lv}>\n",
                    serialize_inlines_to_xhtml(inlines)
                ));
            }
            PreviewBlock::Paragraph { inlines } => {
                chapter_body.push_str(&format!(
                    "<p>{}</p>\n",
                    serialize_inlines_to_xhtml(inlines)
                ));
            }
            PreviewBlock::BlockQuote { inlines } => {
                chapter_body.push_str(&format!(
                    "<blockquote><p>{}</p></blockquote>\n",
                    serialize_inlines_to_xhtml(inlines)
                ));
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if !rendered.png_bytes.is_empty() {
                        let rel_img = format!("images/mermaid_{b_idx}.png");
                        let zip_img = format!("OEBPS/{rel_img}");
                        zip.start_file(&zip_img, deflate_opts)
                            .map_err(|e| e.to_string())?;
                        zip.write_all(&rendered.png_bytes)
                            .map_err(|e| e.to_string())?;
                        image_manifest_items.push_str(&format!(
                            "\n    <item id=\"mermaid_{b_idx}\" href=\"{rel_img}\" media-type=\"image/png\"/>"
                        ));
                        chapter_body.push_str(&format!(
                            "<div class=\"mermaid\"><img src=\"{rel_img}\" alt=\"Mermaid Diagram\"/></div>\n"
                        ));
                        continue;
                    }
                }
                chapter_body.push_str(&format!(
                    "<pre><code>{}</code></pre>\n",
                    xml_escape(code)
                ));
            }
            PreviewBlock::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                chapter_body.push_str(&format!("<{tag}>\n"));
                for item in items {
                    let prefix = match item.task {
                        Some(true) => "☑ ",
                        Some(false) => "☐ ",
                        None => "",
                    };
                    chapter_body.push_str(&format!(
                        "  <li>{prefix}{}</li>\n",
                        serialize_inlines_to_xhtml(&item.inlines)
                    ));
                }
                chapter_body.push_str(&format!("</{tag}>\n"));
            }
            PreviewBlock::Table { headers, rows } => {
                chapter_body.push_str("<table>\n");
                if !headers.is_empty() {
                    chapter_body.push_str("  <thead><tr>");
                    for h in headers {
                        chapter_body.push_str(&format!("<th>{}</th>", xml_escape(h)));
                    }
                    chapter_body.push_str("</tr></thead>\n");
                }
                chapter_body.push_str("  <tbody>\n");
                for r in rows {
                    chapter_body.push_str("    <tr>");
                    for c in r {
                        chapter_body.push_str(&format!("<td>{}</td>", xml_escape(c)));
                    }
                    chapter_body.push_str("</tr>\n");
                }
                chapter_body.push_str("  </tbody>\n</table>\n");
            }
            PreviewBlock::Image { alt, url } => {
                chapter_body.push_str(&format!(
                    "<p>[{} ({})]</p>\n",
                    xml_escape(alt),
                    xml_escape(url)
                ));
            }
        }
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
{chapter_body}
</body>
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
// 5. PURE-RUST PAGINATED PDF 1.4 (.pdf) EXPORTER WITH EMBEDDED MERMAID IMAGES
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

fn wrap_plain_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur.push_str(word);
        } else if cur.chars().count() + 1 + word.chars().count() <= max_chars {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

struct PdfImageXObject {
    name: String,
    width: u32,
    height: u32,
    rgb_bytes: Vec<u8>,
}

/// Generates a paginated PDF 1.4 binary document (`%PDF-1.4`) from the AST `PreviewBlock`s,
/// including headings, paragraphs, code blocks, lists, tables, and rasterized Mermaid diagrams.
pub fn export_to_pdf(blocks: &[PreviewBlock], title: &str) -> Vec<u8> {
    let page_w = 595.28_f32;
    let page_h = 841.89_f32;
    let margin_x = 54.0_f32;
    let top_y = page_h - 56.0;
    let bottom_y = 56.0_f32;

    let mut pages_ops: Vec<String> = Vec::new();
    let mut cur_ops = String::with_capacity(4096);
    let mut cur_y = top_y;
    let mut mermaid_cache = MermaidCache::new();
    let mut embedded_images: Vec<PdfImageXObject> = Vec::new();

    let ensure_space = |needed: f32,
                        cur_y: &mut f32,
                        cur_ops: &mut String,
                        pages_ops: &mut Vec<String>| {
        if *cur_y - needed < bottom_y {
            pages_ops.push(std::mem::take(cur_ops));
            *cur_y = top_y;
        }
    };

    // Document header banner
    if !title.is_empty() {
        cur_ops.push_str(&format!(
            "BT /F2 10 Tf 0.45 0.40 0.32 rg {margin_x:.1} {:.1} Td ({}) Tj ET\n",
            cur_y,
            pdf_win_ansi_escape(title)
        ));
        cur_y -= 18.0;
    }

    for block in blocks {
        match block {
            PreviewBlock::Heading { level, inlines, .. } => {
                let (font_size, leading) = match level {
                    1 => (20.0_f32, 26.0_f32),
                    2 => (16.0_f32, 22.0_f32),
                    3 => (13.5_f32, 18.0_f32),
                    _ => (12.0_f32, 16.0_f32),
                };
                ensure_space(leading + 14.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                cur_y -= 8.0;
                let full_text: String = inlines.iter().map(|i| i.text.as_str()).collect();
                for line in wrap_plain_text(&full_text, 64) {
                    ensure_space(leading, &mut cur_y, &mut cur_ops, &mut pages_ops);
                    cur_ops.push_str(&format!(
                        "BT /F2 {font_size:.1} Tf 0.18 0.20 0.22 rg {margin_x:.1} {cur_y:.1} Td ({}) Tj ET\n",
                        pdf_win_ansi_escape(&line)
                    ));
                    cur_y -= leading;
                }
                if *level == 1 {
                    cur_ops.push_str(&format!(
                        "0.80 0.76 0.70 RG 0.8 w {margin_x:.1} {:.1} m {:.1} {:.1} l S\n",
                        cur_y + 8.0,
                        page_w - margin_x,
                        cur_y + 8.0
                    ));
                    cur_y -= 6.0;
                }
            }
            PreviewBlock::Paragraph { inlines } => {
                let full_text: String = inlines.iter().map(|i| i.text.as_str()).collect();
                let has_bold = inlines.iter().any(|i| i.bold);
                let has_italic = inlines.iter().any(|i| i.italic);
                let font_tag = if has_bold {
                    "/F2"
                } else if has_italic {
                    "/F3"
                } else {
                    "/F1"
                };
                for line in wrap_plain_text(&full_text, 78) {
                    ensure_space(16.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                    cur_ops.push_str(&format!(
                        "BT {font_tag} 11 Tf 0.18 0.20 0.22 rg {margin_x:.1} {cur_y:.1} Td ({}) Tj ET\n",
                        pdf_win_ansi_escape(&line)
                    ));
                    cur_y -= 15.5;
                }
                cur_y -= 6.0;
            }
            PreviewBlock::BlockQuote { inlines } => {
                let full_text: String = inlines.iter().map(|i| i.text.as_str()).collect();
                let wrapped = wrap_plain_text(&full_text, 72);
                let block_h = (wrapped.len() as f32) * 15.0 + 6.0;
                ensure_space(block_h, &mut cur_y, &mut cur_ops, &mut pages_ops);
                let top_bar = cur_y + 10.0;
                let bot_bar = cur_y - block_h + 14.0;
                cur_ops.push_str(&format!(
                    "0.78 0.35 0.09 RG 2.2 w {:.1} {top_bar:.1} m {:.1} {bot_bar:.1} l S\n",
                    margin_x + 4.0,
                    margin_x + 4.0
                ));
                for line in wrapped {
                    cur_ops.push_str(&format!(
                        "BT /F3 11 Tf 0.35 0.30 0.24 rg {:.1} {cur_y:.1} Td ({}) Tj ET\n",
                        margin_x + 16.0,
                        pdf_win_ansi_escape(&line)
                    ));
                    cur_y -= 15.0;
                }
                cur_y -= 6.0;
            }
            PreviewBlock::CodeBlock { info, code } => {
                let lang = info.split_whitespace().next().unwrap_or("");
                if lang.eq_ignore_ascii_case("mermaid") {
                    let rendered = mermaid_cache.get_or_render_themed(code, MermaidTheme::Light);
                    if rendered.width > 0 && rendered.height > 0 && !rendered.rgba_pixels.is_empty()
                    {
                        let img_idx = embedded_images.len() + 1;
                        let img_name = format!("Im{img_idx}");
                        let mut rgb =
                            Vec::with_capacity((rendered.width * rendered.height * 3) as usize);
                        for chunk in rendered.rgba_pixels.chunks_exact(4) {
                            rgb.push(chunk[0]);
                            rgb.push(chunk[1]);
                            rgb.push(chunk[2]);
                        }
                        let max_disp_w = page_w - margin_x * 2.0;
                        let scale = (max_disp_w / rendered.width as f32).min(0.65);
                        let disp_w = rendered.width as f32 * scale;
                        let disp_h = rendered.height as f32 * scale;
                        ensure_space(disp_h + 16.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                        let img_bottom_y = cur_y - disp_h;
                        cur_ops.push_str(&format!(
                            "q {disp_w:.2} 0 0 {disp_h:.2} {margin_x:.2} {img_bottom_y:.2} cm /{img_name} Do Q\n"
                        ));
                        cur_y = img_bottom_y - 12.0;
                        embedded_images.push(PdfImageXObject {
                            name: img_name,
                            width: rendered.width,
                            height: rendered.height,
                            rgb_bytes: rgb,
                        });
                        continue;
                    }
                }
                let code_lines: Vec<&str> = code.lines().collect();
                let box_h = (code_lines.len().max(1) as f32) * 13.5 + 12.0;
                ensure_space(box_h + 8.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                let box_bottom = cur_y - box_h + 10.0;
                cur_ops.push_str(&format!(
                    "0.95 0.94 0.91 rg {margin_x:.1} {box_bottom:.1} {:.1} {box_h:.1} re f\n",
                    page_w - margin_x * 2.0
                ));
                for cline in code_lines {
                    cur_ops.push_str(&format!(
                        "BT /F4 9.5 Tf 0.20 0.20 0.22 rg {:.1} {cur_y:.1} Td ({}) Tj ET\n",
                        margin_x + 8.0,
                        pdf_win_ansi_escape(cline)
                    ));
                    cur_y -= 13.5;
                }
                cur_y -= 10.0;
            }
            PreviewBlock::List { ordered, items } => {
                for (idx, item) in items.iter().enumerate() {
                    let prefix = match item.task {
                        Some(true) => "[x] ".to_string(),
                        Some(false) => "[ ] ".to_string(),
                        None if *ordered => format!("{}. ", idx + 1),
                        None => "- ".to_string(),
                    };
                    let item_txt: String = item.inlines.iter().map(|i| i.text.as_str()).collect();
                    let full = format!("{prefix}{item_txt}");
                    for line in wrap_plain_text(&full, 74) {
                        ensure_space(15.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                        cur_ops.push_str(&format!(
                            "BT /F1 11 Tf 0.18 0.20 0.22 rg {:.1} {cur_y:.1} Td ({}) Tj ET\n",
                            margin_x + 10.0,
                            pdf_win_ansi_escape(&line)
                        ));
                        cur_y -= 15.0;
                    }
                }
                cur_y -= 6.0;
            }
            PreviewBlock::Table { headers, rows } => {
                if !headers.is_empty() {
                    ensure_space(18.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                    let hdr_line = headers.join("  |  ");
                    cur_ops.push_str(&format!(
                        "BT /F2 10.5 Tf 0.15 0.18 0.20 rg {margin_x:.1} {cur_y:.1} Td ({}) Tj ET\n",
                        pdf_win_ansi_escape(&hdr_line)
                    ));
                    cur_y -= 16.0;
                }
                for row in rows {
                    ensure_space(16.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                    let row_line = row.join("  |  ");
                    cur_ops.push_str(&format!(
                        "BT /F1 10 Tf 0.20 0.22 0.25 rg {margin_x:.1} {cur_y:.1} Td ({}) Tj ET\n",
                        pdf_win_ansi_escape(&row_line)
                    ));
                    cur_y -= 15.0;
                }
                cur_y -= 8.0;
            }
            PreviewBlock::Image { alt, url } => {
                ensure_space(16.0, &mut cur_y, &mut cur_ops, &mut pages_ops);
                cur_ops.push_str(&format!(
                    "BT /F3 10.5 Tf 0.30 0.35 0.45 rg {margin_x:.1} {cur_y:.1} Td ({}) Tj ET\n",
                    pdf_win_ansi_escape(&format!("[Immagine: {alt} ({url})]"))
                ));
                cur_y -= 16.0;
            }
        }
    }

    if !cur_ops.is_empty() || pages_ops.is_empty() {
        pages_ops.push(cur_ops);
    }

    // Object numbering:
    // 1: Catalog
    // 2: Pages
    // 3: Font F1 (Helvetica)
    // 4: Font F2 (Helvetica-Bold)
    // 5: Font F3 (Helvetica-Oblique)
    // 6: Font F4 (Courier)
    // 7 .. 7 + N_img: Image XObjects
    // Following: Page + Content pairs
    let mut objects: Vec<Vec<u8>> = Vec::new();
    let img_start_obj = 7usize;
    let pages_start_obj = img_start_obj + embedded_images.len();

    // Obj 1: Catalog
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    // Obj 2: Pages (placeholder, filled below)
    let mut kids_refs = String::new();
    for i in 0..pages_ops.len() {
        let page_obj_id = pages_start_obj + i * 2;
        kids_refs.push_str(&format!("{page_obj_id} 0 R "));
    }
    objects.push(
        format!(
            "<< /Type /Pages /Kids [ {}] /Count {} >>",
            kids_refs,
            pages_ops.len()
        )
        .into_bytes(),
    );

    // Obj 3..6: Built-in Type1 Fonts
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique /Encoding /WinAnsiEncoding >>".to_vec());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>".to_vec());

    // Image XObjects
    let mut xobj_dict = String::new();
    for (idx, img) in embedded_images.iter().enumerate() {
        let obj_id = img_start_obj + idx;
        xobj_dict.push_str(&format!("/{} {} 0 R ", img.name, obj_id));
        let mut obj_bytes = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
            img.width,
            img.height,
            img.rgb_bytes.len()
        )
        .into_bytes();
        obj_bytes.extend_from_slice(&img.rgb_bytes);
        obj_bytes.extend_from_slice(b"\nendstream");
        objects.push(obj_bytes);
    }

    // Page & Content objects
    for (i, content_str) in pages_ops.iter().enumerate() {
        let content_obj_id = pages_start_obj + i * 2 + 1;
        let page_dict = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_w:.2} {page_h:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R >> /XObject << {xobj_dict}>> >> /Contents {content_obj_id} 0 R >>"
        );
        objects.push(page_dict.into_bytes());

        let content_bytes = content_str.as_bytes();
        let mut stream_obj =
            format!("<< /Length {} >>\nstream\n", content_bytes.len()).into_bytes();
        stream_obj.extend_from_slice(content_bytes);
        stream_obj.extend_from_slice(b"endstream");
        objects.push(stream_obj);
    }

    // Serialize full PDF binary with byte-accurate xref table
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
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
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
|:---|:---|:---|
| PDF | Pure Rust PDF 1.4 | OK |
| ODT | ODF 1.2 + zip | OK |

## 2. Diagramma Mermaid

```mermaid
flowchart LR
  A[Markdown AST] --> B{Esportatore}
  B -->|PDF| C[Documento PDF]
  B -->|ODT| D[Archivio ODF]
```
";

    #[test]
    fn test_render_markdown_to_html() {
        let html = render_markdown_to_html("# Title\n\nall you can ~~eat~~ drink\n");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<del>eat</del>"));
    }

    #[test]
    fn test_all_five_exporters_pdf_docx_odt_latex_epub() {
        let buf = DocumentBuffer::from_str_content(RICH_TEST_MARKDOWN);

        // 1. PDF (.pdf)
        let pdf_bytes = export_document_bytes(&buf, MarkdownDialect::Gfm, "Test Export", ExportFormat::Pdf)
            .expect("PDF export failed");
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
        assert!(
            pdf_bytes.len() > 2000,
            "Expected substantial PDF with embedded Mermaid XObject, got {} bytes",
            pdf_bytes.len()
        );
        assert!(pdf_bytes.ends_with(b"%%EOF\n"));

        // 2. Word (.docx)
        let docx_bytes = export_document_bytes(&buf, MarkdownDialect::Gfm, "Test Export", ExportFormat::Docx)
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

        // 3. OpenDocument Text (.odt)
        let odt_bytes = export_document_bytes(&buf, MarkdownDialect::Gfm, "Test Export", ExportFormat::Odt)
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
        assert!(odt_content.contains("Pictures/mermaid_"));

        // 4. LaTeX (.tex)
        let tex_bytes = export_document_bytes(&buf, MarkdownDialect::Gfm, "Test Export", ExportFormat::Latex)
            .expect("LaTeX export failed");
        let tex_str = String::from_utf8(tex_bytes).expect("valid UTF-8 .tex");
        assert!(tex_str.contains("\\documentclass[11pt,a4paper]{article}"));
        assert!(tex_str.contains("\\begin{document}"));
        assert!(tex_str.contains("\\section{Documento Completo di Test}"));
        assert!(tex_str.contains("\\subsection{1. Tabella e Lista}"));
        assert!(
            tex_str.contains(r"100\% \& \$50\_1"),
            "Expected strict LaTeX escaping of '100% & $50_1', got:\n{tex_str}"
        );

        // 5. EPUB (.epub)
        let epub_bytes = export_document_bytes(&buf, MarkdownDialect::Gfm, "Test Export", ExportFormat::Epub)
            .expect("EPUB export failed");
        let mut epub_zip = ZipArchive::new(Cursor::new(&epub_bytes)).expect("valid EPUB zip");
        {
            let first = epub_zip.by_index(0).expect("first entry in EPUB");
            assert_eq!(first.name(), "mimetype");
            assert_eq!(first.compression(), CompressionMethod::Stored);
        }
        let mut opf = String::new();
        epub_zip
            .by_name("OEBPS/content.opf")
            .expect("OEBPS/content.opf must exist in .epub")
            .read_to_string(&mut opf)
            .expect("read content.opf");
        assert!(opf.contains("Test Export"));
        let mut ch1 = String::new();
        epub_zip
            .by_name("OEBPS/chapter1.xhtml")
            .expect("OEBPS/chapter1.xhtml must exist")
            .read_to_string(&mut ch1)
            .expect("read chapter1.xhtml");
        assert!(ch1.contains("Documento Completo di Test"));
        assert!(ch1.contains("images/mermaid_"));
    }
}
