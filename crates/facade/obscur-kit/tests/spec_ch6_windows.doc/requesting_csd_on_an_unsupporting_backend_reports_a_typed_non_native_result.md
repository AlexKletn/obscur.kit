§6.4 Custom chrome: "Safe areas and system-frame fallback are supported."
with §6.2 "Operations return typed results."

On a backend without client-side decorations the request reports
`unsupported` (or `emulated` if a fallback frame is really drawn) so the
application can tell its custom title bar not to draw, and the introspected
capability agrees.
