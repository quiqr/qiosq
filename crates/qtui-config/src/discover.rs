//! Discover candidate Quiqr data folders for the `qtui init` flow.
//!
//! Both Quiqr editions record their storage location authoritatively: the
//! Electron desktop app writes `instance_settings.json` (`storage.dataFolder`)
//! in its per-OS application config dir, and the Server uses its module config.
//! This module reads the Electron settings plus a fixed set of fallback paths,
//! de-duplicates them, and annotates each with its source, whether it is a valid
//! Quiqr library, and its site count. It is read-only and pure (the home/config
//! roots are injectable so it is fully unit-testable without touching the real
//! environment).

use std::path::{Path, PathBuf};

/// Where a candidate data folder came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Read from the Electron desktop app's `instance_settings.json`.
    ElectronSettings,
    /// A built-in fallback location.
    Fallback,
}

/// A discovered candidate Quiqr data folder, annotated for the init UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirCandidate {
    /// Absolute path to the candidate data folder.
    pub path: PathBuf,
    /// How this candidate was found.
    pub source: Source,
    /// Whether it looks like a Quiqr library (has `sites/` with >=1 site).
    pub valid: bool,
    /// Number of sites found (0 when not a valid library).
    pub site_count: usize,
}

/// Discover candidates using the real environment (home + per-OS config dir).
pub fn discover() -> Vec<DataDirCandidate> {
    discover_in(dirs::home_dir().as_deref(), dirs::config_dir().as_deref())
}

/// Discover candidates against the given `home` and app-`config_dir` roots
/// (injected for tests). `config_dir` is the per-OS application config dir
/// (e.g. `~/.config` on Linux); the Electron settings live at
/// `<config_dir>/quiqr/instance_settings.json`.
pub fn discover_in(home: Option<&Path>, config_dir: Option<&Path>) -> Vec<DataDirCandidate> {
    let mut raw: Vec<(PathBuf, Source)> = Vec::new();

    // 1. Electron settings -> storage.dataFolder.
    if let Some(cfg) = config_dir {
        let settings = cfg.join("quiqr").join("instance_settings.json");
        if let Some(data_folder) = read_electron_data_folder(&settings) {
            if let Some(p) = expand_tilde(&data_folder, home) {
                raw.push((p, Source::ElectronSettings));
            }
        }
    }

    // 2. Fallback locations under home.
    if let Some(h) = home {
        raw.push((h.join("Quiqr"), Source::Fallback));
        raw.push((h.join("Quiqr Data"), Source::Fallback));
    }

    // De-dup by resolved absolute path, keeping the first (richer) source.
    let mut out: Vec<DataDirCandidate> = Vec::new();
    for (path, source) in raw {
        let resolved = resolve_abs(&path);
        if out.iter().any(|c| resolve_abs(&c.path) == resolved) {
            continue;
        }
        let site_count = count_sites(&path);
        out.push(DataDirCandidate {
            path,
            source,
            valid: site_count > 0,
            site_count,
        });
    }
    out
}

/// Read `storage.dataFolder` from a Quiqr `instance_settings.json`, tolerantly:
/// a missing file, invalid JSON, or an absent key all yield `None`.
fn read_electron_data_folder(settings: &Path) -> Option<String> {
    let text = std::fs::read_to_string(settings).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value
        .get("storage")?
        .get("dataFolder")?
        .as_str()
        .map(str::to_string)
}

/// Expand a leading `~` (or `~/…`) to the home directory. Other paths are
/// returned as-is. Returns `None` only if a `~` is present but `home` is unknown.
fn expand_tilde(path: &str, home: Option<&Path>) -> Option<PathBuf> {
    if path == "~" {
        return home.map(Path::to_path_buf);
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home.map(|h| h.join(rest));
    }
    Some(PathBuf::from(path))
}

/// Best-effort absolute resolution for de-dup: canonicalize when the path
/// exists, else return it as given (already absolute for our inputs).
fn resolve_abs(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Lightweight site count mirroring the storage rule: the number of
/// `<data>/sites/<name>/` directories that contain a `config.json`. Kept local
/// (no `qtui-storage` dependency) so `qtui-config` stays foundational; the
/// binary uses the authoritative `enumerate_sites` at runtime.
fn count_sites(data_dir: &Path) -> usize {
    let sites = data_dir.join("sites");
    let Ok(entries) = std::fs::read_dir(&sites) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.path().join("config.json").is_file())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create `<data>/sites/<name>/config.json` so `count_sites` counts it.
    fn add_site(data_dir: &Path, name: &str) {
        let d = data_dir.join("sites").join(name);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("config.json"), r#"{"key":"x"}"#).unwrap();
    }

    fn write_settings(config_dir: &Path, json: &str) {
        let q = config_dir.join("quiqr");
        fs::create_dir_all(&q).unwrap();
        fs::write(q.join("instance_settings.json"), json).unwrap();
    }

    #[test]
    fn reads_electron_data_folder_and_expands_tilde() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let cfg = home.join(".config");
        fs::create_dir_all(&cfg).unwrap();
        // Electron points at ~/Quiqr; populate it with one site.
        write_settings(&cfg, r#"{"storage":{"type":"fs","dataFolder":"~/Quiqr"}}"#);
        add_site(&home.join("Quiqr"), "blog");

        let cands = discover_in(Some(&home), Some(&cfg));
        let electron = cands
            .iter()
            .find(|c| c.source == Source::ElectronSettings)
            .expect("electron candidate");
        assert_eq!(electron.path, home.join("Quiqr"));
        assert!(electron.valid);
        assert_eq!(electron.site_count, 1);
    }

    #[test]
    fn includes_fallbacks_and_dedups_against_electron() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let cfg = home.join(".config");
        fs::create_dir_all(&cfg).unwrap();
        // Electron dataFolder == the ~/Quiqr fallback -> must appear once.
        write_settings(&cfg, r#"{"storage":{"dataFolder":"~/Quiqr"}}"#);
        add_site(&home.join("Quiqr"), "a");

        let cands = discover_in(Some(&home), Some(&cfg));
        let quiqr_hits = cands
            .iter()
            .filter(|c| c.path == home.join("Quiqr"))
            .count();
        assert_eq!(quiqr_hits, 1, "~/Quiqr de-duped across sources");
        // The "~/Quiqr Data" fallback is still present (as a separate candidate).
        assert!(cands.iter().any(|c| c.path == home.join("Quiqr Data")));
    }

    #[test]
    fn tolerates_missing_and_malformed_settings() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let cfg = home.join(".config");
        fs::create_dir_all(&cfg).unwrap();

        // No settings file at all -> only fallbacks, no panic.
        let cands = discover_in(Some(&home), Some(&cfg));
        assert!(cands.iter().all(|c| c.source == Source::Fallback));

        // Malformed JSON -> still no electron candidate.
        write_settings(&cfg, "{ not json");
        let cands = discover_in(Some(&home), Some(&cfg));
        assert!(cands.iter().all(|c| c.source == Source::Fallback));

        // Valid JSON but no dataFolder key -> no electron candidate.
        write_settings(&cfg, r#"{"storage":{"type":"fs"}}"#);
        let cands = discover_in(Some(&home), Some(&cfg));
        assert!(cands.iter().all(|c| c.source == Source::Fallback));
    }

    #[test]
    fn nonexistent_or_empty_candidate_is_invalid() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        // No data folders exist at all.
        let cands = discover_in(Some(&home), None);
        assert!(!cands.is_empty());
        assert!(cands.iter().all(|c| !c.valid && c.site_count == 0));
    }
}
