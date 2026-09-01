A face standing in for one an application vendored.

The vendored *mono* face, deliberately: the bundled last-resort face is
`NotoSans-Regular`, and using those exact bytes would make
`install_text_stack` treat the application's face as the bundled one and skip
registering the sans-serif fallback the control assertion needs.
