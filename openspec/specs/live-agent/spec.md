# live-agent Specification

## Purpose
TBD - created by archiving change live-agent. Update Purpose after archive.
## Requirements
### Requirement: Agent implementation chosen from config

The host SHALL choose the agent implementation from configuration: when
`agent.command` names the in-tree fake agent, it SHALL use the in-process fake
agent; otherwise it SHALL use the real rmux-backed agent. Both are driven through
the same `Agent` abstraction, so the rest of the loop is identical.

#### Scenario: Fake agent selected for the test command
- **WHEN** `agent.command` points at the `fake-agent` binary
- **THEN** the host uses the in-process fake agent

#### Scenario: Real agent selected otherwise
- **WHEN** `agent.command` is any other command (e.g. `claude`)
- **THEN** the host uses the rmux-backed agent

### Requirement: Live session started lazily and pinned to the site

In interactive mode the host SHALL start the agent at most once — lazily, on the
first "Ask AI" — with its working directory pinned to the opened site, and SHALL
reuse that session for the rest of the run. The user SHALL never be attached to
the raw session.

#### Scenario: Agent starts on first Ask AI and is reused
- **WHEN** the user triggers "Ask AI" twice during a session
- **THEN** the agent session is started once (pinned to the site) and the second
  request reuses the same session

### Requirement: Intent is sent to the live session

"Ask AI" SHALL send the `@{path} I want to do the following… ` intent to the live
agent session (not spawn a one-shot process), so the agent can be continued.
Sending the intent SHALL NOT itself write any site file.

#### Scenario: Ask AI sends the intent to the running agent
- **WHEN** the user triggers "Ask AI" on the open file
- **THEN** the live agent receives the `@{path} …` intent and no site file is
  created by the act of sending

### Requirement: The agent pane streams live output

While an agent session is running, the interactive loop SHALL refresh the agent
pane from the agent's snapshot each render tick, so new output appears as it
arrives, and SHALL recognise task completion via the configured sentinel.

#### Scenario: Output appears as it is produced
- **WHEN** the live agent produces output over time
- **THEN** the right pane reflects the latest snapshot on subsequent ticks

#### Scenario: Completion is detected via the sentinel
- **WHEN** the agent's output contains the configured completion sentinel
- **THEN** the loop treats the task as complete

### Requirement: Scripted and headless runs keep the deterministic path

The headless/scripted (`--script`) and end-to-end paths SHALL continue to use the
deterministic subprocess fake-agent, requiring no rmux daemon and no real LLM, so
`nix flake check` remains green and offline.

#### Scenario: Scripted run needs no daemon
- **WHEN** `qtui --script …` runs with the fake agent configured
- **THEN** it completes the flow without starting an rmux daemon or contacting a
  real agent

