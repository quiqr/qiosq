//! TestBackend snapshot + state-machine transition tests for the WP5.1 shell.
//!
//! These assert on *content* (does the legend row contain "Ask AI"? does the
//! left pane list "about.md"?) rather than committing exact buffer dumps, so
//! small layout tweaks don't break them — except one end-to-end layout check.

use std::path::PathBuf;

use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::Terminal;

use qtui_model::{Collection, MenuEntry, MenuGroup, NavigationModel, Single};
use qtui_storage::{ContentNode, Site};
use qtui_ui::{legend, render, update, Action, AppState, Mode, NavView};

// ----- fixtures --------------------------------------------------------------

fn sites() -> Vec<Site> {
    vec![
        Site {
            name: "alpha".into(),
            path: PathBuf::from("/data/alpha"),
        },
        Site {
            name: "beta".into(),
            path: PathBuf::from("/data/beta"),
        },
    ]
}

fn content() -> Vec<ContentNode> {
    vec![
        ContentNode::Dir {
            name: "posts".into(),
            rel_path: PathBuf::from("posts"),
            children: vec![ContentNode::File {
                name: "hello.md".into(),
                rel_path: PathBuf::from("posts/hello.md"),
            }],
        },
        ContentNode::File {
            name: "about.md".into(),
            rel_path: PathBuf::from("about.md"),
        },
    ]
}

fn model() -> NavigationModel {
    NavigationModel {
        menu: vec![MenuGroup {
            key: "Content".into(),
            title: "Content".into(),
            entries: vec![
                MenuEntry::Collection("posts".into()),
                MenuEntry::Single("about".into()),
            ],
        }],
        singles: vec![Single {
            key: "about".into(),
            title: "About".into(),
            file: Some("/content/about.md".into()),
            fields: vec![],
            merge_partial: None,
        }],
        collections: vec![Collection {
            key: "posts".into(),
            title: "Posts".into(),
            folder: Some("content/post".into()),
            fields: vec![],
            merge_partial: None,
        }],
        warnings: vec![],
    }
}

/// Build an app already opened on a site, in Browse with the given nav view.
fn opened(nav: NavView) -> AppState {
    let mut app = AppState::new(sites(), nav == NavView::SchemaMenu);
    update(&mut app, key(KeyCode::Enter)); // SiteList -> Browse
    app.set_open_site(content(), model());
    app
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::from(code)
}

/// Render `app` to an 80x24 TestBackend and return the buffer as plain text.
fn render_to_string(app: &AppState) -> String {
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|f| render(f, app)).unwrap();
    let buf = term.backend().buffer().clone();
    let mut s = String::new();
    for y in 0..buf.area().height {
        for x in 0..buf.area().width {
            s.push_str(buf[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

// ----- layout ----------------------------------------------------------------

#[test]
fn layout_has_two_panes_and_a_legend_row() {
    let app = AppState::new(sites(), false);
    let screen = render_to_string(&app);

    // Left pane (Sites) and right pane (Agent) both present.
    assert!(
        screen.contains("Sites"),
        "left pane titled Sites:\n{screen}"
    );
    assert!(
        screen.contains("Agent"),
        "right pane titled Agent:\n{screen}"
    );
    // Bottom row is the legend.
    let last = screen.lines().last().unwrap_or("");
    assert!(
        last.contains("Quit"),
        "legend row on bottom line, got: {last:?}"
    );
}

#[test]
fn right_pane_is_an_agent_placeholder() {
    let app = AppState::new(sites(), false);
    let screen = render_to_string(&app);
    assert!(
        screen.contains("agent pane"),
        "right pane should be a labelled agent placeholder:\n{screen}"
    );
}

// ----- legend per mode -------------------------------------------------------

fn legend_labels(app: &AppState) -> Vec<&'static str> {
    legend(app).into_iter().map(|e| e.label).collect()
}

#[test]
fn browse_legend_offers_preview_and_toggle_no_save() {
    let app = opened(NavView::ContentTree);
    let labels = legend_labels(&app);
    assert!(labels.contains(&"Preview"));
    assert!(labels.contains(&"Toggle Nav"));
    assert!(!labels.contains(&"Save"));
    assert!(!labels.contains(&"Discard"));
}

#[test]
fn viewfile_legend_offers_ask_ai_and_no_edit_verb() {
    let mut app = opened(NavView::ContentTree);
    // Move to about.md (row 2: posts/, hello.md, about.md) and open it.
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Enter));
    assert_eq!(app.mode(), &Mode::ViewFile);

    let labels = legend_labels(&app);
    assert!(
        labels.contains(&"Ask AI"),
        "ViewFile must offer Ask AI: {labels:?}"
    );
    // The viewer is read-only: no in-place edit verb.
    for forbidden in ["Edit", "Save", "Write", "Delete"] {
        assert!(
            !labels.contains(&forbidden),
            "ViewFile must not expose {forbidden}"
        );
    }
    // And it renders that it is read-only.
    assert!(render_to_string(&app).contains("read-only"));
}

#[test]
fn agent_legend_offers_save_discard_back() {
    let mut app = opened(NavView::ContentTree);
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Enter)); // -> ViewFile (about.md)
    let action = update(&mut app, key(KeyCode::F(6))); // Ask AI -> Agent
    assert_eq!(
        action,
        Some(Action::AskAi {
            file: "about.md".into()
        })
    );
    assert_eq!(app.mode(), &Mode::Agent);

    let labels = legend_labels(&app);
    assert!(labels.contains(&"Save"));
    assert!(labels.contains(&"Discard"));
    assert!(labels.contains(&"Back"));
}

