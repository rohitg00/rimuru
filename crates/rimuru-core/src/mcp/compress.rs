use serde_json::Value;

pub enum CompressionStrategy {
    Auto,
    Truncate,
    Summarize,
    JsonPaths,
    ErrorsOnly,
    TreeView,
}

pub struct CompressionResult {
    pub compressed: Value,
    pub original_tokens: u64,
    pub compressed_tokens: u64,
    pub strategy_used: String,
    pub savings_percent: f64,
}

fn estimate_tokens(value: &Value) -> u64 {
    serde_json::to_string(value)
        .map(|s| s.len() as u64 / 4)
        .unwrap_or(0)
}

fn estimate_tokens_str(s: &str) -> u64 {
    s.len() as u64 / 4
}

pub fn compress(input: &Value, strategy: CompressionStrategy, max_tokens: u64) -> CompressionResult {
    let original_tokens = estimate_tokens(input);

    if original_tokens <= max_tokens {
        return CompressionResult {
            compressed: input.clone(),
            original_tokens,
            compressed_tokens: original_tokens,
            strategy_used: "none".to_string(),
            savings_percent: 0.0,
        };
    }

    let strategy = match strategy {
        CompressionStrategy::Auto => pick_strategy(input, original_tokens),
        other => other,
    };

    let (compressed, strategy_name) = match strategy {
        CompressionStrategy::Truncate => (truncate(input, max_tokens), "truncate"),
        CompressionStrategy::Summarize => (summarize(input, max_tokens), "summarize"),
        CompressionStrategy::JsonPaths => (json_paths(input, max_tokens), "json_paths"),
        CompressionStrategy::ErrorsOnly => (errors_only(input, max_tokens), "errors_only"),
        CompressionStrategy::TreeView => (tree_view(input, max_tokens), "tree_view"),
        CompressionStrategy::Auto => unreachable!(),
    };

    let compressed_tokens = estimate_tokens(&compressed);

    if original_tokens > 0 && compressed_tokens < original_tokens / 10 {
        let fallback = truncate(input, max_tokens);
        let fallback_tokens = estimate_tokens(&fallback);
        let savings = if original_tokens > 0 {
            (1.0 - fallback_tokens as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };
        return CompressionResult {
            compressed: fallback,
            original_tokens,
            compressed_tokens: fallback_tokens,
            strategy_used: "truncate_fallback".to_string(),
            savings_percent: savings,
        };
    }

    let savings = if original_tokens > 0 {
        (1.0 - compressed_tokens as f64 / original_tokens as f64) * 100.0
    } else {
        0.0
    };

    CompressionResult {
        compressed,
        original_tokens,
        compressed_tokens,
        strategy_used: strategy_name.to_string(),
        savings_percent: savings,
    }
}

fn pick_strategy(input: &Value, tokens: u64) -> CompressionStrategy {
    match input {
        Value::Object(map) if map.len() > 50 || tokens > 5000 => CompressionStrategy::JsonPaths,
        Value::Array(arr) if arr.len() > 50 || tokens > 5000 => CompressionStrategy::JsonPaths,
        Value::String(s) => pick_string_strategy(s),
        Value::Object(_) | Value::Array(_) => {
            let s = serde_json::to_string_pretty(input).unwrap_or_default();
            if has_error_lines(&s) {
                CompressionStrategy::ErrorsOnly
            } else {
                CompressionStrategy::JsonPaths
            }
        }
        _ => CompressionStrategy::Truncate,
    }
}

fn pick_string_strategy(s: &str) -> CompressionStrategy {
    if has_error_lines(s) {
        CompressionStrategy::ErrorsOnly
    } else if looks_like_file_listing(s) {
        CompressionStrategy::TreeView
    } else {
        CompressionStrategy::Summarize
    }
}

fn has_error_lines(s: &str) -> bool {
    let error_count = s
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("error")
                || lower.contains("fail")
                || lower.contains("panic")
                || lower.contains("traceback")
                || lower.contains("warning")
                || lower.contains("warn")
        })
        .count();
    error_count >= 2
}

fn looks_like_file_listing(s: &str) -> bool {
    let path_lines = s
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.contains('/')
                && !trimmed.contains("http")
                && (trimmed.ends_with('/') || trimmed.contains('.'))
        })
        .count();
    let total_lines = s.lines().count().max(1);
    path_lines as f64 / total_lines as f64 > 0.5
}

fn truncate(input: &Value, max_tokens: u64) -> Value {
    let s = match input {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(input).unwrap_or_default(),
    };

    let max_chars = (max_tokens * 3) as usize;
    if s.len() <= max_chars {
        return Value::String(s);
    }

    let total_tokens = estimate_tokens_str(&s);
    let truncated = &s[..max_chars.min(s.len())];
    let kept_tokens = estimate_tokens_str(truncated);

    Value::String(format!(
        "{}\n... [truncated, showing ~{}/{} tokens]",
        truncated, kept_tokens, total_tokens
    ))
}

