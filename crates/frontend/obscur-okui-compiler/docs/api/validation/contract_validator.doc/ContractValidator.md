Validates a component instance against its [`ComponentContract`] (spec
§2.12, §2.13).

This is the **file** front end's adapter onto the shared rule in
[`contract_rule`](crate::validation::contract_rule): it describes a parsed
[`Element`] as a plain [`ElementUse`], runs [`check_contract`], and resolves
each violation's [`ViolationAnchor`] to a source byte offset. The rule itself
lives elsewhere precisely so the `okui!` macro — which has no syntax tree —
can reach the same verdict from `syn` tokens.
