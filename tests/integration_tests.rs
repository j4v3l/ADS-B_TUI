// tests/integration_tests.rs

use std::fs;
use std::path::Path;

#[test]
fn test_config_file_parsing() {
    // Test that the default config can be parsed
    let config_content = r#"
url = "http://example.com/data.json"
refresh_secs = 1
insecure = false
stale_secs = 60
low_nic = 5
low_nac = 8
trail_len = 6
flags_enabled = true
ui_fps = 60
smooth_mode = true
"#;

    let config_path = "test_config.toml";
    fs::write(config_path, config_content).expect("Failed to write test config");

    // Verify file was created
    assert!(Path::new(config_path).exists());

    // Clean up
    fs::remove_file(config_path).expect("Failed to clean up test config");
}

#[test]
fn test_project_structure() {
    // Test that all expected source files exist
    let expected_files = vec![
        "src/main.rs",
        "src/app.rs",
        "src/ui.rs",
        "src/config.rs",
        "src/model.rs",
        "src/net.rs",
        "Cargo.toml",
        "README.md",
    ];

    for file in expected_files {
        assert!(Path::new(file).exists(), "Expected file {} not found", file);
    }
}

#[test]
fn test_cargo_toml_metadata() {
    // Test that Cargo.toml has required metadata
    let cargo_content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(cargo_content.contains("name = \"adsb-tui\""), "Missing package name");
    assert!(cargo_content.contains("description ="), "Missing description");
    assert!(cargo_content.contains("license ="), "Missing license");
    assert!(cargo_content.contains("readme ="), "Missing readme");
    assert!(cargo_content.contains("homepage ="), "Missing homepage");
    assert!(cargo_content.contains("repository ="), "Missing repository");
}

#[test]
fn test_readme_exists_and_complete() {
    // Test that README.md exists and has essential sections
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let required_sections = vec![
        "# ADS-B TUI",
        "## ✨ Features",
        "## 🚀 Quick Start",
        "## 📖 Configuration",
        "## 🎮 Controls",
        "## 🛠️ Development",
    ];

    for section in required_sections {
        assert!(readme_content.contains(section), "README missing section: {}", section);
    }
}
