A style part a component exports across its scope boundary.

Per §4.15, components may export style parts and states so that outside CSS
can target them (like `::part()`), and the host may override an exported
part. An [`ExportedStylePart`] is the style-side counterpart to a component's
exported-part contract: it names the exported part and the visual states it
exposes for the given scope. The actual selector matching against these parts
lives in the matcher / `obscur-css`.
