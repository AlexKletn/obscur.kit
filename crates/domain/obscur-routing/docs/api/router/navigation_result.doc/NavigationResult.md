The typed outcome of a navigation through the router (spec §6.13).

A navigation either resolves a path to a registered target
([`Matched`](NavigationResult::Matched), carrying the target id, the params
the pattern captured, and the matched path) or finds no registered pattern
([`NotFound`](NavigationResult::NotFound), a deep-link miss carrying the path
that failed). The [`is_matched`](NavigationResult::is_matched),
[`target`](NavigationResult::target), and
[`params`](NavigationResult::params) helpers inspect the outcome without
matching on it.
