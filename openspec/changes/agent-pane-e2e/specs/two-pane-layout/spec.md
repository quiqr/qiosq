## MODIFIED Requirements

### Requirement: The right pane is the agent view

The UI SHALL reserve the right pane for the coding-agent session view. It SHALL
render the agent's current output (the latest snapshot supplied by the host) when
the agent is running, and SHALL fall back to a neutral agent-pane label when there
is no output yet — so the user always understands where the agent appears and the
two-pane shape is preserved. The user SHALL never be attached to the raw session.

#### Scenario: Right pane shows agent output when available
- **WHEN** the host has supplied agent output and the app is rendered
- **THEN** the right pane shows that output

#### Scenario: Right pane shows a label when there is no output
- **WHEN** the agent has produced no output yet
- **THEN** the right pane shows a neutral agent-pane label, not blank and not the
  left pane's content
