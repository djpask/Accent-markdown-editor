use resvg::{tiny_skia, usvg};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, OnceLock};

/// Visual theme for Mermaid diagram rendering, matching Accent's 3 themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MermaidTheme {
    Light,
    #[default]
    Sepia,
    Dark,
}

#[derive(Debug, Clone, Copy)]
struct DiagramColors {
    bg: &'static str,
    node_fill: &'static str,
    node_stroke: &'static str,
    diamond_fill: &'static str,
    diamond_stroke: &'static str,
    edge_stroke: &'static str,
    label_bg: &'static str,
    text_fg: &'static str,
}

impl MermaidTheme {
    fn palette(self) -> DiagramColors {
        match self {
            MermaidTheme::Light => DiagramColors {
                bg: "#F8F9FA",
                node_fill: "#FFFFFF",
                node_stroke: "#4A5568",
                diamond_fill: "#FFF5EB",
                diamond_stroke: "#D9621F",
                edge_stroke: "#4A5568",
                label_bg: "#F1F5F9",
                text_fg: "#2E3436",
            },
            MermaidTheme::Sepia => DiagramColors {
                bg: "#F3EBDD",
                node_fill: "#FBF7F0",
                node_stroke: "#7C5E30",
                diamond_fill: "#F6E4CC",
                diamond_stroke: "#C85A17",
                edge_stroke: "#6B4F24",
                label_bg: "#EFE4D2",
                text_fg: "#4F3915",
            },
            MermaidTheme::Dark => DiagramColors {
                bg: "#232327",
                node_fill: "#333339",
                node_stroke: "#94A3B8",
                diamond_fill: "#3E3127",
                diamond_stroke: "#ED8936",
                edge_stroke: "#A0AEC0",
                label_bg: "#2D2D33",
                text_fg: "#EEEEEE",
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedMermaidDiagram {
    pub content_hash: u64,
    pub theme: MermaidTheme,
    pub svg: String,
    pub width: u32,
    pub height: u32,
    pub rgba_pixels: Vec<u8>,
    pub png_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeShape {
    Rect,
    Rounded,
    Diamond,
    Circle,
}

#[derive(Debug, Clone)]
struct FlowNode {
    label: String,
    shape: NodeShape,
}

#[derive(Debug, Clone)]
struct FlowEdge {
    from: String,
    to: String,
    label: Option<String>,
    dashed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowDirection {
    TopDown,
    LeftRight,
}

#[derive(Debug, Clone)]
struct SeqMessage {
    from: String,
    to: String,
    text: String,
    dashed: bool,
}

static SHARED_FONTDB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();

fn shared_fontdb() -> Arc<usvg::fontdb::Database> {
    SHARED_FONTDB
        .get_or_init(|| {
            let mut db = usvg::fontdb::Database::new();
            db.load_system_fonts();
            for candidate in [
                "Adwaita Sans",
                "FreeSans",
                "Nimbus Sans",
                "Cantarell",
                "Noto Sans",
                "DejaVu Sans",
                "Liberation Sans",
            ] {
                let exists = db.faces().any(|f| {
                    f.families
                        .iter()
                        .any(|(name, _)| name.eq_ignore_ascii_case(candidate))
                });
                if exists {
                    db.set_sans_serif_family(candidate);
                    db.set_serif_family(candidate);
                    break;
                }
            }
            db.set_monospace_family("Nimbus Mono PS");
            Arc::new(db)
        })
        .clone()
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
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

fn parse_node_token(raw: &str) -> (String, Option<(String, NodeShape)>) {
    let s = raw.trim();
    if s.is_empty() {
        return (String::new(), None);
    }
    // Check for A((Label)), A{Label}, A([Label]), A(Label), A[Label]
    if let Some(pos) = s.find("((") {
        if let Some(end) = s.rfind("))") {
            let id = s[..pos].trim().to_string();
            let lbl = s[pos + 2..end].trim().trim_matches('"').to_string();
            return (id, Some((lbl, NodeShape::Circle)));
        }
    }
    if let Some(pos) = s.find('{') {
        if let Some(end) = s.rfind('}') {
            let id = s[..pos].trim().to_string();
            let lbl = s[pos + 1..end].trim().trim_matches('"').to_string();
            return (id, Some((lbl, NodeShape::Diamond)));
        }
    }
    if let Some(pos) = s.find('(') {
        if let Some(end) = s.rfind(')') {
            let id = s[..pos].trim().to_string();
            let inner = s[pos + 1..end].trim().trim_start_matches('[').trim_end_matches(']');
            let lbl = inner.trim().trim_matches('"').to_string();
            return (id, Some((lbl, NodeShape::Rounded)));
        }
    }
    if let Some(pos) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            let id = s[..pos].trim().to_string();
            let lbl = s[pos + 1..end].trim().trim_matches('"').to_string();
            return (id, Some((lbl, NodeShape::Rect)));
        }
    }
    let clean_id = s.trim_matches('"').to_string();
    (clean_id, None)
}

fn ensure_node(
    nodes: &mut Vec<FlowNode>,
    index_by_id: &mut HashMap<String, usize>,
    token: &str,
) -> String {
    let (id, meta) = parse_node_token(token);
    if id.is_empty() {
        return String::new();
    }
    if let Some(&idx) = index_by_id.get(&id) {
        if let Some((lbl, shape)) = meta {
            if !lbl.is_empty() {
                nodes[idx].label = lbl;
            }
            nodes[idx].shape = shape;
        }
    } else {
        let (label, shape) = meta.unwrap_or_else(|| (id.clone(), NodeShape::Rect));
        let idx = nodes.len();
        nodes.push(FlowNode { label, shape });
        index_by_id.insert(id.clone(), idx);
    }
    id
}

fn parse_edge_line(
    line: &str,
    nodes: &mut Vec<FlowNode>,
    index_by_id: &mut HashMap<String, usize>,
    edges: &mut Vec<FlowEdge>,
) {
    // Support arrows: `-.->`, `-->`, `==>`, `---` with optional `|label|`
    let arrows = ["-.->", "==>", "-->", "---", "->"];
    for arrow in arrows {
        if let Some(pos) = line.find(arrow) {
            let left_raw = line[..pos].trim();
            let mut right_raw = line[pos + arrow.len()..].trim();
            let mut label: Option<String> = None;
            let dashed = arrow.contains('.');

            // Handle `A -- label --> B`
            let left_token = if let Some(mid_dash) = left_raw.find(" -- ") {
                label = Some(left_raw[mid_dash + 4..].trim().trim_matches('"').to_string());
                left_raw[..mid_dash].trim()
            } else {
                left_raw
            };

            // Handle `-->|label| B`
            if right_raw.starts_with('|') {
                if let Some(end_pipe) = right_raw[1..].find('|') {
                    let lbl = right_raw[1..1 + end_pipe].trim().trim_matches('"');
                    if !lbl.is_empty() {
                        label = Some(lbl.to_string());
                    }
                    right_raw = right_raw[1 + end_pipe + 1..].trim();
                }
            }

            // Check if right_raw itself contains another chained arrow
            let mut next_arrow_pos: Option<usize> = None;
            for next_a in arrows {
                if let Some(p) = right_raw.find(next_a) {
                    next_arrow_pos = Some(next_arrow_pos.map_or(p, |cur| cur.min(p)));
                }
            }

            if let Some(np) = next_arrow_pos {
                let mid_node_raw = right_raw[..np].trim();
                let from_id = ensure_node(nodes, index_by_id, left_token);
                let to_id = ensure_node(nodes, index_by_id, mid_node_raw);
                if !from_id.is_empty() && !to_id.is_empty() {
                    edges.push(FlowEdge {
                        from: from_id,
                        to: to_id,
                        label,
                        dashed,
                    });
                }
                parse_edge_line(right_raw, nodes, index_by_id, edges);
            } else {
                let from_id = ensure_node(nodes, index_by_id, left_token);
                let to_id = ensure_node(nodes, index_by_id, right_raw);
                if !from_id.is_empty() && !to_id.is_empty() {
                    edges.push(FlowEdge {
                        from: from_id,
                        to: to_id,
                        label,
                        dashed,
                    });
                }
            }
            return;
        }
    }
    // Standalone node declaration line
    if !line.starts_with("subgraph") && line != "end" && !line.starts_with("classDef") && !line.starts_with("style ") {
        let _ = ensure_node(nodes, index_by_id, line);
    }
}

fn render_sequence_svg(source: &str, theme: MermaidTheme) -> (String, u32, u32) {
    let pal = theme.palette();
    let mut participants: Vec<(String, String)> = Vec::new();
    let mut part_idx: HashMap<String, usize> = HashMap::new();
    let mut messages: Vec<SeqMessage> = Vec::new();

    let ensure_part = |id: &str,
                       alias: Option<&str>,
                       participants: &mut Vec<(String, String)>,
                       part_idx: &mut HashMap<String, usize>| {
        let key = id.trim().to_string();
        if key.is_empty() {
            return;
        }
        if let Some(&idx) = part_idx.get(&key) {
            if let Some(a) = alias {
                participants[idx].1 = a.trim().trim_matches('"').to_string();
            }
        } else {
            let idx = participants.len();
            let label = alias
                .map(|a| a.trim().trim_matches('"').to_string())
                .unwrap_or_else(|| key.clone());
            participants.push((key.clone(), label));
            part_idx.insert(key, idx);
        }
    };

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("%%") || line.starts_with("sequenceDiagram") {
            continue;
        }
        if let Some(rest) = line
            .strip_prefix("participant ")
            .or_else(|| line.strip_prefix("actor "))
        {
            if let Some(as_pos) = rest.find(" as ") {
                let id = &rest[..as_pos];
                let alias = &rest[as_pos + 4..];
                ensure_part(id, Some(alias), &mut participants, &mut part_idx);
            } else {
                ensure_part(rest, None, &mut participants, &mut part_idx);
            }
            continue;
        }
        for (arrow, dashed) in [("-->>", true), ("->>", false), ("-->", true), ("->", false)] {
            if let Some(pos) = line.find(arrow) {
                let from = line[..pos].trim();
                let rhs = line[pos + arrow.len()..].trim();
                if let Some(colon) = rhs.find(':') {
                    let to = rhs[..colon].trim();
                    let text = rhs[colon + 1..].trim().to_string();
                    ensure_part(from, None, &mut participants, &mut part_idx);
                    ensure_part(to, None, &mut participants, &mut part_idx);
                    messages.push(SeqMessage {
                        from: from.to_string(),
                        to: to.to_string(),
                        text,
                        dashed,
                    });
                }
                break;
            }
        }
    }

    if participants.is_empty() {
        participants.push(("A".to_string(), "Client".to_string()));
        participants.push(("B".to_string(), "Server".to_string()));
    }

    let col_step = 180_u32;
    let pad_x = 90_u32;
    let width = (pad_x * 2 + (participants.len().saturating_sub(1) as u32) * col_step).max(380);
    let top_y = 36_u32;
    let box_h = 38_u32;
    let msg_step = 52_u32;
    let bottom_box_y = top_y + box_h + 28 + (messages.len() as u32) * msg_step;
    let height = bottom_box_y + box_h + 24;

    let mut svg = String::with_capacity(2048);
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
    ));
    svg.push_str(&format!(
        r#"<defs><marker id="seq-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 1 L 10 5 L 0 9 z" fill="{}"/></marker></defs>"#,
        pal.edge_stroke
    ));
    svg.push_str(&format!(
        r#"<rect x="2" y="2" width="{}" height="{}" rx="10" fill="{}" stroke="{}" stroke-width="1"/>"#,
        width - 4,
        height - 4,
        pal.bg,
        pal.node_stroke
    ));

    // Lifelines & Participant boxes
    for (idx, (_id, label)) in participants.iter().enumerate() {
        let cx = pad_x + (idx as u32) * col_step;
        let bx = cx.saturating_sub(64);
        let bw = 128_u32;
        let esc = xml_escape(label);
        svg.push_str(&format!(
            r#"<line x1="{cx}" y1="{}" x2="{cx}" y2="{bottom_box_y}" stroke="{}" stroke-width="1.5" stroke-dasharray="5,5"/>"#,
            top_y + box_h,
            pal.edge_stroke
        ));
        for by in [top_y, bottom_box_y] {
            svg.push_str(&format!(
                r#"<rect x="{bx}" y="{by}" width="{bw}" height="{box_h}" rx="6" fill="{}" stroke="{}" stroke-width="1.8"/>"#,
                pal.node_fill, pal.node_stroke
            ));
            svg.push_str(&format!(
                r#"<text x="{cx}" y="{}" font-family="FreeSans, Adwaita Sans, Nimbus Sans, Cantarell, DejaVu Sans, sans-serif" font-size="13" font-weight="bold" fill="{}" text-anchor="middle">{esc}</text>"#,
                by + 24,
                pal.text_fg
            ));
        }
    }

    // Messages
    for (m_idx, msg) in messages.iter().enumerate() {
        let i_from = part_idx.get(&msg.from).copied().unwrap_or(0);
        let i_to = part_idx.get(&msg.to).copied().unwrap_or(0);
        let x1 = pad_x + (i_from as u32) * col_step;
        let x2 = pad_x + (i_to as u32) * col_step;
        let y = top_y + box_h + 44 + (m_idx as u32) * msg_step;
        let dash_attr = if msg.dashed {
            r#" stroke-dasharray="6,4""#
        } else {
            ""
        };
        let esc_txt = xml_escape(&msg.text);
        let mid_x = (x1 + x2) / 2;
        svg.push_str(&format!(
            r#"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke="{}" stroke-width="1.8"{dash_attr} marker-end="url(#seq-arrow)"/>"#,
            pal.edge_stroke
        ));
        svg.push_str(&format!(
            r#"<text x="{mid_x}" y="{}" font-family="FreeSans, Adwaita Sans, Nimbus Sans, Cantarell, DejaVu Sans, sans-serif" font-size="12" fill="{}" text-anchor="middle">{esc_txt}</text>"#,
            y - 8,
            pal.text_fg
        ));
    }

    svg.push_str("</svg>");
    (svg, width, height)
}

/// Converts Mermaid source text (`flowchart`/`graph` TD/LR or `sequenceDiagram`) into a crisp SVG string.
pub fn render_mermaid_to_svg(source: &str, theme: MermaidTheme) -> (String, u32, u32) {
    let trimmed = source.trim();
    if trimmed.starts_with("sequenceDiagram") {
        return render_sequence_svg(trimmed, theme);
    }

    let pal = theme.palette();
    let mut dir = FlowDirection::TopDown;
    let mut nodes: Vec<FlowNode> = Vec::new();
    let mut index_by_id: HashMap<String, usize> = HashMap::new();
    let mut edges: Vec<FlowEdge> = Vec::new();

    for (line_idx, raw_line) in trimmed.lines().enumerate() {
        let line = raw_line.trim().trim_end_matches(';').trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        if line_idx == 0
            && (line.starts_with("flowchart") || line.starts_with("graph"))
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&d) = parts.get(1) {
                if d.eq_ignore_ascii_case("LR") || d.eq_ignore_ascii_case("RL") {
                    dir = FlowDirection::LeftRight;
                }
            }
            continue;
        }
        parse_edge_line(line, &mut nodes, &mut index_by_id, &mut edges);
    }

