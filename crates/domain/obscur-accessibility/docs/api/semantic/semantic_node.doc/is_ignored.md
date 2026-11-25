Returns whether the node is not exposed to assistive technology.

A node is ignored when it is [`decorative`](Self::decorative) or its
[`state.hidden`](SemanticState::hidden) flag is set (spec §7.1
decorative hiding). Adapters skip ignored nodes entirely.
