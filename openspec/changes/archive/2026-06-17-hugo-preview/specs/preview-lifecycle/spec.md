## ADDED Requirements

### Requirement: Start a preview server for a site

The system SHALL start `hugo server` for an opened site with its working
directory pinned to the site root, bound to `127.0.0.1` on the selected port,
using the configured hugo binary. On success it SHALL return a handle exposing
the served URL.

#### Scenario: Server starts and exposes its URL
- **WHEN** a preview is started for a valid Hugo site fixture
- **THEN** a running server handle is returned whose URL contains the selected
  port

#### Scenario: Served site is reachable
- **WHEN** the preview server has started for a site fixture
- **THEN** an HTTP request to the served URL connects and receives a response

### Requirement: Stop the server on close and on drop

The system SHALL stop the preview server when explicitly closed and SHALL also
ensure the child process is terminated if the handle is dropped without an
explicit stop, so no orphaned `hugo server` process is left running.

#### Scenario: Explicit stop terminates the process and frees the port
- **WHEN** a started preview is stopped
- **THEN** the hugo child process is no longer running and its port becomes
  bindable again

#### Scenario: Dropping the handle stops the server
- **WHEN** a started preview handle is dropped without calling stop
- **THEN** the hugo child process is terminated

### Requirement: At most one server at a time

For the PoC the system SHALL run at most one preview server at a time; starting a
new preview SHALL stop any previously running one.

#### Scenario: Starting a second preview stops the first
- **WHEN** a preview is running and a new preview is started
- **THEN** the previous server is stopped before (or as) the new one starts
