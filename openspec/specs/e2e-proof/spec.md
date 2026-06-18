# e2e-proof Specification

## Purpose
TBD - created by archiving change agent-pane-e2e. Update Purpose after archive.
## Requirements
### Requirement: VM boots Quiqr Server

The `checks.e2e` NixOS VM test SHALL boot a virtual machine running Quiqr Server
via the Quiqr NixOS module from the `nixpkgs-quiqr` input, and SHALL wait for the
service to be available before exercising the flow.

#### Scenario: Quiqr Server service comes up
- **WHEN** the e2e VM boots
- **THEN** the Quiqr Server systemd service reaches an active state and its data
  directory exists

### Requirement: Provision a sample site

The test SHALL provision a sample Quiqr site — including a `quiqr/model/` schema
with at least one Single and one Collection, and serveable Hugo content — into the
VM's Quiqr data directory before running the app.

#### Scenario: Sample site is present in the data dir
- **WHEN** provisioning completes
- **THEN** the data directory contains a site with a `quiqr/model/` schema and a
  `content/` tree

### Requirement: Drive the full flow with the fake agent

The test SHALL run `qtui` in headless/scripted mode against the provisioned site
with the **fake agent** configured (never a real LLM), exercising: site listed →
Hugo preview reachable → schema menu shows a Single and a Collection → a content
file opens read-only → "Ask AI" injects the intent → the fake agent writes a new
content file and emits the completion sentinel.

#### Scenario: Scripted flow completes and the agent writes content
- **WHEN** the scripted run executes against the provisioned site with the fake
  agent
- **THEN** the run completes successfully and the fake agent has written a new
  content file marked with the completion sentinel

### Requirement: Assert the on-disk result

The test SHALL assert the expected on-disk outcome — that the new content file
created by the agent exists in the site — and SHALL pass as part of
`nix flake check`.

#### Scenario: New content file exists on disk
- **WHEN** the scripted flow has completed
- **THEN** the test finds the new content file on disk and the check passes

