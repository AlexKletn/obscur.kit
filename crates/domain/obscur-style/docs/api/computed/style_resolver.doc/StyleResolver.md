Stateless resolver that computes a node's [`ComputedStyle`] from its cascaded
values, its cascaded custom properties, and its parent's computed style.

It carries no state; construct one with [`StyleResolver::new`] (or
[`Default`]) and call [`StyleResolver::resolve`]. Resolution is deterministic,
panic-free, and cycle-safe on `var()`.
