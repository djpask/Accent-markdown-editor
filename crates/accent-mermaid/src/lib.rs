use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedMermaidDiagram {
    pub content_hash: u64,
    pub svg: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, Clone)]
pub struct MermaidCache {
    entries: HashMap<u64, RenderedMermaidDiagram>,
}

impl MermaidCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hash_source(source: &str) -> u64 {
        // FNV-1a 64-bit deterministic hash
        let mut h: u64 = 0xcbf29ce484222325;
        for b in source.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub fn get_or_render(&mut self, source: &str) -> RenderedMermaidDiagram {
        let key = Self::hash_source(source);
        self.entries
            .entry(key)
            .or_insert_with(|| RenderedMermaidDiagram {
                content_hash: key,
                svg: format!(
                    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"320\" height=\"120\"><!-- {key:x} --></svg>"
                ),
                width: 320,
                height: 120,
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
        let d1 = cache.get_or_render("flowchart LR\n  A --> B");
        let d2 = cache.get_or_render("flowchart LR\n  A --> B");
        assert_eq!(d1.content_hash, d2.content_hash);
    }
}
