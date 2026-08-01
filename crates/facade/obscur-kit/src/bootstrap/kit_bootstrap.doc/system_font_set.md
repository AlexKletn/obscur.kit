Reads the host's own `sans-serif` / `serif` / `monospace` faces off disk via
fontconfig, as a [`FontSet`](super::FontSet) ready for
[`install_text_stack`](Self::install_text_stack) (spec §5.18).

This is what "use the system font" has to mean for a toolkit that shapes its
own text: a CSS family name is not enough — the shaper reads `cmap`/`GSUB`
itself, so something must find the *file*. Each generic is registered under
its own [`GenericFamily`](crate::text::font::GenericFamily), so
`font-family: monospace` resolves to the host's mono face rather than to
whatever came first.

Returns `None` when fontconfig is unavailable or nominates nothing loadable.
A caller that needs text regardless must supply bytes itself.

# Scope

**Linux only.** macOS (Core Text) and Windows (DirectWrite) are **not
implemented** — this returns `None` there, and no text will paint from this
path. Both want a real platform API rather than a CLI shell-out, and neither
is built.

# This is not for tests

The faces it returns are whatever the machine happens to have installed, so
any assertion about a metric drifts per machine. On the authoring machine
`fc-match sans-serif` resolves into `/usr/share/fonts/WindowsFonts/`. Tests
build a [`FontSet`](super::FontSet) from the vendored faces in
`assets/fonts/` instead.
