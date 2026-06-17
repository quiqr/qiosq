//! Agent adapter: the bridge between the read-only left pane and the coding
//! agent that does all the writing.
//!
//! The [`Agent`] trait keeps Claude Code swappable (and lets a [`FakeAgent`]
//! drive the bridge in deterministic, offline tests). The observable behaviour
//! lives in pure functions — [`format_intent`] and [`contains_sentinel`] — so it
//! is testable without any agent at all.
//!
//! The real [`RmuxAgent`] runs the agent in a **detached** rmux session with its
//! workdir pinned to the site repo; the user is never attached to the raw
//! session. It is daemon-backed: `rmux-sdk` needs a running rmux daemon that is
//! not yet available in the Nix/CI sandbox (tracked by bean `qiosq-rts9`), so its
//! live path is exercised manually, while the bridge logic is covered through the
//! fake agent.

use thiserror::Error;

/// The intent prefix injected before the user's plain-language request. The
/// trailing space hands the cursor to the user mid-sentence.
const INTENT_SUFFIX: &str = " I want to do the following… ";

/// A rendered snapshot of the agent pane's visible text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaneSnapshot {
    /// The visible lines of the pane, top to bottom.
    pub lines: Vec<String>,
}

impl PaneSnapshot {
    /// The whole snapshot as a single newline-joined string.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

/// Errors from the agent bridge.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent has not been started")]
    NotStarted,

    #[error("agent backend error: {0}")]
    Backend(String),
}

/// Context handed to the agent at start so it can constrain its output to valid
/// Quiqr front matter. For E6 this is an opaque string (built from field schemas
/// by the host); the seam exists so E6 need not be revisited later.
pub type SchemaContext = String;

/// The coding-agent abstraction. Claude Code (over rmux) is the production impl;
/// [`FakeAgent`] is the test impl. All site writes happen inside the agent's own
/// process — implementations of this trait never write site files themselves.
pub trait Agent {
    /// Start a session pinned to `workdir`, optionally seeding `schema_context`.
    fn start(
        &mut self,
        workdir: &std::path::Path,
        schema_context: &SchemaContext,
    ) -> Result<(), AgentError>;

    /// Inject the file intent (`@{path} I want to do the following… `) plus any
    /// extra text, leaving the cursor for the user. Sends text only; writes no
    /// files.
    fn send_intent(&mut self, file_ref: &str, extra: &str) -> Result<(), AgentError>;

    /// The agent's output accumulated so far (what completion is detected over).
    fn output(&self) -> String;

    /// A snapshot of the agent pane's visible text.
    fn snapshot(&self) -> Result<PaneSnapshot, AgentError>;

    /// Whether the configured completion `sentinel` has appeared in the output.
    fn is_complete(&self, sentinel: &str) -> bool {
        contains_sentinel(&self.output(), sentinel)
    }
}

/// Format the intent text injected for `file_ref`:
/// `@{file_ref} I want to do the following… ` (trailing space intentional).
pub fn format_intent(file_ref: &str) -> String {
    format!("@{file_ref}{INTENT_SUFFIX}")
}

/// Whether `output` contains the completion `sentinel`. An empty sentinel never
/// matches (it would otherwise complete immediately).
pub fn contains_sentinel(output: &str, sentinel: &str) -> bool {
    !sentinel.is_empty() && output.contains(sentinel)
}

// ----- FakeAgent (in-process test double) -----------------------------------

/// An in-process [`Agent`] for tests: records what was sent and replays a
/// scripted output buffer. No process, no I/O.
#[derive(Debug, Default)]
pub struct FakeAgent {
    started_in: Option<std::path::PathBuf>,
    /// Everything sent to the agent (intents + extra text), in order.
    pub sent: Vec<String>,
    /// The output the fake "agent" has produced so far.
    output: String,
}

impl FakeAgent {
    /// Create a fake agent with an empty output buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulate the agent emitting `text` (e.g. progress, or the sentinel).
    pub fn emit(&mut self, text: &str) {
        self.output.push_str(text);
    }

    /// The working directory the agent was started in, if any.
    pub fn started_in(&self) -> Option<&std::path::Path> {
        self.started_in.as_deref()
    }
}

impl Agent for FakeAgent {
    fn start(
        &mut self,
        workdir: &std::path::Path,
        _schema_context: &SchemaContext,
    ) -> Result<(), AgentError> {
        self.started_in = Some(workdir.to_path_buf());
        Ok(())
    }

    fn send_intent(&mut self, file_ref: &str, extra: &str) -> Result<(), AgentError> {
        if self.started_in.is_none() {
            return Err(AgentError::NotStarted);
        }
        let mut msg = format_intent(file_ref);
        msg.push_str(extra);
        self.sent.push(msg);
        Ok(())
    }

    fn output(&self) -> String {
        self.output.clone()
    }

    fn snapshot(&self) -> Result<PaneSnapshot, AgentError> {
        Ok(PaneSnapshot {
            lines: self.output.lines().map(str::to_string).collect(),
        })
    }
}

// ----- RmuxAgent (real, daemon-backed; live path is manual) -----------------

pub mod rmux;
pub use rmux::RmuxAgent;
