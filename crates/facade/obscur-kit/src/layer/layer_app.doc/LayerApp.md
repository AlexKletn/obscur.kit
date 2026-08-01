A desktop surface — panel, dock, bar, wallpaper or overlay — described the
same way an [`App`](crate::App) is, and drawn by the same toolkit.

The difference from `App` is entirely in placement: a layer surface has no
title, no decorations and no place in the window stack. It is anchored to
screen edges, sits in one of four depth bands, and can reserve space that
maximised windows keep clear of.
