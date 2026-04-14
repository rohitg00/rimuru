//! Tree-sitter signature indexer (#36).
//!
//! Reduces code-reading tokens by returning only the parts of a
//! source file an agent actually needs for navigation: function
//! signatures, struct / enum / trait headers, type aliases, impl
//! blocks, use statements. Function bodies, closures, and literal
//! constants are stripped.
//!
//! This PR ships a Rust-only MVP backed by tree-sitter-rust. The
//! `LanguageDef` trait lets us plug in tree-sitter grammars for
//! TypeScript, Python, Go, and Java without touching the dispatch
//! layer — each language just lands its own implementation and
//! registers itself in `languages_for_extension`.
//!
//! Three iii functions are exposed:
//!
//! - `rimuru.indexer.outline`        top-level items (no bodies, no docstrings)
//! - `rimuru.indexer.signatures`     every named signature in the file
//! - `rimuru.indexer.extract_symbol` one symbol by name with its immediate context
//!
//! Unsupported languages fall back to returning the full file with
//! `strategy: "full"` so callers always get a usable answer, and a
//! `note` field tells them the indexer isn't doing anything clever.

use std::path::Path;

use iii_sdk::{III, RegisterFunctionMessage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tree_sitter::{Node, Parser};

use super::sysutil::{api_response, extract_input, require_str};
use crate::state::StateKV;

// ---------- language plugin trait ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub kind: String, // fn / struct / enum / trait / type / impl / use / const / static
    pub name: String,
    pub signature: String,
    pub line: usize,
}

/// A language plugin drives the extraction. New languages implement
/// this trait and plug into `languages_for_extension`. The MVP only
/// ships a Rust implementation; leaving the trait in place so the
/// next PR to add TypeScript / Python / Go doesn't have to refactor.
trait LanguageDef {
    fn name(&self) -> &'static str;
    fn parser(&self) -> Parser;
    /// Node kinds that represent "top-level items" — what shows up
    /// in an outline view and what the signatures extractor walks.
    fn is_top_level_item(&self, node: &Node) -> bool;
    /// Render a signature text snippet for an item. Should strip
    /// function bodies, nested constants, attribute bodies, etc.
    fn render_signature(&self, node: &Node, source: &str) -> Option<Signature>;
}

// ---------- Rust implementation ----------

struct RustLanguage;

impl LanguageDef for RustLanguage {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn parser(&self) -> Parser {
        let mut p = Parser::new();
        p.set_language(&tree_sitter_rust::language())
            .expect("tree-sitter-rust language is compatible");
        p
    }

    fn is_top_level_item(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "function_item"
                | "struct_item"
                | "enum_item"
                | "trait_item"
                | "impl_item"
                | "type_item"
                | "const_item"
                | "static_item"
                | "mod_item"
                | "use_declaration"
                | "macro_definition"
        )
    }

    fn render_signature(&self, node: &Node, source: &str) -> Option<Signature> {
        let node_kind = node.kind();

        let (kind, has_body) = match node_kind {
            "function_item" => ("fn", true),
            "struct_item" => ("struct", true),
            "enum_item" => ("enum", true),
            "trait_item" => ("trait", true),
            "impl_item" => ("impl", true),
            "mod_item" => ("mod", true),
            "type_item" => ("type", false),
            "const_item" => ("const", false),
            "static_item" => ("static", false),
            "use_declaration" => ("use", false),
            "macro_definition" => ("macro", false),
            _ => return None,
        };

        let name = match node_kind {
            // Impls don't have a plain name; use the target type.
            "impl_item" => node
                .child_by_field_name("type")
                .and_then(|n| node_text(&n, source))
                .unwrap_or_else(|| "impl".into()),
            "use_declaration" => node
                .child_by_field_name("argument")
                .and_then(|n| node_text(&n, source))
                .unwrap_or_else(|| "use".into()),
            _ => node
                .child_by_field_name("name")
                .and_then(|n| node_text(&n, source))
                .unwrap_or_else(|| "?".into()),
        };

        // Body-bearing items (fn, struct, enum, trait, impl, mod) have
        // their body stripped. Everything else is used verbatim.
        let signature = if has_body {
            strip_body(node, source, "body")
        } else {
            node_text(node, source).unwrap_or_default()
        };

        Some(Signature {
            kind: kind.into(),
            name,
            signature: signature.trim().to_string(),
            line: node.start_position().row + 1,
        })
    }
}

