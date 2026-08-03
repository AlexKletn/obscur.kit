The mouse buttons currently held down.

winit reports one button transition per `MouseInput`, never the set, so nothing
but a running fold knows that two are down at once. Without it every event would
have to claim a single button, and a chord — dragging with the middle button
while the left is held, a right click during a left-drag — would be reported as
whichever transition arrived last.

Mouse only. A touch contact carries its own engagement and never touches this,
or the first finger down would start claiming the cursor's buttons.
