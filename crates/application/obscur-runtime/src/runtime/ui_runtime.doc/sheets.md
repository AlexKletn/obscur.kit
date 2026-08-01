Every source a frame's stylesheets come from: the set the reactive path
holds (installed via [`set_stylesheets`](Self::set_stylesheets), parallel
to the held [`root_view`](Self::root_view)), the last non-empty author set
any frame was rendered with, and the built-in **user-agent (UA) default**
(installed via
[`set_default_stylesheet`](Self::set_default_stylesheet)) that is matched
first — below every author sheet — on **all** render paths
(`run_frame`/`run_due_frame`/`render_now`/the reactive path).

One field rather than the three it replaces, because resolving the list
the matcher runs on reads two of them in one operation — see
[`CascadeSheets`], which is why what a frame remembers can only be the
author half, and why a replaced UA sheet cannot be outlived by a retained
copy of the one before it.
