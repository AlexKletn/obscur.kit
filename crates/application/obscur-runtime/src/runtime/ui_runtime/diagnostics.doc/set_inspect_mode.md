Enables or disables the in-window inspect overlay (spec §7.6).

When on, a composed frame highlights the hovered node's border box so the
running UI can be debugged directly in the window. Toggling requests an
[`FrameReason::Input`] frame so the highlight appears/disappears without
waiting for the next pointer move; a no-op when the state is unchanged.
