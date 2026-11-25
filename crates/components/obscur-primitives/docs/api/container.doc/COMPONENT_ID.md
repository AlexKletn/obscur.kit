Stable component id for the container primitive.

Element primitives share the `0x00C0_00NN` id space, one fixed value per
primitive so the style/reconcile layer can map them without a name
lookup: container=`..01`, (text has no id — see [`Text`]), image=`..03`,
scroll=`..04`, canvas=`..05`, portal=`..06`, layer=`..07`,
foreign surface=`..08`.

[`Text`]: crate::Text
