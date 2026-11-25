Which vendor namespace, if any, a property name is written in.

The convention every browser has followed since the beginning: a name of the
form `-vendor-property` belongs to one implementation, and every other
implementation ignores it. It is what lets one stylesheet carry declarations
for renderers that do not know each other.

Spec §4.13 excludes "vendor hacks", and that exclusion holds: this kit accepts
no other vendor's prefix as an alias for anything of its own. `-webkit-filter`
does not become `filter`. What the classification buys is the other half of the
convention — knowing whose name it is, so that ours can be compiled and
everyone else's can be passed over in silence rather than reported as a fault.
