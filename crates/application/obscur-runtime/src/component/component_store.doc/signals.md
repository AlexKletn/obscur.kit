The current version of every observable source (spec §3.1). A component
records which of these its render read; comparing the two is what
invalidates only the consumers of a value that actually changed.
