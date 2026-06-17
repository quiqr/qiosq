# readonly-file-view Specification

## Purpose
TBD - created by archiving change agent-bridge. Update Purpose after archive.
## Requirements
### Requirement: Display file contents read-only

In `ViewFile` mode the UI SHALL display the contents of the opened file for
reading, with no affordance to edit, save, or delete it in place. The file's
bytes SHALL be supplied to the UI by the host; the UI library SHALL NOT itself
read or write files.

#### Scenario: Opened file contents are shown
- **WHEN** the host supplies the contents of the opened file and the UI is in
  `ViewFile` mode
- **THEN** the viewer renders those contents

#### Scenario: Viewer offers no edit verb
- **WHEN** the UI is in `ViewFile` mode
- **THEN** the only mutating affordance offered is "Ask AI" (routing the change
  to the agent); there is no Edit/Save/Delete verb in the viewer

### Requirement: Ask AI from the viewer routes to the agent

From `ViewFile`, the "Ask AI" affordance SHALL produce a request to send the
opened file's intent to the agent (entering the agent mode), rather than
modifying the file directly.

#### Scenario: Ask AI emits an intent request for the open file
- **WHEN** the user triggers "Ask AI" while viewing a file
- **THEN** the UI emits a request carrying that file's path for the agent and
  does not modify the file

