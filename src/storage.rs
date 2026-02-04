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

pub fn ensure_watchlist_file(path: &Path) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create watchlist dir: {}", parent.display()))?;
    }
    let template = r#"# ADSB-TUI Watchlist
# Each entry is a [[watchlist]] table.
# match: hex | callsign | reg | type | owner | category | route
# mode: exact | prefix | contains
# enabled/notify: true or false

[[watchlist]]
label = "Example: Callsign prefix"
match = "callsign"
value = "SWA"
mode = "prefix"
notify = true
enabled = true
priority = 1

[[watchlist]]
label = "Example: Hex"
match = "hex"
value = "ac6668"
mode = "exact"
notify = true
enabled = true
priority = 0
"#;
    fs::write(path, template)
        .with_context(|| format!("Failed to write watchlist template: {}", path.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_watchlist_file, load_favorites, load_watchlist, save_favorites, save_watchlist,
    };
    use crate::watchlist::WatchEntry;
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        dir.push(format!("adsb-tui-test-{suffix}"));
        let _ = fs::create_dir_all(&dir);
        dir.push(name);
        dir
    }

    #[test]
    fn favorites_roundtrip() {
        let path = temp_file("favorites.txt");
        let mut set = HashSet::new();
        set.insert("abc".to_string());
        set.insert("def".to_string());
        save_favorites(&path, &set).unwrap();
        let loaded = load_favorites(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.contains("abc"));
        assert!(loaded.contains("def"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(path.parent().unwrap());
    }

    #[test]
    fn watchlist_roundtrip() {
        let path = temp_file("watchlist.toml");
        let entries = vec![WatchEntry {
            id: Some("sample".to_string()),
            label: Some("Sample".to_string()),
            match_type: "hex".to_string(),
            value: "ac6668".to_string(),
            enabled: Some(true),
            notify: Some(false),
            priority: Some(3),
            mode: Some("exact".to_string()),
            color: None,
        }];
        save_watchlist(&path, &entries).unwrap();
        let loaded = load_watchlist(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].match_type, "hex");
        assert_eq!(loaded[0].value, "ac6668");
        assert_eq!(loaded[0].notify, Some(false));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(path.parent().unwrap());
    }

    #[test]
    fn watchlist_template_created() {
        let path = temp_file("watchlist-template.toml");
        assert!(!path.exists());
        let created = ensure_watchlist_file(&path).unwrap();
        assert!(created);
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("[[watchlist]]"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(path.parent().unwrap());
    }
}
