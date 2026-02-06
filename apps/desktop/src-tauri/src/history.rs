use serde::{Serialize, Deserialize};
use std::fs::{OpenOptions, File};
use std::io::{Write, BufReader, BufRead};
use std::path::PathBuf;
use chrono::Local;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub text: String,
    pub duration: f32,
}

pub fn get_history_path() -> PathBuf {
    let appdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::Path::new(&appdata).join("Voice2Text").join("data");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("history.jsonl")
}

pub fn append_to_history(text: &str, duration: f32) -> Result<(), String> {
    let path = get_history_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    let entry = HistoryEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Local::now().to_rfc3339(),
        text: text.to_string(),
        duration,
    };

    let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
    writeln!(file, "{}", json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_history(limit: usize, offset: usize, search: Option<String>) -> Vec<HistoryEntry> {
    let path = get_history_path();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut entries: Vec<HistoryEntry> = reader.lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| serde_json::from_str::<HistoryEntry>(&line).ok())
        .collect();

    // Search filter
    if let Some(query) = search {
        let q = query.to_lowercase();
        entries.retain(|e| e.text.to_lowercase().contains(&q));
    }

    entries.reverse(); // Newest first
    entries.into_iter().skip(offset).take(limit).collect()
}

pub fn clear_history() -> Result<(), String> {
    let path = get_history_path();
    std::fs::remove_file(path).map_err(|e| e.to_string())
}
