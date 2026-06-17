//! Exercise `qtui-storage` against the anonymized real-site fixture — a genuine
//! Quiqr/Hugo site shape (real `quiqr/model/`, real-shaped `content/` tree).

use std::path::PathBuf;

use qtui_storage::{content_tree, is_quiqr_site, ContentNode, Site, DEFAULT_HIDDEN_DIRS};

fn real_site_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/real-site")
}

#[test]
fn real_site_is_recognised() {
    let dir = real_site_dir();
    assert!(
        dir.join("config.toml").is_file(),
        "fixture has a Hugo config"
    );
    assert!(dir.join("quiqr").is_dir(), "fixture has a quiqr/ dir");
    assert!(
        is_quiqr_site(&dir),
        "the real-site fixture must be a Quiqr site"
    );
}

#[test]
fn real_site_content_tree() {
    let site = Site {
        name: "real-site".into(),
        path: real_site_dir(),
    };
    let hidden: Vec<String> = DEFAULT_HIDDEN_DIRS.iter().map(|s| s.to_string()).collect();
    let tree = content_tree(&site, &hidden);

    // Top level includes the `post` and `homepage` directories and the
    // `_index.md` / `about.md` files. (Storage uses bare dir names; the UI is
    // what adds a trailing slash for display.)
    let names: Vec<&str> = tree.iter().map(|n| n.name()).collect();
    assert!(names.contains(&"post"), "tree top level: {names:?}");
    assert!(names.contains(&"homepage"), "tree top level: {names:?}");
    assert!(names.contains(&"about.md"), "tree top level: {names:?}");

    // The `post` directory contains the example posts.
    let post = tree
        .iter()
        .find(|n| n.name() == "post")
        .expect("post dir present");
    if let ContentNode::Dir { children, .. } = post {
        let post_files: Vec<&str> = children.iter().map(|n| n.name()).collect();
        assert!(
            post_files.contains(&"about-example.md"),
            "posts: {post_files:?}"
        );
        assert!(
            post_files.contains(&"example-article.md"),
            "posts: {post_files:?}"
        );
    } else {
        panic!("post should be a directory");
    }
}
