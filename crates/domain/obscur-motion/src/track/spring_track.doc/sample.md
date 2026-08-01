Samples the spring blend from `from` toward `to` at progress `t`.

`t` is passed through the spring curve *without* clamping so overshoot is
preserved; `t <= 0` yields exactly `from`. On a variant mismatch between
the endpoints the blend is undefined and this falls back to `to`.
