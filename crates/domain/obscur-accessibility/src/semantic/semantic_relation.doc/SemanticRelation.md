A typed relation from one accessibility node to another (spec §7.1
relations).

Each variant expresses an ARIA-like relationship and carries the
[`NodeId`] of the related retained-tree node as its single target,
retrievable via [`target`](Self::target). Platform adapters translate these
into their native relation types (e.g. `aria-labelledby`,
`aria-controls`).