    if nodes.is_empty() {
        nodes.push(FlowNode {
            label: if trimmed.is_empty() {
                "Mermaid Diagram".to_string()
            } else {
                trimmed.lines().next().unwrap_or("Diagram").to_string()
            },
            shape: NodeShape::Rounded,
        });
    }

    // Assign layered ranks via longest-path BFS from roots
    let n = nodes.len();
    let mut indegree = vec![0usize; n];
    let mut adj = vec![Vec::<usize>::new(); n];
    for e in &edges {
        if let (Some(&u), Some(&v)) = (index_by_id.get(&e.from), index_by_id.get(&e.to)) {
            if u != v {
                adj[u].push(v);
                indegree[v] += 1;
            }
        }
    }

    let mut rank = vec![0usize; n];
    let mut visited = vec![false; n];
    let mut queue: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    if queue.is_empty() {
        queue.push(0);
    }
    let mut head = 0;
    while head < queue.len() {
        let u = queue[head];
        head += 1;
        visited[u] = true;
        for &v in &adj[u] {
            if rank[v] < rank[u] + 1 && rank[u] + 1 < n {
                rank[v] = rank[u] + 1;
                queue.push(v);
            }
        }
    }
    for i in 0..n {
        if !visited[i] {
            rank[i] = i;
        }
    }

