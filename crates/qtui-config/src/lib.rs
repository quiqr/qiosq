//! quiqr-tui configuration: load and validate a single TOML file.
//!
//! Everything environment-specific lives in this config so the PoC is portable
//! to another machine by editing only the file (see `docs/01-architecture.md`
//! §3 and `config/quiqr-tui.example.toml`). Loading is two phases:
//!
//! 1. [`Config::load`] reads the file and parses TOML into typed structs.
//! 2. [`Config::validate`] checks cross-field invariants and resolves the data
//!    dir to an absolute path, returning **one distinct error per invalid
//!    field** so a misconfiguration is diagnosable without reading the source.
//!
//! [`Config::load_and_validate`] runs both and is what callers normally use.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// The filename of the shipped example config, relative to the repo's
/// `config/` directory. Exposed so the binary and tests can locate it.
pub const EXAMPLE_CONFIG_FILENAME: &str = "quiqr-tui.example.toml";

/// Quiqr's default Hugo preview port. The preview range must never include it.
pub const QUIQR_RESERVED_PORT: u16 = 13131;

/// Top-level configuration, mirroring `config/quiqr-tui.example.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub storage: Storage,
    pub preview: Preview,
    pub agent: Agent,
    #[serde(default)]
    pub ui: Ui,
    #[serde(default)]
    pub rmux: Rmux,
}

/// `[storage]` — where Quiqr keeps sites, and what to hide in the browser.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Storage {
    /// Absolute (after validation) path to the Quiqr Data directory.
    pub quiqr_data_dir: String,
    /// Derived/generated/VCS dirs hidden in the read-only content browser.
    #[serde(default = "default_hidden_dirs")]
    pub hidden_dirs: Vec<String>,
}

/// `[preview]` — the `hugo server` lifecycle.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    #[serde(default = "default_hugo_bin")]
    pub hugo_bin: String,
    /// Inclusive `[low, high]` port range. Must exclude [`QUIQR_RESERVED_PORT`].
    pub port_range: [u16; 2],
    #[serde(default = "default_ready_timeout_ms")]
    pub ready_timeout_ms: u64,
}

/// `[agent]` — the coding agent command (Claude Code by default, swappable).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// String the agent prints when a task is finished (completion detection).
    pub completion_sentinel: String,
    #[serde(default)]
    pub sandbox: Sandbox,
}

/// `[agent.sandbox]` — where the agent may and may not write.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sandbox {
    #[serde(default = "default_writable_paths")]
    pub writable_paths: Vec<String>,
    #[serde(default = "default_readonly_paths")]
    pub readonly_paths: Vec<String>,
    #[serde(default = "default_commit_branch_prefix")]
    pub commit_branch_prefix: String,
}

/// `[ui]` — cosmetic WordPerfect-5.1 toggles.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ui {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub show_schema_nav_first: bool,
}

/// `[rmux]` — pin the rmux version (rmux is young).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rmux {
    #[serde(default = "default_rmux_version")]
    pub version: String,
}

/// An error tied to a specific configuration field. Each variant maps to one
/// field so callers (and tests) get a distinct, human-readable message.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("storage.quiqr_data_dir must not be empty (set it to the Quiqr Data directory)")]
    EmptyDataDir,

    #[error(
        "preview.port_range must not include {QUIQR_RESERVED_PORT} \
         (it is reserved for Quiqr's default Hugo server)"
    )]
    PortRangeIncludesReserved,

    #[error("preview.port_range low ({low}) must be <= high ({high})")]
    PortRangeInverted { low: u16, high: u16 },

    #[error("preview.port_range ports must be >= 1 (got [{0}, {1}])")]
    PortRangeZero(u16, u16),

    #[error("agent.command must not be empty (the coding agent to launch)")]
    EmptyAgentCommand,

    #[error("agent.completion_sentinel must not be empty (used to detect task completion)")]
    EmptyCompletionSentinel,
}

/// Top-level error returned by [`Config::load`] and [`Config::load_and_validate`].
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("config file not found or unreadable: {path} ({source})")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("config file is not valid TOML: {path} ({source})")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("config validation failed: {0}")]
    Validate(#[from] ValidationError),
}