fn summarize(input: &Value, _max_tokens: u64) -> Value {
    let s = match input {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(input).unwrap_or_default(),
    };

    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= 15 {
        return Value::String(s);
    }

    let head_count = 10;
    let tail_count = 5;
    let removed = lines.len() - head_count - tail_count;

    let mut result = String::new();
    for line in &lines[..head_count] {
        result.push_str(line);
        result.push('\n');
    }
    result.push_str(&format!("\n... [{} lines removed] ...\n\n", removed));
    for line in &lines[lines.len() - tail_count..] {
        result.push_str(line);
        result.push('\n');
    }

    Value::String(result)
}

fn json_paths(input: &Value, _max_tokens: u64) -> Value {
    compress_value(input, 0)
}

fn compress_value(value: &Value, depth: usize) -> Value {
    match value {
        Value::Object(map) => {
            if depth > 3 {
                let keys: Vec<String> = map.keys().cloned().collect();
                return serde_json::json!({
                    "__depth": depth,
                    "__keys": keys
                });
            }
            let mut result = serde_json::Map::new();
            for (k, v) in map {
                result.insert(k.clone(), compress_value(v, depth + 1));
            }
            Value::Object(result)
        }
        Value::Array(arr) => {
            if arr.len() <= 10 {
                Value::Array(arr.iter().map(|v| compress_value(v, depth + 1)).collect())
            } else {
                let mut result: Vec<Value> = arr[..3]
                    .iter()
                    .map(|v| compress_value(v, depth + 1))
                    .collect();
                result.push(serde_json::json!({
                    "__truncated": format!("{} items removed", arr.len() - 5)
                }));
                for v in &arr[arr.len() - 2..] {
                    result.push(compress_value(v, depth + 1));
                }
                Value::Array(result)
            }
        }
        Value::String(s) => {
            if s.len() > 200 {
                Value::String(format!("{}...", &s[..100.min(s.len())]))
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

fn errors_only(input: &Value, _max_tokens: u64) -> Value {
    let s = match input {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(input).unwrap_or_default(),
    };

    let lines: Vec<&str> = s.lines().collect();
    let mut kept = vec![false; lines.len()];

    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if lower.contains("error")
            || lower.contains("fail")
            || lower.contains("panic")
            || lower.contains("traceback")
            || lower.contains("warning")
            || lower.contains("warn")
        {
            kept[i] = true;
            for j in i.saturating_sub(3)..i {
                kept[j] = true;
            }
        }
    }

    let mut result = String::new();
    let mut last_kept = false;
    let mut kept_count = 0;
    let mut skipped_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if kept[i] {
            if !last_kept && skipped_count > 0 {
                result.push_str(&format!("... [{} lines skipped] ...\n", skipped_count));
                skipped_count = 0;
            }
            result.push_str(line);
            result.push('\n');
            kept_count += 1;
            last_kept = true;
        } else {
            skipped_count += 1;
            last_kept = false;
        }
    }

    if skipped_count > 0 {
        result.push_str(&format!("... [{} lines skipped] ...\n", skipped_count));
    }

    result.push_str(&format!(
        "\n[errors_only: kept {} of {} lines]",
        kept_count,
        lines.len()
    ));

    Value::String(result)
}

fn tree_view(input: &Value, _max_tokens: u64) -> Value {
    let s = match input {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(input).unwrap_or_default(),
    };

    let mut paths: Vec<String> = s
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    paths.sort();
    paths.dedup();

    let mut tree = TreeNode::new("".to_string());
    for path in &paths {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        tree.insert(&parts);
    }

    let mut result = String::new();
    tree.render(&mut result, "", true);

    Value::String(result.trim_end().to_string())
}

struct TreeNode {
    name: String,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(name: String) -> Self {
        Self {
            name,
            children: Vec::new(),
        }
    }

    fn insert(&mut self, parts: &[&str]) {
        if parts.is_empty() {
            return;
        }

        let child = self
            .children
            .iter_mut()
            .find(|c| c.name == parts[0]);

        match child {
            Some(existing) => existing.insert(&parts[1..]),
            None => {
                let mut new_child = TreeNode::new(parts[0].to_string());
                new_child.insert(&parts[1..]);
                self.children.push(new_child);
            }
        }
    }

    fn render(&self, output: &mut String, _prefix: &str, is_root: bool) {
        if is_root {
            for (i, child) in self.children.iter().enumerate() {
                let is_last = i == self.children.len() - 1;
                child.render_node(output, "", is_last);
            }
        }
    }

    fn render_node(&self, output: &mut String, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let display_name = if !self.children.is_empty() {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        };
        output.push_str(&format!("{}{}{}\n", prefix, connector, display_name));

        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        for (i, child) in self.children.iter().enumerate() {
            let child_is_last = i == self.children.len() - 1;
            child.render_node(output, &child_prefix, child_is_last);
        }
    }
}
