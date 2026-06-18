//! WordPerfect-5.1-style two-pane TUI shell.
//!
//! The shell is a **pure core** with terminal I/O kept at the edge: the host
//! binary owns the `Terminal`, reads key events, calls [`update`] to advance the
//! [`AppState`] state machine, then calls [`render`] to draw a frame. This keeps
//! the whole shell exercisable with ratatui's `TestBackend`.
//!
//! Modes form a small state machine — `SiteList → Browse → ViewFile → Agent` —
//! and the bottom **function-key legend** is context-sensitive to the current
//! mode. All write-like verbs (New, Save, Discard, Ask AI) are emitted as
//! [`Action`]s for the agent; the UI never writes site files.

use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use qtui_model::{MenuEntry, NavigationModel};
use qtui_storage::{ContentNode, Site};

/// Which navigation view is active in [`Mode::Browse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavView {
    /// The raw `content/` tree (from `qtui-storage`).
    ContentTree,
    /// The schema-driven Menu of Singles/Collections (from `qtui-model`).
    SchemaMenu,
}

/// The current UI mode. The mode carries the context specific to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Choose a site.
    SiteList,
    /// Browse an opened site; toggles between the two navigation views.
    Browse { nav: NavView },
    /// Read-only file viewer; offers "Ask AI".
    ViewFile,
    /// The agent pane has the cursor.
    Agent,
}

/// A request emitted by the state machine for the host loop / agent to act on.
/// The UI never performs these as file operations itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The user asked to quit the application.
    Quit,
    /// Ask the agent about the given file (the "@path …" intent of E6).
    AskAi { file: String },
    /// Request the agent create new content.
    RequestNew,
    /// Request the agent save / commit its work.
    RequestSave,
    /// Request the agent discard its work.
    RequestDiscard,
}

/// One entry in the function-key legend: a key label paired with an action label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegendEntry {
    pub key: &'static str,
    pub label: &'static str,
}

/// The pure application state. Holds the mode, the data the panes render, and
/// the selection/cursor state. No terminal, no I/O.
#[derive(Debug, Clone)]
pub struct AppState {
    mode: Mode,
    /// The sites shown in `SiteList`.
    sites: Vec<Site>,
    selected_site: usize,
    /// The opened site's content tree (E2) and schema model (E3), set on open.
    content: Vec<ContentNode>,
    model: NavigationModel,
    selected_browse: usize,
    /// The file opened in `ViewFile`, if any (path relative to `content/`).
    open_file: Option<String>,
    /// The Hugo preview URL, surfaced by E5 once a server is running.
    preview_url: Option<String>,
    /// The opened file's text, supplied by the host for the read-only viewer
    /// (E6). The UI never reads files itself.
    open_file_contents: Option<String>,
    /// The agent pane's latest snapshot lines, pushed by the host each tick
    /// (E7). Rendered in the right pane; empty -> a neutral label.
    agent_output: Vec<String>,
    /// Which navigation view a freshly opened site shows first.
    default_nav: NavView,
}

impl AppState {
    /// Create a new app in `SiteList` mode over the given sites.
    ///
    /// `schema_nav_first` mirrors the `ui.show_schema_nav_first` config: when
    /// true an opened site leads with the schema Menu, else the content tree.
    pub fn new(sites: Vec<Site>, schema_nav_first: bool) -> Self {
        Self {
            mode: Mode::SiteList,
            sites,
            selected_site: 0,
            content: Vec::new(),
            model: NavigationModel::default(),
            selected_browse: 0,
            open_file: None,
            preview_url: None,
            open_file_contents: None,
            agent_output: Vec::new(),
            default_nav: if schema_nav_first {
                NavView::SchemaMenu
            } else {
                NavView::ContentTree
            },
        }
    }

    /// The current mode.
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Load an opened site's data (content tree + schema model). The host calls
    /// this when entering `Browse` for a site; provided separately so the state
    /// machine stays free of storage/model I/O.
    pub fn set_open_site(&mut self, content: Vec<ContentNode>, model: NavigationModel) {
        self.content = content;
        self.model = model;
        self.selected_browse = 0;
    }

    /// Set (or clear) the preview URL surfaced in `Browse` (E5).
    pub fn set_preview_url(&mut self, url: Option<String>) {
        self.preview_url = url;
    }

    /// The preview URL, if a server is running.
    pub fn preview_url(&self) -> Option<&str> {
        self.preview_url.as_deref()
    }

    /// The currently selected site, if any.
    pub fn selected_site(&self) -> Option<&Site> {
        self.sites.get(self.selected_site)
    }

    /// The file open in `ViewFile`, if any.
    pub fn open_file(&self) -> Option<&str> {
        self.open_file.as_deref()
    }

    /// Supply the opened file's path and contents for the read-only viewer (E6).
    /// The host reads the bytes; the UI library never touches the filesystem.
    pub fn set_open_file(&mut self, path: impl Into<String>, contents: impl Into<String>) {
        self.open_file = Some(path.into());
        self.open_file_contents = Some(contents.into());
    }

    /// The opened file's contents, if supplied.
    pub fn open_file_contents(&self) -> Option<&str> {
        self.open_file_contents.as_deref()
    }

