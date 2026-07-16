# Vendored W3C specifications

Normative CSS specifications, as plain text, pinned to the exact dated versions this
workspace is audited against. They sit beside `specs/obscur-kit-2-spec-drafts/` (this
project's own spec) because that spec and `docs/conformance-audit.md` cite these by
section number, and a citation nobody can check is a citation nobody should trust.

## Why these are in the repository

The campaign rule is *a factual claim about a dependency must be read, not remembered*.
A normative spec is the strongest form of that: the audit's verdicts are claims about
what CSS **requires**, and recalled CSS is how this workspace acquired several of the
bugs it has since fixed.

This is not hypothetical. Closing audit finding 4-A (`border-radius: 50%`) with the text
open found, in one sitting:

- the `box-shadow` spread rule cited as **§7.1.1**; it is **§6.1.1**;
- the overlapping-curves correction cited as **§5.1** and **§5.5**; it is **§4.5**;
- a normative rule that was simply **missing** — §6.1.1 scales the spread distance by
  `1 + (r−1)³` when a radius is smaller than the spread. Nothing recalled it, and it is
  correct at both endpoints, so no test caught it. It is now documented as open at
  `CornerRadii::offset`.

Three wrong citations and one missing rule, all found by reading rather than recalling.
That is the argument for the 4 MB.

## The case against, and why it does not win

- **Size.** 24 files, 4.0 MB of text — 1.0 MB compressed, which is what git stores. The
  repository's history is already 15 MB, and it already vendors third-party binaries far
  less compressible than text (`assets/fonts/NotoSans-Regular.ttf`).
- **Staleness.** The usual objection to vendoring a *living* standard. It does not apply
  here: every file pins an immutable, dated `/TR/YYYY/` snapshot (see the table below),
  not a floating `/TR/<shortname>/` URL. A vendored dated snapshot is *more* reproducible
  than a link, because the link moves and the snapshot cannot. When the audit should be
  re-based onto a newer draft, that is a deliberate, reviewable commit — which is the
  behaviour we want from a normative reference.
- **Fetch on demand.** Requires network access at exactly the moment someone is checking
  whether a verdict is honest, and silently returns a *different document* over time.

## Licence — read before redistributing

These are third-party documents, included verbatim under W3C's terms, and they are not
covered by this workspace's own licence.

- **23 of 24** carry the **W3C Permissive Document License**: *"Copyright © W3C. W3C
  liability, trademark and permissive document license rules apply."*
  **W3C Document License**: *"Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights
  Reserved. W3C liability, trademark and document use rules apply."* Those terms permit
  verbatim copying with the copyright notice intact but are more restrictive about
  derivative works than the permissive licence.

One caveat recorded rather than glossed: these are **text renderings** of documents
published as HTML, so they are not byte-for-byte the published artefact. The copyright
notice is retained in each file (the header of every file is intact). If strict
it is referenced only by audit entry 4-D, for CSS 2.1 §10.3.5 shrink-to-fit.

## Refreshing

Each file's own header carries its `This version:` URL — the pinned, dated snapshot it
came from. Re-fetch that URL, not the latest, unless the intent is to re-base the audit
onto a newer draft; a version bump changes what the audit's section numbers mean and
should be its own commit.

## Contents

| File | Version | Pinned source |
| --- | --- | --- |
| `css-align-3.txt` | W3C Working Draft, 30 January 2026 | https://www.w3.org/TR/2026/WD-css-align-3-20260130/ |
| `css-animations-1.txt` | W3C Working Draft, 2 March 2023 | https://www.w3.org/TR/2023/WD-css-animations-1-20230302/ |
| `css-backgrounds-3.txt` | W3C Candidate Recommendation Draft, 11 March 2024 | https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/ |
| `css-cascade-5.txt` | W3C Candidate Recommendation Snapshot, 13 January 2022 | https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/ |
| `css-color-4.txt` | W3C Candidate Recommendation Draft, 16 July 2026 | https://www.w3.org/TR/2026/CRD-css-color-4-20260716/ |
| `css-contain-3.txt` | W3C Working Draft, 18 August 2022 | https://www.w3.org/TR/2022/WD-css-contain-3-20220818/ |
| `css-display-3.txt` | W3C Candidate Recommendation Draft, 5 June 2026 | https://www.w3.org/TR/2026/CRD-css-display-3-20260605/ |
| `css-flexbox-1.txt` | W3C Candidate Recommendation Draft, 14 October 2025 | https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/ |
| `css-fonts-4.txt` | W3C Working Draft, 22 April 2026 | https://www.w3.org/TR/2026/WD-css-fonts-4-20260422/ |
| `css-grid-2.txt` | W3C Candidate Recommendation Draft, 26 March 2025 | https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/ |
| `css-images-3.txt` | W3C Candidate Recommendation Draft, 18 December 2023 | https://www.w3.org/TR/2023/CRD-css-images-3-20231218/ |
| `css-inline-3.txt` | W3C Working Draft, 18 December 2024 | https://www.w3.org/TR/2024/WD-css-inline-3-20241218/ |
| `css-overflow-3.txt` | W3C Working Draft, 7 October 2025 | https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/ |
| `css-position-3.txt` | W3C Working Draft, 7 October 2025 | https://www.w3.org/TR/2025/WD-css-position-3-20251007/ |
| `css-sizing-3.txt` | W3C Working Draft, 17 December 2021 | https://www.w3.org/TR/2021/WD-css-sizing-3-20211217/ |
| `css-syntax-3.txt` | W3C Candidate Recommendation Draft, 24 December 2021 | https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/ |
| `css-text-3.txt` | W3C Candidate Recommendation Draft, 8 June 2026 | https://www.w3.org/TR/2026/CRD-css-text-3-20260608/ |
| `css-transforms-1.txt` | W3C Candidate Recommendation, 14 February 2019 | https://www.w3.org/TR/2019/CR-css-transforms-1-20190214/ |
| `css-transitions-1.txt` | W3C Working Draft, 8 January 2026 | https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/ |
| `css-ui-3.txt` | W3C Recommendation, 7 April 2026 | https://www.w3.org/TR/2026/REC-css-ui-3-20260407/ |
| `css-values-4.txt` | W3C Working Draft, 12 March 2024 | https://www.w3.org/TR/2024/WD-css-values-4-20240312/ |
| `css-writing-modes-4.txt` | W3C Candidate Recommendation, 30 July 2019 | https://www.w3.org/TR/2019/CR-css-writing-modes-4-20190730/ |
| `selectors-4.txt` | W3C Working Draft, 22 January 2026 | https://www.w3.org/TR/2026/WD-selectors-4-20260122/ |

## css2.1 was dropped

The CSS 2.1 Recommendation was vendored here and then removed. Every other file
carries the **W3C Permissive Document License**, which allows a derivative with
attribution — and a text rendering of an HTML document is a derivative. CSS 2.1 is
the 2011 Recommendation under the older **W3C Document License**, whose terms are
verbatim-copy, so a text rendering of it is exactly what that licence does not
grant. Nothing in this repository referenced it. If a normative CSS 2.1 citation is
ever needed, link the dated snapshot rather than vendoring a re-rendered copy.
