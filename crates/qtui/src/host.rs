//! Shared host glue: owns the config + services + `AppState`, and performs the
//! side effects behind each UI transition (open site → preview + model; open
//! file → read bytes; ask AI → run the agent). Reused by both run modes.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use qtui_agent::{Agent, FakeAgent, RmuxAgent};
use qtui_config::Config;
use qtui_preview::PreviewServer;
use qtui_storage::{enumerate_sites, Site};
use qtui_ui::AppState;

/// The running host: config, the discovered sites, the active preview (if any),
/// the live agent session (if started), and the UI state the renderer reads.
pub struct Host {
    pub config: Config,
    pub state: AppState,
    sites: Vec<Site>,
    preview: Option<PreviewServer>,
    /// The live coding-agent session, started lazily on the first "Ask AI"
    /// (interactive mode) and reused for the rest of the run.
    agent: Option<Box<dyn Agent>>,
}

impl Host {
    /// Load the config and enumerate sites; start in the site list.
    pub fn load(config_path: &Path) -> Result<Self, String> {
        let config = Config::load_and_validate(config_path).map_err(|e| e.to_string())?;
        let data_dir = PathBuf::from(&config.storage.quiqr_data_dir);
        let sites = enumerate_sites(&data_dir).map_err(|e| e.to_string())?;
        let state = AppState::new(sites.clone(), config.ui.show_schema_nav_first);
        Ok(Self {
            config,
            state,
            sites,
            preview: None,
            agent: None,
        })
    }

    /// The site with the given name, if present.
    pub fn site_by_name(&self, name: &str) -> Option<&Site> {
        self.sites.iter().find(|s| s.name == name)
    }

    /// All discovered sites.
    pub fn sites(&self) -> &[Site] {
        &self.sites
    }

    /// Open a site: start its Hugo preview, load its schema model + content
    /// tree, and push both into the UI state. Surfaces the preview URL.
    pub fn open_site(&mut self, site: &Site) -> Result<(), String> {
        // One preview at a time: stop any previous one first.
        if let Some(prev) = self.preview.take() {
            prev.stop();
        }
        let server = PreviewServer::start(&self.config, &site.path).map_err(|e| e.to_string())?;
        self.state.set_preview_url(Some(server.url().to_string()));
        self.preview = Some(server);

        let content = qtui_storage::content_tree(site, &self.config.storage.hidden_dirs);
        let model = qtui_model::load_model(&site.path);
        self.state.set_open_site(content, model);
        Ok(())
    }

    /// Read a content file (relative to the site's `content/`) and push it into
    /// the read-only viewer. The host reads; the UI never touches the FS.
    pub fn open_file(&mut self, site: &Site, rel_path: &str) -> Result<(), String> {
        let full = site.path.join("content").join(rel_path);
        let contents =
            std::fs::read_to_string(&full).map_err(|e| format!("read {}: {e}", full.display()))?;
        self.state.set_open_file(rel_path, contents);
        Ok(())
    }

    /// Run the configured agent for an intent on `file_ref`, in the site's
    /// working directory: spawn `agent.command + args`, write the injected
    /// intent to its stdin, and wait for the completion sentinel in its output.
    /// Returns the agent's captured output (also pushed into the agent pane).
    ///
    /// This is the non-rmux path used by scripted/e2e runs and by any agent CLI
    /// that reads an intent on stdin (the fake agent does exactly this). The
    /// interactive rmux path lives in `qtui-agent::RmuxAgent`.
    pub fn run_agent_intent(&mut self, site: &Site, file_ref: &str) -> Result<String, String> {
        let intent = qtui_agent::format_intent(file_ref);
        let mut child = Command::new(&self.config.agent.command)
            .args(&self.config.agent.args)
            .current_dir(&site.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn agent '{}': {e}", self.config.agent.command))?;

        // Inject the intent; the harness/agent supplies the rest of the sentence.
        child
            .stdin
            .take()
            .ok_or("agent stdin unavailable")?
            .write_all(intent.as_bytes())
            .map_err(|e| format!("write intent: {e}"))?;

        let output = child
            .wait_with_output()
            .map_err(|e| format!("await agent: {e}"))?;
        let text = String::from_utf8_lossy(&output.stdout).into_owned();

        self.state
            .set_agent_output(text.lines().map(str::to_string).collect());

        if qtui_agent::contains_sentinel(&text, &self.config.agent.completion_sentinel) {
            Ok(text)
        } else {
            Err(format!(
                "agent finished without the completion sentinel {:?}",
                self.config.agent.completion_sentinel
            ))
        }
    }

    // ----- Live agent (E8) ---------------------------------------------------

    /// Build the agent implementation chosen by config: the in-process fake
    /// agent when `agent.command` names the in-tree `fake-agent` binary,
    /// otherwise the real rmux-backed agent (a detached session per site).
    fn build_agent(&self, site: &Site) -> Box<dyn Agent> {
        let is_fake = Path::new(&self.config.agent.command)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == "fake-agent")
            .unwrap_or(false);
        if is_fake {
            Box::new(FakeAgent::new())
        } else {
            // A stable, sanitised session name per site so re-opening reuses it.
            let session = sanitize_session(&format!("qtui-{}", site.name));
            match RmuxAgent::new(session) {
                Ok(a) => Box::new(a),
                // Constructing the runtime failed (very rare): fall back to a
                // fake so the UI still runs rather than crashing.
                Err(_) => Box::new(FakeAgent::new()),
            }
        }
    }