    let mut layers: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (idx, &r) in rank.iter().enumerate() {
        layers.entry(r).or_default().push(idx);
    }

    let num_layers = layers.len().max(1);
    let max_per_layer = layers.values().map(|v| v.len()).max().unwrap_or(1).max(1);

    let (width, height, node_coords) = match dir {
        FlowDirection::TopDown => {
            let col_w = 210_u32;
            let row_h = 104_u32;
            let w = (max_per_layer as u32 * col_w + 80).max(420);
            let h = (num_layers as u32 * row_h + 48).max(160);
            let mut coords = vec![(0.0_f32, 0.0_f32, 148.0_f32, 44.0_f32); n];
            for (l_idx, (_r, members)) in layers.iter().enumerate() {
                let cy = 54.0 + (l_idx as f32) * (row_h as f32);
                let total_span = (members.len() as f32) * (col_w as f32);
                let start_x = (w as f32 - total_span) * 0.5 + (col_w as f32) * 0.5;
                for (m_pos, &node_idx) in members.iter().enumerate() {
                    let cx = start_x + (m_pos as f32) * (col_w as f32);
                    let char_len = nodes[node_idx].label.chars().count() as f32;
                    let nw = (char_len * 8.4 + 38.0).clamp(116.0, 190.0);
                    let nh = if nodes[node_idx].shape == NodeShape::Diamond {
                        54.0
                    } else {
                        42.0
                    };
                    coords[node_idx] = (cx, cy, nw, nh);
                }
            }
            (w, h, coords)
        }
        FlowDirection::LeftRight => {
            let col_w = 210_u32;
            let row_h = 96_u32;
            let w = (num_layers as u32 * col_w + 80).max(460);
            let h = (max_per_layer as u32 * row_h + 56).max(150);
            let mut coords = vec![(0.0_f32, 0.0_f32, 148.0_f32, 44.0_f32); n];
            for (l_idx, (_r, members)) in layers.iter().enumerate() {
                let cx = 110.0 + (l_idx as f32) * (col_w as f32);
                let total_span = (members.len() as f32) * (row_h as f32);
                let start_y = (h as f32 - total_span) * 0.5 + (row_h as f32) * 0.5;
                for (m_pos, &node_idx) in members.iter().enumerate() {
                    let cy = start_y + (m_pos as f32) * (row_h as f32);
                    let char_len = nodes[node_idx].label.chars().count() as f32;
                    let nw = (char_len * 8.4 + 36.0).clamp(112.0, 184.0);
                    let nh = if nodes[node_idx].shape == NodeShape::Diamond {
                        54.0
                    } else {
                        42.0
                    };
                    coords[node_idx] = (cx, cy, nw, nh);
                }
            }
            (w, h, coords)
        }
    };