#[test]
fn legend_changes_with_mode() {
    let mut app = opened(NavView::ContentTree);
    let browse = legend_labels(&app);
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Enter)); // -> ViewFile
    let viewfile = legend_labels(&app);
    assert_ne!(browse, viewfile);
}

// ----- transitions -----------------------------------------------------------

#[test]
fn full_forward_and_back_path() {
    let mut app = AppState::new(sites(), false);
    assert_eq!(app.mode(), &Mode::SiteList);

    update(&mut app, key(KeyCode::Enter)); // SiteList -> Browse
    assert!(matches!(app.mode(), Mode::Browse { .. }));
    app.set_open_site(content(), model());

    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Down));
    update(&mut app, key(KeyCode::Enter)); // Browse -> ViewFile
    assert_eq!(app.mode(), &Mode::ViewFile);

    update(&mut app, key(KeyCode::F(6))); // ViewFile -> Agent
    assert_eq!(app.mode(), &Mode::Agent);

    update(&mut app, key(KeyCode::Esc)); // Agent -> ViewFile
    assert_eq!(app.mode(), &Mode::ViewFile);
    update(&mut app, key(KeyCode::Esc)); // ViewFile -> Browse
    assert!(matches!(app.mode(), Mode::Browse { .. }));
    update(&mut app, key(KeyCode::Esc)); // Browse -> SiteList
    assert_eq!(app.mode(), &Mode::SiteList);
}

#[test]
fn quit_emits_quit_action() {
    let mut app = AppState::new(sites(), false);
    assert_eq!(
        update(&mut app, key(KeyCode::Char('q'))),
        Some(Action::Quit)
    );
}

#[test]
fn tab_toggles_nav_and_stays_in_browse() {
    let mut app = opened(NavView::ContentTree);
    assert_eq!(
        app.mode(),
        &Mode::Browse {
            nav: NavView::ContentTree
        }
    );
    update(&mut app, key(KeyCode::Tab));
    assert_eq!(
        app.mode(),
        &Mode::Browse {
            nav: NavView::SchemaMenu
        }
    );
    update(&mut app, key(KeyCode::Tab));
    assert_eq!(
        app.mode(),
        &Mode::Browse {
            nav: NavView::ContentTree
        }
    );
}

#[test]
fn default_nav_view_is_honoured() {
    // schema_nav_first = true -> opened site leads with the schema menu.
    let app = opened(NavView::SchemaMenu);
    assert_eq!(
        app.mode(),
        &Mode::Browse {
            nav: NavView::SchemaMenu
        }
    );
}

// ----- dual navigation content ----------------------------------------------

#[test]
fn content_tree_view_lists_content_entries() {
    let app = opened(NavView::ContentTree);
    let screen = render_to_string(&app);
    assert!(
        screen.contains("posts/"),
        "content view lists dirs:\n{screen}"
    );
    assert!(
        screen.contains("about.md"),
        "content view lists files:\n{screen}"
    );
    assert!(
        screen.contains("hello.md"),
        "content view lists nested files:\n{screen}"
    );
}

#[test]
fn schema_menu_view_lists_model_menu() {
    let app = opened(NavView::SchemaMenu);
    let screen = render_to_string(&app);
    assert!(screen.contains("Content"), "menu group:\n{screen}");
    assert!(screen.contains("Posts"), "collection title:\n{screen}");
    assert!(screen.contains("About"), "single title:\n{screen}");
}

// ----- preview URL surfacing (E5) -------------------------------------------

#[test]
fn browse_surfaces_the_preview_url() {
    let mut app = opened(NavView::ContentTree);
    // No preview running yet: the URL must not appear.
    assert!(!render_to_string(&app).contains("13150"));

    // Once E5's preview layer sets the URL, Browse surfaces it.
    app.set_preview_url(Some("http://localhost:13150/".into()));
    let screen = render_to_string(&app);
    assert!(
        screen.contains("13150"),
        "Browse should surface the preview URL:\n{screen}"
    );
}
