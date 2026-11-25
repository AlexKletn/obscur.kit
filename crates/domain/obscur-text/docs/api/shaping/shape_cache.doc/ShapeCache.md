The memo table behind [`ParagraphShaper`](crate::shaping::ParagraphShaper):
shaped runs, keyed by the paragraph's text and by the style it was shaped
under, with a bound that evicts *some* of what it holds rather than all of
it.

# Why it is not one `HashMap`

The obvious form — `HashMap<WholeKey, Vec<ShapedRun>>` — costs on both the
paths that matter, and this one is on the frame path for every visible text
run.

**Eviction.** A single map has to be bounded somehow, and the cheap bound is
`if full { clear() }`. That is fine while the working set fits and
catastrophic the moment it does not: a directory listing of 1000 files at
two runs a row is ~2000 distinct paragraphs against a 512-entry bound, so
past the 512th *every* insertion empties the table and the next scrolled
frame re-shapes every visible run from scratch — permanently, with a hit
rate near zero rather than the ~99% the visible window should give. The
two-generation scheme here ([`young`](Self::young) / [`old`](Self::old))
drops at most half the table at a rotation, and only the half that has not
been touched since the previous rotation; anything looked at since is
promoted back on its way out. It is O(1) per operation — no recency list,
no scan for a victim.

**Probing.** A whole key has to be *built* to be hashed, which means cloning
the paragraph text and every font-family name into owned storage on every
lookup, hits included. Splitting the key in two removes that: the style half
lives once per group and is compared in place
([`ParagraphStyleKey::matches`](crate::shaping::paragraph_style_key::ParagraphStyleKey::matches)),
and the text half is the key of a `HashMap<Box<str>, _>`, which `&str`
probes borrowed. A hit now allocates nothing at all. The linear scan over
groups is not a regression: the number of live (size, chain, attributes)
combinations on a screen is a handful, while the number of distinct strings
is the whole document.

**Handing the result back.** Entries are `Arc<[ShapedRun]>`, so a hit is a
refcount bump rather than a deep copy of every glyph in the paragraph.