    /// Lazily start the live agent (once), pinned to `site`'s working copy, and
    /// reuse it thereafter. Returns an error string if starting fails.
    pub fn ensure_agent_started(&mut self, site: &Site) -> Result<(), String> {
        if self.agent.is_none() {
            let mut agent = self.build_agent(site);
            agent
                .start(&site.path, &String::new())
                .map_err(|e| format!("start agent: {e}"))?;
            self.agent = Some(agent);
        }
        Ok(())
    }

    /// Send the open file's intent to the live agent session (interactive
    /// "Ask AI"). Starts the agent lazily if needed. Sends text only — writes
    /// no site files.
    pub fn send_live_intent(&mut self, site: &Site, file_ref: &str) -> Result<(), String> {
        self.ensure_agent_started(site)?;
        let agent = self.agent.as_mut().expect("agent started above");
        agent
            .send_intent(file_ref, "")
            .map_err(|e| format!("send intent: {e}"))
    }

    /// Refresh the agent pane from the live session's snapshot (called each
    /// render tick). No-op when no session is running. When the agent has
    /// reported completion (its output contains the configured sentinel) a
    /// status line is appended so the user sees the task finished. Snapshot
    /// errors are surfaced into the pane text rather than propagated, so a
    /// transient failure never crashes the loop.
    pub fn poll_agent(&mut self) {
        let Some(agent) = self.agent.as_ref() else {
            return;
        };
        let complete = agent.is_complete(&self.config.agent.completion_sentinel);
        match agent.snapshot() {
            Ok(snap) => {
                let mut lines = snap.lines;
                if complete {
                    lines.push(String::new());
                    lines.push("— task complete —".to_string());
                }
                self.state.set_agent_output(lines);
            }
            Err(e) => self
                .state
                .set_agent_output(vec![format!("(agent snapshot unavailable: {e})")]),
        }
    }

    /// The running preview URL, if any.
    pub fn preview_url(&self) -> Option<&str> {
        self.preview.as_ref().map(|p| p.url())
    }

    /// Stop the preview server (called on exit). Idempotent.
    pub fn shutdown(&mut self) {
        if let Some(prev) = self.preview.take() {
            prev.stop();
        }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Sanitise a string into a conservative rmux session name: keep ASCII
/// alphanumerics, map everything else to `-`, and avoid an empty result.
fn sanitize_session(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    if s.is_empty() {
        "qtui-session".to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_session_keeps_alnum_and_replaces_rest() {
        assert_eq!(sanitize_session("qtui-my site!"), "qtui-my-site-");
        assert_eq!(sanitize_session("abc123"), "abc123");
        assert_eq!(sanitize_session(""), "qtui-session");
    }

    /// Build a Host over the anonymized real-site fixture (a data dir), with the
    /// given agent command, so we can exercise the live-agent host logic.
    fn host_with_agent(agent_command: &str) -> (tempfile::TempDir, Host) {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../qtui-storage/tests/fixtures/real-site");
        let tmp = tempfile::tempdir().unwrap();
        // Copy the fixture (a data dir) into the temp data dir.
        copy_dir(&fixture, tmp.path());
        let cfg = tmp.path().join("qtui.toml");
        std::fs::write(
            &cfg,
            format!(
                "[storage]\nquiqr_data_dir = \"{}\"\n\
                 [preview]\nport_range = [13140, 13200]\n\
                 [agent]\ncommand = \"{}\"\ncompletion_sentinel = \"<<QTUI_TASK_DONE>>\"\n",
                tmp.path().display(),
                agent_command,
            ),
        )
        .unwrap();
        let host = Host::load(&cfg).unwrap();
        (tmp, host)
    }

    fn copy_dir(from: &Path, to: &Path) {
        for entry in std::fs::read_dir(from).unwrap().flatten() {
            let dst = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                std::fs::create_dir_all(&dst).unwrap();
                copy_dir(&entry.path(), &dst);
            } else {
                std::fs::copy(entry.path(), &dst).unwrap();
            }
        }
    }

    #[test]
    fn build_agent_selects_fake_for_fake_agent_command() {
        let (_tmp, host) = host_with_agent("/some/path/fake-agent");
        let site = host.sites()[0].clone();
        // The fake agent has not connected to anything; starting it succeeds
        // with no daemon. (RmuxAgent would also construct, but selecting fake is
        // what keeps interactive/CI runs daemon-free.)
        let agent = host.build_agent(&site);
        // Observable proxy: a freshly built FakeAgent reports not-started, and
        // its snapshot is empty — no connection attempt.
        assert!(agent.snapshot().is_ok());
        assert_eq!(agent.output(), "");
    }

    #[test]
    fn live_intent_starts_once_and_poll_populates_pane() {
        let (_tmp, mut host) = host_with_agent("fake-agent");
        let site = host.sites()[0].clone();

        // First Ask AI starts the (fake) session and sends the intent.
        host.send_live_intent(&site, "homepage/about.md").unwrap();
        assert!(host.agent.is_some(), "agent started lazily on first intent");

        // A second intent reuses the same session (no re-start, no error).
        host.send_live_intent(&site, "about.md").unwrap();

        // Polling the agent refreshes the pane without crashing (fake agent has
        // no output, so the pane is empty — but the call path works).
        host.poll_agent();
        // The UI received an (empty) agent-output update, not a panic.
        assert!(host.state.agent_output().is_empty());
    }
}
