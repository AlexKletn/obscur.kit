The identity of touchscreen `contact`, minted in a namespace disjoint from
[`MOUSE`](Self::MOUSE).

A platform numbers its contacts from zero, so passing a raw contact id
through [`from_raw`](Self::from_raw) would make **the first finger down**
indistinguishable from the mouse — the one collision that is guaranteed to
happen rather than merely possible. Lifting the contact into the high
namespace keeps the two apart for the whole u64 range a platform can
realistically number contacts in, and it does so without the adapter
having to keep a mapping table (which would make an otherwise pure
converter stateful).

The namespace is a *convention*, not an enforced type: a host that mints
its own ids through `from_raw` with the top bit set can still collide, and
nothing here detects that. The device a contact came from is answered by
[`PointerDevice`](super::PointerDevice) on the event, not by this id.
