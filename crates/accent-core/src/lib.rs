use comrak::nodes::{AstNode, LineColumn, ListType, NodeValue, Sourcepos};
use comrak::{Arena, Options};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

static EMPTY_RANGES_ARC: LazyLock<Arc<[Range<usize>]>> = LazyLock::new(|| Arc::from(&[][..]));

/// Canonical version of Accent synchronized across the Cargo workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default sample document showcasing Accent's multi-dialect Markdown capabilities.
pub const DEFAULT_SAMPLE_MARKDOWN: &str = "\
# Guida ed Esempi di Accent

Accent è un editor e visualizzatore Markdown multipiattaforma
progettato in **Rust** puro per offrire un ambiente di scrittura
rapido, pulito e privo di distrazioni.

### 1.1 Scrittura e Formattazione Inline
Durante la stesura puoi utilizzare **grassetto**, *corsivo*, testo
~~barrato~~, frammenti di `codice inline` e collegamenti ipertestuali
come [Documentazione Ufficiale Rust](https://doc.rust-lang.org).

Quando attivi la modalità **Anteprima durante la digitazione** (`Ctrl+L`),
tutti i marcatori sintattici vengono tradotti visivamente e riappaiono
solo quando posizioni il cursore sulla riga o sulla parola interessata.

## 2. Elenchi, Attività e Citazioni

- [x] Architettura stratificata a cinque livelli (`L1`–`L5`)
- [ ] Ripristino automatico da crash tramite journal atomico
- [ ] Esportazione diretta in HTML5, PDF, DOCX ed EPUB

- Primo elemento di elenco puntato standard
- Secondo elemento con *formattazione* integrata

> La chiarezza tipografica e l'assenza di distrazioni permettono di
> concentrarsi esclusivamente sul contenuto.
> Ogni modifica aggiorna in tempo reale l'albero sintattico e le
> statistiche del documento.

![Schema Architettura Accent](accent_architettura.png)

---

## 3. Esempio di Blocco di Codice

```rust
fn calcola_statistiche(buffer: &DocumentBuffer) -> DocumentStats {
    let stats = buffer.stats();
    assert!(stats.words > 0);
    stats
}
```

## 4. Tabella Riepilogativa dei Componenti

| Componente | Livello | Funzione Principale | Stato |
|:---|:---:|:---|:---:|
| `accent-core` | L1 | Buffer Rope, parser AST `comrak`, Live Spans | Attivo |
| `accent-storage` | L2 | Salvataggio atomico e journal di ripristino | Attivo |
| `accent-mermaid` | L3 | Rendering nativo diagrammi Mermaid | Pianificato |
| `accent-export` | L3 | Esportazione nativa HTML, PDF, DOCX, EPUB | Attivo |
| `accent-session` | L4 | Stato sessione, configurazione, localizzazione | Attivo |
| `accent-ui` | L5 | Interfaccia utente `egui`, temi, scorciatoie | Attivo |

## 5. Architettura del Sistema e Flussi Operativi

Il documento sorgente viene mantenuto in memoria tramite una struttura
dati **Rope** (`ropey::Rope`) che consente inserimenti, cancellazioni
e conversioni tra offset di byte, caratteri Unicode e righe in tempo
logaritmico $O(\\log N)$.

1. **CommonMark Standard**: supporto rigoroso per la specifica base, incluse le regole sugli underscore intraparola (`campo_identificativo_documento__c`).
2. **GitHub Flavored Markdown (GFM)**: tabelle, elenchi di attività (`- [x]`), testo barrato (`~~barrato~~`), autolink e blocchi di avviso.
3. **GitLab Flavored Markdown (GLFM)**: blocchi matematici inline e display, note a piè di pagina e metadati frontmatter.

#### 5.1 Persistenza Atomica e Scorrevolezza
Ogni salvataggio su disco avviene tramite scrittura su file temporaneo,
sincronizzazione fisica (`fsync`) e rinomina atomica per prevenire
qualsiasi perdita di dati anche in caso di interruzione improvvisa.
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
    /// Computes character, word, sentence, and paragraph statistics in a single zero-allocation
    /// $O(N)$ pass, handling both Unix (`\n\n`) and Windows (`\r\n\r\n`) paragraph breaks.
    pub fn compute(text: &str) -> Self {
        let mut characters = 0usize;
        let mut words = 0usize;
        let mut sentences = 0usize;
        let mut paragraphs = 0usize;
        let mut in_word = false;
        let mut in_para_with_non_ws = false;
        let mut consecutive_newlines = 0usize;

        for c in text.chars() {
            characters += 1;
            if c == '\r' {
                in_word = false;
                continue;
            }
            if c == '\n' {
                in_word = false;
                consecutive_newlines += 1;
                if consecutive_newlines >= 2 {
                    in_para_with_non_ws = false;
                }
                continue;
            }
            if c.is_whitespace() {
                in_word = false;
                consecutive_newlines = 0;
            } else {
                if !in_word {
                    words += 1;
                    in_word = true;
                }
                if !in_para_with_non_ws {
                    paragraphs += 1;
                    in_para_with_non_ws = true;
                }
                consecutive_newlines = 0;
                if matches!(c, '.' | '!' | '?') {
                    sentences += 1;
                }
            }
        }

        sentences = sentences.max(usize::from(words > 0));
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

/// Returns `true` if `ch` is a Unicode color emoji scalar rendered via color bitmap/vector slots.
#[inline]
pub fn is_color_emoji_char(ch: char) -> bool {
    let u = ch as u32;
    if u < 0x203C || (0x1F3FB..=0x1F3FF).contains(&u) {
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

/// Returns `true` if `ch` is an invisible emoji variation selector, ZWJ, or skin-tone combiner.
#[inline]
pub fn is_emoji_invisible_combiner(ch: char) -> bool {
    let u = ch as u32;
    u >= 0x200D && matches!(u, 0xFE0E | 0xFE0F | 0x200D | 0x20E3 | 0x1F3FB..=0x1F3FF)
}

/// AST span kind extracted from `comrak` with exact `Sourcepos` byte and char ranges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstSpanKind {
    Heading { level: u8 },
    CodeBlock { fenced: bool, info: String },
    BlockQuote,
    ListItem {
        ordered: bool,
        task: Option<bool>,
        content_char_start: usize,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    ThematicBreak,
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
    pub delimiter_byte_ranges: Arc<[Range<usize>]>,
}

/// Zero-copy segment of the full document text produced for `egui::TextEdit`'s custom `layouter`.
/// Concatenating `as_str(doc)` across all `EditorTextSegment`s reproduces `DocumentBuffer::as_str()` 1:1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EditorTextSegment {
    pub byte_range: Range<usize>,
    /// True if this segment starts at column 0 of a line.
    pub is_line_start: bool,
    pub line_index: usize,
    pub heading_level: Option<u8>,
    pub is_heading_hashes: bool,
    pub blockquote: bool,
    pub list_item: bool,
    pub list_marker: bool,
    pub task_checked: Option<bool>,
    pub code_block: bool,
    pub code_comment: bool,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code_inline: bool,
    pub muted: bool,
    pub link: bool,
    pub link_url_part: bool,
    pub table_pipe: bool,
    pub table_header_cell: bool,
    /// True when Caret-Aware *View-while-typing* collapses this syntax delimiter (`font_size = 0.01`, transparent).
    pub collapsed_delimiter: bool,
    /// True when this segment is a body line inside a collapsed `mermaid` code block in *View-while-typing* mode.
    pub mermaid_collapsed: bool,
}

impl EditorTextSegment {
    #[inline]
    pub fn as_str<'a>(&self, doc_text: &'a str) -> &'a str {
        &doc_text[self.byte_range.clone()]
    }
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
struct UndoCoalesceState {
    next_byte_offset: usize,
    last_instant: Instant,
    count: u8,
}

#[derive(Debug, Clone, Default)]
struct UndoHistory {
    undo_stack: VecDeque<(Rope, usize, usize)>,
    redo_stack: VecDeque<(Rope, usize, usize)>,
    coalesce: Option<UndoCoalesceState>,
}

#[derive(Debug, Clone)]
struct PreparedLiveBase {
    key: (u64, MarkdownDialect),
    ast_spans: Arc<Vec<AstSpan>>,
    delimiters: Vec<Arc<[Range<usize>]>>,
    spans_by_line: Vec<Vec<usize>>,
    byte_spans_by_line: Vec<Vec<usize>>,
    collapsed_live_spans: Arc<Vec<LiveSpan>>,
    all_expanded_live_spans: Option<Arc<Vec<LiveSpan>>>,
    collapsed_segments: Option<(Arc<Vec<EditorTextSegment>>, Vec<Range<usize>>)>,
    all_expanded_segments: Option<Arc<Vec<EditorTextSegment>>>,
    live_lru: VecDeque<(Vec<usize>, Arc<Vec<LiveSpan>>)>,
    seg_lru: VecDeque<(Vec<usize>, Arc<Vec<EditorTextSegment>>)>,
}

#[derive(Debug, Clone, Default)]
struct DocumentAstCache {
    ast_key: Option<(u64, MarkdownDialect)>,
    ast_spans: Arc<Vec<AstSpan>>,
    prepared_base: Option<PreparedLiveBase>,
    preview_key: Option<(u64, MarkdownDialect)>,
    preview_blocks: Arc<Vec<PreviewBlock>>,
    preview_ever_used_for: Option<MarkdownDialect>,
}

fn common_utf8_prefix_suffix(old: &str, new: &str) -> (usize, usize, usize) {
    let old_b = old.as_bytes();
    let new_b = new.as_bytes();
    let min_len = old_b.len().min(new_b.len());
    let mut prefix = old_b[..min_len]
        .iter()
        .zip(&new_b[..min_len])
        .position(|(a, b)| a != b)
        .unwrap_or(min_len);
    while prefix > 0 && !(new.is_char_boundary(prefix) && old.is_char_boundary(prefix)) {
        prefix -= 1;
    }

    let rem_old = old_b.len() - prefix;
    let rem_new = new_b.len() - prefix;
    let rem_min = rem_old.min(rem_new);
    let mut suffix = old_b[old_b.len() - rem_min..]
        .iter()
        .rev()
        .zip(new_b[new_b.len() - rem_min..].iter().rev())
        .position(|(a, b)| a != b)
        .unwrap_or(rem_min);
    while suffix > 0
        && !(new.is_char_boundary(new_b.len() - suffix)
            && old.is_char_boundary(old_b.len() - suffix))
    {
        suffix -= 1;
    }

    (prefix, old_b.len() - suffix, new_b.len() - suffix)
}

#[derive(Debug, Clone)]
pub struct DocumentBuffer {
    rope: Rope,
    revision: u64,
    cached_text: Arc<String>,
    cached_stats: Cell<Option<DocumentStats>>,
    has_emojis_cache: Cell<Option<bool>>,
    cursor_char_offset: usize,
    selection_end_char_offset: usize,
    history: Arc<UndoHistory>,
    cache: RefCell<DocumentAstCache>,
}

impl Default for DocumentBuffer {
    fn default() -> Self {
        Self::from_str_content(DEFAULT_SAMPLE_MARKDOWN)
    }
}

impl DocumentBuffer {
    pub fn from_str_content(text: &str) -> Self {
        let rope = Rope::from_str(text);
        let default_caret = text
            .find("Accent è un editor")
            .map(|b| rope.byte_to_char(b))
            .unwrap_or(0);
        let cached_text = Arc::new(text.to_owned());
        Self {
            rope,
            revision: 1,
            cached_text,
            cached_stats: Cell::new(None),
            has_emojis_cache: Cell::new(None),
            cursor_char_offset: default_caret,
            selection_end_char_offset: default_caret,
            history: Arc::new(UndoHistory::default()),
            cache: RefCell::new(DocumentAstCache::default()),
        }
    }

    /// Releases a shared `Arc<UndoHistory>` reference on an inactive/mirror `TabSlot` before mutating
    /// the active buffer so `Arc::make_mut` never needs to clone the `VecDeque` undo history.
    pub fn release_shared_history(&mut self) {
        if Arc::strong_count(&self.history) > 1 {
            self.history = Arc::new(UndoHistory::default());
        }
    }

    fn push_undo_snapshot_uncoalesced(&mut self) {
        let hist = Arc::make_mut(&mut self.history);
        hist.coalesce = None;
        if hist.undo_stack.len() >= 128 {
            hist.undo_stack.pop_front();
        }
        hist.undo_stack.push_back((
            self.rope.clone(),
            self.cursor_char_offset,
            self.selection_end_char_offset,
        ));
        hist.redo_stack.clear();
    }

    /// Reverts the last text mutation on this buffer, returning `true` if a state was restored.
    pub fn undo(&mut self) -> bool {
        let hist = Arc::make_mut(&mut self.history);
        hist.coalesce = None;
        let Some((prev_rope, prev_cur, prev_sel)) = hist.undo_stack.pop_back() else {
            return false;
        };
        hist.redo_stack.push_back((
            self.rope.clone(),
            self.cursor_char_offset,
            self.selection_end_char_offset,
        ));
        self.rope = prev_rope;
        let len = self.rope.len_chars();
        self.cursor_char_offset = prev_cur.min(len);
        self.selection_end_char_offset = prev_sel.min(len);
        self.sync_after_rope_mutation();
        true
    }

    /// Re-applies the last undone text mutation on this buffer, returning `true` if a state was restored.
    pub fn redo(&mut self) -> bool {
        let hist = Arc::make_mut(&mut self.history);
        hist.coalesce = None;
        let Some((next_rope, next_cur, next_sel)) = hist.redo_stack.pop_back() else {
            return false;
        };
        if hist.undo_stack.len() >= 128 {
            hist.undo_stack.pop_front();
        }
        hist.undo_stack.push_back((
            self.rope.clone(),
            self.cursor_char_offset,
            self.selection_end_char_offset,
        ));
        self.rope = next_rope;
        let len = self.rope.len_chars();
        self.cursor_char_offset = next_cur.min(len);
        self.selection_end_char_offset = next_sel.min(len);
        self.sync_after_rope_mutation();
        true
    }

    fn sync_after_rope_mutation(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.cached_text = Arc::new(self.rope.to_string());
        self.cached_stats.set(None);
        self.has_emojis_cache.set(None);
    }

    fn invalidate_after_in_place_mutation(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.cached_stats.set(None);
        self.has_emojis_cache.set(None);
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Zero-allocation slice of the current document text.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.cached_text.as_str()
    }

    /// Cheap `Arc<String>` handle to the current document text without copying bytes.
    #[inline]
    pub fn text_arc(&self) -> Arc<String> {
        Arc::clone(&self.cached_text)
    }

    pub fn text(&self) -> String {
        (*self.cached_text).clone()
    }

    /// Returns `true` if the current document contains any color emoji codepoints, cached per revision.
    pub fn has_color_emojis(&self) -> bool {
        if let Some(cached) = self.has_emojis_cache.get() {
            return cached;
        }
        let s = self.as_str();
        // Every codepoint >= U+2000 begins with a UTF-8 lead byte >= 0xE2.
        let found = s.as_bytes().iter().any(|&b| b >= 0xE2)
            && s.chars()
                .any(|c| is_color_emoji_char(c) || is_emoji_invisible_combiner(c));
        self.has_emojis_cache.set(Some(found));
        found
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

    pub fn raw_selection(&self) -> (usize, usize) {
        (self.cursor_char_offset, self.selection_end_char_offset)
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
        comrak::markdown_to_html(self.as_str(), &dialect.to_comrak_options())
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
        let mut end = (end_inclusive + 1).min(self.rope.len_bytes());
        while end < self.rope.len_bytes() && !self.as_str().is_char_boundary(end) {
            end += 1;
        }
        if start <= end {
            start..end
        } else {
            start..start
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        self.push_undo_snapshot_uncoalesced();
        let sel = self.selection_char_range();
        let sel_bytes = self.char_to_byte(sel.start)..self.char_to_byte(sel.end);
        if sel.start < sel.end {
            self.rope.remove(sel.clone());
        }
        let idx = sel.start.min(self.rope.len_chars());
        self.rope.insert(idx, text);
        let new_cursor = idx + text.chars().count();
        self.cursor_char_offset = new_cursor;
        self.selection_end_char_offset = new_cursor;
        if let Some(s) = Arc::get_mut(&mut self.cached_text) {
            s.replace_range(sel_bytes, text);
            self.invalidate_after_in_place_mutation();
        } else {
            self.sync_after_rope_mutation();
        }
    }

    /// Wraps the current selection with `prefix` and `suffix` (or inserts `prefix + placeholder + suffix` if empty).
    pub fn wrap_selection(&mut self, prefix: &str, suffix: &str, placeholder: &str) {
        self.push_undo_snapshot_uncoalesced();
        let sel = self.selection_char_range();
        let sel_bytes = self.char_to_byte(sel.start)..self.char_to_byte(sel.end);
        let replacement = if sel.start < sel.end {
            let selected = &self.as_str()[sel_bytes.clone()];
            let rep = format!("{prefix}{selected}{suffix}");
            let sel_char_len = sel.end - sel.start;
            self.rope.remove(sel.clone());
            self.rope.insert(sel.start, &rep);
            let new_start = sel.start + prefix.chars().count();
            let new_end = new_start + sel_char_len;
            self.cursor_char_offset = new_end;
            self.selection_end_char_offset = new_start;
            rep
        } else {
            let rep = format!("{prefix}{placeholder}{suffix}");
            let idx = sel.start.min(self.rope.len_chars());
            self.rope.insert(idx, &rep);
            let new_start = idx + prefix.chars().count();
            let new_end = new_start + placeholder.chars().count();
            self.cursor_char_offset = new_end;
            self.selection_end_char_offset = new_start;
            rep
        };
        if let Some(s) = Arc::get_mut(&mut self.cached_text) {
            s.replace_range(sel_bytes, &replacement);
            self.invalidate_after_in_place_mutation();
        } else {
            self.sync_after_rope_mutation();
        }
    }

    pub fn set_text(&mut self, text: &str) {
        if self.as_str() == text {
            return;
        }
        self.apply_new_text_arc(Arc::new(text.to_owned()));
    }

    /// Updates the buffer from an already-owned `String` without copying its heap buffer,
    /// applying a minimal UTF-8 diff to `self.rope` so all `Rope` undo snapshots share CoW `Arc` chunks.
    pub fn set_text_owned(&mut self, text: String) {
        if self.as_str() == text.as_str() {
            return;
        }
        self.apply_new_text_arc(Arc::new(text));
    }

    fn apply_new_text_arc(&mut self, new_arc: Arc<String>) {
        let (prefix_b, old_end_b, new_end_b) =
            common_utf8_prefix_suffix(self.as_str(), new_arc.as_str());
        let ins_str = &new_arc[prefix_b..new_end_b];
        let rem_bytes = old_end_b - prefix_b;

        let now = Instant::now();
        let mut single_non_ws_char = false;
        if rem_bytes == 0 && !ins_str.is_empty() && ins_str.len() <= 4 {
            let mut chars = ins_str.chars();
            if let (Some(ch), None) = (chars.next(), chars.next()) {
                single_non_ws_char = !ch.is_whitespace();
            }
        }

        let start_char = self.rope.byte_to_char(prefix_b);
        let old_end_char = if rem_bytes == 0 {
            start_char
        } else {
            self.rope.byte_to_char(old_end_b)
        };

        {
            let hist = Arc::make_mut(&mut self.history);
            let should_coalesce = single_non_ws_char
                && !hist.undo_stack.is_empty()
                && hist.coalesce.as_ref().is_some_and(|st| {
                    st.next_byte_offset == prefix_b
                        && st.count < 24
                        && now.duration_since(st.last_instant) <= Duration::from_millis(350)
                });

            if should_coalesce {
                if let Some(st) = hist.coalesce.as_mut() {
                    st.next_byte_offset = prefix_b + ins_str.len();
                    st.last_instant = now;
                    st.count = st.count.saturating_add(1);
                }
                hist.redo_stack.clear();
            } else {
                if hist.undo_stack.len() >= 128 {
                    hist.undo_stack.pop_front();
                }
                hist.undo_stack.push_back((
                    self.rope.clone(),
                    self.cursor_char_offset,
                    self.selection_end_char_offset,
                ));
                hist.redo_stack.clear();
                hist.coalesce = if single_non_ws_char {
                    Some(UndoCoalesceState {
                        next_byte_offset: prefix_b + ins_str.len(),
                        last_instant: now,
                        count: 1,
                    })
                } else {
                    None
                };
            }
        }

        if start_char < old_end_char {
            self.rope.remove(start_char..old_end_char);
        }
        if !ins_str.is_empty() {
            self.rope.insert(start_char, ins_str);
        }

        self.cached_text = new_arc;
        let len = self.rope.len_chars();
        self.cursor_char_offset = self.cursor_char_offset.min(len);
        self.selection_end_char_offset = self.selection_end_char_offset.min(len);
        self.invalidate_after_in_place_mutation();
    }

    /// Returns cached `DocumentStats`, computing lazily at most once per revision.
    pub fn stats(&self) -> DocumentStats {
        if let Some(s) = self.cached_stats.get() {
            return s;
        }
        let computed = DocumentStats::compute(self.as_str());
        self.cached_stats.set(Some(computed));
        computed
    }

    /// Returns the half-open byte range of `line_idx` excluding any trailing `\r` or `\n`.
    pub fn line_content_byte_range(&self, line_idx: usize) -> Range<usize> {
        if line_idx >= self.rope.len_lines() {
            return 0..0;
        }
        let line_start = self.rope.line_to_byte(line_idx);
        let slice = self.rope.line(line_idx);
        let mut content_len = slice.len_bytes();
        while content_len > 0 {
            let b = slice.byte(content_len - 1);
            if b == b'\n' || b == b'\r' {
                content_len -= 1;
            } else {
                break;
            }
        }
        line_start..(line_start + content_len)
    }

    /// Returns cached `Arc<Vec<AstSpan>>` for `(self.revision, dialect)`, parsing via `comrak` only on revision/dialect change.
    pub fn extract_ast_spans_arc(&self, dialect: MarkdownDialect) -> Arc<Vec<AstSpan>> {
        let key = (self.revision, dialect);
        let also_build_preview = {
            let c = self.cache.borrow();
            if c.ast_key == Some(key) {
                return Arc::clone(&c.ast_spans);
            }
            c.preview_ever_used_for == Some(dialect) && c.preview_key != Some(key)
        };

        let text = self.as_str();
        let arena = Arena::new();
        let opts = dialect.to_comrak_options();
        let root = comrak::parse_document(&arena, text, &opts);

        let mut spans = Vec::new();
        self.collect_node_spans(root, &mut spans);
        let arc = Arc::new(spans);

        let maybe_preview = if also_build_preview {
            let mut blocks = Vec::new();
            for child in root.children() {
                self.collect_preview_block(child, &mut blocks);
            }
            Some(Arc::new(blocks))
        } else {
            None
        };

        let mut c = self.cache.borrow_mut();
        c.ast_key = Some(key);
        c.ast_spans = Arc::clone(&arc);
        if let Some(p_arc) = maybe_preview {
            c.preview_key = Some(key);
            c.preview_blocks = p_arc;
        }
        arc
    }

    /// Parses the document using `comrak` with the requested `MarkdownDialect` and returns all AST `Sourcepos` spans.
    pub fn extract_ast_spans(&self, dialect: MarkdownDialect) -> Vec<AstSpan> {
        (*self.extract_ast_spans_arc(dialect)).clone()
    }

    fn compute_span_delimiter_ranges(&self, span: &AstSpan, text: &str) -> Arc<[Range<usize>]> {
        let sr = &span.byte_range;
        if sr.start >= sr.end || sr.end > text.len() {
            return Arc::clone(&EMPTY_RANGES_ARC);
        }
        let mut delimiter_byte_ranges = Vec::new();
        match &span.kind {
            AstSpanKind::Heading { .. } => {
                let lr = self.line_content_byte_range(span.start_line);
                if !lr.is_empty() && lr.end <= text.len() {
                    let raw_line = &text[lr.clone()];
                    let trimmed = raw_line.trim_start_matches([' ', '\t']);
                    let leading_ws = raw_line.len() - trimmed.len();
                    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                    let after_hashes = &trimmed[hash_count..];
                    let space_len = usize::from(after_hashes.starts_with(' '));
                    let prefix_end = lr.start + leading_ws + hash_count + space_len;
                    if hash_count > 0 && prefix_end > lr.start && prefix_end <= lr.end {
                        delimiter_byte_ranges.push(lr.start..prefix_end);
                    }
                }
                if span.end_line > span.start_line {
                    let end_lr = self.line_content_byte_range(span.end_line);
                    if !end_lr.is_empty() && end_lr.end <= text.len() {
                        let end_trimmed = text[end_lr.clone()].trim();
                        if !end_trimmed.is_empty()
                            && (end_trimmed.chars().all(|c| c == '-')
                                || end_trimmed.chars().all(|c| c == '='))
                        {
                            delimiter_byte_ranges.push(end_lr);
                        }
                    }
                }
            }
            AstSpanKind::CodeBlock { fenced: true, .. } => {
                let start_lr = self.line_content_byte_range(span.start_line);
                if !start_lr.is_empty() && start_lr.end <= text.len() {
                    let raw_l = text[start_lr.clone()].trim_start();
                    if raw_l.starts_with("```") || raw_l.starts_with("~~~") {
                        delimiter_byte_ranges.push(start_lr);
                    }
                }
                if span.end_line > span.start_line {
                    let end_lr = self.line_content_byte_range(span.end_line);
                    if !end_lr.is_empty() && end_lr.end <= text.len() {
                        let raw_l = text[end_lr.clone()].trim_start();
                        if raw_l.starts_with("```") || raw_l.starts_with("~~~") {
                            delimiter_byte_ranges.push(end_lr);
                        }
                    }
                }
            }
            AstSpanKind::BlockQuote | AstSpanKind::Alert { .. } => {
                for l_idx in span.start_line..=span.end_line {
                    let lr = self.line_content_byte_range(l_idx);
                    if !lr.is_empty() && lr.end <= text.len() {
                        let raw_l = &text[lr.clone()];
                        let trimmed = raw_l.trim_start_matches([' ', '\t']);
                        if let Some(after_gt) = trimmed.strip_prefix('>') {
                            let leading_ws = raw_l.len() - trimmed.len();
                            let space_len = usize::from(after_gt.starts_with(' '));
                            let mut prefix_end = lr.start + leading_ws + 1 + space_len;
                            if matches!(span.kind, AstSpanKind::Alert { .. })
                                && l_idx == span.start_line
                            {
                                let rest = &text[prefix_end..lr.end];
                                if rest.starts_with("[!") {
                                    if let Some(close_br) = rest.find(']') {
                                        prefix_end += close_br + 1;
                                    }
                                }
                            }
                            delimiter_byte_ranges.push(lr.start..prefix_end);
                        }
                    }
                }
            }
            AstSpanKind::ListItem { ordered, task, .. } => {
                if !*ordered {
                    let lr = self.line_content_byte_range(span.start_line);
                    if !lr.is_empty() && lr.end <= text.len() {
                        let raw_l = &text[lr.clone()];
                        let trimmed = raw_l.trim_start_matches([' ', '\t']);
                        let leading_ws = raw_l.len() - trimmed.len();
                        let is_task_prefix = task.is_some()
                            && trimmed.len() >= 5
                            && trimmed.is_char_boundary(5)
                            && matches!(
                                &trimmed[..5],
                                "- [ ]"
                                    | "- [x]"
                                    | "- [X]"
                                    | "* [ ]"
                                    | "* [x]"
                                    | "* [X]"
                                    | "+ [ ]"
                                    | "+ [x]"
                                    | "+ [X]"
                            );
                        if is_task_prefix {
                            let after_box = &trimmed[5..];
                            let space_len = usize::from(after_box.starts_with(' '));
                            let prefix_end = lr.start + leading_ws + 5 + space_len;
                            delimiter_byte_ranges.push(lr.start..prefix_end);
                        } else if trimmed.starts_with("- ")
                            || trimmed.starts_with("* ")
                            || trimmed.starts_with("+ ")
                        {
                            let prefix_end = lr.start + leading_ws + 2;
                            delimiter_byte_ranges.push(lr.start..prefix_end);
                        }
                    }
                }
            }
            AstSpanKind::Table { .. } => {
                for l_idx in span.start_line..=span.end_line {
                    let lr = self.line_content_byte_range(l_idx);
                    if !lr.is_empty() && lr.end <= text.len() {
                        delimiter_byte_ranges.push(lr);
                    }
                }
            }
            AstSpanKind::ThematicBreak => {
                let lr = self.line_content_byte_range(span.start_line);
                if !lr.is_empty() && lr.end <= text.len() {
                    delimiter_byte_ranges.push(lr);
                }
            }
            AstSpanKind::MathBlock => {
                let slice = &text[sr.clone()];
                if slice.starts_with("$$") && slice.ends_with("$$") && sr.len() >= 4 {
                    delimiter_byte_ranges.push(sr.start..(sr.start + 2));
                    delimiter_byte_ranges.push((sr.end - 2)..sr.end);
                }
            }
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
            }
            | AstSpanKind::Image {
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

        if delimiter_byte_ranges.is_empty() {
            Arc::clone(&EMPTY_RANGES_ARC)
        } else {
            Arc::from(delimiter_byte_ranges.into_boxed_slice())
        }
    }

    fn ensure_prepared_base(&self, dialect: MarkdownDialect) {
        let key = (self.revision, dialect);
        if self
            .cache
            .borrow()
            .prepared_base
            .as_ref()
            .is_some_and(|b| b.key == key)
        {
            return;
        }
        let spans = self.extract_ast_spans_arc(dialect);
        let text = self.as_str();
        let total_lines = self.rope.len_lines().max(1);
        let max_line_idx = total_lines.saturating_sub(1);
        let max_byte = self.rope.len_bytes();

        let mut delimiters = Vec::with_capacity(spans.len());
        let mut spans_by_line: Vec<Vec<usize>> = vec![Vec::new(); total_lines];
        let mut byte_spans_by_line: Vec<Vec<usize>> = vec![Vec::new(); total_lines];
        let mut collapsed_live = Vec::with_capacity(spans.len());

        for (s_idx, span) in spans.iter().enumerate() {
            let delims = self.compute_span_delimiter_ranges(span, text);
            let sl = span.start_line.min(max_line_idx);
            let el = span.end_line.min(max_line_idx);
            for line_vec in &mut spans_by_line[sl..=el] {
                line_vec.push(s_idx);
            }

            let b_sl = self
                .rope
                .byte_to_line(span.byte_range.start.min(max_byte))
                .min(max_line_idx);
            let b_el = self
                .rope
                .byte_to_line(span.byte_range.end.min(max_byte))
                .min(max_line_idx);
            for line_vec in &mut byte_spans_by_line[b_sl..=b_el] {
                line_vec.push(s_idx);
            }

            collapsed_live.push(LiveSpan {
                span: span.clone(),
                visibility: DelimiterVisibility::Collapsed,
                delimiter_byte_ranges: Arc::clone(&delims),
            });
            delimiters.push(delims);
        }

        let mut c = self.cache.borrow_mut();
        c.prepared_base = Some(PreparedLiveBase {
            key,
            ast_spans: spans,
            delimiters,
            spans_by_line,
            byte_spans_by_line,
            collapsed_live_spans: Arc::new(collapsed_live),
            all_expanded_live_spans: None,
            collapsed_segments: None,
            all_expanded_segments: None,
            live_lru: VecDeque::with_capacity(8),
            seg_lru: VecDeque::with_capacity(8),
        });
    }

    fn expanded_span_indices_for_caret(&self, base: &PreparedLiveBase, caret: &Range<usize>) -> Vec<usize> {
        if self.rope.len_bytes() == 0 || base.ast_spans.is_empty() {
            return Vec::new();
        }
        let max_byte = self.rope.len_bytes();
        let max_line = base.byte_spans_by_line.len().saturating_sub(1);
        let c_start_line = self
            .rope
            .byte_to_line(caret.start.min(max_byte))
            .min(max_line);
        let c_end_line = self
            .rope
            .byte_to_line(caret.end.min(max_byte))
            .min(max_line);

        let mut expanded = Vec::new();
        if c_start_line == c_end_line {
            for &s_idx in &base.byte_spans_by_line[c_start_line] {
                let sr = &base.ast_spans[s_idx].byte_range;
                if caret.start <= sr.end && caret.end >= sr.start {
                    expanded.push(s_idx);
                }
            }
        } else {
            for line_candidates in &base.byte_spans_by_line[c_start_line..=c_end_line] {
                for &s_idx in line_candidates {
                    let sr = &base.ast_spans[s_idx].byte_range;
                    if caret.start <= sr.end && caret.end >= sr.start {
                        expanded.push(s_idx);
                    }
                }
            }
            expanded.sort_unstable();
            expanded.dedup();
        }
        expanded
    }

    /// Computes a compact 64-bit signature of the exact set of AST spans currently expanded by `caret_byte_range`.
    /// When the cursor moves inside plain text (or stays within the same span), this signature remains identical,
    /// allowing `accent-ui` to reuse its cached `Galley` without re-shaping typography via `ab_glyph`.
    pub fn live_visibility_signature(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> u64 {
        if !view_while_typing {
            return u64::MAX;
        }
        self.ensure_prepared_base(dialect);
        let c = self.cache.borrow();
        let Some(base) = c.prepared_base.as_ref() else {
            return 0;
        };
        let expanded = self.expanded_span_indices_for_caret(base, &caret_byte_range);
        if expanded.is_empty() {
            return 0;
        }
        let mut h: u64 = 0xcbf29ce484222325;
        for idx in expanded {
            h ^= (idx as u64).wrapping_add(1);
            h = h.wrapping_mul(0x100000001b3);
        }
        if h == 0 || h == u64::MAX {
            h ^ 0x5555_5555_5555_5555
        } else {
            h
        }
    }

    /// Returns cached `Arc<Vec<LiveSpan>>` for `(self.revision, dialect, view_while_typing, expanded_spans)`.
    pub fn compute_live_spans_arc(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> Arc<Vec<LiveSpan>> {
        self.ensure_prepared_base(dialect);

        if !view_while_typing {
            {
                let c = self.cache.borrow();
                if let Some(arc) = c
                    .prepared_base
                    .as_ref()
                    .and_then(|b| b.all_expanded_live_spans.as_ref())
                {
                    return Arc::clone(arc);
                }
            }
            let mut c = self.cache.borrow_mut();
            let base = c.prepared_base.as_mut().unwrap();
            let mut v = (*base.collapsed_live_spans).clone();
            for ls in &mut v {
                ls.visibility = DelimiterVisibility::Expanded;
            }
            let arc = Arc::new(v);
            base.all_expanded_live_spans = Some(Arc::clone(&arc));
            return arc;
        }

        let expanded = {
            let c = self.cache.borrow();
            let base = c.prepared_base.as_ref().unwrap();
            let exp = self.expanded_span_indices_for_caret(base, &caret_byte_range);
            if exp.is_empty() {
                return Arc::clone(&base.collapsed_live_spans);
            }
            if let Some((_, cached_arc)) = base.live_lru.iter().find(|(k, _)| *k == exp) {
                return Arc::clone(cached_arc);
            }
            exp
        };

        let mut c = self.cache.borrow_mut();
        let base = c.prepared_base.as_mut().unwrap();
        let mut v = (*base.collapsed_live_spans).clone();
        for &idx in &expanded {
            if let Some(ls) = v.get_mut(idx) {
                ls.visibility = DelimiterVisibility::Expanded;
            }
        }
        let arc = Arc::new(v);
        if base.live_lru.len() >= 8 {
            base.live_lru.pop_front();
        }
        base.live_lru.push_back((expanded, Arc::clone(&arc)));
        arc
    }

    /// Computes Caret-Aware `LiveSpan`s given the active dialect, whether `view_while_typing` is enabled,
    /// and the current cursor/selection byte range (`caret_byte_range`).
    pub fn compute_live_spans(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> Vec<LiveSpan> {
        (*self.compute_live_spans_arc(dialect, view_while_typing, caret_byte_range)).clone()
    }

    fn collect_node_spans<'a>(&self, node: &'a AstNode<'a>, out: &mut Vec<AstSpan>) {
        let text = self.as_str();
        let data = node.data.borrow();
        let sp = data.sourcepos;
        let mut start_line = sp.start.line.saturating_sub(1);
        let mut end_line = sp.end.line.saturating_sub(1);

        let maybe_kind = match &data.value {
            NodeValue::Heading(h) => Some(AstSpanKind::Heading { level: h.level }),
            NodeValue::CodeBlock(cb) => Some(AstSpanKind::CodeBlock {
                fenced: cb.fenced,
                info: cb.info.clone(),
            }),
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                // Clamp end_line to lines that actually start with `>` (excluding lazy continuation lines)
                let mut valid_end = start_line;
                for l_idx in start_line..=end_line {
                    let lr = self.line_content_byte_range(l_idx);
                    if !lr.is_empty()
                        && lr.end <= text.len()
                        && text[lr].trim_start().starts_with('>')
                    {
                        valid_end = l_idx;
                    }
                }
                end_line = valid_end;
                Some(AstSpanKind::BlockQuote)
            }
            NodeValue::Item(ni) => {
                let ordered = ni.list_type == ListType::Ordered;
                end_line = start_line;
                let lr = self.line_content_byte_range(start_line);
                let mut content_byte_start = lr.start;
                if !lr.is_empty() && lr.end <= text.len() {
                    let raw_l = &text[lr.clone()];
                    let trimmed = raw_l.trim_start_matches([' ', '\t']);
                    let leading_ws = raw_l.len() - trimmed.len();
                    if !ordered
                        && (trimmed.starts_with("- ")
                            || trimmed.starts_with("* ")
                            || trimmed.starts_with("+ "))
                    {
                        content_byte_start = lr.start + leading_ws + 2;
                    } else {
                        content_byte_start = lr.start + leading_ws;
                    }
                }
                let content_char_start = self.byte_to_char(content_byte_start);
                Some(AstSpanKind::ListItem {
                    ordered,
                    task: None,
                    content_char_start,
                })
            }
            NodeValue::TaskItem(symbol) => {
                end_line = start_line;
                let lr = self.line_content_byte_range(start_line);
                let mut content_byte_start = lr.start;
                if !lr.is_empty() && lr.end <= text.len() {
                    let raw_l = &text[lr.clone()];
                    let trimmed = raw_l.trim_start_matches([' ', '\t']);
                    let leading_ws = raw_l.len() - trimmed.len();
                    if trimmed.len() >= 5 && trimmed.is_char_boundary(5) {
                        let after_box = &trimmed[5..];
                        let space_len = usize::from(after_box.starts_with(' '));
                        content_byte_start = lr.start + leading_ws + 5 + space_len;
                    }
                }
                let content_char_start = self.byte_to_char(content_byte_start);
                Some(AstSpanKind::ListItem {
                    ordered: false,
                    task: Some(symbol.is_some()),
                    content_char_start,
                })
            }
            NodeValue::Table(_) => {
                let mut headers = Vec::new();
                let mut rows = Vec::new();
                // Clamp table rows to lines that actually contain `|` so trailing paragraphs without blank lines are not absorbed
                let mut valid_end_line = start_line.saturating_add(1).min(end_line);
                for row_node in node.children() {
                    let row_sp = row_node.data.borrow().sourcepos;
                    let row_line = row_sp.start.line.saturating_sub(1);
                    let lr = self.line_content_byte_range(row_line);
                    let has_pipe =
                        !lr.is_empty() && lr.end <= text.len() && text[lr].contains('|');
                    if !has_pipe && row_line > start_line + 1 {
                        break;
                    }
                    valid_end_line = valid_end_line.max(row_line);
                    let is_header =
                        matches!(row_node.data.borrow().value, NodeValue::TableRow(true));
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
                end_line = valid_end_line;
                Some(AstSpanKind::Table { headers, rows })
            }
            NodeValue::ThematicBreak => Some(AstSpanKind::ThematicBreak),
            NodeValue::Alert(alert) => {
                let mut valid_end = start_line;
                for l_idx in start_line..=end_line {
                    let lr = self.line_content_byte_range(l_idx);
                    if !lr.is_empty()
                        && lr.end <= text.len()
                        && text[lr].trim_start().starts_with('>')
                    {
                        valid_end = l_idx;
                    }
                }
                end_line = valid_end;
                Some(AstSpanKind::Alert {
                    alert_type: format!("{:?}", alert.alert_type),
                })
            }
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
            let mut byte_range = self.sourcepos_to_byte_range(sp);
            if matches!(
                kind,
                AstSpanKind::CodeBlock { .. }
                    | AstSpanKind::BlockQuote
                    | AstSpanKind::Alert { .. }
                    | AstSpanKind::ListItem { .. }
                    | AstSpanKind::Table { .. }
            ) {
                let start_lr = self.line_content_byte_range(start_line);
                let end_lr = self.line_content_byte_range(end_line);
                if start_lr.start <= end_lr.end {
                    byte_range = start_lr.start..end_lr.end;
                }
            } else if matches!(kind, AstSpanKind::CodeInline | AstSpanKind::MathInline)
                && !byte_range.is_empty()
            {
                let delim_b = if matches!(kind, AstSpanKind::CodeInline) {
                    b'`'
                } else {
                    b'$'
                };
                while byte_range.start > 0
                    && self.rope.byte(byte_range.start - 1) == delim_b
                    && byte_range.end < self.rope.len_bytes()
                    && self.rope.byte(byte_range.end) == delim_b
                {
                    byte_range.start -= 1;
                    byte_range.end += 1;
                }
            }
            if !byte_range.is_empty() {
                let char_range =
                    self.byte_to_char(byte_range.start)..self.byte_to_char(byte_range.end);
                start_line = start_line.min(end_line);
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

    /// Returns cached `Arc<Vec<EditorTextSegment>>` in $O(1)$ on cache hit, or patches only the 1..3
    /// lines overlapping expanded spans when the cursor enters/leaves a Markdown span.
    pub fn build_editor_segments_arc(
        &self,
        dialect: MarkdownDialect,
        view_while_typing: bool,
        caret_byte_range: Range<usize>,
    ) -> Arc<Vec<EditorTextSegment>> {
        let text = self.as_str();
        if text.is_empty() {
            return Arc::new(Vec::new());
        }
        self.ensure_prepared_base(dialect);

        if !view_while_typing {
            {
                let c = self.cache.borrow();
                if let Some(arc) = c
                    .prepared_base
                    .as_ref()
                    .and_then(|b| b.all_expanded_segments.as_ref())
                {
                    return Arc::clone(arc);
                }
            }
            let built = {
                let c = self.cache.borrow();
                let base = c.prepared_base.as_ref().unwrap();
                let total_lines = self.rope.len_lines();
                let mut out = Vec::with_capacity(total_lines * 2);
                for (line_idx, line_span_indices) in base.spans_by_line.iter().enumerate() {
                    self.append_line_editor_segments(
                        text,
                        line_idx,
                        &base.ast_spans,
                        &base.delimiters,
                        line_span_indices,
                        &[],
                        true,
                        &mut out,
                    );
                }
                Arc::new(out)
            };
            let mut c = self.cache.borrow_mut();
            c.prepared_base.as_mut().unwrap().all_expanded_segments = Some(Arc::clone(&built));
            return built;
        }

        // Ensure base all-collapsed segments + per-line index ranges exist once per (revision, dialect)
        let needs_collapsed_base = {
            let c = self.cache.borrow();
            c.prepared_base
                .as_ref()
                .is_some_and(|b| b.collapsed_segments.is_none())
        };
        if needs_collapsed_base {
            let (arc_segs, line_ranges) = {
                let c = self.cache.borrow();
                let base = c.prepared_base.as_ref().unwrap();
                let total_lines = self.rope.len_lines();
                let mut out = Vec::with_capacity(total_lines * 2);
                let mut ranges = Vec::with_capacity(total_lines);
                for (line_idx, line_span_indices) in base.spans_by_line.iter().enumerate() {
                    let s0 = out.len();
                    self.append_line_editor_segments(
                        text,
                        line_idx,
                        &base.ast_spans,
                        &base.delimiters,
                        line_span_indices,
                        &[],
                        false,
                        &mut out,
                    );
                    let s1 = out.len();
                    ranges.push(s0..s1);
                }
                (Arc::new(out), ranges)
            };
            let mut c = self.cache.borrow_mut();
            c.prepared_base.as_mut().unwrap().collapsed_segments = Some((arc_segs, line_ranges));
        }

        let expanded = {
            let c = self.cache.borrow();
            let base = c.prepared_base.as_ref().unwrap();
            let exp = self.expanded_span_indices_for_caret(base, &caret_byte_range);
            if exp.is_empty() {
                return Arc::clone(&base.collapsed_segments.as_ref().unwrap().0);
            }
            if let Some((_, cached_arc)) = base.seg_lru.iter().find(|(k, _)| *k == exp) {
                return Arc::clone(cached_arc);
            }
            exp
        };

        let patched_arc = {
            let c = self.cache.borrow();
            let base = c.prepared_base.as_ref().unwrap();
            let (collapsed_segs, line_ranges) = base.collapsed_segments.as_ref().unwrap();
            let max_line_idx = line_ranges.len().saturating_sub(1);

            let min_dirty_line = expanded
                .iter()
                .map(|&idx| base.ast_spans[idx].start_line.min(max_line_idx))
                .min()
                .unwrap_or(0);
            let max_dirty_line = expanded
                .iter()
                .map(|&idx| base.ast_spans[idx].end_line.min(max_line_idx))
                .max()
                .unwrap_or(0);

            let prefix_end = line_ranges[min_dirty_line].start;
            let suffix_start = line_ranges[max_dirty_line].end;

            let mut out = Vec::with_capacity(collapsed_segs.len() + 16);
            out.extend_from_slice(&collapsed_segs[..prefix_end]);
            for line_idx in min_dirty_line..=max_dirty_line {
                self.append_line_editor_segments(
                    text,
                    line_idx,
                    &base.ast_spans,
                    &base.delimiters,
                    &base.spans_by_line[line_idx],
                    &expanded,
                    false,
                    &mut out,
                );
            }
            out.extend_from_slice(&collapsed_segs[suffix_start..]);
            Arc::new(out)
        };

        let mut c = self.cache.borrow_mut();
        let base = c.prepared_base.as_mut().unwrap();
        if base.seg_lru.len() >= 8 {
            base.seg_lru.pop_front();
        }
        base.seg_lru
            .push_back((expanded, Arc::clone(&patched_arc)));
        patched_arc
    }

    #[allow(clippy::too_many_arguments)]
    fn append_line_editor_segments(
        &self,
        text: &str,
        line_idx: usize,
        ast_spans: &[AstSpan],
        delimiters: &[Arc<[Range<usize>]>],
        line_span_indices: &[usize],
        expanded_indices: &[usize],
        all_expanded: bool,
        out: &mut Vec<EditorTextSegment>,
    ) {
        let line_start = self.rope.line_to_byte(line_idx);
        let slice = self.rope.line(line_idx);
        let full_line_bytes = slice.len_bytes();
        if full_line_bytes == 0 {
            return;
        }
        let full_line_end = line_start + full_line_bytes;

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

        let is_span_collapsed = |s_idx: usize| -> bool {
            !all_expanded && expanded_indices.binary_search(&s_idx).is_err()
        };

        let mut heading_span: Option<(u8, bool)> = None;
        let mut code_block_idx: Option<usize> = None;
        let mut in_blockquote = false;
        let mut is_list_item_start = false;

        for &s_idx in line_span_indices {
            let span = &ast_spans[s_idx];
            match &span.kind {
                AstSpanKind::Heading { level } => {
                    heading_span = Some((*level, is_span_collapsed(s_idx)));
                }
                AstSpanKind::CodeBlock { .. } => {
                    code_block_idx = Some(s_idx);
                }
                AstSpanKind::BlockQuote | AstSpanKind::Alert { .. } => {
                    in_blockquote = true;
                }
                AstSpanKind::ListItem { ordered, .. } => {
                    if !*ordered && span.start_line == line_idx {
                        is_list_item_start = true;
                    }
                }
                _ => {}
            }
        }
        let heading_level = heading_span.map(|(lvl, _)| lvl);
        let in_code_block = code_block_idx.is_some();

        if content_len == 0 {
            out.push(EditorTextSegment {
                byte_range: line_start..full_line_end,
                is_line_start: true,
                line_index: line_idx,
                code_block: in_code_block,
                blockquote: in_blockquote,
                ..Default::default()
            });
            return;
        }

        if let Some(cb_idx) = code_block_idx {
            let cb_span = &ast_spans[cb_idx];
            let is_fence_delim = delimiters[cb_idx]
                .iter()
                .any(|dr| dr.start == line_start && dr.end == content_end);
            let is_mermaid_block = matches!(
                &cb_span.kind,
                AstSpanKind::CodeBlock { info, .. }
                    if info.trim().len() >= 7
                        && info.trim().as_bytes()[..7].eq_ignore_ascii_case(b"mermaid")
            );
            let is_cb_collapsed = is_span_collapsed(cb_idx);
            let fence_collapsed = is_fence_delim && is_cb_collapsed;
            let mermaid_body_collapsed = !is_fence_delim && is_mermaid_block && is_cb_collapsed;
            let trimmed_cb = raw_line.trim_start();
            let is_code_comment =
                !is_fence_delim && (trimmed_cb.starts_with('#') || trimmed_cb.starts_with("//"));
            out.push(EditorTextSegment {
                byte_range: line_start..content_end,
                is_line_start: true,
                line_index: line_idx,
                code_block: true,
                code_comment: is_code_comment,
                code_inline: true,
                muted: is_fence_delim || is_code_comment,
                collapsed_delimiter: fence_collapsed || mermaid_body_collapsed,
                mermaid_collapsed: mermaid_body_collapsed,
                ..Default::default()
            });
            if content_end < full_line_end {
                out.push(EditorTextSegment {
                    byte_range: content_end..full_line_end,
                    is_line_start: false,
                    line_index: line_idx,
                    code_block: true,
                    collapsed_delimiter: fence_collapsed || mermaid_body_collapsed,
                    mermaid_collapsed: mermaid_body_collapsed,
                    ..Default::default()
                });
            }
            return;
        }

        let mut content_start = line_start;
        let mut next_is_line_start = true;

        if let Some((level, heading_collapsed)) = heading_span {
            let trimmed = raw_line.trim_start();
            let leading_ws = raw_line.len() - trimmed.len();
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            let after_hashes = &trimmed[hash_count..];
            let space_len = usize::from(after_hashes.starts_with(' '));
            let prefix_end = line_start + leading_ws + hash_count + space_len;

            if hash_count > 0 && prefix_end > line_start {
                out.push(EditorTextSegment {
                    byte_range: line_start..prefix_end,
                    is_line_start: true,
                    line_index: line_idx,
                    heading_level: Some(level),
                    is_heading_hashes: true,
                    bold: true,
                    muted: heading_collapsed,
                    collapsed_delimiter: heading_collapsed,
                    ..Default::default()
                });
                content_start = prefix_end;
                next_is_line_start = false;
            }
        }

        self.slice_editor_inline_segments_indexed(
            text,
            content_start..content_end,
            ast_spans,
            delimiters,
            line_span_indices,
            expanded_indices,
            all_expanded,
            line_idx,
            heading_level,
            in_blockquote,
            is_list_item_start,
            next_is_line_start,
            out,
        );

        if content_end < full_line_end {
            out.push(EditorTextSegment {
                byte_range: content_end..full_line_end,
                is_line_start: false,
                line_index: line_idx,
                heading_level,
                blockquote: in_blockquote,
                list_item: is_list_item_start,
                ..Default::default()
            });
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
        (*self.build_editor_segments_arc(dialect, view_while_typing, caret_byte_range)).clone()
    }

    #[allow(clippy::too_many_arguments)]
    fn slice_editor_inline_segments_indexed(
        &self,
        text: &str,
        range: Range<usize>,
        ast_spans: &[AstSpan],
        delimiters: &[Arc<[Range<usize>]>],
        line_span_indices: &[usize],
        expanded_indices: &[usize],
        all_expanded: bool,
        line_idx: usize,
        heading_level: Option<u8>,
        in_blockquote: bool,
        is_list_item_start: bool,
        first_is_line_start: bool,
        out: &mut Vec<EditorTextSegment>,
    ) {
        if range.is_empty() {
            return;
        }
        let base_bold = heading_level.is_some();

        if line_span_indices.is_empty() {
            out.push(EditorTextSegment {
                byte_range: range,
                is_line_start: first_is_line_start,
                line_index: line_idx,
                heading_level,
                blockquote: in_blockquote,
                list_item: is_list_item_start,
                bold: base_bold,
                ..Default::default()
            });
            return;
        }

        let is_span_expanded = |s_idx: usize| -> bool {
            all_expanded || expanded_indices.binary_search(&s_idx).is_ok()
        };

        let mut boundaries = [0usize; 2];
        boundaries[0] = range.start;
        boundaries[1] = range.end;
        let mut bounds_vec = Vec::with_capacity(8);
        bounds_vec.extend_from_slice(&boundaries);
        let mut list_marker_range: Option<(Range<usize>, Option<bool>)> = None;

        for &s_idx in line_span_indices {
            let span = &ast_spans[s_idx];
            let sr = &span.byte_range;
            if sr.end <= range.start || sr.start >= range.end {
                continue;
            }
            for dr in delimiters[s_idx].iter() {
                if dr.start >= range.start && dr.start <= range.end {
                    bounds_vec.push(dr.start);
                }
                if dr.end >= range.start && dr.end <= range.end {
                    bounds_vec.push(dr.end);
                }
            }
            match &span.kind {
                AstSpanKind::ListItem { ordered, task, .. } if span.start_line == line_idx => {
                    let raw_l = &text[range.clone()];
                    let trimmed = raw_l.trim_start_matches([' ', '\t']);
                    let leading_ws = raw_l.len() - trimmed.len();
                    let m_start = range.start + leading_ws;
                    let m_len = if task.is_some() && trimmed.len() >= 5 && trimmed.is_char_boundary(5) {
                        let after_box = &trimmed[5..];
                        5 + usize::from(after_box.starts_with(' '))
                    } else if !*ordered
                        && (trimmed.starts_with("- ")
                            || trimmed.starts_with("* ")
                            || trimmed.starts_with("+ "))
                    {
                        2
                    } else if *ordered {
                        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
                        if digits > 0
                            && (trimmed[digits..].starts_with(". ")
                                || trimmed[digits..].starts_with(") "))
                        {
                            digits + 2
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    if m_len > 0 && m_start + m_len <= range.end {
                        bounds_vec.push(m_start);
                        bounds_vec.push(m_start + m_len);
                        list_marker_range = Some((m_start..(m_start + m_len), *task));
                    }
                }
                AstSpanKind::Table { .. } if is_span_expanded(s_idx) => {
                    for (off, ch) in text[range.clone()].char_indices() {
                        if ch == '|' {
                            bounds_vec.push(range.start + off);
                            bounds_vec.push(range.start + off + 1);
                        }
                    }
                }
                AstSpanKind::Strong
                | AstSpanKind::Emph
                | AstSpanKind::Strikethrough
                | AstSpanKind::CodeInline
                | AstSpanKind::MathInline => {
                    bounds_vec.push(sr.start.max(range.start));
                    bounds_vec.push(sr.end.min(range.end));
                }
                AstSpanKind::Link {
                    content_byte_range, ..
                } => {
                    bounds_vec.push(sr.start.max(range.start));
                    bounds_vec.push(sr.end.min(range.end));
                    if let Some(cr) = content_byte_range {
                        if cr.start >= range.start && cr.start <= range.end {
                            bounds_vec.push(cr.start);
                        }
                        if cr.end >= range.start && cr.end <= range.end {
                            bounds_vec.push(cr.end);
                        }
                    }
                }
                AstSpanKind::Image {
                    content_byte_range, ..
                } => {
                    bounds_vec.push(sr.start.max(range.start));
                    bounds_vec.push(sr.end.min(range.end));
                    if sr.start + 1 >= range.start && sr.start + 1 <= range.end {
                        bounds_vec.push(sr.start + 1);
                    }
                    if let Some(cr) = content_byte_range {
                        if cr.start >= range.start && cr.start <= range.end {
                            bounds_vec.push(cr.start);
                        }
                        if cr.end >= range.start && cr.end <= range.end {
                            bounds_vec.push(cr.end);
                        }
                    }
                }
                _ => {}
            }
        }

        bounds_vec.sort_unstable();
        bounds_vec.dedup();
        bounds_vec.retain(|&b| text.is_char_boundary(b));

        for (i, pair) in bounds_vec.windows(2).enumerate() {
            let (seg_start, seg_end) = (pair[0], pair[1]);
            if seg_start >= seg_end {
                continue;
            }
            let mut seg = EditorTextSegment {
                byte_range: seg_start..seg_end,
                is_line_start: i == 0 && first_is_line_start,
                line_index: line_idx,
                heading_level,
                blockquote: in_blockquote,
                list_item: is_list_item_start,
                bold: base_bold,
                ..Default::default()
            };

            if let Some((ref mr, task_opt)) = list_marker_range {
                if seg_start >= mr.start && seg_end <= mr.end {
                    seg.list_marker = true;
                    seg.task_checked = task_opt;
                    seg.bold = true;
                }
            }

            for &s_idx in line_span_indices {
                let span = &ast_spans[s_idx];
                let sr = &span.byte_range;
                if seg_start < sr.start || seg_end > sr.end {
                    continue;
                }
                let expanded_flag = is_span_expanded(s_idx);
                let is_delim = delimiters[s_idx]
                    .iter()
                    .any(|dr| seg_start >= dr.start && seg_end <= dr.end);
                if is_delim {
                    if !matches!(span.kind, AstSpanKind::Table { .. }) {
                        seg.muted = true;
                    }
                    if !expanded_flag {
                        seg.collapsed_delimiter = true;
                    }
                }

                match &span.kind {
                    AstSpanKind::Table { .. } if expanded_flag => {
                        let line_slice = text[range.clone()].trim();
                        let is_sep_row = !line_slice.is_empty()
                            && line_slice
                                .chars()
                                .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'));
                        if is_sep_row || &text[seg_start..seg_end] == "|" {
                            seg.table_pipe = true;
                            seg.bold = true;
                        } else if line_idx == span.start_line {
                            seg.table_header_cell = true;
                            seg.bold = true;
                        }
                    }
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
                                seg.link_url_part = true;
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
                                seg.link_url_part = true;
                                seg.italic = true;
                            }
                        } else {
                            seg.muted = true;
                            seg.link_url_part = true;
                            seg.italic = true;
                        }
                    }
                    _ => {}
                }
            }

            out.push(seg);
        }
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
            for dr in ls.delimiter_byte_ranges.iter() {
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

    /// Parses the document into structured `PreviewBlock`s for Split/Full/Windowed Preview rendering, cached by `(revision, dialect)`.
    pub fn parse_preview_blocks_arc(&self, dialect: MarkdownDialect) -> Arc<Vec<PreviewBlock>> {
        let key = (self.revision, dialect);
        let also_build_spans = {
            let mut c = self.cache.borrow_mut();
            c.preview_ever_used_for = Some(dialect);
            if c.preview_key == Some(key) {
                return Arc::clone(&c.preview_blocks);
            }
            c.ast_key != Some(key)
        };

        let text = self.as_str();
        let arena = Arena::new();
        let opts = dialect.to_comrak_options();
        let root = comrak::parse_document(&arena, text, &opts);

        let mut blocks = Vec::new();
        for child in root.children() {
            self.collect_preview_block(child, &mut blocks);
        }
        let arc = Arc::new(blocks);

        let maybe_spans = if also_build_spans {
            let mut spans = Vec::new();
            self.collect_node_spans(root, &mut spans);
            Some(Arc::new(spans))
        } else {
            None
        };

        let mut c = self.cache.borrow_mut();
        c.preview_key = Some(key);
        c.preview_blocks = Arc::clone(&arc);
        if let Some(s_arc) = maybe_spans {
            c.ast_key = Some(key);
            c.ast_spans = s_arc;
        }
        arc
    }

    /// Parses the document into structured `PreviewBlock`s for Split/Full/Windowed Preview rendering.
    pub fn parse_preview_blocks(&self, dialect: MarkdownDialect) -> Vec<PreviewBlock> {
        (*self.parse_preview_blocks_arc(dialect)).clone()
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
                    text: expand_emoji_shortcodes(t),
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
            NodeValue::Math(m) => {
                let formatted = if m.display_math {
                    format!("$${}$$", m.literal)
                } else {
                    format!("${}$", m.literal)
                };
                out.push(PreviewInline {
                    text: formatted,
                    bold,
                    italic: true,
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

/// Looks up a standard GitHub/Markdown emoji shortcode name (without surrounding colons) and returns its Unicode emoji string.
pub fn lookup_emoji_shortcode(shortcode: &str) -> Option<&'static str> {
    match shortcode {
        "rocket" => Some("🚀"),
        "sparkles" => Some("✨"),
        "star" => Some("⭐"),
        "star2" | "glowing_star" => Some("🌟"),
        "fire" => Some("🔥"),
        "zap" | "lightning" => Some("⚡"),
        "crab" | "rust" => Some("🦀"),
        "warning" | "warn" => Some("⚠"),
        "check" | "white_check_mark" | "heavy_check_mark" => Some("✓"),
        "x" | "cross_mark" => Some("✗"),
        "bulb" | "idea" => Some("💡"),
        "heart" => Some("❤"),
        "thumbsup" | "+1" => Some("👍"),
        "thumbsdown" | "-1" => Some("👎"),
        "smile" | "grinning" => Some("😀"),
        "joy" => Some("😂"),
        "wink" => Some("😉"),
        "thinking" => Some("🤔"),
        "eyes" => Some("👀"),
        "tada" | "party" => Some("🎉"),
        "100" => Some("💯"),
        "info" | "information_source" => Some("ℹ"),
        "stop" | "octagonal_sign" => Some("🛑"),
        "book" | "open_book" => Some("📖"),
        "memo" | "pencil" => Some("📝"),
        "link" => Some("🔗"),
        "lock" => Some("🔒"),
        "unlock" => Some("🔓"),
        "key" => Some("🔑"),
        "gear" | "settings" => Some("⚙"),
        "bug" => Some("🐛"),
        "wrench" => Some("🔧"),
        "hammer" => Some("🔨"),
        "package" => Some("📦"),
        "Wave" | "wave" => Some("👋"),
        "clap" => Some("👏"),
        "muscle" => Some("💪"),
        "pray" => Some("🙏"),
        "trophy" => Some("🏆"),
        "crown" => Some("👑"),
        "gem" => Some("💎"),
        "bell" => Some("🔔"),
        "bookmark" => Some("🔖"),
        "pushpin" | "pin" => Some("📌"),
        "calendar" => Some("📅"),
        "hourglass" | "clock" => Some("⏳"),
        "mag" | "magnifying_glass" | "search" => Some("🔍"),
        "chart_with_upwards_trend" => Some("📈"),
        "shield" => Some("🛡"),
        "globe" | "earth" => Some("🌐"),
        "computer" | "laptop" => Some("💻"),
        "coffee" => Some("☕"),
        "pizza" => Some("🍕"),
        "sun" | "sunny" => Some("☀"),
        "moon" => Some("🌙"),
        "cloud" => Some("☁"),
        "rainbow" => Some("🌈"),
        "umbrella" => Some("☂"),
        "snowflake" => Some("❄"),
        "music" | "musical_note" => Some("🎵"),
        "art" => Some("🎨"),
        _ => None,
    }
}

/// Expands `:shortcode:` tokens inside `text` into their Unicode emoji equivalents while leaving unknown `:tokens:` untouched.
pub fn expand_emoji_shortcodes(text: &str) -> String {
    if !text.contains(':') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            let mut end = i + 1;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric()
                    || bytes[end] == b'_'
                    || bytes[end] == b'+'
                    || bytes[end] == b'-')
            {
                end += 1;
            }
            if end > i + 1 && end < bytes.len() && bytes[end] == b':' {
                let candidate = &text[(i + 1)..end];
                if let Some(emoji) = lookup_emoji_shortcode(candidate) {
                    out.push_str(emoji);
                    i = end + 1;
                    continue;
                }
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
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
        assert!(buf.text().starts_with("Test # Guida ed Esempi di Accent"));
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
campo_data_registrazione__c
(campi  campo_identificativo_documento__c,
valore_configurazione_sistema__c, codice_riferimento_interno__c,
parametro_sincronizzazione__c, attributo_personalizzato__c,
indice_archivio_principale__c, stato_elaborazione_finale__c)
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
        let md = "Inizio **parola** e ~~barrato~~ fine\n";
        let buf = DocumentBuffer::from_str_content(md);

        // 1. Caret at byte 0 (outside both **parola** and ~~barrato~~) with view_while_typing = true
        let collapsed_spans = buf.compute_live_spans(MarkdownDialect::Gfm, true, 0..0);
        let strong_span = collapsed_spans
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Strong))
            .expect("Strong span present");
        assert_eq!(strong_span.visibility, DelimiterVisibility::Collapsed);
        assert_eq!(strong_span.delimiter_byte_ranges.len(), 2);

        let segs_collapsed = buf.build_editor_segments(MarkdownDialect::Gfm, true, 0..0);
        // Verify 1:1 text preservation (zero cursor drift)
        let reconstructed: String = segs_collapsed.iter().map(|s| s.as_str(md)).collect();
        assert_eq!(reconstructed, md);
        assert!(segs_collapsed
            .iter()
            .any(|s| s.as_str(md) == "**" && s.collapsed_delimiter));

        // 2. Caret at byte 9 (inside `**parola**`, byte range 7..17) -> Strong becomes Expanded!
        let expanded_spans = buf.compute_live_spans(MarkdownDialect::Gfm, true, 9..9);
        let strong_expanded = expanded_spans
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Strong))
            .expect("Strong span present");
        assert_eq!(strong_expanded.visibility, DelimiterVisibility::Expanded);
        // While ~~barrato~~ remains Collapsed!
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
    fn test_view_while_typing_collapses_headings_tables_quotes_lists_and_code_fences() {
        let md = "\
# Titolo Principale

Paragrafo neutro dove risiede il cursore.

## Sottotitolo

- [x] Attività completata
- Voce elenco

> Citazione importante

```rust
let x = 42;
```

| Colonna A | Colonna B |
|:---|:---|
| Valore 1 | Valore 2 |
";
        let buf = DocumentBuffer::from_str_content(md);
        let neutral_pos = md.find("Paragrafo neutro").unwrap();

        let segs = buf.build_editor_segments(
            MarkdownDialect::Gfm,
            true,
            neutral_pos..(neutral_pos + 1),
        );
        // 1:1 text preservation across all blocks
        let reconstructed: String = segs.iter().map(|s| s.as_str(md)).collect();
        assert_eq!(reconstructed, md);

        // Verify `# ` and `## ` heading prefixes are collapsed when caret is outside
        assert!(segs
            .iter()
            .any(|s| s.is_heading_hashes && s.as_str(md) == "# " && s.collapsed_delimiter));
        assert!(segs
            .iter()
            .any(|s| s.is_heading_hashes && s.as_str(md) == "## " && s.collapsed_delimiter));

        // Verify `- [x] `, `- `, `> `, and ` ```rust ` are collapsed when caret is outside
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "- [x] " && s.collapsed_delimiter));
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "- " && s.collapsed_delimiter));
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "> " && s.collapsed_delimiter));
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "```rust" && s.collapsed_delimiter));

        // Verify table lines are collapsed when caret is outside, and expand when caret enters table
        let live_outside = buf.compute_live_spans(
            MarkdownDialect::Gfm,
            true,
            neutral_pos..(neutral_pos + 1),
        );
        let table_outside = live_outside
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Table { .. }))
            .expect("Table span present");
        assert_eq!(table_outside.visibility, DelimiterVisibility::Collapsed);
        assert_eq!(table_outside.delimiter_byte_ranges.len(), 3);

        let table_pos = md.find("Valore 1").unwrap();
        let live_inside =
            buf.compute_live_spans(MarkdownDialect::Gfm, true, table_pos..(table_pos + 1));
        let table_inside = live_inside
            .iter()
            .find(|s| matches!(s.span.kind, AstSpanKind::Table { .. }))
            .expect("Table span present");
        assert_eq!(table_inside.visibility, DelimiterVisibility::Expanded);
    }

    #[test]
    fn test_classify_lines_outdented_headings_and_code() {
        let buf = DocumentBuffer::default();
        let lines = buf.classify_lines();
        assert!(matches!(
            lines.first(),
            Some(LineKind::Heading { level: 1, .. })
        ));
        assert!(lines.iter().any(|l| matches!(l, LineKind::CodeBody { .. })));
        assert!(lines
            .iter()
            .any(|l| matches!(l, LineKind::BlockQuote { .. })));
    }

    #[test]
    fn test_gfm_reference_example_performance_and_ordered_vs_bullet_lists() {
        let gfm_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/GFM_reference_example.md");
        let content = std::fs::read_to_string(&gfm_path)
            .expect("docs/GFM_reference_example.md should exist in workspace");
        let buf = DocumentBuffer::from_str_content(&content);

        let t0 = std::time::Instant::now();
        let segs_1 = buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, 0..0);
        let cold_ms = t0.elapsed().as_secs_f64() * 1000.0;
        assert!(
            !segs_1.is_empty(),
            "Segments should be non-empty for GFM_reference_example.md"
        );
        // Verify 1:1 text preservation across all 5,050 lines
        let reconstructed: String = segs_1.iter().map(|s| s.as_str(&content)).collect();
        assert_eq!(reconstructed, content);

        // Second call with identical state must hit cache in < 0.5 ms
        let t1 = std::time::Instant::now();
        let segs_2 = buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, 0..0);
        let warm_ms = t1.elapsed().as_secs_f64() * 1000.0;
        assert!(Arc::ptr_eq(&segs_1, &segs_2));
        assert!(
            warm_ms < 0.5,
            "Warm cached call took {warm_ms:.3} ms (expected < 0.5 ms)"
        );
        // Even cold unoptimized debug build over 5,050 lines should complete quickly with O(L+S) indexing
        assert!(
            cold_ms < 450.0,
            "Cold O(L+S) build took {cold_ms:.1} ms (expected < 450 ms in debug)"
        );

        // Verify ordered lists (`1. Primo elemento...`) have `ordered: true` and NO collapsed delimiter ranges,
        // while bullet lists (`* Elemento puntato generico`) have `ordered: false` and collapsed `* ` delimiter!
        let live_spans = buf.compute_live_spans_arc(MarkdownDialect::Gfm, true, 0..0);
        let has_ordered = live_spans.iter().any(|ls| {
            matches!(
                ls.span.kind,
                AstSpanKind::ListItem {
                    ordered: true,
                    task: None,
                    ..
                }
            ) && ls.delimiter_byte_ranges.is_empty()
        });
        assert!(
            has_ordered,
            "Ordered list items must have ordered=true and no collapsed bullet delimiter"
        );

        let has_bullet = live_spans.iter().any(|ls| {
            matches!(
                ls.span.kind,
                AstSpanKind::ListItem {
                    ordered: false,
                    task: None,
                    ..
                }
            ) && !ls.delimiter_byte_ranges.is_empty()
        });
        assert!(
            has_bullet,
            "Bullet list items must have ordered=false and collapsed `* `/` - ` delimiter"
        );

        // Verify table on lines 8-11 does NOT absorb line 12 (`Testo in **grassetto** per dare enfasi.`)
        let first_table = live_spans
            .iter()
            .find(|ls| matches!(ls.span.kind, AstSpanKind::Table { .. }))
            .expect("First GFM table should exist");
        if let AstSpanKind::Table { rows, .. } = &first_table.span.kind {
            assert_eq!(
                rows.len(),
                2,
                "First table must have exactly 2 data rows (#101 and #102), not absorb trailing paragraphs"
            );
        }
    }

    #[test]
    fn test_utf8_multibyte_char_to_byte_caret_expansion_inclusive_boundaries() {
        let doc = "Città perché — **parola** fine";
        let mut buf = DocumentBuffer::from_str_content(doc);

        // Locate the `Strong` span for `**parola**`
        let spans = buf.extract_ast_spans(MarkdownDialect::Gfm);
        let strong_span = spans
            .iter()
            .find(|s| matches!(s.kind, AstSpanKind::Strong))
            .expect("Strong span for **parola** should exist");

        assert_eq!(&doc[strong_span.byte_range.clone()], "**parola**");
        let start_char = strong_span.char_range.start;
        let end_char = strong_span.char_range.end;
        // Verify that multi-byte UTF-8 chars ('à', 'é', '—') before `**parola**` cause byte_range != char_range
        assert!(
            strong_span.byte_range.start > start_char,
            "Multi-byte UTF-8 characters before **parola** must make byte_range.start > char_range.start"
        );

        // Every single char position from the first `*` (start_char) through the position immediately
        // after the last `*` (end_char) INCLUSIVE must expand `**parola**` when converted via char_to_byte!
        for char_pos in 0..=buf.len_chars() {
            buf.set_cursor(char_pos);
            let caret_bytes = buf.selection_byte_range();
            let live = buf.compute_live_spans(MarkdownDialect::Gfm, true, caret_bytes.clone());
            let strong_live = live
                .iter()
                .find(|ls| matches!(ls.span.kind, AstSpanKind::Strong))
                .expect("LiveSpan for Strong should exist");
            let segs = buf.build_editor_segments(MarkdownDialect::Gfm, true, caret_bytes);
            let any_collapsed = segs.iter().any(|s| s.collapsed_delimiter);

            if (start_char..=end_char).contains(&char_pos) {
                assert_eq!(
                    strong_live.visibility,
                    DelimiterVisibility::Expanded,
                    "Expected Expanded at char_pos={char_pos} (within {start_char}..={end_char})"
                );
                assert!(
                    !any_collapsed,
                    "Expected no collapsed delimiters in segments at char_pos={char_pos}"
                );
            } else {
                assert_eq!(
                    strong_live.visibility,
                    DelimiterVisibility::Collapsed,
                    "Expected Collapsed at char_pos={char_pos} (outside {start_char}..={end_char})"
                );
                assert!(
                    any_collapsed,
                    "Expected collapsed `**` delimiters in segments at char_pos={char_pos}"
                );
            }
        }
    }

    #[test]
    fn test_semantic_markup_segment_flags_lists_tables_links_and_comments() {
        let md = "\
1. **Strict Ban:**
- [x] Done task
- [ ] Todo task
[guida](https://accent.dev)

```bash
# Comment line
DISPLAY=:99 apostrophe
```

| Costrutto | Colore |
| :--- | :--- |
| Bold | Oro |
";
        let buf = DocumentBuffer::from_str_content(md);
        // Source mode (view_while_typing = false -> all spans Expanded)
        let segs = buf.build_editor_segments(MarkdownDialect::Gfm, false, 0..0);
        let reconstructed: String = segs.iter().map(|s| s.as_str(md)).collect();
        assert_eq!(reconstructed, md);

        // Ordered list marker `1. `
        assert!(segs.iter().any(|s| s.as_str(md) == "1. " && s.list_marker));
        // Checked task marker `- [x] `
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "- [x] " && s.list_marker && s.task_checked == Some(true)));
        // Unchecked task marker `- [ ] `
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "- [ ] " && s.list_marker && s.task_checked == Some(false)));
        // Link text and URL part
        assert!(segs.iter().any(|s| s.as_str(md) == "guida" && s.link));
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "](https://accent.dev)" && s.link_url_part));
        // Code block comment vs body
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "# Comment line" && s.code_block && s.code_comment));
        assert!(segs
            .iter()
            .any(|s| s.as_str(md) == "DISPLAY=:99 apostrophe" && s.code_block && !s.code_comment));
        // Expanded table pipes and header cells
        assert!(segs.iter().any(|s| s.as_str(md) == "|" && s.table_pipe));
        assert!(segs
            .iter()
            .any(|s| s.as_str(md).contains("Costrutto") && s.table_header_cell));
    }

    #[test]
    fn test_unicode_emoji_basic_and_zwj_ast_alignment() {
        let md = "\
# Titolo 🚀 con Famiglia 👨‍👩‍👧‍👦 e Bandiera 🇮🇹

Paragrafo con **grassetto 🦀👍🏽✨**, *corsivo 👩‍💻*, ~~barrato 🏳️‍🌈~~ e `codice 💡⚡`.

- [x] Task completato ✅ con cuore ❤️
- [ ] Task aperto ⚠️ con stella ⭐
1. Elemento numerato 🔥 con link [Sito 🌐](https://accent.dev/🚀)

> Citazione con bussola 🧭 e ingranaggio ⚙️

| Emoji | Descrizione | Stato |
| :--- | :---: | ---: |
| 🚀 | Razzo spaziale | Attivo ✅ |
| 👨‍👩‍👧‍👦 | Sequenza ZWJ (25 byte) | Testato 💯 |
";
        let buf = DocumentBuffer::from_str_content(md);
        for view_while_typing in [false, true] {
            let segs = buf.build_editor_segments(MarkdownDialect::Gfm, view_while_typing, 0..0);
            let reconstructed: String = segs.iter().map(|s| s.as_str(md)).collect();
            assert_eq!(
                reconstructed, md,
                "1:1 UTF-8 reconstruction must hold with complex ZWJ and flag emojis"
            );

            // Verify every segment's byte_range matches its exact slice in `md`
            for seg in &segs {
                assert!(md.is_char_boundary(seg.byte_range.start));
                assert!(md.is_char_boundary(seg.byte_range.end));
                assert_eq!(&md[seg.byte_range.clone()], seg.as_str(md));
            }
        }

        // Verify byte_to_char and char_to_byte roundtrip at every single char index
        for char_idx in 0..=buf.len_chars() {
            let byte_idx = buf.char_to_byte(char_idx);
            assert!(md.is_char_boundary(byte_idx));
            assert_eq!(buf.byte_to_char(byte_idx), char_idx);
        }
    }

    #[test]
    fn test_unicode_emoji_caret_unfolding_boundary_precision() {
        let md = "Inizio 🚀👨‍👩‍👧‍👦🇮🇹 **grassetto 🦀✨** e `codice 💡` fine\n";
        let mut buf = DocumentBuffer::from_str_content(md);

        let spans = buf.extract_ast_spans(MarkdownDialect::Gfm);
        let strong_span = spans
            .iter()
            .find(|s| matches!(s.kind, AstSpanKind::Strong))
            .expect("Strong span should exist");
        let code_span = spans
            .iter()
            .find(|s| matches!(s.kind, AstSpanKind::CodeInline))
            .expect("CodeInline span should exist");

        assert_eq!(&md[strong_span.byte_range.clone()], "**grassetto 🦀✨**");
        assert_eq!(&md[code_span.byte_range.clone()], "`codice 💡`");

        for char_pos in 0..=buf.len_chars() {
            buf.set_cursor(char_pos);
            let caret_bytes = buf.selection_byte_range();
            let live = buf.compute_live_spans(MarkdownDialect::Gfm, true, caret_bytes.clone());
            let segs = buf.build_editor_segments(MarkdownDialect::Gfm, true, caret_bytes);
            let reconstructed: String = segs.iter().map(|s| s.as_str(md)).collect();
            assert_eq!(reconstructed, md);

            let strong_live = live
                .iter()
                .find(|ls| matches!(ls.span.kind, AstSpanKind::Strong))
                .unwrap();
            let code_live = live
                .iter()
                .find(|ls| matches!(ls.span.kind, AstSpanKind::CodeInline))
                .unwrap();

            if (strong_span.char_range.start..=strong_span.char_range.end).contains(&char_pos) {
                assert_eq!(strong_live.visibility, DelimiterVisibility::Expanded);
            } else {
                assert_eq!(strong_live.visibility, DelimiterVisibility::Collapsed);
            }

            if (code_span.char_range.start..=code_span.char_range.end).contains(&char_pos) {
                assert_eq!(code_live.visibility, DelimiterVisibility::Expanded);
            } else {
                assert_eq!(code_live.visibility, DelimiterVisibility::Collapsed);
            }
        }
    }

    #[test]
    fn test_emoji_shortcodes_expansion_and_preview() {
        let raw = "Lancio :rocket: con :crab: e :sparkles: (ignora :non_esiste: e URL https://a.b:8080/c)";
        let expanded = expand_emoji_shortcodes(raw);
        assert_eq!(
            expanded,
            "Lancio 🚀 con 🦀 e ✨ (ignora :non_esiste: e URL https://a.b:8080/c)"
        );

        let buf = DocumentBuffer::from_str_content("# Titolo :rocket:\n\nTesto con :bulb: e :fire:\n");
        let blocks = buf.parse_preview_blocks(MarkdownDialect::Gfm);
        assert_eq!(blocks.len(), 2);
        if let PreviewBlock::Heading { inlines, .. } = &blocks[0] {
            let joined: String = inlines.iter().map(|i| i.text.as_str()).collect();
            assert_eq!(joined, "Titolo 🚀");
        } else {
            panic!("Expected Heading preview block");
        }
    }

    #[test]
    fn stress_test_unicode_emoji_massive_document_10k_glyphs() {
        // Build a heavy document with > 5,000 complex emojis across 600 lines of mixed Markdown
        let mut big_md = String::with_capacity(120_000);
        let emoji_cluster_line =
            "- [x] **Sprint 🚀✨🦀** con sequenza ZWJ 👨‍👩‍👧‍👦👩‍💻, bandiera 🇮🇹🇪🇺, tono 👍🏽 e `codice 💡⚡🔥` in [Link 🌐](https://accent.dev/🎉)\n";
        for i in 0..600 {
            if i % 20 == 0 {
                big_md.push_str(&format!("## Sezione {i} 🚀🌟🔥🦀\n\n"));
            }
            if i % 15 == 0 {
                big_md.push_str("| Colonna 🚀 | Valore 🦀 |\n| :--- | ---: |\n| Dato 👨‍👩‍👧‍👦 | 100% 💯 |\n\n");
            }
            big_md.push_str(emoji_cluster_line);
        }

        let mut buf = DocumentBuffer::from_str_content(&big_md);
        assert!(buf.len_chars() > 35_000);

        // Cold build and 1:1 verification
        let segs_cold = buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, 0..0);
        let reconstructed: String = segs_cold.iter().map(|s| s.as_str(&big_md)).collect();
        assert_eq!(reconstructed, big_md);

        // Warm cache must be sub-millisecond
        let t_warm = std::time::Instant::now();
        let segs_warm = buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, 0..0);
        let warm_ms = t_warm.elapsed().as_secs_f64() * 1000.0;
        assert!(Arc::ptr_eq(&segs_cold, &segs_warm));
        assert!(warm_ms < 1.0, "Warm cache took {warm_ms:.3} ms");

        // Stress 200 sequential edits with multi-scalar emojis + Undo/Redo roundtrip
        let initial_text = buf.text();
        for step in 0..100 {
            let pos = (step * 137) % buf.len_chars().max(1);
            buf.set_cursor(pos);
            buf.insert_text("🎯👨‍🔬🇮🇹");
        }
        let segs_after_edits =
            buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, buf.selection_byte_range());
        let after_doc = buf.as_str();
        let after_reconstructed: String =
            segs_after_edits.iter().map(|s| s.as_str(after_doc)).collect();
        assert_eq!(after_reconstructed, after_doc);

        // Undo all 100 emoji insertions and verify exact restoration
        while buf.undo() {}
        assert_eq!(buf.text(), initial_text);
    }

    #[test]
    fn test_crlf_paragraph_stats_and_plain_text_caret_stability() {
        let crlf_doc = "Primo paragrafo.\r\n\r\nSecondo paragrafo.\r\n\r\nTerzo paragrafo.";
        let lf_doc = "Primo paragrafo.\n\nSecondo paragrafo.\n\nTerzo paragrafo.";
        let s_crlf = DocumentStats::compute(crlf_doc);
        let s_lf = DocumentStats::compute(lf_doc);
        assert_eq!(s_crlf.paragraphs, 3);
        assert_eq!(s_crlf.words, s_lf.words);
        assert_eq!(s_crlf.sentences, s_lf.sentences);

        let md = "Riga neutra iniziale con molte parole semplici.\n\nSeconda riga con **grassetto** qui.\n";
        let buf = DocumentBuffer::from_str_content(md);
        assert!(!buf.has_color_emojis());
        let segs_a = buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, 0..0);
        let segs_b = buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, 12..12);
        let sig_a = buf.live_visibility_signature(MarkdownDialect::Gfm, true, 0..0);
        let sig_b = buf.live_visibility_signature(MarkdownDialect::Gfm, true, 12..12);
        assert_eq!(sig_a, 0);
        assert_eq!(sig_a, sig_b);
        assert!(
            Arc::ptr_eq(&segs_a, &segs_b),
            "Moving caret across plain text must reuse the exact Arc<Vec<EditorTextSegment>>"
        );

        let bold_off = md.find("grassetto").unwrap();
        let sig_bold =
            buf.live_visibility_signature(MarkdownDialect::Gfm, true, bold_off..bold_off);
        let sig_bold_next = buf.live_visibility_signature(
            MarkdownDialect::Gfm,
            true,
            (bold_off + 2)..(bold_off + 2),
        );
        assert_ne!(sig_bold, sig_a);
        assert_eq!(sig_bold, sig_bold_next);
        let segs_bold_1 =
            buf.build_editor_segments_arc(MarkdownDialect::Gfm, true, bold_off..bold_off);
        let segs_bold_2 = buf.build_editor_segments_arc(
            MarkdownDialect::Gfm,
            true,
            (bold_off + 2)..(bold_off + 2),
        );
        assert!(
            Arc::ptr_eq(&segs_bold_1, &segs_bold_2),
            "Moving caret within the same expanded span must reuse cached patched segments"
        );
    }

    #[test]
    fn test_set_text_owned_cow_undo_and_word_coalescing() {
        let base = "Inizio documento.\n".repeat(500);
        let mut buf = DocumentBuffer::from_str_content(&base);
        let mut working = base.clone();
        // Simulate typing "ciao mondo" one char at a time via set_text_owned
        for ch in "ciao mondo".chars() {
            working.push(ch);
            buf.set_text_owned(working.clone());
        }
        assert_eq!(buf.as_str(), working.as_str());
        // Undo once: "mondo" was coalesced into 1 word undo step!
        assert!(buf.undo());
        assert!(buf.as_str().ends_with("ciao "));
        // Undo until original base
        while buf.undo() {}
        assert_eq!(buf.as_str(), base.as_str());
    }
}