    let mut svg = String::with_capacity(4096);
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
    ));
    svg.push_str(&format!(
        r#"<defs><marker id="flow-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 1 L 10 5 L 0 9 z" fill="{}"/></marker></defs>"#,
        pal.edge_stroke
    ));
    svg.push_str(&format!(
        r#"<rect x="2" y="2" width="{}" height="{}" rx="10" fill="{}" stroke="{}" stroke-width="1"/>"#,
        width - 4,
        height - 4,
        pal.bg,
        pal.node_stroke
    ));

    // Render edges first (behind nodes)
    for edge in &edges {
        let Some(&u) = index_by_id.get(&edge.from) else {
            continue;
        };
        let Some(&v) = index_by_id.get(&edge.to) else {
            continue;
        };
        let (x1_c, y1_c, w1, h1) = node_coords[u];
        let (x2_c, y2_c, w2, h2) = node_coords[v];

        let dx = x2_c - x1_c;
        let dy = y2_c - y1_c;
        let (x1, y1, x2, y2) = if dy.abs() >= dx.abs() {
            let s = if dy >= 0.0 { 1.0 } else { -1.0 };
            (
                x1_c,
                y1_c + s * (h1 * 0.5),
                x2_c,
                y2_c - s * (h2 * 0.5 + 3.0),
            )
        } else {
            let s = if dx >= 0.0 { 1.0 } else { -1.0 };
            (
                x1_c + s * (w1 * 0.5),
                y1_c,
                x2_c - s * (w2 * 0.5 + 3.0),
                y2_c,
            )
        };

        let dash_attr = if edge.dashed {
            r#" stroke-dasharray="5,4""#
        } else {
            ""
        };
        let mx = (x1 + x2) * 0.5;
        let my = (y1 + y2) * 0.5;
        svg.push_str(&format!(
            r#"<path d="M {x1:.1} {y1:.1} L {x2:.1} {y2:.1}" fill="none" stroke="{}" stroke-width="2"{dash_attr} marker-end="url(#flow-arrow)"/>"#,
            pal.edge_stroke
        ));

        if let Some(ref lbl) = edge.label {
            let esc = xml_escape(lbl);
            let lw = (lbl.chars().count() as f32 * 7.2 + 14.0).max(28.0);
            let lh = 20.0_f32;
            svg.push_str(&format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{lw:.1}" height="{lh:.1}" rx="4" fill="{}" stroke="{}" stroke-width="1"/>"#,
                mx - lw * 0.5,
                my - lh * 0.5,
                pal.label_bg,
                pal.node_stroke
            ));
            svg.push_str(&format!(
                r#"<text x="{mx:.1}" y="{:.1}" font-family="FreeSans, Adwaita Sans, Nimbus Sans, Cantarell, DejaVu Sans, sans-serif" font-size="11" font-weight="bold" fill="{}" text-anchor="middle">{esc}</text>"#,
                my + 4.0,
                pal.text_fg
            ));
        }
    }

    // Render nodes
    for (idx, node) in nodes.iter().enumerate() {
        let (cx, cy, nw, nh) = node_coords[idx];
        let x = cx - nw * 0.5;
        let y = cy - nh * 0.5;
        let esc_label = xml_escape(&node.label);

        match node.shape {
            NodeShape::Rect => {
                svg.push_str(&format!(
                    r#"<rect x="{x:.1}" y="{y:.1}" width="{nw:.1}" height="{nh:.1}" rx="4" fill="{}" stroke="{}" stroke-width="2"/>"#,
                    pal.node_fill, pal.node_stroke
                ));
            }
            NodeShape::Rounded => {
                svg.push_str(&format!(
                    r#"<rect x="{x:.1}" y="{y:.1}" width="{nw:.1}" height="{nh:.1}" rx="18" fill="{}" stroke="{}" stroke-width="2"/>"#,
                    pal.node_fill, pal.node_stroke
                ));
            }
            NodeShape::Circle => {
                let r = (nw.max(nh) * 0.45).max(24.0);
                svg.push_str(&format!(
                    r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{}" stroke="{}" stroke-width="2"/>"#,
                    pal.node_fill, pal.node_stroke
                ));
            }
            NodeShape::Diamond => {
                let hw = nw * 0.56;
                let hh = nh * 0.62;
                svg.push_str(&format!(
                    r#"<polygon points="{cx:.1},{:.1} {:.1},{cy:.1} {cx:.1},{:.1} {:.1},{cy:.1}" fill="{}" stroke="{}" stroke-width="2"/>"#,
                    cy - hh,
                    cx + hw,
                    cy + hh,
                    cx - hw,
                    pal.diamond_fill,
                    pal.diamond_stroke
                ));
            }
        }

        svg.push_str(&format!(
            r#"<text x="{cx:.1}" y="{:.1}" font-family="FreeSans, Adwaita Sans, Nimbus Sans, Cantarell, DejaVu Sans, sans-serif" font-size="13" font-weight="bold" fill="{}" text-anchor="middle">{esc_label}</text>"#,
            cy + 4.5,
            pal.text_fg
        ));
    }

    svg.push_str("</svg>");
    (svg, width, height)
}

