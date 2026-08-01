**SPECIFICATION-DERIVED** colour-pipeline cases for obscur.kit §5.7 ("Color").

§5.7 states two properties with numeric consequences: *"linear compositing for
blending, blur and filters"* and *"premultiplied alpha"*. Every expected value
below was produced by an agent reading that section and css-color-4 §10.2 /
§12.3, with **no access to this repository's source**.

Note for readers: §5.7's "linear compositing" **contradicts CSS**, where
`opacity` and simple alpha compositing are defined on gamma-encoded sRGB. The
deriver took the toolkit's own spec at face value. If the toolkit decides CSS
wins, [`sc_03_opacity_composites_in_linear_light`]'s expected value becomes
`128` and §5.7 must be amended — but the expected value here may not be
changed to match whatever the code does.
