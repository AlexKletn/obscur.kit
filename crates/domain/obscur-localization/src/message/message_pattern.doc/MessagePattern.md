A parsed message pattern: literal text, interpolations, and the arm selection
that makes plural, gender and case agreement possible.

# Syntax

A deliberate subset of ICU MessageFormat, spelled the same way so a pattern
here reads the way translators already expect:

```text
Привет, {name}!
{count, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}
{actor, select, feminine{добавила} other{добавил}} файл
нет {noun, select, genitive{файла} other{файл}}
```

- `{name}` interpolates the argument's text.
- `{name, plural, <arm>…}` selects on the argument's [`PluralCategory`] under
  the *formatting locale's* [`PluralRules`]. Arms are CLDR category keywords
  (`zero one two few many other`) or `=<integer>` exact matches, which win over
  the category when they match (`=0{no files} other{# files}`). Inside a
  `plural` arm, `#` interpolates the number.
- `{name, select, <arm>…}` selects by name — this is how *gender* and *case*
  variants are expressed, since both are a choice among arms keyed by a value
  the caller binds.
- Every selector must have an `other` arm; [`parse`](Self::parse) rejects one
  that does not.
- Arms nest: an arm's body is a full pattern.

# No escape syntax

`{`, `}` and (inside a `plural` arm) `#` cannot appear as literal text. ICU
quotes them with apostrophes; this subset does not implement that, and rather
than accept an apostrophe convention it cannot honour, [`parse`](Self::parse)
**rejects** a stray brace. A message that must render a literal brace cannot
be expressed, and will say so at parse time rather than at paint time.
