§6.7 Menus and actions: "Unified Action model:" with "- stable ID;" …
"- enabled;", and "The same action can drive:" / "- native menu;" /
"- kit-rendered menu;" / "- toolbar;" / "- command palette;" / "- tray;" /
"- accessibility;" / "- shortcuts."

One mutation by ID disables every surface at once: the shortcut must not fire
the handler, and the action's own state is the single source of truth every
surface reads.
