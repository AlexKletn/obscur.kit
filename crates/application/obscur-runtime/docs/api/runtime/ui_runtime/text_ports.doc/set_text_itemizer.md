Installs a text `itemizer` on the owned scene coordinator, replacing the
default [`SingleRunItemizer`](obscur_text::shaping::SingleRunItemizer).

Text itemization is a hexagonal port: the runtime only knows the
`Box<dyn TextItemizer>`. Installing a real Unicode script/bidi itemizer
makes a text node split into multiple script runs, which — paired with a
covering [`set_font_resolver`](Self::set_font_resolver) — the fallback
orchestrator shapes and paints as several font-tagged segments. Leaving it
unset keeps the single-run default, preserving headless determinism.
