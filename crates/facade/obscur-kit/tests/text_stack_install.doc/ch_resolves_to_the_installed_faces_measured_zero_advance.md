`ch` must resolve to the installed face's own measured `0` advance.

# Why this asserts against the *sans* face

The obvious test — `font-family: monospace` — is **vacuous here**, and the
coincidence is worth naming so nobody "fixes" this back:
[`MockTextShaper::ADVANCE_RATIO`](obscur_kit::text::shaping::MockTextShaper::ADVANCE_RATIO)
is `0.6`, and Noto Sans Mono's `0` advance is `600/1000` upem — the same ratio,
at every font size. So the mock and the real mono face agree on `1ch` to the
bit, and a mono assertion would pass with the whole install reverted.

Noto Sans's `0` is 572/1000, so it discriminates: 9.152px against the mock's
9.6px, and against the `0.5em` substitute's 8.0px.
