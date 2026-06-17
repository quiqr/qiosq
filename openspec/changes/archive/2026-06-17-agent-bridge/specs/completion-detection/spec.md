## ADDED Requirements

### Requirement: Detect completion via the sentinel

The system SHALL treat an agent task as complete when the configured completion
sentinel appears in the agent's output. Detection SHALL be based on the sentinel
string from configuration, so a deployment can choose its own marker.

#### Scenario: Output containing the sentinel completes
- **WHEN** the agent's output contains the configured sentinel (e.g.
  `<<QTUI_TASK_DONE>>`)
- **THEN** awaiting completion resolves as complete

#### Scenario: Output without the sentinel is not complete
- **WHEN** the agent's output does not yet contain the sentinel
- **THEN** awaiting completion does not report completion

### Requirement: Configurable sentinel

The completion sentinel SHALL be taken from configuration (the value validated by
`config-loading`), not hard-coded, so it can be changed without code changes.

#### Scenario: A custom sentinel is honoured
- **WHEN** the configured sentinel is a custom string and that string appears in
  the agent output
- **THEN** completion is detected on that custom string
