The outcome of posting a [`Notification`] through a [`NotificationProvider`]
(spec §6.12).

Because permission is explicit (spec §6.12), [`Denied`](Self::Denied) is a
first-class outcome distinct from [`Unsupported`](Self::Unsupported): the host
is capable of notifications but policy refused this one, versus the host offers
no notification facility at all.
