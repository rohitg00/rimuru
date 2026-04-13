use std::io::{IsTerminal, Write};
use std::time::Duration;

use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::{Value, json};

use crate::output;

const BAR_WIDTH: usize = 30;
const REFRESH_MS: u64 = 1500;

struct CursorGuard;

impl Drop for CursorGuard {
    fn drop(&mut self) {
        print!("\x1b[?25h");
        let _ = std::io::stdout().flush();
    }
}

pub async fn run(iii: &III, session_filter: Option<String>, watch: bool) -> Result<()> {
    if !watch {
        render_once(iii, session_filter.as_deref()).await?;
        return Ok(());
    }

    let is_tty = std::io::stdout().is_terminal();
    if !is_tty {
        render_once(iii, session_filter.as_deref()).await?;
        return Ok(());
    }

    print!("\x1b[?25l");
    let _ = std::io::stdout().flush();
    let _guard = CursorGuard;

    run_loop(iii, session_filter.as_deref()).await
}

async fn run_loop(iii: &III, session_filter: Option<&str>) -> Result<()> {
    let mut first = true;
    loop {
        if !first {
            print!("\x1b[2J\x1b[H");
            let _ = std::io::stdout().flush();
        }
        first = false;

        if let Err(e) = render_once(iii, session_filter).await {
            eprintln!("error: {}", e);
        }

        println!();
        println!("press ctrl-c to exit  (refresh every {}ms)", REFRESH_MS);

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!();
                return Ok(());
            }
            _ = tokio::time::sleep(Duration::from_millis(REFRESH_MS)) => {}
        }
    }
}

async fn render_once(iii: &III, session_filter: Option<&str>) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.context.utilization".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: Some(10_000),
        })
        .await?;
    let body = output::unwrap_body(result);

    let utilizations = body
        .get("utilizations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if utilizations.is_empty() {
        println!("No active sessions to track.");
        println!(
            "Hint: connect an agent first (`rimuru agents detect && rimuru agents connect <type>`)"
        );
        return Ok(());
    }

    let target = pick_session(&utilizations, session_filter);
    let target = match target {
        Some(t) => t,
        None => {
            println!(
                "Session {} not found in active sessions.",
                session_filter.unwrap_or("?")
            );
            return Ok(());
        }
    };

    render_session(target, utilizations.len());
    Ok(())
}

fn pick_session<'a>(active: &'a [Value], session_filter: Option<&str>) -> Option<&'a Value> {
    if let Some(want) = session_filter {
        return active.iter().find(|u| {
            u.get("session_id")
                .and_then(|v| v.as_str())
                .is_some_and(|sid| sid == want || sid.starts_with(want))
        });
    }
    active
        .iter()
        .max_by_key(|u| u.get("tokens_used").and_then(|v| v.as_u64()).unwrap_or(0))
}

fn render_session(session: &Value, total_active: usize) {
    let sid = session
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let model = session
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let used = session
        .get("tokens_used")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let window = session
        .get("context_window_size")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let pct = session
        .get("utilization_percent")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let near = session
        .get("is_near_limit")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let bar = render_bar(pct);
    let color = pct_color(pct);

    println!(
        "rimuru ctx                       {} active session(s)",
        total_active
    );
    println!("session: {}", short_id(sid));
    println!("model:   {}", model);
    println!();
    println!("{}{}\x1b[0m", color, bar);
    println!(
        "{}{:.1}%   {} / {} tokens\x1b[0m",
        color,
        pct,
        format_tokens(used),
        format_tokens(window)
    );

    if near {
        println!("\x1b[31m! near context limit\x1b[0m");
    }
}

fn render_bar(pct: f64) -> String {
    let filled = ((pct / 100.0) * BAR_WIDTH as f64).round() as usize;
    let filled = filled.min(BAR_WIDTH);
    let empty = BAR_WIDTH - filled;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}

fn pct_color(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "\x1b[31m"
    } else if pct >= 75.0 {
        "\x1b[33m"
    } else {
        "\x1b[32m"
    }
}

fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn short_id(sid: &str) -> String {
    let mut iter = sid.char_indices();
    match iter.nth(12) {
        Some((idx, _)) => format!("{}...", &sid[..idx]),
        None => sid.to_string(),
    }
}
