Installs the resource `loader` + `decoder` the runtime uses to resolve
`background-image: url(...)` images (spec §5.15, §5.16), replacing any
previously installed pair and clearing the resolved-image cache so the new
providers re-resolve every URL from scratch.

Resource loading/decoding are hexagonal ports: the runtime (application
layer) depends only on the domain [`ResourceLoader`] / [`ResourceDecoder`]
traits, never on a concrete adapter. The **facade** builds real adapters
(e.g. a sandbox-rooted `FsResourceLoader` + the `image`-crate `ImageDecoder`)
and injects them here. Leaving them unset is the honest-capability default:
a `url()` background then resolves to nothing and paints nothing — never a
crash.
