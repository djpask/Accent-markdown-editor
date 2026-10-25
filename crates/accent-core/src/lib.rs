use comrak::nodes::{AstNode, LineColumn, ListType, NodeValue, Sourcepos};
use comrak::{Arena, Options};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Default sample document replicating the exact structure and lines observed in Phase 0 Oracle baseline.
pub const DEFAULT_SAMPLE_MARKDOWN: &str = "\
#### Allegati
Aggiungere l'elenco dei nomi dei file eventualmente allegati
(campi  F_Nome_Allegato_Originale__c,
G_Nome_Allegato_Originale__c, H_Nome_Allegato_Originale__c,
I_Nome_Allegato_Originale__c,L_Nome_Allegato_Originale__c,
N_Nome_Allegato_Originale__c, O_Nome_Allegato_Originale__c)

### 4.4 Altre funzioni
Le altre funzioni verranno descritte in seguito ad una prima
realizzazione dell'applicazione.

# Other markdown examples

- [X] Elemento
- [ ] efeee
- [ ] 334


# [sito web repubblica](https://www.repubblica.it)

![Argon rocks!](Argon_rocks%21.png)

> Citazione, riga 2 rigaga 3
> riga lunghissimaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
> altra riga


<fdfdefae rfererea raerae rearaeraegg t  ts>


```
if
then
else
```


all you can ~~eat~~ drink

| Fascicolo | Triage | Comunicazione |
|:---|:---:|:---|
| DB001234 | Media | sì |
| DB001235 | Alta | no |
| DB001236 | Critica | sì |
| DB001237 | Bassa | no |

## 5. Architettura del Sistema e Flussi Operativi

L'editor **Accent** è progettato interamente in *Rust* puro sopra il motore grafico immediato `egui` / `eframe`, garantendo prestazioni native, assenza di dipendenze esterne a runtime e piena compatibilità multipiattaforma.

### 5.1 Gestione del Buffer e AST Multi-Dialetto
Il documento sorgente viene mantenuto in memoria tramite una struttura dati **Rope** (`ropey::Rope`) che consente inserimenti, cancellazioni e conversioni tra offset di byte, caratteri Unicode e righe in tempo logaritmico $O(\\log N)$.

1. **CommonMark Standard**: supporto rigoroso per la specifica base, incluse le regole sugli underscore intraparola (`F_Nome_Allegato_Originale__c`).
2. **GitHub Flavored Markdown (GFM)**: tabelle, elenchi di attività (`- [x]`), testo barrato (`~~barrato~~`), autolink e blocchi di avviso.
3. **GitLab Flavored Markdown (GLFM)**: blocchi matematici inline e display, note a piè di pagina e metadati frontmatter.

```rust
fn verify_document_integrity(buffer: &DocumentBuffer) -> bool {
    let stats = buffer.stats();
    stats.words > 0 && stats.lines > 0
}
```

### 5.2 Modalità di Scrittura e Visualizzazione
Durante la stesura di documenti lunghi è possibile scorrere verticalmente il testo con la rotellina del mouse, la barra di scorrimento laterale o i tasti `PagSu` / `PagGiù` e le frecce direzionali:

- **Solo editor (Markdown sorgente)**: mostra sempre tutti i delimitatori di formattazione (`**`, `*`, `~~`, `` ` ``).
- **Anteprima durante la digitazione**: nasconde i delimitatori di sintassi quando il cursore si trova fuori dallo span attivo e li rivela istantaneamente non appena il cursore entra nella parola.
- **Pannello di anteprima (A metà larghezza / A tutta pagina / A metà altezza / Finestra separata)**: affianca o sostituisce l'editor con il rendering visuale completo di tabelle, citazioni, codice ed elenchi interattivi.

> Nota operativa: tutte le modifiche effettuate nell'editor aggiornano in tempo reale sia le statistiche nella barra inferiore sia il pannello di anteprima sincronizzato.

#### 5.2.1 Requisiti di Persistenza Atomica
Ogni salvataggio su disco avviene tramite scrittura su file temporaneo, sincronizzazione fisica (`fsync`) e rinomina atomica per prevenire qualsiasi perdita di dati anche in caso di interruzione improvvisa.

### 5.3 Tabella di Riepilogo dei Moduli

| Modulo | Livello | Responsabilità Principale | Stato |
|:---|:---:|:---|:---:|
| `accent-core` | L1 | Buffer Rope, parser AST `comrak`, Live Spans | Attivo |
| `accent-storage` | L2 | Salvataggio atomico e journal di ripristino | Attivo |
| `accent-mermaid` | L3 | Rendering nativo diagrammi Mermaid | Pianificato |
| `accent-export` | L3 | Esportazione nativa HTML, PDF, DOCX, EPUB | Attivo |
| `accent-session` | L4 | Stato sessione, configurazione, localizzazione | Attivo |
| `accent-ui` | L5 | Interfaccia utente `egui`, temi, scorciatoie | Attivo |

### 5.4 Note Conclusive e Verifiche di Scorrimento
Questa sezione conclusiva permette di verificare comodamente lo scorrimento verticale fino in fondo al documento di esempio (`oracle_sample.md`), controllando che l'indicatore di posizione, i blocchi di codice e le intestazioni fuori margine mantengano un allineamento perfetto durante lo scroll continuo.
";

/// Interactive tutorial document opened via Main Menu -> "Apri tutorial" (`SPECS.md`).
pub const TUTORIAL_MARKDOWN: &str = "\
# Benvenuto in Accent — Guida Rapida

**Accent** è un editor e visualizzatore Markdown veloce, elegante e privo di distrazioni.

## 1. Modalità di Editing e Anteprima
Usa il pulsante **Anteprima** nella barra superiore per scegliere in modo chiaro e univoco tra:
- **Solo editor (Markdown sorgente)**: visualizza sempre i marcatori Markdown (`**grassetto**`, `*corsivo*`, `~~barrato~~`).
- **Anteprima durante la digitazione**: nasconde automaticamente i delimitatori `**` o `~~` finché il cursore non entra nella parola.
- **Pannello di anteprima**: apre l'anteprima formattata *A metà larghezza*, *A tutta pagina*, *A metà altezza* o *In una finestra separata*.

## 2. Apertura e Salvataggio Documenti
- Fai clic su **Apri ▾** in alto a sinistra per aprire rapidamente un documento recente oppure sfogliare il disco con **Apri documento...**.
- Fai clic su **Salva** (`Ctrl+S`) o **Salva come...** (`Ctrl+Shift+S`) per salvare il file in modo atomico e sicuro.

## 3. Esempi di Formattazione

- [x] Elenco di attività completato
- [ ] Secondo elemento da completare

> «La semplicità è l'ultima sofisticazione.»

```rust
fn main() {
    println!(\"Buona scrittura con Accent!\");
}
```

| Scorciatoia | Azione |
|:---|:---|
| `Ctrl+B` | **Grassetto** |
| `Ctrl+I` | *Corsivo* |
| `Ctrl+F` / `Ctrl+H` | Trova e sostituisci |
| `Ctrl+Shift+P` | Attiva/disattiva pannello di anteprima |
";

/// Supported Markdown dialects in Accent (`SPECS.md` Pillar 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MarkdownDialect {
    CommonMark,
    #[default]
    Gfm,
    Glfm,
}

impl MarkdownDialect {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "commonmark" | "cm" => Some(Self::CommonMark),
            "gfm" | "github" => Some(Self::Gfm),
            "glfm" | "gitlab" => Some(Self::Glfm),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::CommonMark => "CommonMark",
            Self::Gfm => "GitHub Flavored Markdown (GFM)",
            Self::Glfm => "GitLab Flavored Markdown (GLFM)",
        }
    }

    pub fn to_comrak_options(self) -> Options<'static> {
        let mut opts = Options::default();
        opts.parse.smart = false;
        match self {
            Self::CommonMark => {}
            Self::Gfm => {
                opts.extension.table = true;
                opts.extension.tasklist = true;
                opts.extension.strikethrough = true;
                opts.extension.autolink = true;
                opts.extension.alerts = true;
            }
            Self::Glfm => {
                opts.extension.table = true;
                opts.extension.tasklist = true;
                opts.extension.strikethrough = true;
                opts.extension.autolink = true;
                opts.extension.alerts = true;
                opts.extension.math_dollars = true;
                opts.extension.math_code = true;
                opts.extension.footnotes = true;
                opts.extension.front_matter_delimiter = Some("---".to_string());
                opts.extension.multiline_block_quotes = true;
            }
        }
        opts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentStats {
    pub characters: usize,
    pub words: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub reading_seconds: usize,
}

impl DocumentStats {
    pub fn compute(text: &str) -> Self {
        let characters = text.chars().count();
        let words = text.split_whitespace().count();
        let sentences = text
            .chars()
            .filter(|c| matches!(c, '.' | '!' | '?'))
            .count()
            .max(usize::from(words > 0));
        let paragraphs = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count();
        let reading_seconds = (words * 60) / 200;
        Self {
            characters,
            words,
            sentences,
            paragraphs,
            reading_seconds,
        }
    }

    pub fn format_reading_time(&self) -> String {
        let hours = self.reading_seconds / 3600;
        let minutes = (self.reading_seconds % 3600) / 60;
        let seconds = self.reading_seconds % 60;
        format!("{hours}:{minutes:02}:{seconds:02}")
    }
}

/// AST span kind extracted from `comrak` with exact `Sourcepos` byte and char ranges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstSpanKind {
    Heading { level: u8 },
    CodeBlock { fenced: bool, info: String },
    BlockQuote,
    ListItem { task: Option<bool> },
    Table,
    Alert { alert_type: String },
    FrontMatter,
    FootnoteDefinition { name: String },
    MathBlock,
    Strong,
    Emph,
    Strikethrough,
    CodeInline,
    Link {
        url: String,
        content_byte_range: Option<Range<usize>>,
    },
    Image {
        url: String,
        content_byte_range: Option<Range<usize>>,
    },
    MathInline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstSpan {
    pub kind: AstSpanKind,
    pub byte_range: Range<usize>,
    pub char_range: Range<usize>,
    /// 0-based start line index.
    pub start_line: usize,
    /// 0-based inclusive end line index.
    pub end_line: usize,
}

/// Visibility state of Markdown syntax delimiters in Caret-Aware *View-while-typing* mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelimiterVisibility {
    /// Caret is outside the span while *View-while-typing* is active: syntax delimiters are collapsed.
    Collapsed,
    /// Caret is inside/touching the span (or *View-while-typing* is off): raw syntax delimiters are shown.
    Expanded,
}

/// Caret-aware live span with explicit delimiter byte ranges and visibility state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveSpan {
    pub span: AstSpan,
    pub visibility: DelimiterVisibility,
    pub delimiter_byte_ranges: Vec<Range<usize>>,
}

/// Segment of the full document text produced for `egui::TextEdit`'s custom `layouter`.
/// Concatenating `text` across all `EditorTextSegment`s reproduces `DocumentBuffer::text()` 1:1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EditorTextSegment {
    pub text: String,
    pub byte_range: Range<usize>,
    /// True if this segment starts at column 0 of a line.
    pub is_line_start: bool,
    pub line_index: usize,
    pub heading_level: Option<u8>,
    pub is_heading_hashes: bool,
    pub blockquote: bool,
    pub code_block: bool,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code_inline: bool,
    pub muted: bool,
    pub link: bool,
    /// True when Caret-Aware *View-while-typing* collapses this syntax delimiter (`font_size = 0.01`, transparent).
    pub collapsed_delimiter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyledBlockKind {
    Heading { level: u8, hashes: String },
    BlockQuote,
    CodeFence,
    CodeBody,
    Paragraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StyledSegment {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub muted: bool,
    pub link: bool,
    pub collapsed_delimiter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyledLine {
    pub line_index: usize,
    pub byte_range: Range<usize>,
    pub block: StyledBlockKind,
    pub segments: Vec<StyledSegment>,
}

/// Structured inline item for rendered Split/Windowed Preview panes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PreviewInline {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub link_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewListItem {
    pub task: Option<bool>,
    pub inlines: Vec<PreviewInline>,
}

/// Structured block item for rendered Split/Windowed Preview panes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewBlock {
    Heading {
        level: u8,
        inlines: Vec<PreviewInline>,
        is_link_heading: bool,
    },
    Paragraph {
        inlines: Vec<PreviewInline>,
    },
    BlockQuote {
        inlines: Vec<PreviewInline>,
    },
    CodeBlock {
        info: String,
        code: String,
    },
    List {
        ordered: bool,
        items: Vec<PreviewListItem>,
    },
    Image {
        alt: String,
        url: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

/// Legacy compatibility enum used by unit tests; backed by AST classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineKind {
    Heading {
        level: u8,
        hashes: String,
        rest: String,
    },
    BlockQuote {
        line: String,
    },
    CodeFence {
        line: String,
    },
    CodeBody {
        line: String,
    },
    Paragraph {
        line: String,
    },
}

#[derive(Debug, Clone)]
pub struct DocumentBuffer {
    rope: Rope,
    cursor_char_offset: usize,
    selection_end_char_offset: usize,
}

impl Default for DocumentBuffer {
    fn default() -> Self {
        Self::from_str_content(DEFAULT_SAMPLE_MARKDOWN)
    }
}

impl DocumentBuffer {
    pub fn from_str_content(text: &str) -> Self {
        let rope = Rope::from_str(text);
        // Default initial caret at start of `all you can ~~eat~~ drink`
        let default_caret = text
            .find("all you can")
            .map(|b| rope.byte_to_char(b))
            .unwrap_or(0);
        Self {
            rope,
            cursor_char_offset: default_caret,
            selection_end_char_offset: default_caret,
        }
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        let total = self.rope.len_lines();
        if total > 1 && self.rope.line(total - 1).len_bytes() == 0 {
            total - 1
        } else {
            total
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    pub fn cursor(&self) -> usize {
        self.cursor_char_offset
    }

    pub fn selection_char_range(&self) -> Range<usize> {
        let a = self.cursor_char_offset.min(self.selection_end_char_offset);
        let b = self.cursor_char_offset.max(self.selection_end_char_offset);
        a..b
    }

    pub fn selection_byte_range(&self) -> Range<usize> {
        let cr = self.selection_char_range();
        self.char_to_byte(cr.start)..self.char_to_byte(cr.end)
    }

    pub fn cursor_byte(&self) -> usize {
        self.char_to_byte(self.cursor_char_offset)
    }

    pub fn cursor_line(&self) -> usize {
        let c = self.cursor_char_offset.min(self.rope.len_chars());
        self.rope.char_to_line(c)
    }

    pub fn set_cursor(&mut self, offset: usize) {
        let clamped = offset.min(self.rope.len_chars());
        self.cursor_char_offset = clamped;
        self.selection_end_char_offset = clamped;
    }

    pub fn set_selection(&mut self, primary_char: usize, secondary_char: usize) {
        self.cursor_char_offset = primary_char.min(self.rope.len_chars());
        self.selection_end_char_offset = secondary_char.min(self.rope.len_chars());
    }

    pub fn render_html(&self, dialect: MarkdownDialect) -> String {
        comrak::markdown_to_html(&self.text(), &dialect.to_comrak_options())
    }

    /// Converts a byte offset to a character offset in $O(\log N)$ via `ropey::Rope`.
    pub fn byte_to_char(&self, byte_idx: usize) -> usize {
        let clamped = byte_idx.min(self.rope.len_bytes());
        self.rope.byte_to_char(clamped)
    }

    /// Converts a character offset to a byte offset in $O(\log N)$ via `ropey::Rope`.
    pub fn char_to_byte(&self, char_idx: usize) -> usize {
        let clamped = char_idx.min(self.rope.len_chars());
        self.rope.char_to_byte(clamped)
    }

    /// Converts a byte offset to 0-based `(line_idx, byte_col)`.
    pub fn byte_to_line_col(&self, byte_idx: usize) -> (usize, usize) {
        let clamped = byte_idx.min(self.rope.len_bytes());
        let line_idx = self.rope.byte_to_line(clamped);
        let line_start = self.rope.line_to_byte(line_idx);
        (line_idx, clamped.saturating_sub(line_start))
    }

    /// Converts a 1-based `(line, column)` from `comrak::nodes::LineColumn` into a 0-based byte offset.
    pub fn line_col_1based_to_byte(&self, lc: LineColumn) -> usize {
        if lc.line == 0 || self.rope.len_bytes() == 0 {
            return 0;
        }
        let line_idx = (lc.line - 1).min(self.rope.len_lines().saturating_sub(1));
        let line_start = self.rope.line_to_byte(line_idx);
        let line_len = self.rope.line(line_idx).len_bytes();
        let col_offset = lc.column.saturating_sub(1).min(line_len);
        (line_start + col_offset).min(self.rope.len_bytes())
    }

    /// Converts a `comrak` `Sourcepos` (inclusive 1-based start and end `LineColumn`) to a half-open byte range.
    pub fn sourcepos_to_byte_range(&self, sp: Sourcepos) -> Range<usize> {
        if sp.start.line == 0 || sp.end.line == 0 {
            return 0..0;
        }
        let start = self.line_col_1based_to_byte(sp.start);
        let end_inclusive = self.line_col_1based_to_byte(sp.end);
        let end = (end_inclusive + 1).min(self.rope.len_bytes());
        if start <= end {
            start..end
        } else {
            start..start
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        let sel = self.selection_char_range();
        if sel.start < sel.end {
            self.rope.remove(sel.clone());
        }
        let idx = sel.start.min(self.rope.len_chars());
        self.rope.insert(idx, text);
        let new_cursor = idx + text.chars().count();
        self.cursor_char_offset = new_cursor;
        self.selection_end_char_offset = new_cursor;
    }

    /// Wraps the current selection with `prefix` and `suffix` (or inserts `prefix + placeholder + suffix` if empty).
    pub fn wrap_selection(&mut self, prefix: &str, suffix: &str, placeholder: &str) {
        let sel = self.selection_char_range();
        if sel.start < sel.end {
            let selected: String = self.rope.slice(sel.clone()).to_string();
            let replacement = format!("{prefix}{selected}{suffix}");
            self.rope.remove(sel.clone());
            self.rope.insert(sel.start, &replacement);
            let new_start = sel.start + prefix.chars().count();
            let new_end = new_start + selected.chars().count();
            self.cursor_char_offset = new_end;
            self.selection_end_char_offset = new_start;
        } else {
            let replacement = format!("{prefix}{placeholder}{suffix}");
            let idx = sel.start.min(self.rope.len_chars());
            self.rope.insert(idx, &replacement);
            let new_start = idx + prefix.chars().count();
            let new_end = new_start + placeholder.chars().count();
            self.cursor_char_offset = new_end;
            self.selection_end_char_offset = new_start;
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.rope = Rope::from_str(text);
        let len = self.rope.len_chars();
        self.cursor_char_offset = self.cursor_char_offset.min(len);
        self.selection_end_char_offset = self.selection_end_char_offset.min(len);
    }

    pub fn stats(&self) -> DocumentStats {
        DocumentStats::compute(&self.text())
    }

    /// Parses the document using `comrak` with the requested `MarkdownDialect` and returns all AST `Sourcepos` spans.
    pub fn extract_ast_spans(&self, dialect: MarkdownDialect) -> Vec<AstSpan> {
        let text = self.text();
        let arena = Arena::new();
        let opts = dialect.to_comrak_options();
        let root = comrak::parse_document(&arena, &text, &opts);

        let mut spans = Vec::new();
        self.collect_node_spans(root, &mut spans);
        spans
    }

    /// Computes Caret-Aware `LiveSpan`s given the active dialect, whether `view_while_typing` is enabled,
    /// and the current cursor/selection byte range (`caret_byte_range`).
    ///
    /// When `view_while_typing` is true and `caret_byte_range` is strictly outside a span (`caret.end < span.start || caret.start > span.end`),
    /// its syntax delimiters are marked `DelimiterVisibility::Collapsed`; when the caret enters or touches the span,
    /// they immediately become `DelimiterVisibility::Expanded`.
    pub fn compute_live_spans(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> Vec<LiveSpan> {
        let text = self.text();
        let spans = self.extract_ast_spans(dialect);
        let mut out = Vec::with_capacity(spans.len());

        for span in spans {
            let sr = &span.byte_range;
            let caret_touches =
                caret_byte_range.start <= sr.end && caret_byte_range.end >= sr.start;
            let visibility = if view_while_typing && !caret_touches {
                DelimiterVisibility::Collapsed
            } else {
                DelimiterVisibility::Expanded
            };

            let mut delimiter_byte_ranges = Vec::new();
            if sr.start < sr.end && sr.end <= text.len() {
                match &span.kind {
                    AstSpanKind::Strong | AstSpanKind::Strikethrough => {
                        if sr.len() >= 4 {
                            delimiter_byte_ranges.push(sr.start..(sr.start + 2));
                            delimiter_byte_ranges.push((sr.end - 2)..sr.end);
                        }
                    }
                    AstSpanKind::Emph => {
                        if sr.len() >= 2 {
                            delimiter_byte_ranges.push(sr.start..(sr.start + 1));
                            delimiter_byte_ranges.push((sr.end - 1)..sr.end);
                        }
                    }
                    AstSpanKind::CodeInline | AstSpanKind::MathInline => {
                        let slice = &text[sr.clone()];
                        let delim_ch = if matches!(span.kind, AstSpanKind::CodeInline) {
                            '`'
                        } else {
                            '$'
                        };
                        let open_len = slice.chars().take_while(|&c| c == delim_ch).count();
                        let close_len = slice.chars().rev().take_while(|&c| c == delim_ch).count();
                        if open_len > 0 && close_len > 0 && sr.len() > open_len + close_len {
                            delimiter_byte_ranges.push(sr.start..(sr.start + open_len));
                            delimiter_byte_ranges.push((sr.end - close_len)..sr.end);
                        }
                    }
                    AstSpanKind::Link {
                        content_byte_range: Some(cr),
                        ..
                    } => {
                        if cr.start > sr.start && cr.end <= sr.end {
                            delimiter_byte_ranges.push(sr.start..cr.start);
                            delimiter_byte_ranges.push(cr.end..sr.end);
                        }
                    }
                    _ => {}
                }
            }

            out.push(LiveSpan {
                span,
                visibility,
                delimiter_byte_ranges,
            });
        }

        out
    }

    fn collect_node_spans<'a>(&self, node: &'a AstNode<'a>, out: &mut Vec<AstSpan>) {
        let data = node.data.borrow();
        let sp = data.sourcepos;

        let maybe_kind = match &data.value {
            NodeValue::Heading(h) => Some(AstSpanKind::Heading { level: h.level }),
            NodeValue::CodeBlock(cb) => Some(AstSpanKind::CodeBlock {
                fenced: cb.fenced,
                info: cb.info.clone(),
            }),
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                Some(AstSpanKind::BlockQuote)
            }
            NodeValue::Item(_) => Some(AstSpanKind::ListItem { task: None }),
            NodeValue::TaskItem(symbol) => Some(AstSpanKind::ListItem {
                task: Some(symbol.is_some()),
            }),
            NodeValue::Table(_) => Some(AstSpanKind::Table),
            NodeValue::Alert(alert) => Some(AstSpanKind::Alert {
                alert_type: format!("{:?}", alert.alert_type),
            }),
            NodeValue::FrontMatter(_) => Some(AstSpanKind::FrontMatter),
            NodeValue::FootnoteDefinition(def) => Some(AstSpanKind::FootnoteDefinition {
                name: def.name.clone(),
            }),
            NodeValue::Strong => Some(AstSpanKind::Strong),
            NodeValue::Emph => Some(AstSpanKind::Emph),
            NodeValue::Strikethrough => Some(AstSpanKind::Strikethrough),
            NodeValue::Code(_) => Some(AstSpanKind::CodeInline),
            NodeValue::Math(m) => {
                if m.display_math {
                    Some(AstSpanKind::MathBlock)
                } else {
                    Some(AstSpanKind::MathInline)
                }
            }
            NodeValue::Link(link) => {
                let content_byte_range = self.children_byte_range(node);
                Some(AstSpanKind::Link {
                    url: link.url.clone(),
                    content_byte_range,
                })
            }
            NodeValue::Image(img) => {
                let content_byte_range = self.children_byte_range(node);
                Some(AstSpanKind::Image {
                    url: img.url.clone(),
                    content_byte_range,
                })
            }
            _ => None,
        };
        drop(data);

        if let Some(kind) = maybe_kind {
            let byte_range = self.sourcepos_to_byte_range(sp);
            if !byte_range.is_empty() {
                let char_range =
                    self.byte_to_char(byte_range.start)..self.byte_to_char(byte_range.end);
                let start_line = sp.start.line.saturating_sub(1);
                let end_line = sp.end.line.saturating_sub(1);
                out.push(AstSpan {
                    kind,
                    byte_range,
                    char_range,
                    start_line,
                    end_line,
                });
            }
        }

        for child in node.children() {
            self.collect_node_spans(child, out);
        }
    }

    fn children_byte_range<'a>(&self, node: &'a AstNode<'a>) -> Option<Range<usize>> {
        let mut first_start: Option<usize> = None;
        let mut last_end: Option<usize> = None;
        for child in node.children() {
            let sp = child.data.borrow().sourcepos;
            let r = self.sourcepos_to_byte_range(sp);
            if !r.is_empty() {
                first_start = Some(first_start.map_or(r.start, |s| s.min(r.start)));
                last_end = Some(last_end.map_or(r.end, |e| e.max(r.end)));
            }
        }
        match (first_start, last_end) {
            (Some(s), Some(e)) if s <= e => Some(s..e),
            _ => None,
        }
    }

    /// Builds a 1:1 full-document stream of `EditorTextSegment`s for `egui::TextEdit`'s custom `layouter`.
    /// Every character (including `\n`) is preserved in order so `TextEdit` cursor offsets never drift.
    pub fn build_editor_segments(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> Vec<EditorTextSegment> {
        let text = self.text();
        if text.is_empty() {
            return Vec::new();
        }
        let live_spans = self.compute_live_spans(dialect, view_while_typing, caret_byte_range);
        let total_lines = self.rope.len_lines();
        let mut out = Vec::new();

        for line_idx in 0..total_lines {
            let line_start = self.rope.line_to_byte(line_idx);
            let slice = self.rope.line(line_idx);
            let full_line_bytes = slice.len_bytes();
            if full_line_bytes == 0 {
                continue;
            }
            let full_line_end = line_start + full_line_bytes;

            // Separate content from trailing newline ('\n' or '\r\n')
            let mut content_len = full_line_bytes;
            while content_len > 0 {
                let b = slice.byte(content_len - 1);
                if b == b'\n' || b == b'\r' {
                    content_len -= 1;
                } else {
                    break;
                }
            }
            let content_end = line_start + content_len;
            let raw_line = &text[line_start..content_end];

            let heading_level = live_spans.iter().find_map(|ls| match &ls.span.kind {
                AstSpanKind::Heading { level }
                    if ls.span.start_line <= line_idx && line_idx <= ls.span.end_line =>
                {
                    Some(*level)
                }
                _ => None,
            });
            let in_code_block = live_spans.iter().any(|ls| {
                matches!(ls.span.kind, AstSpanKind::CodeBlock { .. })
                    && ls.span.start_line <= line_idx
                    && line_idx <= ls.span.end_line
            });
            let in_blockquote = live_spans.iter().any(|ls| {
                matches!(ls.span.kind, AstSpanKind::BlockQuote)
                    && ls.span.start_line <= line_idx
                    && line_idx <= ls.span.end_line
            });

            if content_len == 0 {
                // Empty line with only newline
                out.push(EditorTextSegment {
                    text: text[line_start..full_line_end].to_string(),
                    byte_range: line_start..full_line_end,
                    is_line_start: true,
                    line_index: line_idx,
                    code_block: in_code_block,
                    blockquote: in_blockquote,
                    ..Default::default()
                });
                continue;
            }

            if in_code_block {
                out.push(EditorTextSegment {
                    text: text[line_start..full_line_end].to_string(),
                    byte_range: line_start..full_line_end,
                    is_line_start: true,
                    line_index: line_idx,
                    code_block: true,
                    code_inline: true,
                    ..Default::default()
                });
                continue;
            }

            let mut content_start = line_start;
            let mut next_is_line_start = true;

            if let Some(level) = heading_level {
                let trimmed = raw_line.trim_start();
                let leading_ws = raw_line.len() - trimmed.len();
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                let after_hashes = &trimmed[hash_count..];
                let space_len = usize::from(after_hashes.starts_with(' '));
                let prefix_end = line_start + leading_ws + hash_count + space_len;

                if prefix_end > line_start {
                    out.push(EditorTextSegment {
                        text: text[line_start..prefix_end].to_string(),
                        byte_range: line_start..prefix_end,
                        is_line_start: true,
                        line_index: line_idx,
                        heading_level: Some(level),
                        is_heading_hashes: true,
                        bold: true,
                        ..Default::default()
                    });
                    content_start = prefix_end;
                    next_is_line_start = false;
                }
            }

            // Slice inline segments across content_start..content_end
            let inline_segs = self.slice_editor_inline_segments(
                &text,
                content_start..content_end,
                &live_spans,
                line_idx,
                heading_level,
                in_blockquote,
                next_is_line_start,
            );
            out.extend(inline_segs);

            // Append trailing newline bytes if present
            if content_end < full_line_end {
                out.push(EditorTextSegment {
                    text: text[content_end..full_line_end].to_string(),
                    byte_range: content_end..full_line_end,
                    is_line_start: false,
                    line_index: line_idx,
                    heading_level,
                    blockquote: in_blockquote,
                    ..Default::default()
                });
            }
        }

        out
    }

    #[allow(clippy::too_many_arguments)]
    fn slice_editor_inline_segments(
        &self,
        text: &str,
        range: Range<usize>,
        live_spans: &[LiveSpan],
        line_idx: usize,
        heading_level: Option<u8>,
        in_blockquote: bool,
        first_is_line_start: bool,
    ) -> Vec<EditorTextSegment> {
        if range.is_empty() {
            return Vec::new();
        }
        let base_bold = heading_level.is_some();

        let mut boundaries = vec![range.start, range.end];
        for ls in live_spans {
            let sr = &ls.span.byte_range;
            if sr.end <= range.start || sr.start >= range.end {
                continue;
            }
            for dr in &ls.delimiter_byte_ranges {
                if dr.start >= range.start && dr.start <= range.end {
                    boundaries.push(dr.start);
                }
                if dr.end >= range.start && dr.end <= range.end {
                    boundaries.push(dr.end);
                }
            }
            match &ls.span.kind {
                AstSpanKind::Strong
                | AstSpanKind::Emph
                | AstSpanKind::Strikethrough
                | AstSpanKind::CodeInline
                | AstSpanKind::MathInline => {
                    boundaries.push(sr.start.max(range.start));
                    boundaries.push(sr.end.min(range.end));
                }
                AstSpanKind::Link {
                    content_byte_range, ..
                } => {
                    boundaries.push(sr.start.max(range.start));
                    boundaries.push(sr.end.min(range.end));
                    if let Some(cr) = content_byte_range {
                        if cr.start >= range.start && cr.start <= range.end {
                            boundaries.push(cr.start);
                        }
                        if cr.end >= range.start && cr.end <= range.end {
                            boundaries.push(cr.end);
                        }
                    }
                }
                AstSpanKind::Image {
                    content_byte_range, ..
                } => {
                    boundaries.push(sr.start.max(range.start));
                    boundaries.push(sr.end.min(range.end));
                    if sr.start + 1 >= range.start && sr.start + 1 <= range.end {
                        boundaries.push(sr.start + 1);
                    }
                    if let Some(cr) = content_byte_range {
                        if cr.start >= range.start && cr.start <= range.end {
                            boundaries.push(cr.start);
                        }
                        if cr.end >= range.start && cr.end <= range.end {
                            boundaries.push(cr.end);
                        }
                    }
                }
                _ => {}
            }
        }

        boundaries.sort_unstable();
        boundaries.dedup();
        boundaries.retain(|&b| text.is_char_boundary(b));

        let mut segments = Vec::new();
        for (i, pair) in boundaries.windows(2).enumerate() {
            let (seg_start, seg_end) = (pair[0], pair[1]);
            if seg_start >= seg_end {
                continue;
            }
            let mut seg = EditorTextSegment {
                text: text[seg_start..seg_end].to_string(),
                byte_range: seg_start..seg_end,
                is_line_start: i == 0 && first_is_line_start,
                line_index: line_idx,
                heading_level,
                blockquote: in_blockquote,
                bold: base_bold,
                ..Default::default()
            };

            for ls in live_spans {
                let sr = &ls.span.byte_range;
                if seg_start < sr.start || seg_end > sr.end {
                    continue;
                }
                let is_delim = ls
                    .delimiter_byte_ranges
                    .iter()
                    .any(|dr| seg_start >= dr.start && seg_end <= dr.end);
                if is_delim {
                    seg.muted = true;
                    if ls.visibility == DelimiterVisibility::Collapsed {
                        seg.collapsed_delimiter = true;
                    }
                }

                match &ls.span.kind {
                    AstSpanKind::Strong => {
                        if !seg.collapsed_delimiter {
                            seg.bold = true;
                        }
                    }
                    AstSpanKind::Emph => {
                        if !seg.collapsed_delimiter {
                            seg.italic = true;
                        }
                    }
                    AstSpanKind::Strikethrough => {
                        if !seg.collapsed_delimiter {
                            seg.strikethrough = true;
                        }
                    }
                    AstSpanKind::CodeInline | AstSpanKind::MathInline => {
                        seg.code_inline = true;
                    }
                    AstSpanKind::Link {
                        content_byte_range, ..
                    } => {
                        if let Some(cr) = content_byte_range {
                            if seg_end <= cr.start {
                                seg.muted = true;
                                seg.bold = false;
                            } else if seg_start >= cr.end {
                                seg.muted = true;
                                seg.italic = true;
                                seg.bold = false;
                            } else {
                                seg.link = true;
                            }
                        } else {
                            seg.link = true;
                        }
                    }
                    AstSpanKind::Image {
                        content_byte_range, ..
                    } => {
                        if seg_start == sr.start && seg_end == sr.start + 1 {
                            // '!'
                        } else if let Some(cr) = content_byte_range {
                            if seg_start >= cr.start && seg_end <= cr.end {
                                // alt text
                            } else {
                                seg.muted = true;
                                seg.italic = true;
                            }
                        } else {
                            seg.muted = true;
                            seg.italic = true;
                        }
                    }
                    _ => {}
                }
            }

            segments.push(seg);
        }

        segments
    }

    /// Produces AST-styled lines for the editor strictly from `comrak` `Sourcepos` spans.
    pub fn styled_lines(&self, dialect: MarkdownDialect) -> Vec<StyledLine> {
        self.styled_lines_caret_aware(dialect, false, 0..0)
    }

    /// Produces AST-styled lines with optional Caret-Aware *View-while-typing* delimiter collapsing.
    pub fn styled_lines_caret_aware(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> Vec<StyledLine> {
        let text = self.text();
        let live_spans = self.compute_live_spans(dialect, view_while_typing, caret_byte_range);
        let num_lines = self.len_lines();
        let mut result = Vec::with_capacity(num_lines);

        for line_idx in 0..num_lines {
            let line_start = self.rope.line_to_byte(line_idx);
            let slice = self.rope.line(line_idx);
            let mut line_len = slice.len_bytes();
            while line_len > 0 {
                let b = slice.byte(line_len - 1);
                if b == b'\n' || b == b'\r' {
                    line_len -= 1;
                } else {
                    break;
                }
            }
            let line_end = line_start + line_len;
            let raw_line = &text[line_start..line_end];

            let heading_span = live_spans.iter().find_map(|ls| match &ls.span.kind {
                AstSpanKind::Heading { level }
                    if ls.span.start_line <= line_idx && line_idx <= ls.span.end_line =>
                {
                    Some(*level)
                }
                _ => None,
            });

            let code_block_span = live_spans.iter().find(|ls| {
                matches!(ls.span.kind, AstSpanKind::CodeBlock { .. })
                    && ls.span.start_line <= line_idx
                    && line_idx <= ls.span.end_line
            });

            let blockquote_span = live_spans.iter().find(|ls| {
                matches!(ls.span.kind, AstSpanKind::BlockQuote)
                    && ls.span.start_line <= line_idx
                    && line_idx <= ls.span.end_line
            });

            if let Some(cb) = code_block_span {
                let is_fenced = matches!(cb.span.kind, AstSpanKind::CodeBlock { fenced: true, .. });
                let trimmed = raw_line.trim_start();
                let is_fence_line = is_fenced
                    && (line_idx == cb.span.start_line || line_idx == cb.span.end_line)
                    && (trimmed.starts_with("```") || trimmed.starts_with("~~~"));
                let block = if is_fence_line {
                    StyledBlockKind::CodeFence
                } else {
                    StyledBlockKind::CodeBody
                };
                result.push(StyledLine {
                    line_index: line_idx,
                    byte_range: line_start..line_end,
                    block,
                    segments: vec![StyledSegment {
                        text: raw_line.to_string(),
                        code: true,
                        ..Default::default()
                    }],
                });
                continue;
            }

            if let Some(level) = heading_span {
                let trimmed = raw_line.trim_start();
                let leading_ws = raw_line.len() - trimmed.len();
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                let hashes = trimmed[..hash_count].to_string();
                let after_hashes = &trimmed[hash_count..];
                let space_len = usize::from(after_hashes.starts_with(' '));
                let content_start = line_start + leading_ws + hash_count + space_len;

                let segments =
                    self.slice_inline_segments(&text, content_start..line_end, &live_spans, true);
                result.push(StyledLine {
                    line_index: line_idx,
                    byte_range: line_start..line_end,
                    block: StyledBlockKind::Heading { level, hashes },
                    segments,
                });
                continue;
            }

            if blockquote_span.is_some() {
                let trimmed = raw_line.trim_start();
                let leading_ws = raw_line.len() - trimmed.len();
                let content_start = line_start + leading_ws;
                let segments =
                    self.slice_inline_segments(&text, content_start..line_end, &live_spans, false);
                result.push(StyledLine {
                    line_index: line_idx,
                    byte_range: line_start..line_end,
                    block: StyledBlockKind::BlockQuote,
                    segments,
                });
                continue;
            }

            let segments =
                self.slice_inline_segments(&text, line_start..line_end, &live_spans, false);
            result.push(StyledLine {
                line_index: line_idx,
                byte_range: line_start..line_end,
                block: StyledBlockKind::Paragraph,
                segments,
            });
        }

        result
    }

    fn slice_inline_segments(
        &self,
        text: &str,
        range: Range<usize>,
        live_spans: &[LiveSpan],
        base_bold: bool,
    ) -> Vec<StyledSegment> {
        if range.is_empty() {
            return Vec::new();
        }

        let mut boundaries = vec![range.start, range.end];
        for ls in live_spans {
            let sr = &ls.span.byte_range;
            if sr.end <= range.start || sr.start >= range.end {
                continue;
            }
            for dr in &ls.delimiter_byte_ranges {
                if dr.start >= range.start && dr.start <= range.end {
                    boundaries.push(dr.start);
                }
                if dr.end >= range.start && dr.end <= range.end {
                    boundaries.push(dr.end);
                }
            }
            match &ls.span.kind {
                AstSpanKind::Strong
                | AstSpanKind::Emph
                | AstSpanKind::Strikethrough
                | AstSpanKind::CodeInline
                | AstSpanKind::MathInline => {
                    boundaries.push(sr.start.max(range.start));
                    boundaries.push(sr.end.min(range.end));
                }
                AstSpanKind::Link {
                    content_byte_range, ..
                } => {
                    boundaries.push(sr.start.max(range.start));
                    boundaries.push(sr.end.min(range.end));
                    if let Some(cr) = content_byte_range {
                        if cr.start >= range.start && cr.start <= range.end {
                            boundaries.push(cr.start);
                        }
                        if cr.end >= range.start && cr.end <= range.end {
                            boundaries.push(cr.end);
                        }
                    }
                }
                AstSpanKind::Image {
                    content_byte_range, ..
                } => {
                    boundaries.push(sr.start.max(range.start));
                    boundaries.push(sr.end.min(range.end));
                    if sr.start + 1 >= range.start && sr.start + 1 <= range.end {
                        boundaries.push(sr.start + 1);
                    }
                    if let Some(cr) = content_byte_range {
                        if cr.start >= range.start && cr.start <= range.end {
                            boundaries.push(cr.start);
                        }
                        if cr.end >= range.start && cr.end <= range.end {
                            boundaries.push(cr.end);
                        }
                    }
                }
                _ => {}
            }
        }

        boundaries.sort_unstable();
        boundaries.dedup();
        boundaries.retain(|&b| text.is_char_boundary(b));

        let mut segments: Vec<StyledSegment> = Vec::new();
        for pair in boundaries.windows(2) {
            let (seg_start, seg_end) = (pair[0], pair[1]);
            if seg_start >= seg_end {
                continue;
            }
            let slice_str = &text[seg_start..seg_end];

            let mut seg = StyledSegment {
                text: slice_str.to_string(),
                bold: base_bold,
                ..Default::default()
            };

            for ls in live_spans {
                let sr = &ls.span.byte_range;
                if seg_start < sr.start || seg_end > sr.end {
                    continue;
                }
                let is_delim = ls
                    .delimiter_byte_ranges
                    .iter()
                    .any(|dr| seg_start >= dr.start && seg_end <= dr.end);
                if is_delim {
                    seg.muted = true;
                    if ls.visibility == DelimiterVisibility::Collapsed {
                        seg.collapsed_delimiter = true;
                    }
                }

                match &ls.span.kind {
                    AstSpanKind::Strong => seg.bold = true,
                    AstSpanKind::Emph => seg.italic = true,
                    AstSpanKind::Strikethrough => seg.strikethrough = true,
                    AstSpanKind::CodeInline | AstSpanKind::MathInline => seg.code = true,
                    AstSpanKind::Link {
                        content_byte_range, ..
                    } => {
                        if let Some(cr) = content_byte_range {
                            if seg_end <= cr.start {
                                seg.muted = true;
                                seg.bold = false;
                            } else if seg_start >= cr.end {
                                seg.muted = true;
                                seg.italic = true;
                                seg.bold = false;
                            } else {
                                seg.link = true;
                            }
                        } else {
                            seg.link = true;
                        }
                    }
                    AstSpanKind::Image {
                        content_byte_range, ..
                    } => {
                        if seg_start == sr.start && seg_end == sr.start + 1 {
                            // Leading '!'
                        } else if let Some(cr) = content_byte_range {
                            if seg_start >= cr.start && seg_end <= cr.end {
                                // Image alt text
                            } else {
                                seg.muted = true;
                                seg.italic = true;
                            }
                        } else {
                            seg.muted = true;
                            seg.italic = true;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(last) = segments.last_mut() {
                if last.bold == seg.bold
                    && last.italic == seg.italic
                    && last.strikethrough == seg.strikethrough
                    && last.code == seg.code
                    && last.muted == seg.muted
                    && last.link == seg.link
                    && last.collapsed_delimiter == seg.collapsed_delimiter
                {
                    last.text.push_str(&seg.text);
                    continue;
                }
            }
            segments.push(seg);
        }

        segments
    }

    /// Parses the document into structured `PreviewBlock`s for Split/Full/Windowed Preview rendering.
    pub fn parse_preview_blocks(&self, dialect: MarkdownDialect) -> Vec<PreviewBlock> {
        let text = self.text();
        let arena = Arena::new();
        let opts = dialect.to_comrak_options();
        let root = comrak::parse_document(&arena, &text, &opts);

        let mut blocks = Vec::new();
        for child in root.children() {
            self.collect_preview_block(child, &mut blocks);
        }
        blocks
    }

    fn collect_preview_block<'a>(&self, node: &'a AstNode<'a>, out: &mut Vec<PreviewBlock>) {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Heading(h) => {
                let mut inlines = Vec::new();
                let mut is_link_heading = false;
                for c in node.children() {
                    if matches!(c.data.borrow().value, NodeValue::Link(_)) {
                        is_link_heading = true;
                    }
                    Self::collect_preview_inlines(c, &mut inlines, false, false, false, None);
                }
                out.push(PreviewBlock::Heading {
                    level: h.level,
                    inlines,
                    is_link_heading,
                });
            }
            NodeValue::Paragraph => {
                // Check if this paragraph is a standalone image
                if node.children().count() == 1 {
                    if let Some(first) = node.first_child() {
                        if let NodeValue::Image(img) = &first.data.borrow().value {
                            let mut alt_inlines = Vec::new();
                            for c in first.children() {
                                Self::collect_preview_inlines(
                                    c,
                                    &mut alt_inlines,
                                    false,
                                    false,
                                    false,
                                    None,
                                );
                            }
                            let alt: String =
                                alt_inlines.iter().map(|i| i.text.as_str()).collect();
                            out.push(PreviewBlock::Image {
                                alt,
                                url: img.url.clone(),
                            });
                            return;
                        }
                    }
                }
                let mut inlines = Vec::new();
                for c in node.children() {
                    Self::collect_preview_inlines(c, &mut inlines, false, false, false, None);
                }
                if !inlines.is_empty() {
                    out.push(PreviewBlock::Paragraph { inlines });
                }
            }
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                let mut inlines = Vec::new();
                for c in node.children() {
                    Self::collect_preview_inlines(c, &mut inlines, false, false, false, None);
                }
                out.push(PreviewBlock::BlockQuote { inlines });
            }
            NodeValue::CodeBlock(cb) => {
                out.push(PreviewBlock::CodeBlock {
                    info: cb.info.clone(),
                    code: cb.literal.trim_end_matches('\n').to_string(),
                });
            }
            NodeValue::List(list) => {
                let ordered = list.list_type == ListType::Ordered;
                let mut items = Vec::new();
                for item_node in node.children() {
                    let task = match item_node.data.borrow().value {
                        NodeValue::TaskItem(sym) => Some(sym.is_some()),
                        _ => None,
                    };
                    let mut inlines = Vec::new();
                    for c in item_node.children() {
                        Self::collect_preview_inlines(c, &mut inlines, false, false, false, None);
                    }
                    items.push(PreviewListItem { task, inlines });
                }
                out.push(PreviewBlock::List { ordered, items });
            }
            NodeValue::Table(_) => {
                let mut headers = Vec::new();
                let mut rows = Vec::new();
                for row_node in node.children() {
                    let is_header = matches!(row_node.data.borrow().value, NodeValue::TableRow(true));
                    let mut cells = Vec::new();
                    for cell_node in row_node.children() {
                        let mut cell_inlines = Vec::new();
                        for c in cell_node.children() {
                            Self::collect_preview_inlines(
                                c,
                                &mut cell_inlines,
                                false,
                                false,
                                false,
                                None,
                            );
                        }
                        let cell_text: String =
                            cell_inlines.iter().map(|i| i.text.as_str()).collect();
                        cells.push(cell_text);
                    }
                    if is_header {
                        headers = cells;
                    } else {
                        rows.push(cells);
                    }
                }
                out.push(PreviewBlock::Table { headers, rows });
            }
            _ => {}
        }
    }

    fn collect_preview_inlines<'a>(
        node: &'a AstNode<'a>,
        out: &mut Vec<PreviewInline>,
        bold: bool,
        italic: bool,
        strikethrough: bool,
        link_url: Option<String>,
    ) {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Text(t) => {
                out.push(PreviewInline {
                    text: t.clone(),
                    bold,
                    italic,
                    strikethrough,
                    code: false,
                    link_url,
                });
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                out.push(PreviewInline {
                    text: " ".to_string(),
                    bold,
                    italic,
                    strikethrough,
                    code: false,
                    link_url,
                });
            }
            NodeValue::Code(c) => {
                out.push(PreviewInline {
                    text: c.literal.clone(),
                    bold,
                    italic,
                    strikethrough,
                    code: true,
                    link_url,
                });
            }
            NodeValue::HtmlInline(raw) => {
                out.push(PreviewInline {
                    text: raw.clone(),
                    bold,
                    italic,
                    strikethrough,
                    code: false,
                    link_url,
                });
            }
            NodeValue::Strong => {
                for c in node.children() {
                    Self::collect_preview_inlines(
                        c,
                        out,
                        true,
                        italic,
                        strikethrough,
                        link_url.clone(),
                    );
                }
            }
            NodeValue::Emph => {
                for c in node.children() {
                    Self::collect_preview_inlines(
                        c,
                        out,
                        bold,
                        true,
                        strikethrough,
                        link_url.clone(),
                    );
                }
            }
            NodeValue::Strikethrough => {
                for c in node.children() {
                    Self::collect_preview_inlines(c, out, bold, italic, true, link_url.clone());
                }
            }
            NodeValue::Link(l) => {
                for c in node.children() {
                    Self::collect_preview_inlines(
                        c,
                        out,
                        bold,
                        italic,
                        strikethrough,
                        Some(l.url.clone()),
                    );
                }
            }
            _ => {
                for c in node.children() {
                    Self::collect_preview_inlines(
                        c,
                        out,
                        bold,
                        italic,
                        strikethrough,
                        link_url.clone(),
                    );
                }
            }
        }
    }

    /// Classifies lines using the AST engine (`MarkdownDialect::Gfm` by default).
    pub fn classify_lines(&self) -> Vec<LineKind> {
        let styled = self.styled_lines(MarkdownDialect::Gfm);
        styled
            .into_iter()
            .map(|sl| {
                let joined: String = sl.segments.iter().map(|s| s.text.as_str()).collect();
                match sl.block {
                    StyledBlockKind::Heading { level, hashes } => LineKind::Heading {
                        level,
                        hashes,
                        rest: joined,
                    },
                    StyledBlockKind::BlockQuote => LineKind::BlockQuote { line: joined },
                    StyledBlockKind::CodeFence => LineKind::CodeFence { line: joined },
                    StyledBlockKind::CodeBody => LineKind::CodeBody { line: joined },
                    StyledBlockKind::Paragraph => LineKind::Paragraph { line: joined },
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_buffer_and_stats() {
        let mut buf = DocumentBuffer::default();
        let stats = buf.stats();
        assert!(stats.words > 40);
        assert!(stats.characters > 200);
        assert!(stats.paragraphs >= 8);

        buf.set_cursor(0);
        buf.insert_text("Test ");
        assert!(buf.text().starts_with("Test #### Allegati"));
    }

    #[test]
    fn test_rope_byte_char_line_conversions() {
        let buf = DocumentBuffer::from_str_content("prima riga\nseconda è qui\nterza");
        assert_eq!(buf.len_lines(), 3);
        let char_idx = buf.byte_to_char(11);
        assert_eq!(char_idx, 11);
        assert_eq!(buf.char_to_byte(char_idx), 11);
        assert_eq!(buf.byte_to_line_col(11), (1, 0));
    }

    #[test]
    fn test_intraword_underscores_never_trigger_bold_or_emphasis() {
        let sample = "\
F_1a_data_violazione_il__c
(campi  F_Nome_Allegato_Originale__c,
G_Nome_Allegato_Originale__c, H_Nome_Allegato_Originale__c,
I_Nome_Allegato_Originale__c,L_Nome_Allegato_Originale__c,
N_Nome_Allegato_Originale__c, O_Nome_Allegato_Originale__c)
";
        let buf = DocumentBuffer::from_str_content(sample);
        for dialect in [
            MarkdownDialect::CommonMark,
            MarkdownDialect::Gfm,
            MarkdownDialect::Glfm,
        ] {
            let spans = buf.extract_ast_spans(dialect);
            assert!(
                !spans
                    .iter()
                    .any(|s| matches!(s.kind, AstSpanKind::Strong | AstSpanKind::Emph)),
                "Intraword `__c` must NEVER produce Strong or Emph AST spans in {dialect:?}"
            );

            let styled = buf.styled_lines(dialect);
            for line in styled {
                for seg in line.segments {
                    assert!(
                        !seg.bold && !seg.italic,
                        "Segment {:?} was falsely styled bold/italic",
                        seg.text
                    );
                }
            }
        }
    }

    #[test]
    fn test_multi_dialect_ast_spans_commonmark_gfm_glfm() {
        let md = "# Heading\n\n**bold** and ~~strike~~\n\n$x + y = z$\n";
        let buf = DocumentBuffer::from_str_content(md);

        let cm_spans = buf.extract_ast_spans(MarkdownDialect::CommonMark);
        assert!(cm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::Heading { level: 1 })));
        assert!(cm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::Strong)));
        assert!(!cm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::Strikethrough)));
        assert!(!cm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::MathInline)));

        let gfm_spans = buf.extract_ast_spans(MarkdownDialect::Gfm);
        assert!(gfm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::Strikethrough)));
        assert!(!gfm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::MathInline)));

        let glfm_spans = buf.extract_ast_spans(MarkdownDialect::Glfm);
        assert!(glfm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::Strikethrough)));
        assert!(glfm_spans
            .iter()
            .any(|s| matches!(s.kind, AstSpanKind::MathInline)));
    }

    #[test]
    fn test_caret_aware_live_spans_collapsed_vs_expanded_and_zero_drift() {
        let md = "Inizio **parola** e ~~eat~~ fine\n";
        let buf = DocumentBuffer::from_str_content(md);

        // 1. Caret at byte 0 (outside both **parola** and ~~eat~~) with view_while_typing = true
        let collapsed_spans = buf.compute_live_spans(MarkdownDialect::Gfm, true, 0..0);
        let strong_span = collapsed_spans
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Strong))
            .expect("Strong span present");
        assert_eq!(strong_span.visibility, DelimiterVisibility::Collapsed);
        assert_eq!(strong_span.delimiter_byte_ranges.len(), 2);

        let segs_collapsed = buf.build_editor_segments(MarkdownDialect::Gfm, true, 0..0);
        // Verify 1:1 text preservation (zero cursor drift)
        let reconstructed: String = segs_collapsed.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(reconstructed, md);
        assert!(segs_collapsed
            .iter()
            .any(|s| s.text == "**" && s.collapsed_delimiter));

        // 2. Caret at byte 9 (inside `**parola**`, byte range 7..17) -> Strong becomes Expanded!
        let expanded_spans = buf.compute_live_spans(MarkdownDialect::Gfm, true, 9..9);
        let strong_expanded = expanded_spans
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Strong))
            .expect("Strong span present");
        assert_eq!(strong_expanded.visibility, DelimiterVisibility::Expanded);
        // While ~~eat~~ (bytes 20..27) remains Collapsed!
        let strike_still_collapsed = expanded_spans
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Strikethrough))
            .expect("Strikethrough span present");
        assert_eq!(
            strike_still_collapsed.visibility,
            DelimiterVisibility::Collapsed
        );
    }

    #[test]
    fn test_classify_lines_outdented_headings_and_code() {
        let buf = DocumentBuffer::default();
        let lines = buf.classify_lines();
        assert!(matches!(
            lines.first(),
            Some(LineKind::Heading { level: 4, .. })
        ));
        assert!(lines.iter().any(|l| matches!(l, LineKind::CodeBody { .. })));
        assert!(lines
            .iter()
            .any(|l| matches!(l, LineKind::BlockQuote { .. })));
    }
}
