Creates a compiler whose lowered source map records spans under
`source_name` (the logical `.okui` source identifier).

The contract registry is seeded with the built-in primitive contracts
([`ContractRegistry::builtins`]) so a default compile already rejects
unknown attributes/events on the built-in primitives (spec §2.13);
[`with_registry`](Self::with_registry) replaces it wholesale.
