A filesystem-backed [`ResourceLoader`] that reads bytes for
[`ResourceSource::File`] paths from disk (spec §5.15 file, §2.7 filesystem
loader).

This adapter handles only filesystem paths: the bytes-in-hand kinds
([`ResourceSource::Embedded`], [`ResourceSource::Memory`]) are the
`InlineLoader`'s job, and [`ResourceSource::Url`] / [`ResourceSource::Document`]
require a host loader. When constructed with a sandbox root via
[`FsResourceLoader::with_root`], `File` paths resolve relative to that root and
may not escape it through `..` traversal; without a root ([`FsResourceLoader::new`])
the path is used as given (absolute or relative to the process working
directory).
