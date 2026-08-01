Compiles `document`, running each pass in order and accumulating every
diagnostic into the returned [`CompileResult`] (spec §2.15).

# Pipeline

1. **Name resolution** — every reference is resolved against a symbol
   table seeded with the [`defining`](Self::defining) symbols.
2. **Type checking** — no whole-document pass exists yet: the type layer
   is a per-expression vocabulary/algebra pending an expression AST
   (§2.9/§2.10), so nothing is emitted here. The
   [`Type`](CompileDiagnostic::Type) variant reserves its place in the
   diagnostic sum for when that pass lands.
3. **Validation** — expression restrictions (§2.10), capability grants
   (§2.13), the handler references against the declared
   [`CommandSurface`] (§2.13's "string method names", silent until a caller
   declares one — see [`claiming_command`](Self::claiming_command)), the
   §2.11 collection rule (a `<VirtualFor>` is refused, see
   [`CollectionValidator`]), the §2.12 outlet rule (an authored
   `<slot name="…"/>` carries a literal name and nothing else, see
   [`OutletValidator`]) and, for every element whose tag has a registered
   contract, the contract surface (required props, unknown
   attributes/events, allowed children — §2.13).
4. **Lowering** — the document is lowered to a [`UiIrDocument`]; on a hard
   lowering error the document is `None` but all earlier diagnostics are
   still returned.
