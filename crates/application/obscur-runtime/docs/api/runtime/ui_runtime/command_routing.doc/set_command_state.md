Sets the live [`CommandState`] of a registered command, returning whether
a command was registered under `command` (spec §6.7).

Because [`dispatch_command`](Self::dispatch_command) consults this state,
disabling or hiding a command here makes it inert on every surface at
once. Setting state does not itself schedule a frame; the next frame
rebuilds any menu from the registry.
