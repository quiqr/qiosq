## ADDED Requirements

### Requirement: Render the agent output in the right pane

The UI SHALL render the coding agent's current output in the right pane, supplied
by the host as the latest snapshot of the agent session, refreshed as new output
arrives. The user SHALL never be attached to the raw agent/multiplexer session;
only its rendered snapshot is shown.

#### Scenario: Agent output appears in the right pane
- **WHEN** the host supplies the agent's latest snapshot lines and the UI renders
- **THEN** the right pane shows those lines instead of the static placeholder

#### Scenario: Empty agent output falls back to a label
- **WHEN** no agent output has been produced yet
- **THEN** the right pane shows a neutral agent-pane label rather than being blank
  or borrowing the left pane's content

### Requirement: Snapshots drive the render, not attachment

The agent pane SHALL be updated by pushing snapshot text into the UI state; the UI
SHALL NOT open, attach to, or read the underlying session directly. This keeps the
render path the same whether the agent is real (rmux) or fake.

#### Scenario: The same render path works for a fake agent
- **WHEN** a fake agent's snapshot lines are pushed into the UI state
- **THEN** the right pane renders them identically to a real agent's snapshot
