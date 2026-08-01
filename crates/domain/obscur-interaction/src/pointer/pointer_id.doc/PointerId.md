Stable identity for one pointer or contact point (spec §4.7).

A single interaction session may involve several concurrent pointers: a
mouse plus multiple simultaneous touch contacts. Each pointer carries a
`PointerId` so multi-pointer (touch) input can be tracked and routed
independently. By convention the mouse is [`PointerId::MOUSE`] (raw `0`) and
every touch contact is minted a distinct id. This is a *local* newtype:
the interaction crate mints and compares these handles itself. The numeric
value carries no meaning outside the subsystem that mints it.
