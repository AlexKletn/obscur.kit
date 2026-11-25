The severity/intent of a [`DialogRequest::Message`](super::DialogRequest::Message) (spec §6.9).

A provider maps the level to the platform's native styling — an icon, a
sound, or an accent — so an informational notice reads differently from an
error. The domain stays presentation-agnostic and only names the intent.
