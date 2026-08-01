A value that can be blended toward another value of the same type (spec
§4.17).

This is the foundation for the motion engine's *arbitrary typed
interpolation*: any presentation value that implements `Interpolatable` can
be animated by feeding a normalized progress `t` to [`interpolate`]. The
engine layers timelines, tracks and drivers on top, but at the bottom every
animated quantity reduces to this one pure operation.

[`interpolate`]: Interpolatable::interpolate
