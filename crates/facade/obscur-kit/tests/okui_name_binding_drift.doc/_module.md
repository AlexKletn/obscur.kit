Drift guard: the frontend `name_binding` registry mirrors the raw handle
values of their canonical sources.

`obscur-ui-ir::name_binding` must not depend on the component tier, so it
hard-codes the reserved `obscur-view` symbols and the widget/primitive
`COMPONENT_ID`s. This facade test — which *can* see every crate — pins each
mirrored constant to its real source, so a change to any canonical id fails
here instead of silently desynchronising the two `.okui` lowering paths.