fn node_text(node: &Node, source: &str) -> Option<String> {
    node.utf8_text(source.as_bytes())
        .ok()
        .map(|s| s.to_string())
}

/// Return the full node text minus the named `body_field` child,
/// collapsed onto one line and suffixed with `{...}` so a multi-line
/// signature reads naturally in the outline.
fn strip_body(node: &Node, source: &str, body_field: &str) -> String {
    let Some(full) = node_text(node, source) else {
        return String::new();
    };
    let Some(body) = node.child_by_field_name(body_field) else {
        return full;
    };
    let offset = body.start_byte().saturating_sub(node.start_byte());
    if offset == 0 || offset > full.len() {
        return full;
    }
    let header: String = full[..offset]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    format!("{} {{...}}", header)
}

// ---------- language dispatch ----------

fn language_for_extension(ext: &str) -> Option<Box<dyn LanguageDef>> {
    match ext {
        "rs" => Some(Box::new(RustLanguage)),
        // Future: "ts" => Box::new(TypeScriptLanguage), etc.
        _ => None,
    }
}

fn extract_ext(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

// ---------- extraction ----------

fn collect_signatures(lang: &dyn LanguageDef, source: &str) -> Vec<Signature> {
    let mut parser = lang.parser();
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let root = tree.root_node();
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if lang.is_top_level_item(&node)
            && let Some(sig) = lang.render_signature(&node, source)
        {
            out.push(sig);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    out.sort_by_key(|s| s.line);
    out
}

fn extract_symbol(lang: &dyn LanguageDef, source: &str, symbol_name: &str) -> Option<String> {
    let mut parser = lang.parser();
    let tree = parser.parse(source, None)?;
    let root = tree.root_node();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if lang.is_top_level_item(&node)
            && let Some(sig) = lang.render_signature(&node, source)
            && sig.name == symbol_name
        {
            return node_text(&node, source);
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                stack.push(child);
            }
        }
    }
    None
}

// ---------- iii functions ----------

pub fn register(iii: &III, kv: &StateKV) {
    register_outline(iii, kv);
    register_signatures(iii, kv);
    register_extract_symbol(iii, kv);
}

fn read_path(input: &Value) -> Result<(String, String), iii_sdk::IIIError> {
    let path = require_str(input, "path")?;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| iii_sdk::IIIError::Handler(format!("read {}: {}", path, e)))?;
    Ok((path, content))
}

fn register_outline(iii: &III, _kv: &StateKV) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.indexer.outline".to_string()),
        move |input: Value| async move {
            let input = extract_input(input);
            let (path, content) = read_path(&input)?;

            let ext = extract_ext(&path);
            let Some(lang) = ext.as_deref().and_then(language_for_extension) else {
                return Ok(api_response(json!({
                    "path": path,
                    "strategy": "full",
                    "content": content,
                    "note": format!(
                        "no tree-sitter plugin for extension {:?}; returning full content",
                        ext.unwrap_or_default()
                    ),
                })));
            };

            let sigs = collect_signatures(&*lang, &content);
            // Outline = signatures grouped into a compact text blob
            // that mirrors the file's top-level shape. One item per line.
            let outline: String = sigs
                .iter()
                .map(|s| format!("{:>4}  {}", s.line, s.signature))
                .collect::<Vec<_>>()
                .join("\n");

            Ok(api_response(json!({
                "path": path,
                "language": lang.name(),
                "strategy": "outline",
                "item_count": sigs.len(),
                "outline": outline,
                "original_bytes": content.len(),
                "outline_bytes": outline.len(),
                "reduction_percent": if content.is_empty() {
                    0.0
                } else {
                    (1.0 - (outline.len() as f64 / content.len() as f64)) * 100.0
                },
            })))
        },
    );
}

fn register_signatures(iii: &III, _kv: &StateKV) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.indexer.signatures".to_string()),
        move |input: Value| async move {
            let input = extract_input(input);
            let (path, content) = read_path(&input)?;

            let ext = extract_ext(&path);
            let Some(lang) = ext.as_deref().and_then(language_for_extension) else {
                return Ok(api_response(json!({
                    "path": path,
                    "strategy": "full",
                    "content": content,
                    "signatures": [],
                    "note": "no tree-sitter plugin for this extension",
                })));
            };

            let sigs = collect_signatures(&*lang, &content);
            Ok(api_response(json!({
                "path": path,
                "language": lang.name(),
                "strategy": "signatures",
                "signatures": sigs,
                "count": sigs.len(),
                "original_bytes": content.len(),
            })))
        },
    );
}

