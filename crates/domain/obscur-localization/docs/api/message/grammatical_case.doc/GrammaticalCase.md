A grammatical case, as a CLDR value name.

Open for the same reason [`GrammaticalGender`](super::GrammaticalGender) is,
only more so: `specs/cldr/grammaticalFeatures.xml` lists **28** distinct case
values across its locales, and the eight Russian uses (`nominative genitive
dative accusative instrumental prepositional vocative locative`) are not a
superset of anyone else's — Hungarian alone contributes `delative`,
`sublative`, `superessive`, `terminative` and more.

Case variants matter to a UI toolkit for a plain reason: a message that
interpolates a noun cannot use the dictionary form everywhere. Russian «нет
{file}» needs the genitive of the word that «{file}» is nominative for. The
mechanism is the same as gender's — the case selects an arm by name — and the
*catalogue*, which is the only party that knows the actual word forms, holds
them.
