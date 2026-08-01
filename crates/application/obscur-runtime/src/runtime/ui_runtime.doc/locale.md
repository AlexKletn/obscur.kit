The application's current locale (spec §7.2), `und` until a host calls
[`set_locale`](Self::set_locale).

Retained so [`locale`](Self::locale) can answer it; the value that *acts*
is the language hint `set_locale` pushes into the paragraph orchestrator,
which is where shaping reads it from.
