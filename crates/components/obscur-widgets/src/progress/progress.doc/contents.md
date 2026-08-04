The bar itself — one `<canvas>` naming the built-in painter.

Factored out so the mounted component and the Rust builder emit the same
children, which is what makes mounting idempotent on a built bar.
