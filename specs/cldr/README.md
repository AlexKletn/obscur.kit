# Vendored CLDR data

The normative Unicode CLDR sources `obscur-localization` (spec §7.2) is transcribed
from, pinned to one dated release. They sit beside `specs/w3c/` for the same reason
that directory exists: *a factual claim about a spec must be read, not remembered*,
and a citation nobody can check is a citation nobody should trust.

All files are taken verbatim from **CLDR release-48**:

| File | Pinned source |
| --- | --- |
| `plurals.xml` | https://github.com/unicode-org/cldr/blob/release-48/common/supplemental/plurals.xml |
| `scriptMetadata.txt` | https://github.com/unicode-org/cldr/blob/release-48/common/properties/scriptMetadata.txt |
| `grammaticalFeatures.xml` | https://github.com/unicode-org/cldr/blob/release-48/common/supplemental/grammaticalFeatures.xml |

The operand definitions (`n i v w f t c`) and the rule grammar these files are read
against are TR35 part 3 (Numbers), §Language Plural Rules — *not* vendored, because
nothing in this workspace is transcribed from its prose:
https://github.com/unicode-org/cldr/blob/release-48/docs/ldml/tr35-numbers.md

## What each file backs

- **`plurals.xml`** — every cardinal rule in
  `crates/domain/obscur-localization/src/plural/plural_rules.rs`. Each rule function
  there quotes its CLDR line verbatim in its doc comment, and its `locales="…"`
  attribute is the language list the function is registered under, so a reader can
  `grep` this file and check the transcription by eye. The `@integer` sample lists on
  each `pluralRule` are transcribed into that file's tests.
- **`scriptMetadata.txt`** — field 6 (`RTL`) is the whole of `RTL_SCRIPTS` in
  `src/locale/script_direction.rs` (37 scripts).
- **`grammaticalFeatures.xml`** — the value spaces of `GrammaticalCase` (28 values)
  and `GrammaticalGender` (7 values) in `src/message/`. These are open sets on
  purpose: the file gives Russian eight cases and three genders but Polish five
  *genders* (`animate inanimate personal feminine neuter`), so a closed Rust enum
  would be wrong on arrival.

## `likelySubtags.xml` is referenced but not vendored

`RTL_LANGUAGES` in `src/locale/script_direction.rs` (258 languages) is **derived**,
not transcribed: it is every bare language subtag in

  https://github.com/unicode-org/cldr/blob/release-48/common/supplemental/likelySubtags.xml

whose likely script is one of `scriptMetadata.txt`'s RTL scripts. The file is 900 KB
— thirteen times the rest of this directory combined — and no line of it is quoted
anywhere, so the pinned URL plus the derivation carries the same audit value at 0.7%
of the cost. The derivation is exactly:

```python
# language -> likely script, for bare language subtags only
for m in re.finditer(r'<likelySubtag from="([^"]+)" to="([^"]+)"', likely_subtags_xml):
    frm, to = m.group(1), m.group(2)
    if "_" in frm:                      # bare language subtags only ("ar", not "ar_EG")
        continue
    parts = to.split("_")
    if len(parts) > 1 and parts[1] in rtl_scripts:   # rtl_scripts from scriptMetadata.txt field 6
        rtl_languages.add(frm)
```

Re-run it against a newer release to refresh the table; that is a deliberate,
reviewable commit, because a version bump changes what a locale resolves to.

## Licence

CLDR data files are published by Unicode, Inc. under **`Unicode-3.0`**; the terms are
at https://www.unicode.org/copyright.html. They are third-party documents and are not
covered by this workspace's own licence. Every file here is byte-for-byte the published
artefact (unlike `specs/w3c/`, which holds text renderings of HTML), so whatever header
Unicode shipped is intact.

**What each file actually carries** — checked, not assumed, because the first version of
this section claimed an SPDX line in "the header of each file" and only one has one:

| file | copyright header | `SPDX-License-Identifier` |
|---|---|---|
| `plurals.xml` | yes | **yes** — `Unicode-3.0` |
| `scriptMetadata.txt` | yes (© 1991–2016) | **no** |
| `grammaticalFeatures.xml` | yes (© 2019) | **no** |

The two without an SPDX line are almost certainly under the same terms — they ship in
the same CLDR release, from the same publisher, and Unicode applies one licence to the
repository — but that is an inference, not a declaration in the file. Stated so that
anyone auditing the vendored third-party text finds the gap here rather than
discovering the README was wrong about it.
