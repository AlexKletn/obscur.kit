The current [`StateGeneration`] of each observable source (spec §3.1).

State enters the UI through typed props, local component state, and
**read-only signal/observable sources**. This is the version book for the
third: an application bumps a source when the value behind it changes, and a
component records which sources its render read. Comparing the two is what
makes "dependency tracking invalidates only consumers of changed values" true
— a source that did not advance invalidates nobody, and a component that read
nothing is never invalidated.

# It holds versions, not values

Deliberately: the toolkit provides no store, and a component owns its own
data (§2.1). A `SignalId` is an opaque name for "something the app can
change"; what it names, and where that value lives, is entirely the
application's business. This only answers *has it changed since you looked*.

An unknown source reads as [`StateGeneration::INITIAL`], so tracking a source
nobody has bumped yet is well-defined rather than an error.
