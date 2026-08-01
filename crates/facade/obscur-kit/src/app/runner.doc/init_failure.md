A window/GPU failure recorded on `resumed` (which cannot return a `Result`)
so [`run`] can surface it instead of returning `Ok` on a blank window.
