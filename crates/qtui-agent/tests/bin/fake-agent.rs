//! A tiny scripted stand-in for the coding agent, for deterministic, offline
//! tests (and the E7 e2e harness) — never a real LLM.
//!
//! Behaviour:
//! - Reads one line of "intent" from stdin (the `@{path} …` the left pane
//!   injects, finished by the harness).
//! - Optionally writes a known file: if `QTUI_FAKE_WRITE` is set to a path, it
//!   writes a short marker there (simulating the agent writing content).
//! - Prints the completion sentinel so the bridge's `wait_for_text` / sentinel
//!   detection resolves. The sentinel is taken from `QTUI_FAKE_SENTINEL` (default
//!   `<<QTUI_TASK_DONE>>`).
//!
//! This binary intentionally has no dependency on the rest of the workspace; it
//! is a self-contained process the tests spawn.

use std::io::{Read, Write};

fn main() {
    let sentinel =
        std::env::var("QTUI_FAKE_SENTINEL").unwrap_or_else(|_| "<<QTUI_TASK_DONE>>".to_string());

    // Consume whatever the harness sends (the finished intent). We don't need to
    // act on it for the PoC fake; reading it proves the pipe works.
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);

    // Simulate "the agent wrote content" when asked to.
    if let Ok(path) = std::env::var("QTUI_FAKE_WRITE") {
        let body = format!("written by fake-agent\nintent: {}\n", input.trim());
        // Best-effort; if the parent dir is missing the test will notice.
        let _ = std::fs::write(&path, body);
    }

    let mut out = std::io::stdout();
    let _ = writeln!(out, "fake-agent: done");
    // Print the sentinel last so a reader waiting for it sees a completed task.
    let _ = writeln!(out, "{sentinel}");
    let _ = out.flush();
}
