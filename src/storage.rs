use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::watchlist::{WatchEntry, WatchlistFile};

pub fn load_favorites(path: &Path) -> Result<HashSet<String>> {
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read favorites: {}", path.display()))?;
    let mut set = HashSet::new();
    for line in content.lines() {
        let value = line.trim().to_ascii_lowercase();
        if !value.is_empty() {
            set.insert(value);
        }
    }
    Ok(set)
}

pub fn save_favorites(path: &Path, favorites: &HashSet<String>) -> Result<()> {
    let mut list: Vec<String> = favorites.iter().cloned().collect();
    list.sort();
    let content = list.join("\n");
    fs::write(path, content)
        .with_context(|| format!("Failed to write favorites: {}", path.display()))?;
    Ok(())
}

pub fn load_watchlist(path: &Path) -> Result<Vec<WatchEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read watchlist: {}", path.display()))?;
    let file: WatchlistFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse watchlist: {}", path.display()))?;
    Ok(file.watchlist)
}

pub fn save_watchlist(path: &Path, entries: &[WatchEntry]) -> Result<()> {
    let file = WatchlistFile {
        watchlist: entries.to_vec(),
    };
    let content = toml::to_string_pretty(&file)
        .with_context(|| format!("Failed to serialize watchlist: {}", path.display()))?;
    fs::write(path, content)
        .with_context(|| format!("Failed to write watchlist: {}", path.display()))?;
    Ok(())
}
