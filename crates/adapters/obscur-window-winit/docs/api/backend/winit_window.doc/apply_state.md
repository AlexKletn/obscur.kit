Maps a domain [`WindowState`] onto winit's separate minimize / maximize /
fullscreen toggles.

The domain models presentation as one mutually-exclusive state, whereas
winit exposes three independent setters; [`WindowState::Normal`] therefore
clears all three, and each other variant sets exactly its own.
[`WindowState::Fullscreen`] uses borderless fullscreen on the current
monitor, the portable form that needs no monitor/video-mode handle.
