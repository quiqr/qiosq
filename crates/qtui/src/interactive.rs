//! Interactive mode: a crossterm raw-mode terminal loop driving the WP5.1 UI.
//!
//! Reads key events, advances the `qtui-ui` state machine, performs the side
//! effects each transition implies through the [`Host`], and renders each tick.
//! The agent pane (interactive path) is left to the rmux-backed agent in a later
//! refinement; here it shows the host's last agent output, if any.

use std::io::stdout;
use std::path::Path;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::{execute, terminal};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

use qtui_ui::{render, update, Action, Mode};

use crate::host::Host;

pub fn run(config_path: &Path, site_name: Option<&str>) -> Result<(), String> {
    let mut host = Host::load(config_path)?;

    // If a site was named on the command line, pre-select it so Enter opens it.
    if let Some(name) = site_name {
        if host.site_by_name(name).is_none() {
            return Err(format!("site {name:?} not found"));
        }
    }

    // Enter the alternate screen + raw mode; restore on the way out no matter
    // what (a guard would be nicer, but explicit teardown keeps deps minimal).
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    let mut out = stdout();
    execute!(out, terminal::EnterAlternateScreen).map_err(|e| e.to_string())?;
    let mut term = Terminal::new(CrosstermBackend::new(out)).map_err(|e| e.to_string())?;

    let loop_result = event_loop(&mut term, &mut host);

    // Teardown (best-effort) before propagating any loop error.
    let _ = terminal::disable_raw_mode();
    let _ = execute!(term.backend_mut(), terminal::LeaveAlternateScreen);
    host.shutdown();
    loop_result
}

fn event_loop<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    host: &mut Host,
) -> Result<(), String> {
    loop {
        // Refresh the live agent pane every tick (no-op when no session is
        // running) so its output streams even between keypresses.
        host.poll_agent();

        term.draw(|f| render(f, &host.state))
            .map_err(|e| e.to_string())?;

        // Wait briefly for a key; on timeout, loop to re-poll + redraw.
        if !event::poll(Duration::from_millis(200)).map_err(|e| e.to_string())? {
            continue;
        }
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Note the mode *before* update so we can detect open transitions.
        let before = host.state.mode().clone();
        let action = update(&mut host.state, key);

        handle_transition(host, &before)?;

        if let Some(action) = action {
            match action {
                Action::Quit => return Ok(()),
                Action::AskAi { file } => {
                    // Send the intent to the LIVE agent session (started lazily,
                    // pinned to the site, reused across turns). Errors surface in
                    // the agent pane rather than crashing the UI; the next tick's
                    // poll_agent streams the response.
                    if let Some(site) = current_site(host) {
                        if let Err(e) = host.send_live_intent(&site, &file) {
                            host.state
                                .set_agent_output(vec![format!("agent error: {e}")]);
                        }
                    }
                }
                // New/Save/Discard are agent requests; wiring them to the rmux
                // session is a later refinement. They are intentionally no-ops in
                // the host loop for now (the UI already routed them as requests).
                Action::RequestNew | Action::RequestSave | Action::RequestDiscard => {}
            }
        }
    }
}

/// React to a mode change caused by the last key: when the UI entered Browse for
/// a site, open it (preview + model); when it entered ViewFile, load the file.
fn handle_transition(host: &mut Host, before: &Mode) -> Result<(), String> {
    match (before, host.state.mode().clone()) {
        // SiteList -> Browse: open the selected site.
        (Mode::SiteList, Mode::Browse { .. }) => {
            if let Some(site) = host.state.selected_site().cloned() {
                host.open_site(&site)?;
            }
        }
        // Browse -> ViewFile: load the opened file's contents.
        (Mode::Browse { .. }, Mode::ViewFile) => {
            if let (Some(site), Some(rel)) = (
                current_site(host),
                host.state.open_file().map(str::to_string),
            ) {
                // The UI set open_file to the path; fill in its contents.
                let _ = host.open_file(&site, &rel);
            }
        }
        _ => {}
    }
    Ok(())
}

/// The site currently being browsed (the UI's selected site).
fn current_site(host: &Host) -> Option<qtui_storage::Site> {
    host.state.selected_site().cloned()
}
