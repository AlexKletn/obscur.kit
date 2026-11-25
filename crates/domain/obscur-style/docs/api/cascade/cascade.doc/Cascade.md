The cascade engine (§4.14).

Given the declarations that matched a node, [`Cascade::resolve`] picks, for
each [`PropertyId`], the declaration with the greatest
[`CascadePriority`] and returns its value.

The engine is stateless; it is a zero-sized handle whose only job is to host
the resolution algorithm.

# Cascaded, not computed

[`Cascade::resolve`] produces the **cascaded value** per property: the raw
winning value straight from the source declaration. Turning a cascaded value
into a **computed value** — resolving the CSS-wide keywords
(`inherit` / `initial` / `unset`), inheriting from the parent element, and
substituting `var()` custom-property references — is the job of the next
sub-step (computed style) and is deliberately **not** done here.
