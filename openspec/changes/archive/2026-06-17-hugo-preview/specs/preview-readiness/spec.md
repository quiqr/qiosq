## ADDED Requirements

### Requirement: Detect readiness from server output

The system SHALL determine that the preview server is ready by reading its output
for Hugo's readiness line ("Web Server is available at …") and SHALL surface the
URL reported there (or the URL derived from the selected port).

#### Scenario: Readiness line surfaces the URL
- **WHEN** Hugo prints its "Web Server is available at http://localhost:<port>/"
  line
- **THEN** the preview reports ready and exposes that URL

### Requirement: Time out when readiness is not reached

The system SHALL stop waiting after the configured `preview.ready_timeout_ms` and
return a clear error if the readiness line has not appeared, and SHALL not leave a
hung child process behind.

#### Scenario: Timeout yields an error and no orphan
- **WHEN** the server does not become ready within the configured timeout
- **THEN** starting the preview returns a timeout error and the child process is
  terminated
