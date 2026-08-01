Returns the typed activation category of this target (spec §6.11).

A [`File`](Self::File) or [`Document`](Self::Document) activates as
[`ActivationKind::File`]. A [`Uri`](Self::Uri) is classified by its scheme:
the conventional `http`, `https`, `mailto`, and `file` schemes are
[`ActivationKind::Uri`], and every other (custom/app) scheme is treated as
a [`ActivationKind::DeepLink`]. This is the simplified §6.11 rule: rather
than consult a registry, any non-standard scheme is assumed to be an app
deep link.
