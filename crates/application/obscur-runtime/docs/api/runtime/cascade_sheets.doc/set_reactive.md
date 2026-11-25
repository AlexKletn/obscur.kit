Installs the stylesheets the reactive frame path matches, replacing any
previously installed set.

Installing an **empty** set does not un-style the application: the frame
that supplies it takes the retained author set instead, exactly as a bare
`render_now(view, &[])` does. Clearing the styling of a running
application is therefore not something this can express — see
[`author_for_frame`](Self::author_for_frame), which is where that rule
lives.
