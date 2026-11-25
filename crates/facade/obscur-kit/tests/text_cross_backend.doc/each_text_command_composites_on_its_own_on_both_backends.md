Two text commands are two composites on both backends.

The guard debt 4 asked for, in the form that needs no GPU. Its CPU half is a
pixel: two half-alpha commands over the same glyph reach 192/255, which is a
measurement of "one composite per command" and not an assumption about it. Its
Vello half is the encoding: two commands must leave two glyph runs, because a
run is one path and one draw object.

Watched failing before it was trusted. With the reverted fold temporarily put
back into `SceneConverter`'s `DrawText` arm, the Vello half read one run against
two, and the GPU sibling `the_backends_composite_two_commands_the_same` read
vello 128 against cpu 192.
