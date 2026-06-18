//! quiqr-tui (codename *qiosq*) — the kiosk binary.
//!
//! The host wires the services together behind the `qtui-ui` state machine:
//! load the config, enumerate sites (`qtui-storage`), build the navigation model
//! (`qtui-model`), start/stop the Hugo preview (`qtui-preview`) on site
//! open/close, and drive the agent (`qtui-agent`). Run modes:
//!
//! - **interactive** (default): a crossterm raw-mode terminal loop.
//! - **headless** (`--script <steps>`): the same transitions driven by a step
//!   list with no TTY — what the e2e test runs.
//! - **init** (`qtui init`): discover the Quiqr storage (or ask), then write the
//!   config to `~/.config/qiosq/config.toml`.
//!
//! Config resolution: `--config <path>` wins; else the default
//! `~/.config/qiosq/config.toml`; else (interactive) run `init`.
//!
//! Usage:
//!   qtui --version
//!   qtui init [--force]
//!   qtui [--config <path>] [--site <name>]                 # interactive
//!   qtui [--config <path>] --script open-site,open-file,ask-ai,await   # headless

mod host;
mod init;
mod interactive;
mod script;

use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || matches!(args[0].as_str(), "--version" | "-V" | "version") {
        // A bare `qtui` with no args is interactive (resolve config below); only
        // an explicit version request short-circuits.
        if args.first().map(String::as_str) == Some("--version")
            || args.first().map(String::as_str) == Some("-V")
            || args.first().map(String::as_str) == Some("version")
        {
            println!("qtui {VERSION}");
            return ExitCode::SUCCESS;
        }
    }

    // `qtui init …` is a distinct subcommand.
    if args.first().map(String::as_str) == Some("init") {
        let force = args.iter().any(|a| a == "--force");
        return match init::run(force) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("qtui init: {e}");
                ExitCode::FAILURE
            }
        };
    }

    let opts = match Options::parse(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("qtui: {e}");
            eprintln!(
                "usage: qtui [--version] | qtui init [--force] | \
                 qtui [--config <path>] [--site <name>] [--script <steps>]"
            );
            return ExitCode::from(2);
        }
    };

    // Resolve the config path: explicit --config, else the default location.
    let config = match resolve_config(opts.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("qtui: {e}");
            return ExitCode::FAILURE;
        }
    };

    let result = if let Some(steps) = &opts.script {
        script::run(&config, opts.site.as_deref(), steps)
    } else {
        interactive::run(&config, opts.site.as_deref())
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("qtui: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Resolve which config to load: the explicit `--config`, else the default
/// `~/.config/qiosq/config.toml` if it exists. Errors with guidance otherwise.
fn resolve_config(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    match qtui_config::default_config_path() {
        Some(p) if p.exists() => Ok(p),
        Some(p) => Err(format!(
            "no config found at {} — run `qtui init` to create one (or pass --config)",
            p.display()
        )),
        None => Err("could not determine the default config location; pass --config".into()),
    }
}

/// Parsed command-line options for the run modes (not `init`).
struct Options {
    config: Option<PathBuf>,
    site: Option<String>,
    /// Comma-separated headless step list, when in scripted mode.
    script: Option<Vec<String>>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut config = None;
        let mut site = None;
        let mut script = None;
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--config" => {
                    config = Some(it.next().ok_or("--config needs a path")?.into());
                }
                "--site" => {
                    site = Some(it.next().ok_or("--site needs a name")?.clone());
                }
                "--script" => {
                    let steps = it.next().ok_or("--script needs a step list")?;
                    script = Some(steps.split(',').map(|s| s.trim().to_string()).collect());
                }
                other => return Err(format!("unknown argument {other:?}")),
            }
        }
        Ok(Self {
            config,
            site,
            script,
        })
    }
}
