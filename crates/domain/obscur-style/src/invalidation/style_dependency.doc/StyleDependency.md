Records what a single node's computed style depended on (§4.3).

A node's style depends on: whether it inherited any value from its parent
([`inherits_from_parent`](Self::inherits_from_parent)), which custom
properties it read ([`custom_properties`](Self::custom_properties)), and which
interaction states its matching depends on
([`pseudo_states`](Self::pseudo_states)). A later change consults this record
via the `affected_by_*` methods to decide whether the node must be restyled;
those methods are the gate that would keep invalidation confined to affected
consumers (§4.14).

Would, not does — see the module docs: nothing builds one of these, so the
runtime keeps no such record alongside a node's cached style.
