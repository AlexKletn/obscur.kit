The files that export several main entities today, each with its reason.

**A debt register, not a permission slip** — identical in spirit to the
ledger in `file_size.rs`. Every entry here must also carry [`MARKER`] in the
file itself, so a reader of the code sees the exception without reading
xtask. Paths are workspace-relative and use `/` on every platform.
