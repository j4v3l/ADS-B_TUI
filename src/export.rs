use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::App;
use crate::model::seen_seconds;

pub fn export_csv(app: &App, indices: &[usize]) -> Result<String> {
    let filename = format!("adsb-snapshot-{}.csv", Local::now().format("%Y%m%d-%H%M%S"));
    let mut path = export_path(&filename)?;
    if path.exists() {
        path = unique_path(&path);
    }

    let mut lines = Vec::new();
    lines.push("hex,flight,reg,type,alt_baro,alt_geom,gs,track,lat,lon,seen,messages".to_string());
    for idx in indices {
        let ac = &app.data.aircraft[*idx];
        lines.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_field(ac.hex.as_deref()),
            csv_field(ac.flight.as_deref()),
            csv_field(ac.r.as_deref()),
            csv_field(ac.t.as_deref()),
            opt_i64(ac.alt_baro),
            opt_i64(ac.alt_geom),
            opt_f64(ac.gs, 1),
            opt_f64(ac.track, 1),
            opt_f64(ac.lat, 4),
            opt_f64(ac.lon, 4),
            opt_f64(seen_seconds(ac), 1),
            opt_u64(ac.messages)
        ));
    }

    fs::write(&path, lines.join("\n"))
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(path.to_string_lossy().to_string())
}

pub fn export_json(app: &App) -> Result<String> {
    let filename = format!(
        "adsb-snapshot-{}.json",
        Local::now().format("%Y%m%d-%H%M%S")
    );
    let mut path = export_path(&filename)?;
    if path.exists() {
        path = unique_path(&path);
    }

    let payload = serde_json::to_string_pretty(&app.data)?;
    fs::write(&path, payload).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(path.to_string_lossy().to_string())
}

fn opt_i64(value: Option<i64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn opt_u64(value: Option<u64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn opt_f64(value: Option<f64>, precision: usize) -> String {
    value
        .map(|v| format!("{v:.precision$}", precision = precision))
        .unwrap_or_default()
}

fn csv_field(value: Option<&str>) -> String {
    let text = value.unwrap_or("");
    let guarded = guard_csv_formula(text);
    if guarded.contains(',')
        || guarded.contains('"')
        || guarded.contains('\n')
        || guarded.contains('\r')
    {
        format!("\"{}\"", guarded.replace('"', "\"\""))
    } else {
        guarded
    }
}

fn guard_csv_formula(text: &str) -> String {
    let trimmed = text.trim_start_matches([' ', '\t']);
    let mut chars = trimmed.chars();
    if matches!(chars.next(), Some('=' | '+' | '-' | '@')) {
        format!("'{}", text)
    } else {
        text.to_string()
    }
}

fn export_path(filename: &str) -> Result<PathBuf> {
    let dir = PathBuf::from("exports");
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    Ok(dir.join(filename))
}

fn unique_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let mut i = 1;
    loop {
        let name = if ext.is_empty() {
            format!("{stem}-{i}")
        } else {
            format!("{stem}-{i}.{ext}")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::csv_field;

    #[test]
    fn csv_field_guards_formulas() {
        assert_eq!(csv_field(Some("=1+1")), "'=1+1");
        assert_eq!(csv_field(Some("+SUM(A1:A2)")), "'+SUM(A1:A2)");
        assert_eq!(csv_field(Some("  -42")), "'  -42");
    }

    #[test]
    fn csv_field_quotes_special_chars() {
        assert_eq!(csv_field(Some("a,b")), "\"a,b\"");
        assert_eq!(csv_field(Some("line\nbreak")), "\"line\nbreak\"");
        assert_eq!(csv_field(Some("quote\"here")), "\"quote\"\"here\"");
    }
}
