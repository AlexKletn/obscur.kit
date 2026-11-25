What the desktop said, as CSS custom properties an author can read.

**Only what was said.** A value the desktop did not state produces no property
at all, rather than a default chosen here — that is the same contract
[`SystemAppearance`](obscur_system::appearance::SystemAppearance) keeps, carried
one step further so a stylesheet can see it.

### The limit worth knowing before designing around this

The natural way to use these is `background: var(--obscur-sys-window-bg)`, and
**that form does not work today**: a shorthand whose value holds `var()` is
dropped whole (`docs/layer-consumer-audit.md`, finding 2-C). Longhands read the
variable — `background-color`, `border-color`, `color` — so a sheet written in
longhands works. So does the guard an author would reach for,
`var(--obscur-sys-accent, #06c)` — except that the fallback is discarded, so a
desktop that stated no accent paints the initial value rather than the fallback.

Both are open defects rather than design, and the ordering matters: a design
system built on these variables in shorthand form will look transcribed
carelessly, with no message to say otherwise.
