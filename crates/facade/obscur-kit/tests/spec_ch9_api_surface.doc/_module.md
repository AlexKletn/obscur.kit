**SPECIFICATION-DERIVED** cases for §1.1 ("Concrete window-system, GPU,
filesystem, networking and platform types never leak upward."), §1.3
("Unused backends and optional integrations must not be linked.") and §2.3
("Events are typed. String-based method dispatch is not used.") — the
invariants behind the §9.6 guardrails "leak platform types into core" and
"introduce untyped string dispatch", and the §9.1 non-goal "arbitrary
networking in the core".

These are linkage and API-surface claims, not behaviour claims, so they are
asserted against the repository's own manifests and public signatures.

Every expectation was written by an agent reading the specification with **no
access to this repository's source**. A failure here is a defect report about
the engine, and must not be "corrected" by editing the expected value.
