Sets the application's current locale (spec §7.2).

This is how a host tells the runtime what language its content is in.
Nothing reads the operating system here — §1.1 puts anything that touches
the environment in an adapter, and no adapter ships a locale reader yet, so
the host is the source of truth.

# What it actually changes

The locale's **language subtag** becomes the BCP-47 hint on every
[`ShapingRequest`](obscur_text::shaping::ShapingRequest) the paragraph
orchestrator issues, in both the measure and the compose pass (they share
the orchestrator, so they cannot disagree). That field has existed since
shaping was written and nothing has ever set it: the HarfRust adapter reads
it into `buffer.set_language`, which drives `locl` — the OpenType feature
that gives Serbian Cyrillic its own italic **б** where Russian Cyrillic
keeps the Russian one, from the same face and the same code points. Without
a language the shaper guesses from the text, and for Cyrillic it guesses
Russian.

# What it does not change

The base **direction**. `direction` still comes from computed CSS alone and
still defaults to left-to-right for an `ar` locale whose stylesheet is
silent — see
[`direction_for`](crate::orchestration::direction_for) for why that is not
wired here and what wiring it would cost. Message formatting is likewise
not the runtime's: a [`Localizer`](obscur_localization::catalog::Localizer)
is held by the application, which calls it before handing text to a widget.

# Why this does not touch the measure cache

The sibling shaping-port setters all drop
[`measure_cache`](Self::set_text_shaper) explicitly, because a port swap
changes measured sizes and nothing else would notice. A locale switch does
too — a language-sensitive substitution can be a different width — but the
language is a member of [`LayoutInputs`](crate::runtime::LayoutInputs), so the
next frame's classifier sees the mismatch, runs a **full** layout, and a full
layout clears the measure cache itself. Clearing it here as well would be a
second mechanism
for one guarantee, and the one that carries it is the one the boxes depend
on: without the `LayoutInputs` member the retained *boxes* would be reused
whatever the cache did, and the frame would paint newly-shaped glyphs into
sizes measured under the outgoing language.
