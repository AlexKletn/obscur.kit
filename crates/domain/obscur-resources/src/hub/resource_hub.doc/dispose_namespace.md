Disposes every resource in `namespace`, ignoring reference counts, and
returns how many were removed (spec §5.15 disposal / hot-reload scope).

A namespace is a bulk teardown scope — a closed document or unloaded
plugin — so this evicts all its entries and their deduplication indices
regardless of how many outstanding references they hold.
