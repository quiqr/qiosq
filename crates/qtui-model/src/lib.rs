//! Quiqr schema -> navigation model.
//!
//! Epic E3 fills this in: parse `quiqr/model/base.yaml`, merge
//! `quiqr/model/includes/*` (each a top-level root: collections, menu, singles,
//! dynamics), and produce a `NavigationModel` plus per-collection/single field
//! schemas used to constrain the agent prompt. The model is read-only and must
//! tolerate partial/legacy/malformed schemas without panicking.
//!
//! In E1 this is an empty-but-compiling stub establishing the crate boundary.
