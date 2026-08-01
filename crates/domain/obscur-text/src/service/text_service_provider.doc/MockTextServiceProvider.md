A deterministic, in-memory spellchecking [`TextServiceProvider`] for tests and
reference use (spec §5.13).

This is a **test / reference stub, not a real linguistic engine**. It supports
only [`ServiceKind::Spellcheck`], checking each whitespace-delimited token of
the request text against an in-memory [`known`](Self::known) dictionary
(case-insensitive). Any token absent from the dictionary is flagged with a
[`DiagnosticKind::Spelling`] / [`DiagnosticSeverity::Warning`] diagnostic whose
range is the token's **document** byte range (its offset within the request
text, translated by [`request.range().start`](LanguageServiceRequest::range)).

It is fully deterministic: the dictionary is a [`BTreeSet`] and tokens are
scanned left to right, so a given request always yields the same ordered
diagnostics. [`detect_language`](TextServiceProvider::detect_language) returns
`Some("en")` when any dictionary word appears, else `None`.
