Inheritance classification for CSS properties.

Per the Obscur Kit 2 draft (§4.14), inheritance is decided *per property*:
some properties inherit from their parent by default (e.g. `color`), while
most do not (e.g. `opacity`, `background-color`). This module models that
binary classification as a small, `Copy` enum so the cascade can reason
about default inheritance without any string comparison.
