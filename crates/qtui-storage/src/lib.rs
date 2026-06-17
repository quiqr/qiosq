//! Quiqr/Hugo read-only storage layer.
//!
//! Given the configured Quiqr data directory this crate:
//!
//! - [`enumerate_sites`] — discovers the sites (a subdir is a site when it has a
//!   Hugo config **and** a `quiqr/` folder), and
//! - [`content_tree`] — builds a read-only, filtered tree of a site's `content/`
//!   directory, hiding derived/generated/VCS directories.
//!
//! This crate **never writes** — site mutation is the agent's job (the
//! single-writer rule from `docs/01-architecture.md`). It is a pure crate: no
//! ratatui/rmux dependencies.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// The default set of directory names hidden from the content tree. Mirrors the
/// `storage.hidden_dirs` default in `config/quiqr-tui.example.toml`.
pub const DEFAULT_HIDDEN_DIRS: &[&str] = &["public", "resources", ".quiqr-cache", ".git", "themes"];

/// Errors from the storage layer.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Quiqr data directory not found or unreadable: {path} ({source})")]
    DataDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// A discovered Quiqr site: its directory name and absolute path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// The site's directory name (used as its display name).
    pub name: String,
    /// Absolute path to the site's root directory.
    pub path: PathBuf,
}

/// A node in a site's read-only `content/` tree. Paths are relative to
/// `content/` so the type does not depend on where the site lives on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentNode {
    /// A directory with its (already filtered + ordered) children.
    Dir {
        name: String,
        rel_path: PathBuf,
        children: Vec<ContentNode>,
    },
    /// A file leaf.
    File { name: String, rel_path: PathBuf },
}

impl ContentNode {
    /// The node's base name.
    pub fn name(&self) -> &str {
        match self {
            ContentNode::Dir { name, .. } | ContentNode::File { name, .. } => name,
        }
    }

    /// The node's path relative to `content/`.
    pub fn rel_path(&self) -> &Path {
        match self {
            ContentNode::Dir { rel_path, .. } | ContentNode::File { rel_path, .. } => rel_path,
        }
    }

    fn is_dir(&self) -> bool {
        matches!(self, ContentNode::Dir { .. })
    }
}

/// Return `true` if `dir` looks like a Quiqr site: it must contain a Hugo site
/// configuration (a `config.*` or `hugo.*` file, or a `config/` directory) **and**
/// a `quiqr/` directory.
pub fn is_quiqr_site(dir: &Path) -> bool {
    has_quiqr_dir(dir) && has_hugo_config(dir)
}

fn has_quiqr_dir(dir: &Path) -> bool {
    dir.join("quiqr").is_dir()
}

fn has_hugo_config(dir: &Path) -> bool {
    // A `config/` directory (Hugo's directory-based config) counts.
    if dir.join("config").is_dir() {
        return true;
    }
    // Otherwise any `config.*` or `hugo.*` file (toml/yaml/json/…).
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(stem) = name.split('.').next() {
            // Require an extension (`config.toml`), not a bare `config` file.
            if (stem == "config" || stem == "hugo") && name.contains('.') {
                return true;
            }
        }
    }
    false
}

/// Enumerate the Quiqr sites directly under `data_dir`, sorted by name.
///
/// Returns an empty list (not an error) when the directory exists but holds no
/// site-shaped subdirectories. Returns [`StorageError::DataDir`] naming the path
/// when `data_dir` is missing or unreadable. Never writes.
pub fn enumerate_sites(data_dir: &Path) -> Result<Vec<Site>, StorageError> {
    let entries = std::fs::read_dir(data_dir).map_err(|source| StorageError::DataDir {
        path: data_dir.to_path_buf(),
        source,
    })?;

    let mut sites = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        // Only immediate subdirectories are candidates.
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if is_quiqr_site(&path) {
            let name = entry.file_name().to_string_lossy().into_owned();
            sites.push(Site { name, path });
        }
    }
    sites.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sites)
}

/// Build the read-only content tree for `site`, hiding any directory whose base
/// name is in `hidden_dirs` (at every depth). Returns an empty `Vec` when the
/// site has no `content/` directory. Files are never hidden by this rule. Does
/// not follow symlinked directories. Never writes.
pub fn content_tree(site: &Site, hidden_dirs: &[String]) -> Vec<ContentNode> {
    let content_root = site.path.join("content");
    if !content_root.is_dir() {
        return Vec::new();
    }
    read_dir_filtered(&content_root, Path::new(""), hidden_dirs)
}

