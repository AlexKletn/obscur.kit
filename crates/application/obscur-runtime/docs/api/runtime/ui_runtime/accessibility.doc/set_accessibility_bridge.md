Installs the platform [`AccessibilityBridge`] every frame publishes its
semantic projection to, replacing any previously installed one, and pushes
the current tree to it immediately (spec §7.1).

This is the runtime half of the §7.1 output path: from here on, each frame
that changes the retained tree or moves focus rebuilds the semantic
projection and pushes it — the application never calls
[`push_accessibility`](Self::push_accessibility) itself, exactly as it
never calls the vector decoder. Accessibility is a **port**, so the runtime
depends only on the domain trait; the facade builds the real adapter (an
`AccessKitBridge`) and injects it here.

The immediate push is what makes installation order not matter: install
before the first frame and that frame publishes, install after and the tree
already on screen is published now rather than at the next unrelated
change.

# What this does not do

Installing a bridge does not reach an assistive technology by itself. The
bridge converts; the **platform** push (`accesskit_winit` / `accesskit_unix`
and the window they need) is the host's. Draining
[`poll_actions`](AccessibilityBridge::poll_actions) back into input is
nobody's yet: the runtime does not, so an action an assistive technology
requests reaches the bridge and stops there (§7.1 L27 is unbuilt — this is
the outbound half only).

Leaving this unset is the honest-capability default: no semantic tree is
built at all, and a frame pays one `Option::is_none` check for the privilege.