    /// Push the agent pane's latest snapshot lines (E7). The host calls this
    /// each tick with `Agent::snapshot()` output; the right pane renders them.
    pub fn set_agent_output(&mut self, lines: Vec<String>) {
        self.agent_output = lines;
    }

    /// The agent pane's current snapshot lines.
    pub fn agent_output(&self) -> &[String] {
        &self.agent_output
    }
}

/// Advance the state machine for a key event, returning an [`Action`] when the
/// event implies one the host must handle. Pure: no I/O, deterministic.
pub fn update(state: &mut AppState, key: KeyEvent) -> Option<Action> {
    match key.code {
        // Quit from anywhere.
        KeyCode::Char('q') => return Some(Action::Quit),

        // Back: pop one step along SiteList ← Browse ← ViewFile ← Agent.
        KeyCode::Esc => {
            state.mode = match &state.mode {
                Mode::SiteList => Mode::SiteList,
                Mode::Browse { .. } => Mode::SiteList,
                Mode::ViewFile => Mode::Browse {
                    nav: state.default_nav,
                },
                Mode::Agent => Mode::ViewFile,
            };
        }

        // Selection movement (lists are vertical).
        KeyCode::Up => move_selection(state, -1),
        KeyCode::Down => move_selection(state, 1),

        // Toggle the Browse navigation view.
        KeyCode::Tab => {
            if let Mode::Browse { nav } = &state.mode {
                let next = match nav {
                    NavView::ContentTree => NavView::SchemaMenu,
                    NavView::SchemaMenu => NavView::ContentTree,
                };
                state.mode = Mode::Browse { nav: next };
            }
        }

        // Forward transitions (Enter = open).
        KeyCode::Enter => return open_forward(state),

        // Function keys: the write-like verbs are requests, never file ops.
        KeyCode::F(6) => {
            if state.mode == Mode::ViewFile {
                if let Some(file) = state.open_file.clone() {
                    state.mode = Mode::Agent;
                    return Some(Action::AskAi { file });
                }
            }
        }
        KeyCode::F(3) => return Some(Action::RequestNew),
        KeyCode::F(7) if state.mode == Mode::Agent => return Some(Action::RequestSave),
        KeyCode::F(9) if state.mode == Mode::Agent => return Some(Action::RequestDiscard),
        _ => {}
    }
    None
}

fn move_selection(state: &mut AppState, delta: isize) {
    let len = match &state.mode {
        Mode::SiteList => state.sites.len(),
        Mode::Browse { nav } => browse_row_count(state, *nav),
        _ => 0,
    };
    if len == 0 {
        return;
    }
    let sel = match &state.mode {
        Mode::SiteList => &mut state.selected_site,
        Mode::Browse { .. } => &mut state.selected_browse,
        _ => return,
    };
    let next = (*sel as isize + delta).clamp(0, len as isize - 1);
    *sel = next as usize;
}

/// Handle Enter: open the selected site (SiteList→Browse) or file
/// (Browse→ViewFile). Returns no action (transitions are internal); the host
/// observes the new mode and loads data via [`AppState::set_open_site`].
fn open_forward(state: &mut AppState) -> Option<Action> {
    match &state.mode {
        Mode::SiteList => {
            if state.selected_site().is_some() {
                state.mode = Mode::Browse {
                    nav: state.default_nav,
                };
                state.selected_browse = 0;
            }
        }
        // Only the content-tree view opens files directly; opening a schema
        // entry is resolved to its path by the host (E5/E6) — for the shell we
        // open the selected content-tree file. Directory rows are not openable.
        Mode::Browse {
            nav: NavView::ContentTree,
        } => {
            if let Some(rel) = content_rows(&state.content)
                .get(state.selected_browse)
                .and_then(|row| row.rel_path.clone())
            {
                // Store the true path relative to content/, not the display row.
                state.open_file = Some(rel.to_string_lossy().into_owned());
                state.mode = Mode::ViewFile;
            }
        }
        _ => {}
    }
    None
}

// ----- Row helpers (flatten the data each mode lists) -----------------------

/// A flattened content-tree row: what to display, and the file path (relative to
/// `content/`) it opens — `None` for directory rows, which are not openable.
struct ContentRow {
    display: String,
    rel_path: Option<PathBuf>,
}

fn browse_row_count(state: &AppState, nav: NavView) -> usize {
    match nav {
        NavView::ContentTree => content_rows(&state.content).len(),
        NavView::SchemaMenu => menu_rows(&state.model).len(),
    }
}

/// Flatten the content tree to display rows (depth-indented), each carrying its
/// real `rel_path` for files (and `None` for directories).
fn content_rows(nodes: &[ContentNode]) -> Vec<ContentRow> {
    fn walk(nodes: &[ContentNode], depth: usize, out: &mut Vec<ContentRow>) {
        for n in nodes {
            let indent = "  ".repeat(depth);
            match n {
                ContentNode::Dir { name, children, .. } => {
                    out.push(ContentRow {
                        display: format!("{indent}{name}/"),
                        rel_path: None,
                    });
                    walk(children, depth + 1, out);
                }
                ContentNode::File { name, rel_path } => out.push(ContentRow {
                    display: format!("{indent}{name}"),
                    rel_path: Some(rel_path.clone()),
                }),
            }
        }
    }
    let mut out = Vec::new();
    walk(nodes, 0, &mut out);
    out
}

