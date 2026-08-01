A filesystem-backed [`ResourceLoader`] adapter (spec §5.15 sources, §2.7
adapters — "Filesystem loader").

Spec §5.15 lists `file` among a resource's sources, and states the kit's core
performs no filesystem access itself — an interface adapter does. This module
is that adapter: [`FsResourceLoader`] implements the pure
[`ResourceLoader`] port using only `std`, reading the encoded bytes behind a
[`ResourceSource::File`] from disk. An optional sandbox root confines resolved
paths to a directory subtree (spec §2.7 adapters keep host reach explicit).
