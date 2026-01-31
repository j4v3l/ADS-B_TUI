use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEntry {
    pub id: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "match")]
    pub match_type: String,
    pub value: String,
    pub enabled: Option<bool>,
    pub notify: Option<bool>,
    pub priority: Option<i64>,
    pub mode: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchlistFile {
    #[serde(default)]
    pub watchlist: Vec<WatchEntry>,
}

impl WatchEntry {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    pub fn notify_enabled(&self) -> bool {
        self.notify.unwrap_or(true)
    }

    pub fn priority(&self) -> i64 {
        self.priority.unwrap_or(0)
    }

    pub fn match_mode(&self) -> &str {
        self.mode.as_deref().unwrap_or("exact")
    }

    pub fn entry_id(&self) -> String {
        if let Some(id) = &self.id {
            if !id.trim().is_empty() {
                return id.trim().to_string();
            }
        }
        let label = self.label.as_deref().unwrap_or("");
        if !label.trim().is_empty() {
            return label.trim().to_string();
        }
        format!("{}:{}", self.match_type, self.value)
    }
}
