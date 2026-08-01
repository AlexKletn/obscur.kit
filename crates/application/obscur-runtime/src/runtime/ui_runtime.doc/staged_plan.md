The phase plan a committed transaction asked for, awaiting the frame that
will honour it ([`apply_transaction`](Self::apply_transaction)).

[`None`] on every frame nobody committed to, which is the overwhelming
majority: those classify from their [`FrameReason`] exactly as before.
