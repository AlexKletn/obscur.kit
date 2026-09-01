The port a platform implements to answer what its desktop looks like.

One method that reads, and one that says how much of an answer to expect. It is
a **pull**: nothing is cached here, nothing subscribes, and a host that wants a
fresh answer asks again.

`support` matters more than it usually does, because
[`SystemAppearance`](super::SystemAppearance) is all `Option`s and an empty one
is ambiguous on its own — it could mean the desktop was asked and said nothing,
or that there was nothing to ask. The support level tells those apart.
