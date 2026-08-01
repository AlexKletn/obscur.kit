Strips every `#[cfg(test)] mod tests { ... }` block from `source` by brace
counting, so a scan of public API surface never reads unit-test code.