impl Config {
    /// Read and parse the TOML file at `path`. Does **not** validate.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|source| LoadError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&text).map_err(|source| LoadError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Validate cross-field invariants and resolve the data dir to an absolute
    /// path. Returns the first failing field's error; the validation order is
    /// stable so a given config always reports the same field first.
    pub fn validate(&mut self) -> Result<(), ValidationError> {
        // storage.quiqr_data_dir: non-empty, resolved to absolute.
        if self.storage.quiqr_data_dir.trim().is_empty() {
            return Err(ValidationError::EmptyDataDir);
        }
        self.storage.quiqr_data_dir = resolve_absolute(&self.storage.quiqr_data_dir);

        // preview.port_range: valid, ordered, excludes the reserved port.
        let [low, high] = self.preview.port_range;
        if low == 0 || high == 0 {
            return Err(ValidationError::PortRangeZero(low, high));
        }
        if low > high {
            return Err(ValidationError::PortRangeInverted { low, high });
        }
        if (low..=high).contains(&QUIQR_RESERVED_PORT) {
            return Err(ValidationError::PortRangeIncludesReserved);
        }

        // agent.command + completion_sentinel: non-empty.
        if self.agent.command.trim().is_empty() {
            return Err(ValidationError::EmptyAgentCommand);
        }
        if self.agent.completion_sentinel.is_empty() {
            return Err(ValidationError::EmptyCompletionSentinel);
        }

        Ok(())
    }

    /// Load from `path` and validate in one step.
    pub fn load_and_validate(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let mut config = Self::load(path)?;
        config.validate()?;
        Ok(config)
    }
}

/// Resolve `dir` to an absolute path. If already absolute, return it as-is;
/// otherwise join it onto the current working directory. We deliberately do not
/// require the directory to exist at config-load time (the Quiqr Server may
/// create it later); existence is checked by the storage layer in E2.
fn resolve_absolute(dir: &str) -> String {
    let p = Path::new(dir);
    if p.is_absolute() {
        return dir.to_string();
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(p).to_string_lossy().into_owned(),
        // No cwd (extremely rare): fall back to the original value rather than
        // failing — validation is about the config, not the process state.
        Err(_) => dir.to_string(),
    }
}

fn default_hidden_dirs() -> Vec<String> {
    ["public", "resources", ".quiqr-cache", ".git", "themes"]
        .into_iter()
        .map(String::from)
        .collect()
}
fn default_hugo_bin() -> String {
    "hugo".to_string()
}
fn default_ready_timeout_ms() -> u64 {
    15_000
}
fn default_writable_paths() -> Vec<String> {
    vec!["content".to_string(), "static".to_string()]
}
fn default_readonly_paths() -> Vec<String> {
    vec![
        "quiqr".to_string(),
        "public".to_string(),
        "resources".to_string(),
    ]
}
fn default_commit_branch_prefix() -> String {
    "qtui/".to_string()
}
fn default_theme() -> String {
    "wp51".to_string()
}
fn default_rmux_version() -> String {
    "0.1".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            writable_paths: default_writable_paths(),
            readonly_paths: default_readonly_paths(),
            commit_branch_prefix: default_commit_branch_prefix(),
        }
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_schema_nav_first: true,
        }
    }
}

