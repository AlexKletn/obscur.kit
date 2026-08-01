The multi-font text engine the compose phase shapes every paragraph through,
and the phase's one piece of state.

Text is shaped through a [`ParagraphShaper`] — the multi-font fallback
orchestrator composing an itemizer, a font resolver, and an inner
[`TextShaper`]. The default composition (a [`SingleRunItemizer`], an empty
[`MockFontResolver`], and a cache-wrapped [`MockTextShaper`]) reproduces the
historical single-font behavior exactly, so headless determinism is preserved;
a host installs a real engine, a Unicode itemizer, and a coverage registry
through [`SceneCoordinator`](super::SceneCoordinator)'s setters, which delegate
here.

The three ports are one entity rather than three fields because **the same
engine must shape a run in the measure pass and in the compose pass**: the
layout coordinator borrows this shaper (through
[`SceneCoordinator::paragraph`](super::SceneCoordinator::paragraph)) to size a
text box, and the compose walk shapes the drawn glyphs from it. Swapping the
inner shaper without dropping the memoized runs, or hyphenating in one pass and
not the other, makes a measured box height and a painted line count disagree —
which is why [`set_shaper`](Self::set_shaper) rewraps in a **fresh**
[`CachingShaper`] rather than replacing the engine underneath a live cache.

Because `ParagraphShaper` holds `dyn` ports it is neither `Clone` nor `Debug`,
so this type is not `Clone` and carries a manual [`fmt::Debug`].
