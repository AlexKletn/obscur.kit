Counts lines above the `#[cfg(test)] mod tests {` block, excluding Rustdoc
line comments.

The whole file when there is no such block. The visibility is optional
because several crates write `pub(crate) mod tests` so a sibling entity's
tests can borrow a fixture builder.
