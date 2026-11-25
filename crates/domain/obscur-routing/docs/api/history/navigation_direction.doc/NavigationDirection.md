The way a navigation moves through the history stack (spec §6.13
back/forward).

Every navigation is one of four moves. Two of them *mutate* the stack by
adding or replacing an entry ([`Push`](NavigationDirection::Push),
[`Replace`](NavigationDirection::Replace)), and two of them merely move the
history cursor over entries that already exist
([`Back`](NavigationDirection::Back),
[`Forward`](NavigationDirection::Forward)). The
[`mutates_stack`](NavigationDirection::mutates_stack),
[`is_backward`](NavigationDirection::is_backward), and
[`is_forward`](NavigationDirection::is_forward) predicates classify a
direction without the caller matching on it.
