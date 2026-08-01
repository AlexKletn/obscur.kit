The sensitivity classification of a piece of UI or data (spec §6.17, §7.12).

[`PrivacyClass`] tags a region, field, or value with how sensitive it is, so
the runtime can decide whether to exclude it from screen capture / sharing
(§6.17) and whether to redact it from diagnostic snapshots (§7.12). Variants
are ordered least → most sensitive, so the derived [`Ord`] ranks sensitivity
and the predicates below are monotone in it.
