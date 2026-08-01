§6.3 Activation and focus: "The application may request:" / "1. window
activation;" / "2. element focus." with "Possible results:" / "- activated;"
/ "- denied;" / "- unsupported;" / "- failed."

A stale handle and a non-focusable node both fail to focus, and neither
changes the focused node. Neither call panics.
