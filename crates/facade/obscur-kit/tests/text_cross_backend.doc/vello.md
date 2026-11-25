A Vello renderer on this machine's adapter, with the face registered — or a
panic naming the missing adapter.

Reachable only from an `#[ignore]`d test, so calling it is a statement that this
machine has a GPU; if it has none, the run is broken and says so rather than
printing a line and returning, which is a pass wearing a skip's clothes.
