use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Pdf,
    Html,
    Odt,
    Docx,
    Epub,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_to_html() {
        let html = render_markdown_to_html("# Title\n\nall you can ~~eat~~ drink\n");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<del>eat</del>"));
    }
}
