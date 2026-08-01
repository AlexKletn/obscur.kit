The scroll-reset token each virtual list last presented. When a frame's marker
carries a different token, the list's content has been replaced, so its
container's `scroll_top` is zeroed before the window is computed — the offset
belonged to the old content. The app is the only party that can know the model
changed, and the view is its only channel here, so it drives this.
