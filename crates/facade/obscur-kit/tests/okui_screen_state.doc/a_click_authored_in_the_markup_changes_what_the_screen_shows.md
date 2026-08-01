The whole loop, from markup to pixels: the screen's own `on:click=increment`
dispatches the command, the handler writes the table, and the next frame's
lowering shows it. A counter in a standalone `.okui` file, which is exactly what
was impossible.
