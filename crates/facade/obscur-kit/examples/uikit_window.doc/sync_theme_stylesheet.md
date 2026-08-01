Reconciles the installed stylesheet with the shared `active_theme` a tab
handler may have just changed: if it differs from [`applied_theme`], swap in
`themes()[i]` (a command handler cannot reach `&mut runtime`, so this is the
window's job) and request a structural frame so the reactive builder rebuilds
the re-skinned view. Returns `true` when a swap happened.

[`applied_theme`]: Self::applied_theme
