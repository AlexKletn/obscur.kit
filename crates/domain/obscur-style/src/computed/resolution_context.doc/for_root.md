Builds the context for resolving the **root** node itself.

On the root element `rem` refers to the *initial* `font-size` (`medium`),
not to the root's own computed one — otherwise `html { font-size: 2rem }`
would be defined in terms of itself.
