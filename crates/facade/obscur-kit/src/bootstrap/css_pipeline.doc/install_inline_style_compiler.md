Arms the runtime's `style` attribute, once.

The runtime holds the cascade but not the grammar — an application-tier crate
cannot depend on a frontend one (§1.11) — so it accepts the compiler as a
function and this is the one place that owns both halves. The consequence
worth knowing: `style="…"` means something exactly where a CSS pipeline was
installed, and is carried but never compiled where one was not.

Guarded on [`UiRuntime::has_inline_style_compiler`], because `install` runs
again on every live stylesheet replacement and a re-install would discard every
block compiled so far for no gain — an inline block does not depend on the
sheets it outranks.
