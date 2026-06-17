//! Integration test: the shipped `config/quiqr-tui.example.toml` must load and
//! validate. This guards against the example drifting from the loader (and
//! against regressions like the stray heredoc `EOF` that was in the bootstrap
//! bundle). Maps to the `config-loading` spec scenario "Example config loads
//! successfully".

use std::path::PathBuf;

use qtui_config::{Config, EXAMPLE_CONFIG_FILENAME};

/// Path to the repo's `config/` dir, derived from this crate's manifest dir
/// (`crates/qtui-config`) so the test is independent of the cwd.
fn example_config_path() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("crate is two levels below the repo root");
    repo_root.join("config").join(EXAMPLE_CONFIG_FILENAME)
}

#[test]
fn shipped_example_loads_and_validates() {
    let path = example_config_path();
    assert!(
        path.exists(),
        "expected example config at {}",
        path.display()
    );

    let cfg = Config::load_and_validate(&path)
        .unwrap_or_else(|e| panic!("example config should load and validate: {e}"));

    // Spot-check the values the example documents, so a silent reshape is caught.
    assert_eq!(cfg.preview.port_range, [13140, 13200]);
    assert!(
        !cfg.preview
            .port_range
            .contains(&qtui_config::QUIQR_RESERVED_PORT),
        "example must not include the reserved port"
    );
    assert_eq!(cfg.agent.command, "claude");
    assert_eq!(cfg.agent.completion_sentinel, "<<QTUI_TASK_DONE>>");
    assert!(
        std::path::Path::new(&cfg.storage.quiqr_data_dir).is_absolute(),
        "data dir should be resolved to absolute"
    );
}

#[test]
fn missing_file_reports_path() {
    let err =
        Config::load("/nonexistent/quiqr-tui.toml").expect_err("loading a missing file must error");
    let msg = err.to_string();
    assert!(
        msg.contains("/nonexistent/quiqr-tui.toml"),
        "error should name the missing path, got: {msg}"
    );
}
