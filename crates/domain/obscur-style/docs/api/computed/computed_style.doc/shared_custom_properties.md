The custom-property set as a shared handle, for a child that inherits it whole.

Custom properties inherit, so in a themed application every node carries the
root's set. Copying it per node meant one clone of every value into an
intermediate map and a second clone out of it, plus two hash maps grown from
empty — for values that were identical to the parent's. A node that declares
none of its own can share instead, and pay a refcount.
