Installs `hyphenator` as the hyphenation port used to break an over-wide
word, replacing the default (no hyphenation) (spec §5.10).

This is the seam the runtime's
[`set_hyphenator`](crate::runtime::UiRuntime::set_hyphenator) drives. The
same hyphenator sizes text in the measure pass and paints it in the
compose pass, so a hyphenated line's measured height and painted line count
agree. Leaving it unset preserves headless determinism.
