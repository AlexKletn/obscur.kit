§6.15 "Typed environment values: - safe areas;" / "CSS environment values may
expose safe-area insets."

The bottom inset resolves independently of the top one: a host reporting a
34px bottom inset gives `#root` a used `padding-bottom` of 34px.
