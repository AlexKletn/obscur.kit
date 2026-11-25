What happens when a new transition targets an element that is *already*
transitioning (spec §4.17 "interruption").

The motion engine must decide how an incoming transition reconciles with the
one currently in flight on the same element. This enum names the four
policies; the runtime reads it to either replace, turn around, blend into, or
suppress the incoming transition. Choosing to continue from the element's
present mid-animation value (rather than snapping) is what makes rapid,
repeated interactions look seamless.
