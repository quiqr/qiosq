//! Shared host glue: owns the config + services + `AppState`, and performs the
//! side effects behind each UI transition (open site → preview + model; open
//! file → read bytes; ask AI → run the agent). Reused by both run modes.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use qtui_config::Config;
use qtui_preview::PreviewServer;
use qtui_storage::{enumerate_sites, Site};
use qtui_ui::AppState;

/// The running host: config, the discovered sites, the active preview (if any),
/// and the UI state the renderer reads.
pub struct Host {
    pub config: Config,
    pub state: AppState,
    sites: Vec<Site>,
    preview: Option<PreviewServer>,
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
