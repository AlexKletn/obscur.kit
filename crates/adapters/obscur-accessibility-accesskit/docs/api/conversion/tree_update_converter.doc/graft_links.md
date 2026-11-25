Walks `node`'s captured child links, attaching each present child under
`ancestor` and recursing into it.

An ignored child emits no AccessKit node, so it cannot be anyone's parent:
the walk descends **through** it with `ancestor` unchanged, so its own
children graft onto the nearest non-ignored ancestor instead of being
dropped along with it. `visited` bounds the walk on a malformed (cyclic)
snapshot; `owned` records what has been attached so a later `Owns` relation
or the orphan sweep cannot attach it a second time.
