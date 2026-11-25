A stable symbol derived from a **name** rather than a hand-assigned index:
the FNV-1a 32-bit hash of `name`, lifted into the high namespace (top bit
set) so it never collides with a low-range reserved symbol.

This is what lets an author name a command or a list instead of inventing a
unique integer for it: `App::on_command(InternedSymbol::named("increment"),
…)` in place of `InternedSymbol::from_raw(0x1002)`. The scheme is deliberately
**identical** to the frontend's `fallback_symbol`, so a handler registered
under `named("save")` is reached by an `on:click={save}` written in markup —
the two meet on the name without either side knowing about the other. A drift
guard in `obscur-ui-ir` pins the two implementations equal.

The braced spelling is deliberate here, and it is not an expression: an
`on:` value is a command *name* on both front ends (`okui!` accepts only the
braced form; a `.okui` file also accepts the bare `on:click=save`). The
quoted `on:click="save"` this comment used to show is what §2.3 calls
string-based method dispatch, and the `.okui` contract validator refuses it.

`const`, so it can name a `const` symbol handle at the definition site.
