Registers a canvas painter under `name`, the spelling CSS reaches it by.

This is the other half of the `-obscur-painter` property. A stylesheet rule
names a routine, and until an application supplies one under that name the
element draws nothing — the property named a painter no application had a
supported way to register.

```ignore
App::new("charts")
    .stylesheet("chart{width:240px;height:80px;-obscur-painter:sparkline}")
    .painter("sparkline", |region, _paint| {
        let size = region.size();
        region.painter().fill_rect(
            Rect::new(Point::new(0.0, 0.0), size),
            Brush::solid(Color::rgba(0.2, 0.5, 1.0, 1.0)),
        );
        // Whether this painter wants the next frame: a still drawing says
        // `false`, an animating one says `true` and is asked again.
        false
    })
    .run()
```

The painter is handed the box **layout** gave the element — not the size any
constructor hinted at — and that node's own properties, so one registration
serves every element on the screen that names it.

Registration is by key and the last one wins, so a name registered twice keeps
the second painter. Use [`painter_id`] where the canvas names its painter as a
property rather than through CSS, which is what the shipped widgets do.

[`painter_id`]: App::painter_id
