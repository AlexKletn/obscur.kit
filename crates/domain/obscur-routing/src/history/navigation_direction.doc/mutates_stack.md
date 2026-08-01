Returns whether this direction adds or replaces a history entry
([`Push`](NavigationDirection::Push) or
[`Replace`](NavigationDirection::Replace)).

[`Back`](NavigationDirection::Back) and
[`Forward`](NavigationDirection::Forward) only move the cursor over
existing entries and therefore return `false`.
