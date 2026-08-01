Creates the envelope for a [`PointerEvent`], carrying its payload, its
trust and its timestamp through unchanged, with no target yet (spec §2.4).

The target is deliberately absent: hit-testing is the runtime's job, not
the payload's, so whoever resolves the hit seats it with
[`at_target`](Self::at_target). The [`origin`](Self::origin) follows the
device — a touchscreen contact is [`EventOrigin::Touch`], everything else
is [`EventOrigin::Pointer`] — because §2.4 lists touch as its own source
and its hover behaviour differs.
