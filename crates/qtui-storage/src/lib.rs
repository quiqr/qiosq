//! Quiqr/Hugo read-only storage layer.
//!
//! Epic E2 fills this in: locate the Quiqr data dir from config, enumerate
//! sites (a subdir with `config.*` + a `quiqr/` folder), and expose a
//! read-only `content/` tree with derived/generated dirs hidden. This crate
//! never writes — site mutation happens only through the agent.
//!
//! In E1 this is an empty-but-compiling stub establishing the crate boundary.
