use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

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
