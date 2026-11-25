Checks every element in the parsed tree against its component's contract
(spec §2.12, §2.13), reporting each violation at the token that caused it.

# This front end used to check nothing

`okui!` calls itself the spec's *primary* authoring surface, and it ran **no**
contract validation at all: `okui! { <Button laebl="OK"/> }` compiled clean
while the very same document in a `.okui` file was rejected. §2.13's "unknown
attributes, events and string method names are compile errors" held on the
fallback path only. The rule was never the problem — it was written against
the file parser's AST, which this front end does not have — so it now lives in
[`check_contract`] over plain data and both front ends reach it.

# What a proc-macro can and cannot see

The registry is [`ContractRegistry::builtins`]: the primitives and shipped
widgets, whose surfaces are compiled into this macro. A **user** component's
contract cannot be *seen* here and no amount of wiring will change that —
`MyProps::contract_surface()` is a function that runs when the built crate
runs, and this code runs while that crate is still being compiled.

The asymmetry with the file front end is **narrower than this doc used to
claim**. It said the file front end "takes its registry as an argument and so
can be handed one; a macro has no argument to take it in", which is true about
the argument and misleading about the reachability: on the `.okui` **build**
path there is nothing in scope to pass. `build.rs` of crate X cannot call
`X::CardProps::contract_surface()` either, and the edge that would let it —
`[build-dependencies] X` inside X — is rejected by cargo as a `cyclic package
dependency`. For a component defined in the same crate as the document that
uses it, both front ends are equally blind. The file path wins only when the
component lives in a *different* crate, or when the consumer is a running
binary rather than a compiler (the `.okui` language server, which is where
[`ContractRegistry::register_surface`] is actually reachable).

None of that means a user contract is unenforceable here — only unreadable.
A macro can *emit* a check the host crate's own `rustc` resolves (a
field-access probe against the props type yields a real `E0609` at the
author's typo, in the same crate). That is deferral rather than
introspection; it needs a tag→type mapping this repo does not have, and it is
noted in the audit's §2-A rather than attempted here.