/// Rasterizes an SVG string into unpremultiplied RGBA8 pixels and encoded PNG bytes via `resvg`.
pub fn rasterize_svg(svg: &str, scale: f32) -> Option<(u32, u32, Vec<u8>, Vec<u8>)> {
    let mut opts = usvg::Options::default();
    opts.fontdb = shared_fontdb();
    let tree = usvg::Tree::from_str(svg, &opts).ok()?;
    let size = tree.size().to_int_size();
    let px_w = ((size.width() as f32) * scale).round().max(1.0) as u32;
    let px_h = ((size.height() as f32) * scale).round().max(1.0) as u32;

    let mut pixmap = tiny_skia::Pixmap::new(px_w, px_h)?;
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let png_bytes = pixmap.encode_png().ok()?;
    // Convert premultiplied RGBA from tiny-skia to unpremultiplied RGBA for egui::ColorImage
    let mut rgba = Vec::with_capacity((px_w * px_h * 4) as usize);
    for px in pixmap.pixels() {
        let c = px.demultiply();
        rgba.push(c.red());
        rgba.push(c.green());
        rgba.push(c.blue());
        rgba.push(c.alpha());
    }

    Some((px_w, px_h, rgba, png_bytes))
}

#[derive(Debug, Default, Clone)]
pub struct MermaidCache {
    entries: HashMap<(u64, MermaidTheme), Arc<RenderedMermaidDiagram>>,
}

