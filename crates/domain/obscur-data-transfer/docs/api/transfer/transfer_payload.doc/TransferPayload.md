The full content moved by a transfer (spec §6.20, §6.21).

A payload always carries a [`DataOffer`] — the system-facing MIME
representations any application can consume. It may *additionally* carry a
typed *internal* payload: an app-chosen type tag plus the app's serialized
value. This is the fast path for same-application transfers (spec §6.21 "one
drag may expose both an internal typed payload and system MIME"): the source
and target app agree on the tag and exchange the serialized value directly,
avoiding a round-trip through MIME serialization. The toolkit stores the
internal type and data opaquely and never interprets them.
