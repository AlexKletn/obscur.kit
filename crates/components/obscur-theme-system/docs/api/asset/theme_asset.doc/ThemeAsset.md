A named font/icon/image/stylesheet resource a theme bundles (spec §2.6).

Each asset pairs a name and an [`AssetKind`] with an [`AssetSource`] that is
either embedded bytes or a host-resolved path. Because the payloads
([`Arc<[u8]>`](Arc) and [`Box<str>`]) are `Eq`, this type derives `Debug`,
`Clone`, `PartialEq`, and `Eq`.