/// Flatten the schema menu to display rows (group headers + their entries).
fn menu_rows(model: &NavigationModel) -> Vec<String> {
    let mut out = Vec::new();
    for group in &model.menu {
        out.push(group.title.clone());
        for entry in &group.entries {
            let label = match entry {
                MenuEntry::Single(k) => model
                    .single(k)
                    .map(|s| s.title.clone())
                    .unwrap_or_else(|| k.clone()),
                MenuEntry::Collection(k) => model
                    .collection(k)
                    .map(|c| c.title.clone())
                    .unwrap_or_else(|| k.clone()),
            };
            out.push(format!("  {label}"));
        }
    }
    out
}

// ----- Legend ----------------------------------------------------------------

/// The context-sensitive function-key legend for the current mode.
pub fn legend(state: &AppState) -> Vec<LegendEntry> {
    let e = |key, label| LegendEntry { key, label };
    match &state.mode {
        Mode::SiteList => vec![e("Enter", "Open"), e("q", "Quit")],
        Mode::Browse { .. } => vec![
            e("Enter", "Open"),
            e("Tab", "Toggle Nav"),
            e("F3", "New"),
            e("F5", "Preview"),
            e("Esc", "Back"),
            e("q", "Quit"),
        ],
        // ViewFile is read-only: it offers "Ask AI" and NO edit verb.
        Mode::ViewFile => vec![e("F6", "Ask AI"), e("Esc", "Back"), e("q", "Quit")],
        Mode::Agent => vec![
            e("F7", "Save"),
            e("F9", "Discard"),
            e("Esc", "Back"),
            e("q", "Quit"),
        ],
    }
}

// ----- Render ----------------------------------------------------------------

/// Render the whole shell into `frame`. Draws a left pane, a right (agent) pane,
/// and a bottom legend row. Performs no terminal I/O of its own.
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    let work = rows[0];
    let legend_row = rows[1];

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(work);

    render_left(frame, panes[0], state);
    render_right(frame, panes[1], state);
    render_legend(frame, legend_row, state);
}

fn render_left(frame: &mut Frame, area: Rect, state: &AppState) {
    // In Browse, reserve a one-row strip above the list to surface the live
    // preview URL (when a server is running). Rendering it on its own line keeps
    // the full URL readable instead of truncating it inside the narrow border
    // title.
    let show_preview = matches!(state.mode, Mode::Browse { .. }) && state.preview_url().is_some();
    let (preview_area, list_area) = if show_preview {
        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        (Some(parts[0]), parts[1])
    } else {
        (None, area)
    };

    if let Some(pa) = preview_area {
        let url = state.preview_url().unwrap_or("");
        let line = Line::from(vec![
            Span::styled("Preview: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(url.to_string()),
        ]);
        frame.render_widget(Paragraph::new(line), pa);
    }

    let (title, rows, selected) = match &state.mode {
        Mode::SiteList => (
            "Sites".to_string(),
            state
                .sites
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>(),
            state.selected_site,
        ),
        Mode::Browse { nav } => {
            let title = match nav {
                NavView::ContentTree => "Content",
                NavView::SchemaMenu => "Menu",
            };
            let rows = match nav {
                NavView::ContentTree => content_rows(&state.content)
                    .into_iter()
                    .map(|r| r.display)
                    .collect(),
                NavView::SchemaMenu => menu_rows(&state.model),
            };
            (title.to_string(), rows, state.selected_browse)
        }
        Mode::ViewFile => {
            // Title names the open file; the body is its contents, read-only.
            let title = match state.open_file() {
                Some(path) => format!("{path} (read-only)"),
                None => "View (read-only)".to_string(),
            };
            let rows = match state.open_file_contents() {
                Some(contents) => contents.lines().map(str::to_string).collect(),
                None => vec!["(no file contents)".to_string()],
            };
            (title, rows, 0)
        }
        Mode::Agent => ("Browse".to_string(), Vec::new(), 0),
    };

    let items: Vec<ListItem> = rows.into_iter().map(ListItem::new).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_symbol("> ");
    let mut st = ListState::default();
    st.select(Some(selected));
    frame.render_stateful_widget(list, list_area, &mut st);
}

fn render_right(frame: &mut Frame, area: Rect, state: &AppState) {
    // The agent pane renders the agent's latest snapshot (pushed by the host via
    // `set_agent_output`), falling back to a neutral label when there is none.
    let block = Block::default().borders(Borders::ALL).title("Agent");
    let text = if state.agent_output().is_empty() {
        "(agent pane — Claude Code appears here)".to_string()
    } else {
        state.agent_output().join("\n")
    };
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_legend(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut spans = Vec::new();
    for entry in legend(state) {
        spans.push(Span::styled(
            entry.key,
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
        ));
        spans.push(Span::raw(format!(" {}  ", entry.label)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
