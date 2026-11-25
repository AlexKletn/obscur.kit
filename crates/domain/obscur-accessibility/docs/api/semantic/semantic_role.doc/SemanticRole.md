The semantic role of an accessibility node (spec §7.1).

A role is the ARIA-like classification a platform adapter maps onto its
native accessibility API (UI Automation control types, AX roles, AT-SPI
roles, or web `role` attributes). It covers the common cross-platform set;
[`Unknown`](Self::Unknown) is the neutral default for nodes whose role has
not been established.

Two coarse predicates classify roles without exhaustive matching:
[`is_interactive`](Self::is_interactive) for roles a user can operate, and
[`is_container`](Self::is_container) for roles that primarily group other
nodes.
