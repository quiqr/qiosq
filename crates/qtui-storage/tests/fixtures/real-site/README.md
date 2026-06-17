# real-site fixture

An **anonymized** copy of a real Quiqr/Hugo site, kept as a shared test fixture
so `qtui-storage` and `qtui-model` can be exercised against a realistic on-disk
shape (a genuine `quiqr/model/` schema with `menu`/`singles`/`collections`, and a
real-shaped `content/` tree) rather than only hand-built minimal fixtures.

Provenance: derived from a public Quiqr site; all author names, domains, and
prose have been replaced with generic placeholders. The Hugo theme and generated
output (`public/`, `resources/`) are intentionally omitted — only the structure
the storage/model layers read is kept. No theme is included, so this fixture is
for the read-only storage/schema layers, not for running `hugo server` (the E7
VM provisions a serveable site separately).
