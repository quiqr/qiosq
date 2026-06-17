## ADDED Requirements

### Requirement: Context-sensitive legend per mode

The UI SHALL display a function-key legend whose entries depend on the current
mode, mirroring the WordPerfect-5.1 bottom legend. Each entry SHALL pair a key
label (e.g. `F6`) with an action label (e.g. `Ask AI`). The legend SHALL change
when the mode changes.

#### Scenario: Browse legend offers preview and open
- **WHEN** the mode is `Browse`
- **THEN** the legend includes a Preview entry (F5) and an entry to toggle the
  navigation view, and does not include a Save or Discard entry

#### Scenario: ViewFile legend offers Ask AI and never an edit verb
- **WHEN** the mode is `ViewFile`
- **THEN** the legend includes an "Ask AI" entry (F6) and includes no verb that
  edits the file in place (the viewer is read-only)

#### Scenario: Agent legend offers Save, Discard, and Back
- **WHEN** the mode is `Agent`
- **THEN** the legend includes Save (F7), Discard (F9), and a Back entry

#### Scenario: Legend reflects a mode change
- **WHEN** the mode transitions from `Browse` to `ViewFile`
- **THEN** the rendered legend changes to the `ViewFile` legend
