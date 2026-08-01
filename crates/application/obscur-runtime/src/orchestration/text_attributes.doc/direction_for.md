Resolves a text node's base [`TextDirection`] from its computed `direction`
(§4.14, §5.10).

Reads [`PropertyId::Direction`] — a [`PropertyValue::Keyword`] backed by one
of the reserved [`layout_keyword`] handles — returning
[`TextDirection::RightToLeft`] when it is `DIRECTION_RTL` and
[`TextDirection::LeftToRight`] otherwise (including `DIRECTION_LTR` and an
absent property). The itemizer uses this as the paragraph's base direction
and assigns bidi levels from it; the same value drives both the measure and
compose shaping passes so they agree.

# An absent `direction` ignores the locale, and that is a known gap

§7.2 requires localization to integrate with RTL, and
[`LanguageTag::direction`](obscur_localization::locale::LanguageTag::direction)
answers what a locale's base direction *is* — but nothing here asks it. An
application whose locale is `ar` and whose stylesheet never mentions
`direction` still lays its Arabic out left-to-right.

It is not wired because the honest fix is not small, and a half-wired base
direction is worse than none: this helper feeds **both** the measure pass
(`layout_coordinator`) and the compose pass (`scene_coordinator`), and a
locale default that reached one and not the other would size boxes for one
direction and paint the other into them. The two candidate fixes are
(a) thread the locale direction into `direction_for` — the measure path reaches
this only through `layout_coordinator`'s recursive `measure` chain, so each of
the 21 functions there that already thread `&ParagraphShaper` would have to
thread the direction beside it, plus the ~35 sites that forward it; or
(b) seed the root node's cascaded
`direction` so the existing inheritance carries it (`PropertyId::Direction` is
in the resolver's inherited set), which needs `StyleInputs` to distinguish a
UA-level default from the author cascade that owns it today. Neither is this
change's size.

The locale's language *is* wired, through
[`UiRuntime::set_locale`](crate::runtime::UiRuntime::set_locale) →
[`ParagraphShaper::set_language`](obscur_text::shaping::ParagraphShaper::set_language).
