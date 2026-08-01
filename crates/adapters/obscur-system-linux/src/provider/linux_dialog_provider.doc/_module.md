A Linux [`DialogProvider`] that presents dialogs via `zenity` (spec §6.9).

[`LinuxDialogProvider`] implements the pure
[`DialogProvider`](obscur_system::dialog::DialogProvider) port by shelling out
to `zenity` with [`std::process::Command`] and no external crates. Each
[`DialogRequest`](obscur_system::dialog::DialogRequest) maps to a `zenity`
argv (see [`build_args`]); the provider runs the tool synchronously, captures
its output, and maps the exit status onto a
[`DialogResult`](obscur_system::dialog::DialogResult).

There is no `Failed` variant in the result vocabulary, so a spawn/IO error is
reported best-effort as [`DialogResult::Dismissed`] (as if the user closed the
dialog), a missing tool as [`DialogResult::Unsupported`], and a user cancel as
[`DialogResult::Dismissed`]. A portal-based implementation
(`xdg-desktop-portal` via `ashpd`) is a future refinement (spec §2.7
target-specific providers); the shell-out path here is dependency-free.
