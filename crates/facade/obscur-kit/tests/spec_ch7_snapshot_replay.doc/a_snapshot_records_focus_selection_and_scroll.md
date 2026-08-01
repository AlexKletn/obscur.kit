§7.12, verbatim: "Development/diagnostic mode may serialize snapshots of:" /
"- focus;" / "- selection;" / "- scroll;" / "- animation;".

FIXTURE REPAIRED 2026-07-19: nothing here is weakened. The case set a scroll
offset of 120 on `.panel` and read back 0, and the shared `CSS` above styles
`.panel` as `width: 400px; height: 300px` with no `overflow` declaration — so
it is not a scroll container, `set_scroll_top` is correctly a no-op on it,
and there was no scroll state for any snapshot to record. The case now uses
its own stylesheet in which the panel actually scrolls (49 rows of 20px
against a 300px viewport, so 120 is well inside range). The assertion is
unchanged.
