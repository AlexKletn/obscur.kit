Pure converters from the domain semantic tree to AccessKit types (§7.1).

Each converter is stateless and deterministic, honoring the spec §7.1
determinism boundary: identical domain input always yields identical
AccessKit output. [`RoleConverter`] and [`ActionConverter`] map the leaf
enums; [`NodeConverter`] builds a whole [`accesskit::Node`]; and
[`TreeUpdateConverter`] assembles a full [`accesskit::TreeUpdate`].
