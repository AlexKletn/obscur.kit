A lookup from component/tag name to its [`ComponentContract`], so the
[`ContractValidator`] can find the allowed surface of each element instance
(spec §2.13).

A contract is keyed by its [`component`](ComponentContract::component) name.
An element whose tag has no registered contract is left unchecked (the
registry does not constrain it).
