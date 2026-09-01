`font-family: "X"` in an author sheet, where `X` is a name and not a generic
(spec §5.18, audit 2-F).

# What was wrong

A runner installed exactly three faces — the host's `sans-serif`, `serif` and
`monospace`, each registered under its *generic* family — so every named
family in a sheet resolved to nothing. That is partly correct behaviour when
the font is absent; what made it a defect is that it was equally true when the
font was present. An application had no way to add a face under a name, and a
font the machine had installed was never looked up by name, so installing the
font changed nothing.

# The two halves, tested separately

* **The application's own faces.** A face vendored into the binary is the only
  answer that works on a machine that does not have the font, and it is the
  one an application designed against a mockup actually needs. Tested against
  the vendored `NotoSansMono` under a fictitious name, so the assertions are
  exact glyph-run identities rather than anything the machine supplies.
* **The host's faces, by name.** Tested through fontconfig, skipping where
  there is none, and with the machine's *own* serif as the subject so the
  family is installed by construction and the test states no fact about what
  is on any particular box. What it pins is that the face pulled in is the
  file listed under that family and not a substitute — `fc-match` answers
  every name, which is why the lookup goes through `fc-list`.
