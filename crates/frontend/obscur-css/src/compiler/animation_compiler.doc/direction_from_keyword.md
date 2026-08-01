Maps a `<single-animation-direction>` keyword to the direction iteration 1
plays in plus whether iterations alternate (css-animations-1 §3.6).

`alternate` starts normal and flips; `alternate-reverse` starts reversed and
flips. Because iteration 1 is the first one played, this is exactly the
spec's one-based parity: "iterations start counting from 1".
