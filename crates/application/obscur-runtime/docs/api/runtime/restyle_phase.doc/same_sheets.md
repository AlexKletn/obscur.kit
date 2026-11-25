Whether `now` is the very set `last` was, sheet for sheet.

Compares [`Stylesheet::stamp`](obscur_style::matching::Stylesheet::stamp),
which is identity, not content: the two sides of this question are always
clones of one compiled set, so a structural comparison could only ever walk
every selector of every rule and return `true`. It answers `false` for sheets
recompiled to identical bytes, and that is the safe direction — a `false` here
costs one full restyle, a wrong `true` keeps last frame's styles forever.
