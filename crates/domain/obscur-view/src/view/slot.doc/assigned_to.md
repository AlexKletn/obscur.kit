Whether `node` is assigned to the slot called `name`, where the empty name is
the default slot.

Content that carries no `slot="…"` is assigned to the default slot and to no
named one; a text run, which can carry no property and therefore no
assignment, is always content of the default slot. Content that names a slot
reaches that slot only — naming one is how an author says "not the default".