fn read_dir_filtered(abs_dir: &Path, rel_dir: &Path, hidden: &[String]) -> Vec<ContentNode> {
    let Ok(entries) = std::fs::read_dir(abs_dir) else {
        // Unreadable subdirectory: tolerate by yielding no children rather than
        // panicking (a real site must never crash the browser).
        return Vec::new();
    };

    let mut nodes = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // Use the entry's own file type; do NOT traverse symlinks as dirs.
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let rel_path = rel_dir.join(&name);

        if file_type.is_dir() {
            if hidden.iter().any(|h| h == &name) {
                continue;
            }
            let children = read_dir_filtered(&entry.path(), &rel_path, hidden);
            nodes.push(ContentNode::Dir {
                name,
                rel_path,
                children,
            });
        } else if file_type.is_file() {
            nodes.push(ContentNode::File { name, rel_path });
        }
        // Symlinks and other types are skipped (read-only browser of real files).
    }

    // Directories first, then files; each alphabetically — a stable order.
    nodes.sort_by(|a, b| match (a.is_dir(), b.is_dir()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name().cmp(b.name()),
    });
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Build a Quiqr-site-shaped directory under `root/name`:
    /// a `config.toml`, a `quiqr/` dir, and an optional `content/` layout given
    /// as a list of relative paths (trailing `/` => directory, else file).
    fn make_site(root: &Path, name: &str, content: &[&str]) -> PathBuf {
        let site = root.join(name);
        fs::create_dir_all(site.join("quiqr")).unwrap();
        fs::write(site.join("config.toml"), "title = \"t\"\n").unwrap();
        for item in content {
            let p = site.join("content").join(item.trim_end_matches('/'));
            if item.ends_with('/') {
                fs::create_dir_all(&p).unwrap();
            } else {
                fs::create_dir_all(p.parent().unwrap()).unwrap();
                fs::write(&p, b"body").unwrap();
            }
        }
        site
    }

    fn site_of(path: PathBuf) -> Site {
        Site {
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            path,
        }
    }

    fn default_hidden() -> Vec<String> {
        DEFAULT_HIDDEN_DIRS.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn enumerate_returns_only_sites_sorted() {
        let tmp = TempDir::new().unwrap();
        make_site(tmp.path(), "zeta", &[]);
        make_site(tmp.path(), "alpha", &[]);
        // An unrelated directory (no quiqr/, no config).
        fs::create_dir_all(tmp.path().join("not-a-site/sub")).unwrap();

        let sites = enumerate_sites(tmp.path()).unwrap();
        let names: Vec<_> = sites.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["alpha", "zeta"]);
        assert!(sites.iter().all(|s| s.path.is_absolute()));
    }

    #[test]
    fn enumerate_empty_dir_is_ok_and_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(enumerate_sites(tmp.path()).unwrap().is_empty());
    }

    #[test]
    fn enumerate_missing_data_dir_errors_with_path() {
        let missing = Path::new("/definitely/not/here/quiqr-data");
        let err = enumerate_sites(missing).unwrap_err();
        assert!(err.to_string().contains("quiqr-data"), "got: {err}");
    }

    #[test]
    fn site_detection_requires_both_markers() {
        let tmp = TempDir::new().unwrap();

        // Both markers -> site.
        let ok = make_site(tmp.path(), "ok", &[]);
        assert!(is_quiqr_site(&ok));

        // Hugo config but no quiqr/.
        let no_quiqr = tmp.path().join("no-quiqr");
        fs::create_dir_all(&no_quiqr).unwrap();
        fs::write(no_quiqr.join("config.toml"), "x = 1").unwrap();
        assert!(!is_quiqr_site(&no_quiqr));

        // quiqr/ but no Hugo config.
        let no_cfg = tmp.path().join("no-cfg");
        fs::create_dir_all(no_cfg.join("quiqr")).unwrap();
        assert!(!is_quiqr_site(&no_cfg));
    }

    #[test]
    fn site_detection_accepts_hugo_config_variants() {
        let tmp = TempDir::new().unwrap();

        // hugo.yaml + quiqr/
        let a = tmp.path().join("a");
        fs::create_dir_all(a.join("quiqr")).unwrap();
        fs::write(a.join("hugo.yaml"), "title: t").unwrap();
        assert!(is_quiqr_site(&a));

        // config/ directory + quiqr/
        let b = tmp.path().join("b");
        fs::create_dir_all(b.join("quiqr")).unwrap();
        fs::create_dir_all(b.join("config")).unwrap();
        assert!(is_quiqr_site(&b));

        // A bare `config` *file* (no extension) does NOT count.
        let c = tmp.path().join("c");
        fs::create_dir_all(c.join("quiqr")).unwrap();
        fs::write(c.join("config"), "nope").unwrap();
        assert!(!is_quiqr_site(&c));
    }

    #[test]
    fn content_tree_represents_nested_structure() {
        let tmp = TempDir::new().unwrap();
        let path = make_site(tmp.path(), "s", &["about.md", "posts/hello.md"]);
        let tree = content_tree(&site_of(path), &default_hidden());

        // Dir `posts` before file `about.md` (dirs first), each correct.
        assert_eq!(tree.len(), 2);
        match &tree[0] {
            ContentNode::Dir { name, children, .. } => {
                assert_eq!(name, "posts");
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].name(), "hello.md");
                assert_eq!(children[0].rel_path(), Path::new("posts/hello.md"));
            }
            other => panic!("expected posts dir first, got {other:?}"),
        }
        assert_eq!(tree[1].name(), "about.md");
    }

    #[test]
    fn content_tree_missing_content_is_empty() {
        let tmp = TempDir::new().unwrap();
        let path = make_site(tmp.path(), "s", &[]); // no content/
        assert!(content_tree(&site_of(path), &default_hidden()).is_empty());
    }

    #[test]
    fn hidden_dirs_omitted_at_top_level_and_nested() {
        let tmp = TempDir::new().unwrap();
        let path = make_site(
            tmp.path(),
            "s",
            &[
                "posts/hello.md",
                ".git/",            // top-level hidden
                "posts/resources/", // nested hidden
            ],
        );
        let tree = content_tree(&site_of(path), &default_hidden());

        // Top level: only `posts`, `.git` omitted.
        assert_eq!(tree.iter().map(|n| n.name()).collect::<Vec<_>>(), ["posts"]);

        // Nested: `resources` omitted, only `hello.md` remains under posts.
        match &tree[0] {
            ContentNode::Dir { children, .. } => {
                assert_eq!(
                    children.iter().map(|n| n.name()).collect::<Vec<_>>(),
                    ["hello.md"]
                );
            }
            other => panic!("expected posts dir, got {other:?}"),
        }
    }

    #[test]
    fn custom_hidden_set_overrides_default() {
        let tmp = TempDir::new().unwrap();
        let path = make_site(tmp.path(), "s", &["drafts/", "posts/"]);
        // Hide only `drafts`; `posts` stays. `.git` etc. are NOT in this set.
        let hidden = vec!["drafts".to_string()];
        let tree = content_tree(&site_of(path), &hidden);
        assert_eq!(tree.iter().map(|n| n.name()).collect::<Vec<_>>(), ["posts"]);
    }

    /// The storage layer must never mutate the site on disk (single-writer
    /// rule). Snapshot the data dir's recursive listing before and after the
    /// read operations and assert it is unchanged.
    #[test]
    fn operations_never_write_to_disk() {
        let tmp = TempDir::new().unwrap();
        make_site(
            tmp.path(),
            "site-a",
            &["about.md", "posts/hello.md", ".git/"],
        );
        make_site(tmp.path(), "site-b", &[]);

        let before = listing(tmp.path());

        let sites = enumerate_sites(tmp.path()).unwrap();
        for site in &sites {
            let _ = content_tree(site, &default_hidden());
        }
        // Re-run to be thorough.
        let _ = enumerate_sites(tmp.path()).unwrap();

        let after = listing(tmp.path());
        assert_eq!(
            before, after,
            "storage layer must not create/modify/delete files"
        );
    }

    /// Sorted recursive listing of paths under `root` (relative), for snapshotting.
    fn listing(root: &Path) -> Vec<PathBuf> {
        fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
            let mut entries: Vec<_> = fs::read_dir(dir).unwrap().flatten().collect();
            entries.sort_by_key(|e| e.path());
            for e in entries {
                let p = e.path();
                out.push(p.strip_prefix(root).unwrap().to_path_buf());
                if e.file_type().unwrap().is_dir() {
                    walk(&p, root, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(root, root, &mut out);
        out
    }
}
