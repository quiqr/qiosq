## ADDED Requirements

### Requirement: Two navigation views in Browse

In `Browse` mode the UI SHALL offer two navigation views of the open site — the
raw `content/` tree (from `qtui-storage`) and the schema-driven Menu of Singles
and Collections (from `qtui-model`) — and SHALL render exactly one at a time.

#### Scenario: Content tree view lists content entries
- **WHEN** `Browse` shows the content-tree view for a site with `posts/` and
  `about.md`
- **THEN** the left pane lists those content entries

#### Scenario: Schema menu view lists the model's menu
- **WHEN** `Browse` shows the schema-menu view for a site whose model has a
  "Content" group with Singles/Collections
- **THEN** the left pane lists that menu's groups and entries

### Requirement: Toggle between the two views

The UI SHALL provide an action to toggle the `Browse` navigation between the
content-tree view and the schema-menu view, preserving `Browse` mode across the
toggle. The configured default view SHALL be honoured when a site is opened.

#### Scenario: Toggling switches the active view
- **WHEN** the toggle action is triggered while showing the content tree
- **THEN** the schema menu becomes the active view and the mode remains `Browse`

#### Scenario: Default navigation view is configurable
- **WHEN** a site is opened with the "schema nav first" preference enabled
- **THEN** the schema-menu view is shown first
