Sets the contract registry the compiler consults (fluent), **replacing**
the built-in set [`new`](Self::new) seeded.

Pass `ContractRegistry::builtins().with_surface(&MyProps::contract_surface())`
to add a user component's §2.13 surface while keeping the built-ins;
`ContractRegistry::new()` drops them.
