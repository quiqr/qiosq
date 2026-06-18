//! Exercise `qtui-storage` against the anonymized real-site fixture — laid out
//! like a real Quiqr data folder (`sites/<name>/{config.json, main/…}`).

use std::path::PathBuf;

use qtui_storage::{content_tree, enumerate_sites, ContentNode, DEFAULT_HIDDEN_DIRS};

/// The fixture root acts as the Quiqr **data directory**.
fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/real-site")
}

#[test]
fn real_site_is_enumerated_with_its_working_copy() {
    let sites = enumerate_sites(&data_dir()).unwrap();
    let names: Vec<&str> = sites.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["examplesite"], "should find the one fixture site");

    // The resolved path is the working copy (sites/examplesite/main), where
    // quiqr/ and content/ live.
    let site = &sites[0];
    assert!(
        site.path.ends_with("sites/examplesite/main"),
        "{:?}",
        site.path
    );
    assert!(site.path.join("quiqr/model/base.yaml").is_file());
    assert!(site.path.join("content").is_dir());
}

#[test]
fn real_site_content_tree() {
    let sites = enumerate_sites(&data_dir()).unwrap();
    let site = sites.into_iter().find(|s| s.name == "examplesite").unwrap();

    let hidden: Vec<String> = DEFAULT_HIDDEN_DIRS.iter().map(|s| s.to_string()).collect();
    let tree = content_tree(&site, &hidden);

    // Top level includes the `post` and `homepage` directories and `about.md`.
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
