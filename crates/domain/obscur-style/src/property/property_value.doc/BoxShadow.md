An ordered `box-shadow` list, in author order: the value of `box-shadow`.
Each [`BoxShadow`] carries its offsets, blur, spread, optional color, and
`inset` flag; `box-shadow: none` lowers to an **empty** list. CSS paints
the first-listed shadow on top, so the scene coordinator emits the list in
reverse order (spec §5.2).
