A style part exported across a scope boundary, with its exposed states
(§4.15).

The host may override an exported part, so this only *declares* the contract.
It owns a `Vec` of states, hence `Clone` rather than `Copy`.
