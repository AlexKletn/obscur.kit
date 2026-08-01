`minmax(min, max)`: sized by `max`, floored at `min`.

Each bound is itself a [`TrackSize`] (conventionally a length, a
percentage, `fr`, or one of the content-sized keywords); an `fr` bound is
only meaningful as the `max`.
