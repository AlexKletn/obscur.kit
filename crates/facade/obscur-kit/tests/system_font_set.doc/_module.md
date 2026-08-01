The desktop production path: `KitBootstrap::system_font_set` reads the host's
own faces off disk through fontconfig (spec §5.18).

# These tests are deliberately weak, and that is the point

Everything here depends on what the running machine has installed, so nothing
can assert a metric, a glyph id, or a family name without drifting per machine
— on the authoring machine `fc-match sans-serif` resolves into
`/usr/share/fonts/WindowsFonts/`. So this file asserts only the two properties
that must hold on *any* machine: whatever comes back is **loadable**, and
whatever comes back **shapes real glyphs**. The numeric assertions live in
`text_stack_install.rs` against the vendored faces, which is the split the
owner's policy asks for: system fonts for production, vendored fonts for tests.

A machine with no fontconfig skips rather than fails — a bare CI container is a
legitimate host, not a broken one.
