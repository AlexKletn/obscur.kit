Whether `ctx` permits this extension to run — the capability check of
spec §5.3.

This is a **compound gate**: the context must have granted
[`Self::required_capability`], *and*, if any of the extension's passes
[`requires_shaders`](CustomPass::requires_shaders), the context must also
allow custom shaders
([`TrustedRenderContext::allows_custom_shaders`]). An extension with no
shader passes needs only the capability grant.
