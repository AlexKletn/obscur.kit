The resource ports that are on by default, where their features are compiled in.

Filesystem images and vector resources are defaults-on because an application
that names an image expects it to load. The blocking executor is opt-in per
§3.3 in general, but on by default here: both of these runners own the process,
which is the case §3.3's caution is not about.
