Renders one real frame of `v` under `css` in a fresh `w x h` runtime whose
**host reports `insets`** as its safe area (spec §6.15's typed environment
values).

The setup half of a §6.15 case: the section requires both that a host supply
safe-area insets and that CSS `env()` expose them, and a case asserting the
second has to perform the first. This states no expectation — it only reports
what the case's prose says the host reports.
