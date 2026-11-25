A style scope: the boundary that scopes a component's or module's CSS.

Per §4.15, global stylesheets are allowed but component stylesheets are
scoped by default. The one substantive rule this type encodes is the
boundary-crossing rule: an isolated scope lets *inherited* and *custom*
properties cross its boundary and contains everything else. Custom
properties always cross because they always inherit; inherited properties
cross because inheritance flows through the tree regardless of scope; other
(non-inherited) properties are contained by an isolated scope.

Scope-aware selector matching and module rewriting happen elsewhere (in the
matcher / `obscur-css`); this type only models scope identity and the
visibility of a property across the boundary.
