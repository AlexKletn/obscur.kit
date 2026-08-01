Reads take `self` by value and writes take `&mut self`: the type is four
scalars and `Copy`, so a read costs the same either way and the signature says
which half of the API it is in.
