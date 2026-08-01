Checks one element instance against `contract`, returning every violation it
commits (spec §2.12, §2.13).

The checks, in order:

- **required props** — every required prop must appear as an attribute;
- **unknown attributes** — when the prop surface is closed, every non-event
  attribute must be declared or [reserved](is_reserved_attribute);
- **unknown events** — when the event surface is closed, every `on:<event>`
  must be declared;
- **allowed children** — when the contract restricts children, every member's
  tag must be listed, and a contract declaring [`ChildPolicy::None`] refuses
  members and content alike;
- **slots** (§2.12) — every member's `slot="…"` must name a declared slot and
  satisfy that slot's own child restriction, and each slot's
  [`SlotCardinality`] must hold.
