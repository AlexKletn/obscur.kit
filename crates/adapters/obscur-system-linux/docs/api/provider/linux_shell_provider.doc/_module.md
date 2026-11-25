A Linux [`ShellProvider`] that opens targets via `xdg-open` (spec §6.11).

[`LinuxShellProvider`] implements the pure
[`ShellProvider`](obscur_system::shell::ShellProvider) port by shelling out to
the conventional freedesktop opener (`xdg-open`) with
[`std::process::Command`] and no external crates. The open is fire-and-forget:
`xdg-open` hands the target to the desktop and detaches, so a successful spawn
reports [`ShellResult::Opened`] without waiting — the actual application launch
is asynchronous and outside this provider's observation.

A portal-based implementation (`xdg-desktop-portal` via `ashpd`) is a future
refinement (spec §2.7 target-specific providers); the shell-out path here is
dependency-free and synchronous.
