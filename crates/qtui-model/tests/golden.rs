//! Golden-file tests over realistic `quiqr/model/` fixtures, asserting the
//! produced `NavigationModel`. Covers a full schema, a partial one (collections
//! only), a malformed include (skipped with a warning), a directory-based
//! include root, and a site with no model at all.

use std::path::PathBuf;

use qtui_model::{load_model, MenuEntry};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn full_schema_builds_expected_model() {
    let model = load_model(&fixture("full"));

    // Collections: pages, posts (with folders).
    let pages = model.collection("pages").expect("pages collection");
    assert_eq!(pages.title, "Pages");
    assert_eq!(pages.folder.as_deref(), Some("content/page/"));
    let posts = model.collection("posts").expect("posts collection");
    assert_eq!(posts.folder.as_deref(), Some("content/post"));

    // Singles: frontpage (file + fields), mainConfig (file + _mergePartial).
    let frontpage = model.single("frontpage").expect("frontpage single");
    assert_eq!(frontpage.title, "Homepage");
    assert_eq!(frontpage.file.as_deref(), Some("/content/_index.md"));
    let main_config = model.single("mainConfig").expect("mainConfig single");
    assert_eq!(
        main_config.merge_partial.as_deref(),
        Some("single_conf_params")
    );

    // Field title falls back to key when omitted.
    let subheadline = frontpage
        .fields
        .iter()
        .find(|f| f.key == "header_subheadline")
        .expect("subheadline field");
    assert_eq!(subheadline.title, "header_subheadline");
    assert_eq!(subheadline.type_, "string");

    // Nested fields preserved (posts.images -> thumb).
    let images = posts
        .fields
        .iter()
        .find(|f| f.key == "images")
        .expect("images field");
    assert_eq!(images.type_, "bundle-manager");
    assert_eq!(images.fields.len(), 1);
    assert_eq!(images.fields[0].key, "thumb");

    // Menu order and resolution.
    assert_eq!(model.menu.len(), 2);
    let content = &model.menu[0];
    assert_eq!(content.key, "Content");
    assert_eq!(
        content.entries,
        vec![
            MenuEntry::Collection("pages".into()),
            MenuEntry::Collection("posts".into()),
            MenuEntry::Single("frontpage".into()),
        ]
    );
    let settings = &model.menu[1];
    // `mainConfig` resolves to a single; `ghost` references nothing -> dropped.
    assert_eq!(
        settings.entries,
        vec![MenuEntry::Single("mainConfig".into())]
    );

    // The dangling `ghost` reference produced a warning.
    assert!(
        model.warnings.iter().any(|w| w.contains("ghost")),
        "expected a warning about the dangling 'ghost' menu item, got {:?}",
        model.warnings
    );
}

#[test]
fn partial_schema_collections_only() {
    let model = load_model(&fixture("partial"));
    assert!(model.menu.is_empty(), "no menu in the partial fixture");
    assert!(
        model.singles.is_empty(),
        "no singles in the partial fixture"
    );
    assert_eq!(model.collections.len(), 1);
    assert_eq!(
        model.collection("notes").unwrap().folder.as_deref(),
        Some("content/notes")
    );
}

#[test]
fn malformed_include_is_skipped_with_warning() {
    let model = load_model(&fixture("malformed"));

    // The good collection still parses.
    assert!(model.collection("good").is_some());
    // The malformed menu is omitted, not fatal.
    assert!(model.menu.is_empty(), "malformed menu must be omitted");
    // ...and recorded as a warning naming the file.
    assert!(
        model.warnings.iter().any(|w| w.contains("menu.yaml")),
        "expected a warning naming menu.yaml, got {:?}",
        model.warnings
    );
}

#[test]
fn directory_include_root_is_merged_in_order() {
    let model = load_model(&fixture("dir-include"));
    let keys: Vec<&str> = model.singles.iter().map(|s| s.key.as_str()).collect();
    // 01-about.yaml before 02-contact.yaml (name-sorted file order).
    assert_eq!(keys, ["about", "contact"]);
    assert_eq!(
        model.single("about").unwrap().file.as_deref(),
        Some("/content/about.md")
    );
}

#[test]
fn missing_model_yields_empty() {
    // A temp dir with no quiqr/model/ at all.
    let tmp = std::env::temp_dir().join("qtui-model-no-schema-xyz");
    let _ = std::fs::create_dir_all(&tmp);
    let model = load_model(&tmp);
    assert_eq!(model, Default::default());
    let _ = std::fs::remove_dir_all(&tmp);
}
