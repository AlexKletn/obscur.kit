How a window occupies the screen when presented (spec §6.5).

[`FullscreenMode::Normal`] is the windowed baseline — not fullscreen at all.
The remaining variants are the fullscreen forms;
[`FullscreenMode::Exclusive`] is honored only where the platform offers a
true exclusive mode, and otherwise degrades to a borderless equivalent.
