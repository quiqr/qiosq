## ADDED Requirements

### Requirement: Choose a free port in the configured range

The system SHALL select a port for the preview server from within the configured
`preview.port_range` (inclusive), choosing one that is currently free by probing
with a TCP bind. When no port in the range is free, it SHALL return a clear error
naming the range.

#### Scenario: Picks a free port in range
- **WHEN** a port is requested for a range with at least one free port
- **THEN** the returned port lies within the range and was bindable at selection
  time

#### Scenario: Skips a port already in use
- **WHEN** the lowest port in the range is already bound by another listener
- **THEN** the selected port is a different, free port within the range

#### Scenario: No free port errors clearly
- **WHEN** every port in the range is occupied
- **THEN** selection returns an error that names the configured range

### Requirement: Never select Quiqr's reserved port

The system SHALL never select port `13131`, even if it falls within the
configured range and is free, because that is Quiqr Server's default Hugo port.

#### Scenario: 13131 is skipped within a range that includes it
- **WHEN** the configured range includes `13131` and it is free
- **THEN** the selected port is not `13131`
