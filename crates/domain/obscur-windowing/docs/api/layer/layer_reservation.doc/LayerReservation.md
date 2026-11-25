The region a layer surface asks other windows to keep clear of.

This is what makes a panel *work*: a maximised window stops at the panel's edge
instead of underneath it.

It is three cases rather than a signed number because
[`Ignore`](LayerReservation::Ignore) is not a quantity — it is a categorically
different request, and a bare integer invites writing the platform's sentinel
by accident and getting a surface that ignores every other panel on the screen.
The encoding a platform happens to use for these three cases belongs to that
platform's backend and is not visible here.

**Not every platform can honour this.** Reserving screen space is an ordinary
request on some and impossible on others, so a backend answers
[`LayerFeature::Reservation`](crate::port::LayerFeature::Reservation) honestly
rather than accepting the setting and dropping it.
