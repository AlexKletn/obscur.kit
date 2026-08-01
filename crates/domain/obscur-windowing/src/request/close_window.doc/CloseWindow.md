An application's intent to close a window (spec §6.1).

A graceful [`CloseWindow::request`] mirrors a
[`WindowEvent::CloseRequested`](crate::window::WindowEvent::CloseRequested):
it asks the window to close and lets any application-level close
confirmation run. A [`CloseWindow::force`] request bypasses that
confirmation and closes the window unconditionally.
