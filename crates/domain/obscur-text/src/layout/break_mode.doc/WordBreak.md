The computed `word-break` value (CSS Text 3 §5.2).

`word-break` selects the *soft-wrap* granularity within a run of text.
[`Normal`](Self::Normal) uses the ordinary rules (break between words, and —
via the default breaker — between adjacent CJK ideographs).
[`BreakAll`](Self::BreakAll) permits a break between any two typographic
units, so a long word wraps greedily mid-word. [`KeepAll`](Self::KeepAll)
forbids breaking between the letters of CJK-context text (its main observable
effect versus `normal`).
