The document declared an IR schema version outside the range this codec
can read (spec §2.15 "precise diagnostics for newer unsupported IR"). The
payload carries the document's declared version and the codec's supported
range so the diagnostic can name both.
