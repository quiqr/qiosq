//! Agent adapter.
//!
//! Epic E6 fills this in: a `trait Agent`
//! (`start`/`send_intent`/`await_completion`/`snapshot`) with a Claude Code
//! implementation over `rmux-sdk` — a detached session, the agent in pane
//! `(0,0)`, workdir pinned to the opened site repo, restricted permissions.
//! `send_intent` injects `@{path} I want to do the following… ` and hands the
//! cursor to the user; completion is detected via the configured sentinel.
//!
//! In E1 this is an empty-but-compiling stub establishing the crate boundary.
