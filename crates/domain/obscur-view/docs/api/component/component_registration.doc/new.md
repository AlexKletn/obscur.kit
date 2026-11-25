Builds a registration for `factory` at `version`.

The [`ComponentId`] and [`ComponentContract`] are taken from the factory
itself (via [`ComponentFactory::component_id`] and
[`ComponentFactory::contract`]), keeping the entry internally consistent
by construction.