impl Default for Rmux {
    fn default() -> Self {
        Self {
            version: default_rmux_version(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal valid TOML config, with one field overridable for error tests.
    fn valid_toml() -> String {
        r#"
[storage]
quiqr_data_dir = "/tmp/quiqr-data"

[preview]
port_range = [13140, 13200]

[agent]
command = "claude"
completion_sentinel = "<<QTUI_TASK_DONE>>"
"#
        .to_string()
    }

    fn parse(toml_str: &str) -> Config {
        toml::from_str(toml_str).expect("test TOML should parse")
    }

    #[test]
    fn minimal_valid_config_validates_and_fills_defaults() {
        let mut cfg = parse(&valid_toml());
        cfg.validate().expect("valid config should validate");

        // Defaults applied for omitted sections/fields.
        assert_eq!(cfg.preview.hugo_bin, "hugo");
        assert_eq!(cfg.preview.ready_timeout_ms, 15_000);
        assert_eq!(cfg.agent.sandbox.writable_paths, vec!["content", "static"]);
        assert_eq!(cfg.ui.theme, "wp51");
        assert!(cfg.ui.show_schema_nav_first);
        assert_eq!(cfg.rmux.version, "0.1");
    }

    #[test]
    fn relative_data_dir_is_resolved_absolute() {
        let toml_str = valid_toml().replace("/tmp/quiqr-data", "Quiqr Data");
        let mut cfg = parse(&toml_str);
        cfg.validate().unwrap();
        assert!(
            Path::new(&cfg.storage.quiqr_data_dir).is_absolute(),
            "relative data dir should resolve to absolute, got {:?}",
            cfg.storage.quiqr_data_dir
        );
    }

    #[test]
    fn empty_data_dir_errors() {
        let toml_str = valid_toml().replace("/tmp/quiqr-data", "");
        let mut cfg = parse(&toml_str);
        assert_eq!(cfg.validate(), Err(ValidationError::EmptyDataDir));
    }

    #[test]
    fn port_range_including_reserved_errors() {
        let toml_str = valid_toml().replace("[13140, 13200]", "[13100, 13140]");
        let mut cfg = parse(&toml_str);
        assert_eq!(
            cfg.validate(),
            Err(ValidationError::PortRangeIncludesReserved)
        );
    }

    #[test]
    fn inverted_port_range_errors() {
        let toml_str = valid_toml().replace("[13140, 13200]", "[13200, 13140]");
        let mut cfg = parse(&toml_str);
        assert_eq!(
            cfg.validate(),
            Err(ValidationError::PortRangeInverted {
                low: 13200,
                high: 13140
            })
        );
    }

    #[test]
    fn zero_port_errors() {
        let toml_str = valid_toml().replace("[13140, 13200]", "[0, 13200]");
        let mut cfg = parse(&toml_str);
        assert_eq!(
            cfg.validate(),
            Err(ValidationError::PortRangeZero(0, 13200))
        );
    }

    #[test]
    fn empty_agent_command_errors() {
        let toml_str = valid_toml().replace("command = \"claude\"", "command = \"\"");
        let mut cfg = parse(&toml_str);
        assert_eq!(cfg.validate(), Err(ValidationError::EmptyAgentCommand));
    }

    #[test]
    fn empty_completion_sentinel_errors() {
        let toml_str = valid_toml().replace("\"<<QTUI_TASK_DONE>>\"", "\"\"");
        let mut cfg = parse(&toml_str);
        assert_eq!(
            cfg.validate(),
            Err(ValidationError::EmptyCompletionSentinel)
        );
    }

    #[test]
    fn each_invalid_field_has_a_distinct_message() {
        // One config per single-field defect; assert all messages are unique.
        let cases = [
            valid_toml().replace("/tmp/quiqr-data", ""),
            valid_toml().replace("[13140, 13200]", "[13100, 13140]"),
            valid_toml().replace("[13140, 13200]", "[13200, 13140]"),
            valid_toml().replace("[13140, 13200]", "[0, 13200]"),
            valid_toml().replace("command = \"claude\"", "command = \"\""),
            valid_toml().replace("\"<<QTUI_TASK_DONE>>\"", "\"\""),
        ];
        let mut messages = Vec::new();
        for toml_str in cases {
            let mut cfg = parse(&toml_str);
            let err = cfg.validate().expect_err("each case is invalid");
            messages.push(err.to_string());
        }
        let unique: std::collections::HashSet<_> = messages.iter().cloned().collect();
        assert_eq!(
            unique.len(),
            messages.len(),
            "every invalid field must have a distinct message: {messages:#?}"
        );
    }

    #[test]
    fn missing_required_section_is_a_parse_error() {
        // No [agent] section: serde should refuse rather than silently default.
        let toml_str = r#"
[storage]
quiqr_data_dir = "/tmp/x"
[preview]
port_range = [13140, 13200]
"#;
        let parsed: Result<Config, _> = toml::from_str(toml_str);
        assert!(parsed.is_err(), "missing [agent] must fail to parse");
    }

    #[test]
    fn unknown_field_is_rejected() {
        let toml_str = format!("{}\nbogus = true\n", valid_toml());
        let parsed: Result<Config, _> = toml::from_str(&toml_str);
        assert!(parsed.is_err(), "unknown top-level field must be rejected");
    }
}
