Computes which nodes must be restyled when interaction state changes (§4.3).

# Why a structural reach, and why it is sound

The selector matcher navigates a tree **only** through
[`MatchElement`](obscur_style::selector::MatchElement), whose entire
navigation surface is `parent()` and `previous_sibling_element()`. So while
matching node `X`, the states the matcher can read are those of `X` itself and
of the nodes reachable from `X` by repeatedly taking a parent or a *preceding*
sibling. Inverting that relation: node `N`'s interaction state can only be read
while matching

* `N` itself and its descendants (they reach `N` through `parent()`), and
* the siblings that **follow** `N`, and their descendants (they reach `N`
  through `previous_sibling_element()`, possibly after some `parent()` hops).

which is exactly `subtree(N) ∪ ⋃ subtree(S)` over `N`'s following siblings —
what [`expand`](Self::expand) marks. The trait bound is what makes this a proof
rather than an enumeration of today's combinators: no implementation of the
matcher can consult a node outside that closure, because it has no method with
which to reach one. The selector-list pseudo-classes `:is()`, `:where()`,
`:not()` and `:nth-child(An+B of S)` are inside the closure too — each
evaluates its argument through the *same* seam, against the subject itself or
against a preceding sibling. `:has()` — the one selector that would reach
*downward*, and so let a node's state be read from outside the closure — is
modelled only for specificity:
[`PseudoClassSelector::Has`](obscur_style::selector::PseudoClassSelector::Has)
is answered `false` by the matcher unconditionally, as is the `of S` form of
`:nth-last-child()`, which would reach forward across siblings. **Wiring
either of them up invalidates the bound above and must widen this scope in
the same change.**

# Inheritance is covered by construction

A restyled node can change values its descendants inherit. That needs no extra
pass here because the marked set is **downward-closed**: every node it contains
is marked together with its whole subtree, so a descendant of anything that can
change is already in the set.

# What it does not narrow

The set is a sound **upper** bound, not a minimal one. It does not consult
which pseudo-classes the installed sheets actually mention, nor
[`StyleDependency`](obscur_style::invalidation::StyleDependency) — the per-node
record of what a node's matching really depended on — because nothing populates
that record during matching yet. Hovering a node inside a large subtree
therefore still restyles that subtree even when no rule mentions `:hover`.
