use std::io::{Read, Write};

use anyhow::Result;
use rimuru_core::mcp::compress::{self, CompressionStrategy};
use serde_json::Value;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SlimStrategy {
    Auto,
    Truncate,
    Summarize,
    Json,
    Errors,
    Tree,
}

impl From<SlimStrategy> for CompressionStrategy {
    fn from(s: SlimStrategy) -> Self {
        match s {
            SlimStrategy::Auto => CompressionStrategy::Auto,
            SlimStrategy::Truncate => CompressionStrategy::Truncate,
            SlimStrategy::Summarize => CompressionStrategy::Summarize,
            SlimStrategy::Json => CompressionStrategy::JsonPaths,
            SlimStrategy::Errors => CompressionStrategy::ErrorsOnly,
            SlimStrategy::Tree => CompressionStrategy::TreeView,
        }
    }
}

pub fn run(strategy: SlimStrategy, max_tokens: u64, stats: bool) -> Result<()> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    let input_value: Value = serde_json::from_str(&buf).unwrap_or(Value::String(buf.clone()));

    let result = compress::compress(&input_value, strategy.into(), max_tokens);

    let output = match &result.compressed {
        Value::String(s) => s.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    };

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;
    if !output.ends_with('\n') {
        handle.write_all(b"\n")?;
    }
    handle.flush()?;

    if stats {
        let stderr = std::io::stderr();
        let mut err = stderr.lock();
        writeln!(
            err,
            "[slim] strategy={} original={} compressed={} savings={:.1}%",
            result.strategy_used,
            result.original_tokens,
            result.compressed_tokens,
            result.savings_percent
        )?;
    }

    Ok(())
}
