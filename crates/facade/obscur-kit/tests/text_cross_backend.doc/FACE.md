The face both backends shape against: the workspace's own Noto Sans.

Bundled bytes rather than a system lookup, so the glyph ids below mean the same
thing on every machine, and so the CPU backend takes its real outline path
(`draw_glyph_outlines`) rather than the box-coverage fallback it uses when a run
names no registered font.
