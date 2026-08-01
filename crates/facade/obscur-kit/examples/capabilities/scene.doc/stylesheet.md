The authored stylesheet exercising every ported CSS feature. Returned as a
`&'static str` so all three examples compile it through the real `obscur-css`
frontend. The window driver additionally layers a per-frame `.xform-box`
override on top (see [`crate`] and `capabilities_window.rs`).
