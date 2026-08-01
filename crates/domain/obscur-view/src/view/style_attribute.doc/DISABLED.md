Reserved symbol for a control's `disabled` structural state — the `:disabled`
pseudo-class input (spec §4.13). Unlike `class`/`id`/`type` (selector inputs
matched as strings), this is a *boolean* state the element carries about
itself: present with a truthy [`ViewValue`] means disabled. A widget emits it
**only when true**, so toggling it off drops the attribute and the extracted
[`StyleAttributes`] change, driving a restyle. Kept in the same reserved
low-symbol namespace so it survives reconciliation onto the retained node.
