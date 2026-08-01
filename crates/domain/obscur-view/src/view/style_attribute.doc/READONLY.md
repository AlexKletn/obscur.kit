Reserved symbol for a text control's `readonly` structural state — the HTML
`readonly` attribute (WHATWG HTML §attr-input-readonly). **Distinct from
[`DISABLED`]:** a readonly control is still *focusable* and its text is still
*selectable / copyable*; it merely cannot be *edited* (no insert/delete, no IME
commit, no paste). Boolean state (present + truthy = readonly); emitted **only
when true** so toggling it drops the attribute and re-styles. Same reserved
namespace as `disabled`/`checked`, so it survives reconciliation onto the
retained node, where the runtime reads it to gate *only* the mutation path.
