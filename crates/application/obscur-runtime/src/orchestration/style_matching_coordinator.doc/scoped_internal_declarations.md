The cascade for a component-internal `node`: the system default (UA) sheet
applies in full, only `::part()` rules the host exports cross from any other
origin, and every other ordinary selector is contained.

# Why [`CascadeOrigin::SystemDefault`] is not scoped

§4.15 scopes *stylesheets an author wrote* — that is what "internal classes
do not need global uniqueness" and "components may export style parts" are
about. The system default sheet is not one of those: it is the built-in
definition of what a `<button>`, an `<input>` or an `<a>` **is**, and it sits
at the lowest cascade origin (§4.14), below every author declaration. Scoping
it out does not encapsulate anything — nothing outside authored it, so
nothing outside can be leaking in — it merely means a base element rendered
by a component is not that element any more. The external consumer audit's
finding 2-1 caught exactly this shape: a `<button>` inside a component
computed `color: rgb(0,0,0)`, the initial value, where the same `<button>`
outside got the UA sheet's ink.

The `source_order` counter advances over **every** encountered declaration
(matching or not), exactly as [`StyleMatcher::match_element_in`] does, so
document-order tie-breaks are identical for the declarations that do cross.
