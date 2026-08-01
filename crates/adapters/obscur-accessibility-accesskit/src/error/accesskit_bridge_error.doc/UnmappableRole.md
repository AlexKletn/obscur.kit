A semantic role had no AccessKit equivalent and could not be mapped. The
payload names the offending role for diagnostics. The built-in
[`RoleConverter`](crate::conversion::RoleConverter) never produces this
(it always falls back to [`accesskit::Role::Unknown`]); it exists for
stricter callers that reject approximations.
