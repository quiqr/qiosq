//! Hugo preview lifecycle.
//!
//! Epic E5 fills this in: on site open, start `hugo server` on a free port in
//! the configured range (never Quiqr's default `:13131`), detect readiness,
//! surface the URL to the UI, and stop the server on close/exit. One server at
//! a time for the PoC.
//!
//! In E1 this is an empty-but-compiling stub establishing the crate boundary.
