Registers a component kind so nodes naming its
[`ComponentId`](obscur_identity::ComponentId) instantiate it (spec §2.14).

Static registration is the default. Once registered, every reconciled node
whose element names this kind mounts an instance — which is rendered with
its typed props decoded from the node's authored properties, and whose
local state lives for as long as the node does.

Registering nothing keeps the whole component path inert, so a host that
drives its tree from a root view builder alone is unaffected.

# Errors

Returns [`RegistryError`](obscur_view::component::RegistryError) when a
kind is already registered under the same id.
