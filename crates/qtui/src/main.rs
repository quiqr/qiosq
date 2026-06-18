//! quiqr-tui (codename *qiosq*) — the kiosk binary.
//!
//! The host wires the services together behind the `qtui-ui` state machine:
//! load the config, enumerate sites (`qtui-storage`), build the navigation model
//! (`qtui-model`), start/stop the Hugo preview (`qtui-preview`) on site
//! open/close, and drive the agent (`qtui-agent`). Two run modes:
//!
//! - **interactive** (default): a crossterm raw-mode terminal loop.
//! - **headless** (`--script <steps>`): the same transitions driven by a step
//!   list with no TTY — what the e2e test runs.
//!
//! Usage:
//!   qtui --version
//!   qtui --config <path> [--site <name>]                 # interactive
//!   qtui --config <path> --script open-site,open-file,ask-ai,await   # headless

mod host;
mod interactive;
mod script;

use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || matches!(args[0].as_str(), "--version" | "-V" | "version") {
        println!("qtui {VERSION}");
        return ExitCode::SUCCESS;
    }

    let opts = match Options::parse(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("qtui: {e}");
            eprintln!(
                "usage: qtui [--version] [--config <path>] [--site <name>] [--script <steps>]"
            );
            return ExitCode::from(2);
        }
    };

    let result = if let Some(steps) = &opts.script {
        script::run(&opts.config, opts.site.as_deref(), steps)
    } else {
        interactive::run(&opts.config, opts.site.as_deref())
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("qtui: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Parsed command-line options.
struct Options {
    config: std::path::PathBuf,
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
            config: config.ok_or("--config is required")?,
            site,
            script,
        })
    }
}
