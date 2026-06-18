---
# qiosq-j5p7
title: 'E12 — Read-only viewer: markdown highlight + file-type gate'
status: todo
type: epic
created_at: 2026-06-18T10:07:49Z
updated_at: 2026-06-18T10:07:49Z
parent: qiosq-csrd
---

Goal: 'syntax highlight markdown files' and 'do not allow opening other files than markdown (later we will allow json/yaml/tomls).'

## Tasks (draft)
- [ ] Restrict the viewer/open action to .md files for now; clearly skip/deny others (config-driven allowlist so json/yaml/toml can be enabled later).
- [ ] Markdown syntax highlighting in the read-only viewer (a ratatui-friendly highlighter; keep qtui-ui pure/testable).

## Tests
- [ ] Opening a non-.md file is refused; .md opens. Highlight render smoke test.
