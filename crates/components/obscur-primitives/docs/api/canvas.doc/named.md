Names the painter that draws this canvas.

The symbol is a handle, not a closure: a `ViewNode` is data — cloneable and
comparable — and a drawing callback is neither. The painter itself is
registered on the runtime under the same symbol, exactly as a virtual list's
row builder is.
