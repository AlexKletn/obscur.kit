Creates the envelope for a [`KeyEvent`], with no target yet (spec §2.6).

[`KeyEvent`] carries no trust of its own — unlike [`PointerEvent`], which
has three producers and therefore has to say which one it came from — so
the envelope starts [`Untrusted`](EventTrust::Untrusted) and whoever
received the OS key confers trust with [`as_trusted`](Self::as_trusted).
The day a key-injection path exists, that call is what it must not make.
