//! WordPerfect-5.1-style two-pane TUI.
//!
//! Epic E4 fills this in: a two-pane ratatui layout (left browser/launcher,
//! right agent snapshot), a persistent context-sensitive function-key legend,
//! and a mode state machine (`SiteList` -> `Browse` -> `ViewFile` -> `Agent`).
//! All write-like verbs are requests routed to the agent, never direct file
//! ops. E7 renders the agent pane via `ratatui-rmux`.
//!
//! In E1 this is an empty-but-compiling stub establishing the crate boundary.
