Registers a unified [`Command`] descriptor and binds its shortcut, if it
declares one (spec §6.7).

This is the single registration that lights up every surface at once. The
command's [`id`](Command::id) is the app-owned symbol the application
already registers a handler for via [`on_command`](Self::on_command) and
binds `on-click` to in the View, so:

- its [`shortcut`](Command::shortcut) is bound into the
  [`CommandRouter`] here — no separate [`bind_command`](Self::bind_command)
  call, and no chance of the menu and the keyboard disagreeing about
  which chord runs the action;
- its [`title`](Command::title) and enabled-ness drive a kit-rendered
  menu built with `Menu::from_registry`;
- its [`CommandState`] gates [`dispatch_command`](Self::dispatch_command),
  so [`set_command_state`](Self::set_command_state) disabling it stops the
  shortcut, the menu item, and the click together.

Re-registering a symbol replaces its descriptor and resets its state.
Registering does not itself schedule a frame.