fn register_extract_symbol(iii: &III, _kv: &StateKV) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.indexer.extract_symbol".to_string()),
        move |input: Value| async move {
            let input = extract_input(input);
            let (path, content) = read_path(&input)?;
            let symbol = require_str(&input, "symbol")?;

            let ext = extract_ext(&path);
            let Some(lang) = ext.as_deref().and_then(language_for_extension) else {
                return Ok(api_response(json!({
                    "path": path,
                    "symbol": symbol,
                    "strategy": "full",
                    "content": content,
                    "note": "no tree-sitter plugin; returning full file",
                })));
            };

            match extract_symbol(&*lang, &content, &symbol) {
                Some(text) => Ok(api_response(json!({
                    "path": path,
                    "language": lang.name(),
                    "symbol": symbol,
                    "strategy": "symbol",
                    "content": text,
                }))),
                None => Ok(api_response(json!({
                    "path": path,
                    "language": lang.name(),
                    "symbol": symbol,
                    "strategy": "not_found",
                    "content": "",
                }))),
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
use std::collections::HashMap;

/// A canonical thing.
pub struct Widget {
    pub name: String,
    count: u32,
}

impl Widget {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            count: 0,
        }
    }

    fn bump(&mut self) {
        self.count += 1;
        for _ in 0..10 {
            println!("bump {}", self.count);
        }
    }
}

pub enum Flavor {
    Sweet,
    Sour,
    Bitter,
}

pub trait Describe {
    fn describe(&self) -> String;
}

type Callback = fn(&str) -> u32;

const MAX: u32 = 42;
"#;

    #[test]
    fn rust_signatures_strip_bodies() {
        let lang = RustLanguage;
        let sigs = collect_signatures(&lang, SAMPLE);

        let names: Vec<&str> = sigs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Widget"), "missing Widget: {:?}", names);
        assert!(names.contains(&"new"), "missing new");
        assert!(names.contains(&"bump"), "missing bump");
        assert!(names.contains(&"Flavor"), "missing Flavor");
        assert!(names.contains(&"Describe"), "missing Describe");
        assert!(names.contains(&"Callback"), "missing Callback");
        assert!(names.contains(&"MAX"), "missing MAX");

        // Body-bearing signatures must end with the "{...}" placeholder
        // and must not contain the body's inner content.
        let bump = sigs.iter().find(|s| s.name == "bump").unwrap();
        assert!(
            bump.signature.contains("{...}"),
            "bump body not stripped: {:?}",
            bump.signature
        );
        assert!(
            !bump.signature.contains("count += 1"),
            "bump still has body: {:?}",
            bump.signature
        );

        // Top-level-only collection: `new` and `bump` are inside `impl Widget`,
        // so they should appear AND the impl block itself should too.
        assert!(sigs.iter().any(|s| s.kind == "impl"));
    }

    #[test]
    fn rust_extract_symbol_returns_full_body() {
        let lang = RustLanguage;
        let text = extract_symbol(&lang, SAMPLE, "bump").expect("found bump");
        // Signature must be part of what we returned, not just body text.
        // "bump {" inside the println! would satisfy a weaker check and
        // hide a regression where extract_symbol only returned the body.
        assert!(
            text.contains("fn bump(&mut self)"),
            "missing function signature: {}",
            text
        );
        assert!(
            text.contains("count += 1"),
            "missing function body: {}",
            text
        );
    }

    #[test]
    fn rust_extract_symbol_missing_returns_none() {
        let lang = RustLanguage;
        assert!(extract_symbol(&lang, SAMPLE, "does_not_exist").is_none());
    }

    #[test]
    fn unsupported_extension_returns_none() {
        assert!(language_for_extension("py").is_none());
        assert!(language_for_extension("rs").is_some());
    }

    #[test]
    fn outline_is_smaller_than_source() {
        let lang = RustLanguage;
        let sigs = collect_signatures(&lang, SAMPLE);
        let outline: String = sigs
            .iter()
            .map(|s| format!("{:>4}  {}", s.line, s.signature))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            outline.len() < SAMPLE.len(),
            "outline should be smaller than source: {} vs {}",
            outline.len(),
            SAMPLE.len()
        );
    }
}
