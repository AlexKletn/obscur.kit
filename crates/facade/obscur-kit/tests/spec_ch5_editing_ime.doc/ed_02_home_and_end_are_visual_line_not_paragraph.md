§5.11 "- word/line/document navigation;" / "- Home/End;". Home and End are
**visual-line** operations, not paragraph ones: on wrapped text they must stop
at the wrap, not at a `\n` or the buffer end.