impl MermaidCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hash_source(source: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in source.trim().as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub fn get_or_render(&mut self, source: &str) -> RenderedMermaidDiagram {
        self.get_or_render_themed(source, MermaidTheme::Sepia)
            .as_ref()
            .clone()
    }

    pub fn get_or_render_themed(
        &mut self,
        source: &str,
        theme: MermaidTheme,
    ) -> Arc<RenderedMermaidDiagram> {
        let key = (Self::hash_source(source), theme);
        self.entries
            .entry(key)
            .or_insert_with(|| {
                let (svg, logical_w, logical_h) = render_mermaid_to_svg(source, theme);
                let (width, height, rgba_pixels, png_bytes) =
                    rasterize_svg(&svg, 1.5).unwrap_or_else(|| {
                        (
                            logical_w,
                            logical_h,
                            vec![240; (logical_w * logical_h * 4) as usize],
                            Vec::new(),
                        )
                    });
                Arc::new(RenderedMermaidDiagram {
                    content_hash: key.0,
                    theme,
                    svg,
                    width,
                    height,
                    rgba_pixels,
                    png_bytes,
                })
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mermaid_cache_deterministic() {
        let mut cache = MermaidCache::new();
        let d1 = cache.get_or_render("flowchart LR\n  A[Input] --> B{Valid?} -->|Yes| C(Output)");
        let d2 = cache.get_or_render("flowchart LR\n  A[Input] --> B{Valid?} -->|Yes| C(Output)");
        assert_eq!(d1.content_hash, d2.content_hash);
        assert!(d1.svg.contains("<svg"));
        assert!(d1.svg.contains("Input"));
        assert!(d1.svg.contains("Valid?"));
        assert!(d1.svg.contains("Output"));
        assert!(
            d1.png_bytes.len() > 1000,
            "Expected valid PNG > 1000 bytes, got {}",
            d1.png_bytes.len()
        );
        assert_eq!(&d1.png_bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(
            d1.rgba_pixels.len(),
            (d1.width * d1.height * 4) as usize
        );
    }

    #[test]
    fn test_mermaid_sequence_diagram_rendering() {
        let mut cache = MermaidCache::new();
        let seq = "sequenceDiagram\n  participant U as Utente\n  participant A as Accent\n  U->>A: Digita Markdown\n  A-->>U: Live Preview + WAL";
        let rendered = cache.get_or_render_themed(seq, MermaidTheme::Dark);
        assert!(rendered.svg.contains("Utente"));
        assert!(rendered.svg.contains("Live Preview + WAL"));
        assert!(rendered.png_bytes.len() > 1000);
    }
}
