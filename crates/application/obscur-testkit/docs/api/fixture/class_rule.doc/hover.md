A rule matching `.class:hover`.

The fixture sheets use this to make a hover *visible*: a class that is sized but
unpainted until it is hovered produces a paint command **iff** the pseudo-class
fired end to end, so the display list answers the question the test is asking.
