The size and count limits a capability places on a transfer (spec §6.20).

Clipboard and drag-and-drop integrate capability limits: a policy may cap the
total byte volume a transfer may carry, the number of items, or both. Each
bound is optional — `None` means "no limit on this axis" — so the default is
entirely [`TransferLimit::unlimited`]. The predicates report whether a
proposed transfer fits, without performing the transfer.
