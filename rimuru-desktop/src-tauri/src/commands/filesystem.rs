use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Serialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
}

#[derive(Serialize, Clone)]
pub struct DirectoryStats {
    pub file_count: u64,
    pub folder_count: u64,
    pub total_size: u64,
}

#[derive(Serialize, Clone)]
pub struct GitInfo {
    pub branch: String,
    pub is_clean: bool,
    pub remote_url: Option<String>,
    pub status_summary: String,
}

#[tauri::command]
pub async fn read_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries: Vec<DirEntry> = Vec::new();
    let read = fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in read {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        entries.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

#[tauri::command]
pub async fn get_directory_stats(path: String) -> Result<DirectoryStats, String> {
    let mut file_count: u64 = 0;
    let mut folder_count: u64 = 0;
    let mut total_size: u64 = 0;

    for entry in WalkDir::new(&path)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            folder_count += 1;
        } else {
            file_count += 1;
            total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    folder_count = folder_count.saturating_sub(1);

    Ok(DirectoryStats {
        file_count,
        folder_count,
        total_size,
    })
}

#[tauri::command]
pub async fn get_git_info(path: String) -> Result<GitInfo, String> {
    let run = |args: &[&str]| -> Option<String> {
        Command::new("git")
            .args(args)
            .current_dir(&path)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };

    let branch = run(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let status_output = run(&["status", "--porcelain"]).unwrap_or_default();
    let is_clean = status_output.is_empty();
    let remote_url = run(&["remote", "get-url", "origin"]);

    let status_summary = if is_clean {
        "Working tree clean".into()
    } else {
        let changes = status_output.lines().count();
        format!(
            "{} file{} changed",
            changes,
            if changes == 1 { "" } else { "s" }
        )
    };

    Ok(GitInfo {
        branch,
        is_clean,
        remote_url,
        status_summary,
    })
}

#[tauri::command]
pub async fn read_file_preview(path: String, max_lines: Option<u32>) -> Result<String, String> {
    let max = max_lines.unwrap_or(50) as usize;
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().take(max).filter_map(|l| l.ok()).collect();
    Ok(lines.join("\n"))
}
