[`App::view_with`](crate::App::view_with) was used with
[`App::layer`](crate::App::layer).

The frame-aware root exists so a view can see a viewport that only the window
knows and that changes as the user drags. A desktop surface is sized by the
platform before the first frame and reports every later change through the
loop, so there is nothing here the plain [`App::view`](crate::App::view) does
not already have.
