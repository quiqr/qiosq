//! Quiqr schema → navigation model.
//!
//! Parses a site's `quiqr/model/base.yaml` and `quiqr/model/includes/*` into a
//! read-only [`NavigationModel`]: an ordered Menu of [`Single`]s and
//! [`Collection`]s, each mapped to the content file/folder it represents, with
//! per-entity [`Field`] schemas for constraining agent output.
//!
//! Parsing is **tolerant** — real sites carry partial, legacy, and occasionally
//! malformed schemas, and the browser must never crash on one. Unknown keys are
//! ignored, a malformed include is skipped (recorded in
//! [`NavigationModel::warnings`]), `_mergePartial` references are recorded but
//! never fetched (no network), and an absent/empty model yields an empty
//! `NavigationModel`. The model is read-only; this crate never writes.

use std::path::{Path, PathBuf};

use serde_yaml::Value;

// The include roots recognised under `quiqr/model/includes/` are `menu`,
// `singles`, `collections`, and `dynamics`. Each may be a `<root>.yaml` file or
// a `<root>/` directory of YAML files; see `load_model` and `read_root`.

/// The assembled, read-only navigation model for a site.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NavigationModel {
    /// Menu groups, in schema order.
    pub menu: Vec<MenuGroup>,
    /// Singles, keyed by their schema `key`, in discovery order.
    pub singles: Vec<Single>,
    /// Collections, keyed by their schema `key`, in discovery order.
    pub collections: Vec<Collection>,
    /// Non-fatal problems encountered while parsing (malformed includes,
    /// dangling menu references, …). Surfaced for tests and diagnostics.
    pub warnings: Vec<String>,
}

/// One top-level menu group (e.g. "Content") with its entries, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuGroup {
    pub key: String,
    pub title: String,
    pub entries: Vec<MenuEntry>,
}

/// A resolved menu entry: a reference to a single or a collection by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    Single(String),
    Collection(String),
}

/// A schema-defined single content entity (e.g. the About page).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Single {
    pub key: String,
    pub title: String,
    /// The content file this single edits, if declared (`file:` in the schema).
    pub file: Option<String>,
    pub fields: Vec<Field>,
    /// A recorded `_mergePartial` reference; never fetched.
    pub merge_partial: Option<String>,
}

/// A schema-defined set of like items (e.g. blog posts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    pub key: String,
    pub title: String,
    /// The folder this collection manages (`folder:` in the schema).
    pub folder: Option<String>,
    pub fields: Vec<Field>,
    pub merge_partial: Option<String>,
}

/// A field definition. `type_` is a free string (Quiqr has many widget types).
/// Nested fields are preserved (e.g. a `bundle-manager` containing a `thumb`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: String,
    pub title: String,
    pub type_: String,
    pub fields: Vec<Field>,
}

impl NavigationModel {
    /// Look up a single by key.
    pub fn single(&self, key: &str) -> Option<&Single> {
        self.singles.iter().find(|s| s.key == key)
    }

    /// Look up a collection by key.
    pub fn collection(&self, key: &str) -> Option<&Collection> {
        self.collections.iter().find(|c| c.key == key)
    }
}

/// Load and assemble the navigation model for the site rooted at `site_dir`.
///
/// Reads `<site_dir>/quiqr/model/`. Never returns an error for content problems;
/// an absent or empty model yields an empty [`NavigationModel`], and malformed
/// includes are skipped and recorded in [`NavigationModel::warnings`].
pub fn load_model(site_dir: &Path) -> NavigationModel {
    let model_dir = site_dir.join("quiqr").join("model");
    let mut model = NavigationModel::default();
    if !model_dir.is_dir() {
        return model;
    }

    let includes = model_dir.join("includes");

    // Collections and singles first, so menu resolution can reference them.
    for value in read_root(&includes, "collections", &mut model.warnings) {
        if let Some(c) = parse_collection(&value, &mut model.warnings) {
            model.collections.push(c);
        }
    }
    for value in read_root(&includes, "singles", &mut model.warnings) {
        if let Some(s) = parse_single(&value, &mut model.warnings) {
            model.singles.push(s);
        }
    }

    // Menu: resolve each item's key against the known singles/collections.
    // Collect into locals first so we are not borrowing `model` immutably (for
    // the lookups) and mutably (for `warnings`) at the same time.
    let menu_values = read_root(&includes, "menu", &mut model.warnings);
    let single_keys: Vec<&str> = model.singles.iter().map(|s| s.key.as_str()).collect();
    let collection_keys: Vec<&str> = model.collections.iter().map(|c| c.key.as_str()).collect();
    let mut groups = Vec::new();
    let mut menu_warnings = Vec::new();
    for group in &menu_values {
        if let Some(g) = parse_menu_group(group, &single_keys, &collection_keys, &mut menu_warnings)
        {
            groups.push(g);
        }
    }
    model.menu = groups;
    model.warnings.extend(menu_warnings);

    // `dynamics` is parsed leniently for completeness but not yet surfaced in
    // the menu (no UI consumer in M1). Touch the root so a malformed file is
    // still recorded as a warning.
    let _ = read_root(&includes, "dynamics", &mut model.warnings);

    model
}

