Reserved [`ComponentId`] of a **slot outlet** — the marker a component's
output uses to say "the content assigned to this slot goes here".

Nothing is ever registered under this id and no node ever retains it:
[`project_slots`] replaces every outlet with the content assigned to its
name before the output reaches the reconciler.

It sits in a reserved range of its own (`0x00B0_xxxx`), below the primitive
(`0x00C0_xxxx`) and widget (`0x00D0_xxxx`) ranges and clear of the top-bit
namespace `user_component_id` hashes into, so it can alias neither a shipped
component nor an author's.
