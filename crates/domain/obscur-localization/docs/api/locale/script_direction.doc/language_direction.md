Returns the base [`TextDirection`] implied by a `language` subtag alone, for
a tag that names no script.

This is CLDR's likely-script inference collapsed to the one bit this layer
needs. An unlisted language answers [`TextDirection::LeftToRight`] — which is
a *guess*, and the reason [`LanguageTag::parse`](crate::locale::LanguageTag::parse)
keeps an explicit script subtag when the caller supplies one.
