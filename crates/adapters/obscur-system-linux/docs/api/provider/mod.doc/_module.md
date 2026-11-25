Linux system-service providers backed by conventional CLI tools (spec §6.9
dialogs, §6.11 shell, §6.12 notifications; §2.7 target-specific providers).

Each provider implements one `obscur_system` port by shelling out — via
[`std::process::Command`] with no external crates — to the standard Linux tool
for that job: [`LinuxDialogProvider`] over `zenity`, [`LinuxShellProvider`]
over `xdg-open`, and [`LinuxNotificationProvider`] over `notify-send`.
Capability queries probe `PATH` without executing anything, so they never pop a
GUI. Portal-based implementations (`xdg-desktop-portal` via `ashpd`) are a
documented future refinement.
