## ADDED Requirements

### Requirement: Modes and their order

The UI SHALL implement a mode state machine with the modes `SiteList`, `Browse`,
`ViewFile`, and `Agent`. The app SHALL start in `SiteList`. Forward transitions
SHALL follow `SiteList → Browse → ViewFile → Agent`, and a Back action SHALL
return to the previous mode in that order.

#### Scenario: App starts in SiteList
- **WHEN** the app state is created
- **THEN** its mode is `SiteList`

#### Scenario: Opening a site enters Browse
- **WHEN** a site is selected in `SiteList` and the open action is triggered
- **THEN** the mode becomes `Browse` for that site

#### Scenario: Opening a file enters ViewFile
- **WHEN** a file is selected in `Browse` and the open action is triggered
- **THEN** the mode becomes `ViewFile` for that file

#### Scenario: Ask AI enters Agent
- **WHEN** the Ask AI action is triggered in `ViewFile`
- **THEN** the mode becomes `Agent`

#### Scenario: Back returns to the previous mode
- **WHEN** the Back action is triggered in `ViewFile`
- **THEN** the mode returns to `Browse`

### Requirement: Key events drive transitions via a pure reducer

The UI SHALL translate key events into typed actions/transitions through a pure
function that takes the current state and a key and returns the next state (and
any emitted action), without performing I/O — so the state machine is testable
deterministically.

#### Scenario: A key event yields a deterministic next state
- **WHEN** the reducer is given a state and a key event
- **THEN** it returns the resulting state (and any action) with no side effects,
  the same way every time for the same inputs

#### Scenario: Quit is available
- **WHEN** the quit key is pressed
- **THEN** the reducer emits a quit action the host loop can act on
