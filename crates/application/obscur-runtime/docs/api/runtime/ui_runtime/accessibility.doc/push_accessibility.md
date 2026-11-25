Pushes the current focus and semantic snapshot to a platform
[`AccessibilityBridge`], feeding the retained tree's accessibility
projection into UI Automation / AX / AT-SPI / web a11y (spec §7.1).

Focus goes first because it is not carried *by* the snapshot as far as an
adapter is concerned: the semantic nodes record which one is
[`focused`](obscur_accessibility::semantic::SemanticState::focused), but an
adapter converts a tree against the focus target it was last *told*, so a
snapshot pushed alone would arrive with the platform's focus still on
whatever it was — for AccessKit, the root.

The runtime calls this itself for the bridge installed by
[`set_accessibility_bridge`](Self::set_accessibility_bridge); it is public
for a host that drives a bridge of its own instead.
