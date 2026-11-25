Collects only a node's **in-flow** children (`Static` / `Relative`).

Out-of-flow children ([`Position::Absolute`](obscur_layout::model::Position::Absolute) / [`Position::Fixed`](obscur_layout::model::Position::Fixed)) are
dropped: they neither reach the parent's flow/flex/grid algorithm nor
contribute to its content size (spec §4.1). This is the child list the
measure pass hands the algorithm, so its indices are stable.
