//! quiqr-tui (codename *Reveal*) — the kiosk binary.
//!
//! Epic E1: this entrypoint only reports its version, proving the workspace
//! wires together and `nix build` produces a runnable binary. Later epics load
//! the config, construct the storage/model/preview/agent services, and run the
//! UI loop and mode state machine here.

// Bring every workspace member into the dependency graph so the binary build
// fails if any of them stops compiling. `qtui-config` is exercised for real;
// the rest are stubs until their epics (E2–E7).
use qtui_agent as _;
use qtui_model as _;
use qtui_preview as _;
use qtui_storage as _;
use qtui_ui as _;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("--version" | "-V" | "version") | None => {
            println!("qtui {VERSION}");
        }
        Some(other) => {
            eprintln!("qtui {VERSION}: unknown argument {other:?}");
            eprintln!("usage: qtui [--version]");
            std::process::exit(2);
        }
    }

    // Touch qtui-config's public surface so the link is real, not just declared.
    debug_assert!(qtui_config::EXAMPLE_CONFIG_FILENAME.ends_with(".toml"));
}
