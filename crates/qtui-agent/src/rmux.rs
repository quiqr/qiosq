//! The real, daemon-backed agent over `rmux-sdk`.
//!
//! `RmuxAgent` runs the coding agent in a **detached** rmux session (pane
//! `(0,0)`) with its workdir pinned to the opened site. It wraps the async SDK
//! behind a blocking facade so callers and tests stay synchronous.
//!
//! **Runtime requirement (`qiosq-rts9`):** `rmux-sdk` connects to a running rmux
//! daemon. That daemon is not yet available in the Nix/CI sandbox, so the live
//! path here is exercised manually (see the `#[ignore]`d test below), while the
//! bridge *logic* is covered through [`crate::FakeAgent`] in CI. This type still
//! compiles against the real SDK so API drift is caught at build time.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rmux_sdk::{EnsureSession, EnsureSessionPolicy, Rmux, SessionName};
use tokio::runtime::Runtime;

use crate::{format_intent, Agent, AgentError, PaneSnapshot, SchemaContext};

/// A coding-agent session driven over the rmux daemon.
#[derive(Debug)]
pub struct RmuxAgent {
    rt: Runtime,
    session_name: String,
    default_timeout: Duration,
    workdir: Option<PathBuf>,
    /// Output accumulated from pane snapshots (the daemon is the source of truth;
    /// we cache the latest visible text for `output`/`is_complete`).
    last_output: String,
}

impl RmuxAgent {
    /// Create an agent that will drive the rmux session named `session_name`.
    /// Does not connect yet — [`Agent::start`] connects and ensures the session.
    pub fn new(session_name: impl Into<String>) -> Result<Self, AgentError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AgentError::Backend(format!("tokio runtime: {e}")))?;
        Ok(Self {
            rt,
            session_name: session_name.into(),
            default_timeout: Duration::from_secs(5),
            workdir: None,
            last_output: String::new(),
        })
    }

    /// Connect to (or start) the daemon and ensure a detached session whose pane
    /// working directory is `workdir`. Separated from the trait method so the
    /// async body is in one place.
    fn ensure(&self, workdir: &Path) -> Result<(), AgentError> {
        let name = SessionName::new(&self.session_name)
            .map_err(|e| AgentError::Backend(format!("session name: {e}")))?;
        let timeout = self.default_timeout;
        let workdir = workdir.to_path_buf();
        self.rt.block_on(async move {
            let rmux = Rmux::builder()
                .default_timeout(timeout)
                .connect_or_start()
                .await
                .map_err(|e| AgentError::Backend(format!("connect_or_start: {e}")))?;
            rmux.ensure_session(
                EnsureSession::named(name)
                    .policy(EnsureSessionPolicy::CreateOrReuse)
                    .detached(true)
                    .working_directory(workdir.to_string_lossy().into_owned()),
            )
            .await
            .map_err(|e| AgentError::Backend(format!("ensure_session: {e}")))?;
            Ok::<(), AgentError>(())
        })
    }

    fn refresh_snapshot(&self) -> Result<PaneSnapshot, AgentError> {
        let name = SessionName::new(&self.session_name)
            .map_err(|e| AgentError::Backend(format!("session name: {e}")))?;
        let timeout = self.default_timeout;
        self.rt.block_on(async move {
            let rmux = Rmux::builder()
                .default_timeout(timeout)
                .connect_or_start()
                .await
                .map_err(|e| AgentError::Backend(format!("connect_or_start: {e}")))?;
            let session = rmux
                .ensure_session(EnsureSession::named(name).policy(EnsureSessionPolicy::ReuseOnly))
                .await
                .map_err(|e| AgentError::Backend(format!("reuse session: {e}")))?;
            let snap = session
                .pane(0, 0)
                .snapshot()
                .await
                .map_err(|e| AgentError::Backend(format!("snapshot: {e}")))?;
            Ok::<PaneSnapshot, AgentError>(PaneSnapshot {
                lines: snap.visible_lines(),
            })
        })
    }

    fn send_text(&self, text: &str) -> Result<(), AgentError> {
        let name = SessionName::new(&self.session_name)
            .map_err(|e| AgentError::Backend(format!("session name: {e}")))?;
        let timeout = self.default_timeout;
        let text = text.to_string();
        self.rt.block_on(async move {
            let rmux = Rmux::builder()
                .default_timeout(timeout)
                .connect_or_start()
                .await
                .map_err(|e| AgentError::Backend(format!("connect_or_start: {e}")))?;
            let session = rmux
                .ensure_session(EnsureSession::named(name).policy(EnsureSessionPolicy::ReuseOnly))
                .await
                .map_err(|e| AgentError::Backend(format!("reuse session: {e}")))?;
            session
                .pane(0, 0)
                .send_text(text)
                .await
                .map_err(|e| AgentError::Backend(format!("send_text: {e}")))?;
            Ok::<(), AgentError>(())
        })
    }
}

impl Agent for RmuxAgent {
    fn start(&mut self, workdir: &Path, _schema_context: &SchemaContext) -> Result<(), AgentError> {
        self.ensure(workdir)?;
        self.workdir = Some(workdir.to_path_buf());
        Ok(())
    }

    fn send_intent(&mut self, file_ref: &str, extra: &str) -> Result<(), AgentError> {
        if self.workdir.is_none() {
            return Err(AgentError::NotStarted);
        }
        // Inject the intent prefix + any extra text, WITHOUT a trailing newline,
        // so the cursor is left mid-line for the user to finish the sentence.
        let mut msg = format_intent(file_ref);
        msg.push_str(extra);
        self.send_text(&msg)
    }

    fn output(&self) -> String {
        // Best-effort: refresh from the live pane; fall back to the cache.
        match self.refresh_snapshot() {
            Ok(snap) => snap.text(),
            Err(_) => self.last_output.clone(),
        }
    }

    fn snapshot(&self) -> Result<PaneSnapshot, AgentError> {
        self.refresh_snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_without_a_daemon() {
        // Constructing the agent must not require a daemon (no connection yet).
        let agent = RmuxAgent::new("qtui-test").expect("construct");
        assert_eq!(agent.session_name, "qtui-test");
        assert!(agent.workdir.is_none());
    }

    // The live path needs a running rmux daemon (bean qiosq-rts9), which is not
    // available in CI. Run manually with: `cargo test -p qtui-agent -- --ignored`
    // in an environment with the rmux daemon installed.
    #[test]
    #[ignore = "needs a running rmux daemon (qiosq-rts9)"]
    fn live_start_and_send() {
        let dir = std::env::temp_dir();
        let mut agent = RmuxAgent::new("qtui-live-test").expect("construct");
        agent.start(&dir, &String::new()).expect("start");
        agent
            .send_intent("content/x.md", "hello")
            .expect("send_intent");
        let _ = agent.snapshot().expect("snapshot");
    }
}
