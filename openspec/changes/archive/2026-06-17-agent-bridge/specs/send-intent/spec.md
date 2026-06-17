## ADDED Requirements

### Requirement: Inject the file intent prefix

When the user asks the agent about a file, the system SHALL send the agent an
intent that begins with `@{path} I want to do the following… `, where `{path}` is
the file the user opened, and SHALL then leave the cursor in the agent input for
the user to continue typing in plain language.

#### Scenario: Intent carries the @path prefix
- **WHEN** the user triggers "Ask AI" on the file `content/posts/hello.md`
- **THEN** the text sent to the agent begins with
  `@content/posts/hello.md I want to do the following… `

#### Scenario: Cursor is handed to the user
- **WHEN** the intent has been injected
- **THEN** the agent input retains the injected prefix and awaits further user
  input rather than being submitted immediately

### Requirement: The intent is a request, not a file write

Sending an intent SHALL NOT itself read, create, modify, or delete any site file;
it only forwards text to the agent, preserving the single-writer rule (only the
agent process writes).

#### Scenario: Injecting an intent writes no files
- **WHEN** an intent is sent for a file
- **THEN** no site file is created, modified, or deleted by the act of sending
