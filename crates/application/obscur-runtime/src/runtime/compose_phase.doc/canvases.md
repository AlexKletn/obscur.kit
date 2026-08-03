What each canvas's painter drew this frame, by node.

Resolved before this phase, like the images and the vectors, because a painter
is application code that needs `&mut` and the compose walk takes `&self`.