/// Read an include root (`<root>.yaml` file or `<root>/` directory) and return
/// the top-level sequence entries it defines. Each root is expected to be a YAML
/// sequence; a mapping or scalar yields a warning and no entries. A malformed or
/// unreadable file is skipped with a warning.
fn read_root(includes: &Path, root: &str, warnings: &mut Vec<String>) -> Vec<Value> {
    let file = includes.join(format!("{root}.yaml"));
    let dir = includes.join(root);

    let mut files: Vec<PathBuf> = Vec::new();
    if file.is_file() {
        files.push(file);
    }
    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut yamls: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && matches!(p.extension().and_then(|e| e.to_str()), Some("yaml" | "yml"))
                })
                .collect();
            yamls.sort();
            files.extend(yamls);
        }
    }

    let mut out = Vec::new();
    for path in files {
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_yaml::from_str::<Value>(&text) {
                Ok(Value::Sequence(seq)) => out.extend(seq),
                Ok(_) => warnings.push(format!(
                    "{}: expected a YAML sequence, ignoring",
                    path.display()
                )),
                Err(e) => warnings.push(format!("{}: invalid YAML, skipped ({e})", path.display())),
            },
            Err(e) => warnings.push(format!("{}: unreadable, skipped ({e})", path.display())),
        }
    }
    out
}

fn parse_collection(v: &Value, warnings: &mut Vec<String>) -> Option<Collection> {
    let key = require_key(v, "collection", warnings)?;
    Some(Collection {
        title: str_field(v, "title").unwrap_or_else(|| key.clone()),
        folder: str_field(v, "folder"),
        fields: parse_fields(v.get("fields")),
        merge_partial: str_field(v, "_mergePartial"),
        key,
    })
}

fn parse_single(v: &Value, warnings: &mut Vec<String>) -> Option<Single> {
    let key = require_key(v, "single", warnings)?;
    Some(Single {
        title: str_field(v, "title").unwrap_or_else(|| key.clone()),
        file: str_field(v, "file"),
        fields: parse_fields(v.get("fields")),
        merge_partial: str_field(v, "_mergePartial"),
        key,
    })
}

/// Parse a `fields:` value (a sequence) into `Field`s, recursively. A non-
/// sequence or absent value yields an empty list.
fn parse_fields(v: Option<&Value>) -> Vec<Field> {
    let Some(Value::Sequence(seq)) = v else {
        return Vec::new();
    };
    seq.iter().filter_map(parse_field).collect()
}

fn parse_field(v: &Value) -> Option<Field> {
    // A field without a key is meaningless; skip it.
    let key = str_field(v, "key")?;
    Some(Field {
        title: str_field(v, "title").unwrap_or_else(|| key.clone()),
        // Default the widget type to "string" when omitted (Quiqr's default).
        type_: str_field(v, "type").unwrap_or_else(|| "string".to_string()),
        fields: parse_fields(v.get("fields")),
        key,
    })
}

fn parse_menu_group(
    v: &Value,
    single_keys: &[&str],
    collection_keys: &[&str],
    warnings: &mut Vec<String>,
) -> Option<MenuGroup> {
    let key = require_key(v, "menu group", warnings)?;
    let title = str_field(v, "title").unwrap_or_else(|| key.clone());

    let mut entries = Vec::new();
    if let Some(Value::Sequence(items)) = v.get("menuItems") {
        for item in items {
            let Some(item_key) = str_field(item, "key") else {
                continue;
            };
            if collection_keys.contains(&item_key.as_str()) {
                entries.push(MenuEntry::Collection(item_key));
            } else if single_keys.contains(&item_key.as_str()) {
                entries.push(MenuEntry::Single(item_key));
            } else {
                warnings.push(format!(
                    "menu group '{key}': item '{item_key}' references no known single or collection, dropped"
                ));
            }
        }
    }

    Some(MenuGroup {
        key,
        title,
        entries,
    })
}

/// Extract a string field from a YAML mapping, if present and string-valued.
fn str_field(v: &Value, name: &str) -> Option<String> {
    v.get(name)?.as_str().map(|s| s.to_string())
}

/// Require a non-empty `key`; record a warning and return `None` otherwise.
fn require_key(v: &Value, what: &str, warnings: &mut Vec<String>) -> Option<String> {
    match str_field(v, "key") {
        Some(k) if !k.is_empty() => Some(k),
        _ => {
            warnings.push(format!("{what} entry without a 'key', skipped"));
            None
        }
    }
}
