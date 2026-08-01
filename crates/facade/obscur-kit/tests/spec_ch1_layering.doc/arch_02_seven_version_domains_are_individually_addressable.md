§1.2: "Independent version domains: Rust public API; supported CSS subset;
UI IR; diagnostics protocol; resource manifests; plugin contracts;
extension/provider contracts."

WEAKENED 2026-07-19: this originally asserted `domains.len() == 7` — that all
seven named domains expose a separately queryable version value. §1.2's
heading is "Independent version domains:" followed by the seven bullets, and
that is a *policy* statement: these things version independently of one
another and of the kit. It does not say each is addressable at runtime, and
three of the seven ("supported CSS subset", "resource manifests", "plugin
contracts", "extension/provider contracts") have no artifact that could carry
one today. What is kept is the half the policy really does assert, and the
half that discriminates: no reachable domain version is the kit's own.
Candidate spec amendment: state whether each version domain must expose a
queryable version constant, and where — this is the difference between a
documentation promise and a compatibility-negotiation surface.
