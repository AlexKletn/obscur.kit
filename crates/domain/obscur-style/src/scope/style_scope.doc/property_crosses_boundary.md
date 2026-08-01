Returns `true` when a property is of a kind that crosses a scope
boundary — i.e. it inherits by default (§4.15).

This is independent of this scope's kind: it classifies the *property*.
Inherited properties cross because inheritance flows through the tree
regardless of scope. Custom properties always inherit, so they always
cross (they are represented among the inherited properties).
Non-inherited properties do not cross and are contained by an isolated
scope.
