A handle with nothing queued.

`App::control` hands out clones of the one it
made in `App::new`; construct one directly only when
wiring a runtime by hand (a guest host, or a test).
