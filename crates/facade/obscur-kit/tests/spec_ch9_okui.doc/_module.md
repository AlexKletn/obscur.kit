**SPECIFICATION-DERIVED** cases for §2.10 Expressions ("Forbidden:
- assignment; - unconstrained arbitrary function calls; - I/O; - async work;
- direct service access;"), §2.9 ("- no business logic."), §2.12 ("Templates
are stateless and lifecycle-free.") and §2.13 ("Unknown attributes, events and
string method names are compile errors.") — the invariants behind the §9.1
non-goals "automatic execution of business logic inside `.okui`" and "a full
HTML compatibility layer".

Every expected value was produced by an agent reading the specification with
**no access to this repository's source**. A failure here is a defect report
about the engine, and must not be "corrected" by editing the expected value.
