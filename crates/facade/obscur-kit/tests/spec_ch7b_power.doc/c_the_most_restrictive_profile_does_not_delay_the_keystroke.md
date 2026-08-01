§7.11, verbatim: "It is off by default and must not compromise critical
responsiveness."

The narrowest reading "responsiveness" bears on its own: latency from an
input event to the frame that reflects it. Run R supplies the most
restrictive `PowerProfile` the API offers; run F supplies the no-saving
profile. Both feed the identical event script — a key press of `a` delivered
at tick 10, then nothing — and the clock is advanced tick by tick.
Measured in ticks from delivery, not milliseconds.
