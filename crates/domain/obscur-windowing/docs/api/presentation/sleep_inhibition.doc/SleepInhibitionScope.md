What a sleep-inhibition request holds off (spec §6.5).

`SleepInhibitionScope` is a tightly-coupled part of [`SleepInhibition`](super::SleepInhibition): it
names what a request keeps awake and has no meaning apart from it, so it is
colocated here rather than given its own module.
