Checks that every handler a document references is a command somebody claims
(spec §2.13, "Unknown attributes, events and **string method names** are
compile errors").

# What it walks, and what it reads

Every element in the document, recursively, and for each of its `on:<event>`
attributes the handler reference: the *same* string the lowering's own
`attribute_value_text` hands it, which is the string
`name_binding::fallback_symbol` hashes into the command symbol the runtime
dispatches on. Reading it through that one function is deliberate — a checker
with its own extraction could admit a name the IR does not carry, and a
validator that disagrees with the artefact is worse than none.

All three authored spellings are therefore covered by construction:
`on:click=increment`, `on:click={increment}` and (already refused as string
dispatch by the [`ContractValidator`](super::ContractValidator))
`on:click="increment"`.

# Only a declared surface can be judged

The verdict comes from a [`CommandSurface`], and the default
[`Unchecked`](CommandSurface::Unchecked) surface reports nothing at all. A
command is claimed by a runtime registration, which does not exist while
anything is compiling, so the compiler cannot discover the set — the caller
declares it, exactly as it declares granted capabilities. Guessing instead
would turn a silent no-op into a false compile error, which is the worse of the
two failures.

# The empty-handler case

`on:click` with no value at all lowers to an empty handler string, which hashes
to a symbol nothing can ever register. It is reported as its own diagnostic
rather than as an unclaimed name, because "the command '' is not claimed" would
name nothing to fix.
