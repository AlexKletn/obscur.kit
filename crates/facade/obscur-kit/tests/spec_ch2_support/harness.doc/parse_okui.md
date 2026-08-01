Parses `source` as a `.okui` document.

# Panics

Panics when the source does not parse into a document at all — a parse
failure is a different verdict from the compile diagnostics under test, and
silently swallowing it would make every rejection case vacuous.
