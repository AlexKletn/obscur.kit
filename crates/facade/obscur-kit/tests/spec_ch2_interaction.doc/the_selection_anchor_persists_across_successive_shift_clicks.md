§2.7: "- anchor and active item;" and "- range selection;"

SETUP REPAIRED 2026-07-19: this drove `SelectionPolicy::default()`, which is
`SelectionMode::Single` — and for Single, collapsing a range to its last item
is the documented, correct behaviour, so the case asserted multi-selection
against a policy that forbids it. §2.7 lists "- single selection;" and
"- multi-selection;" as separate capabilities, and a range gesture is only
meaningful under the latter, so the policy is now explicitly Multiple. The
assertions are unchanged.
