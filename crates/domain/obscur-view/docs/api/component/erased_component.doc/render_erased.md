Decodes `props` into the component's typed props and renders it.

# Errors

Returns the [`PropsError`] from decoding when the node's properties do not
satisfy the component's typed props. Render itself cannot fail — a
component that wants to signal failure does so through its error boundary.
