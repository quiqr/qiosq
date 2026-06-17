# agent-trait Specification

## Purpose
TBD - created by archiving change agent-bridge. Update Purpose after archive.
## Requirements
### Requirement: The Agent abstraction

The system SHALL define an `Agent` abstraction that can start a session pinned to
a working directory, receive an intent for a file, report when a task is
complete, and produce a snapshot of its current output. The coding agent SHALL
sit behind this abstraction so the implementation (Claude Code over rmux) can be
swapped — in particular for a fake agent in tests.

#### Scenario: A fake agent satisfies the abstraction
- **WHEN** a fake agent implementation is used in place of the real one
- **THEN** the rest of the system drives it through the same `Agent` operations
  without code changes

### Requirement: Sessions are detached and workdir-pinned

The real agent implementation SHALL run the agent in a detached session with its
working directory pinned to the opened site's repository, so the user is never
attached to a raw multiplexer session and the agent cannot act outside the site.

#### Scenario: Start pins the working directory
- **WHEN** the agent is started for a given site directory
- **THEN** the session's working directory is that site directory

#### Scenario: The user is never attached to the raw session
- **WHEN** the agent session is running
- **THEN** the system interacts with it via snapshots and sent input, never by
  attaching the user to the underlying session

