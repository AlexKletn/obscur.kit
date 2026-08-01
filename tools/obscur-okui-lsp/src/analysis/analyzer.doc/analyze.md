Analyzes the document at `uri`, returning every diagnostic in LSP shape.

Returns an empty vector if no document is open at `uri`. Parse errors are
reported first; only when the source parses cleanly is it compiled and
the compiler's diagnostics appended.
