The computed styles resolved for the last produced frame, retained so a
frame that skips the style phase (e.g. an idle diagnostics capture, or a
future paint-only class) can reuse them instead of re-resolving (§3.4
phase 5 incremental recalculation).
