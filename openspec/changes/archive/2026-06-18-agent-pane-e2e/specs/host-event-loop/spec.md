## ADDED Requirements

### Requirement: Runnable host that wires the services

The `qtui` binary SHALL run an event loop that loads the configuration,
enumerates sites via the storage layer, builds the navigation model via the model
layer, renders the UI state machine, starts and stops the Hugo preview on site
open and close, and drives the agent — so that the PoC is an actually runnable
two-pane application, not just libraries.

#### Scenario: Binary boots into the site list
- **WHEN** `qtui` is run with a valid config pointing at a data directory
- **THEN** it loads, enumerates the sites, and presents the site-list view

#### Scenario: Opening a site starts the preview and shows navigation
- **WHEN** a site is opened in the running app
- **THEN** the preview server is started for it, its URL is surfaced, and the
  navigation (content tree / schema menu) is shown

#### Scenario: Exiting stops the preview
- **WHEN** the app exits
- **THEN** any running preview server is stopped (no orphaned Hugo process)

### Requirement: Headless scripted mode

The `qtui` binary SHALL support a headless/scripted mode (a `--script` flag or
equivalent) that runs the flow without a live terminal, accepting a sequence of
steps and using the configured agent, so the end-to-end test can drive it
deterministically.

#### Scenario: Scripted run exercises the flow without a TTY
- **WHEN** `qtui` is run in scripted mode with a step sequence and the fake agent
  configured
- **THEN** it performs the steps (open site, open file, ask AI, await completion)
  without requiring an interactive terminal and exits with a success code
