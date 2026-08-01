§6.15 "Typed environment values: - safe areas;" / "CSS environment values may
expose safe-area insets."

`#root { padding-top: env(safe-area-inset-top) }` under a host reporting a
44px top inset yields a used `padding-top` of 44px, and the content box
starts at y=44. The declaration is not dropped as invalid.
