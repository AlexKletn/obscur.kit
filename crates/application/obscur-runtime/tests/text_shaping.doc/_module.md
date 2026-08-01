The text-shaping port, driven from outside: what the runtime asks the shaper and
the itemizer, how often, and what comes back.

Every test here installs a stub through a public `set_…` seam and reads the
result out of the composed display list, because the shaper's answer is only
worth anything if it survives the whole measure → arrange → compose path and the
caches in between. A stub emitting a fixed glyph id proves the injection reaches
paint; a `MockFontResolver` plus a two-script itemizer proves font *fallback*
does, producing two font-tagged runs on one line from one text node; and with no
ports installed at all, a text node still renders as one run on the primary face
with glyph ids equal to the Unicode scalars — the default path unchanged.

Two of the eleven are about *cost*, and they are the reason the counting stubs
exist. A text node is shaped in the measure pass, again in arrange, and again in
compose. The caching shaper and the shared paragraph cache must collapse all
three into **one** call to the underlying engine, per unique run and per frame,
and a tally cell read after the stub was boxed into the runtime is the only way
to see it.

The rest is the **language** the shaper is told (§7.2), which is a different
question from which shaper it is. The default locale is `und`, not `en` — the
Anglocentric default exists here to be refused — and until `set_locale` is
called no shaping request carries a language at all, which is the state this
work found: the field existed, the adapter read it, and no production path had
ever set it. What reaches the port is the language *subtag*, `ru` and not
`ru-RU`, because OpenType has no opinion about a region.

The last two are the cache assertions that give `set_locale` its teeth. A
locale-sensitive shaper — Serbian glyphs twice as wide as anyone's — makes a
language a *layout* input, one no computed-style diff can see. Switching locale
must therefore drop the retained measures as well as the paragraph cache, or
either one would happily serve the previous language's runs and the box would
keep the old width forever.
