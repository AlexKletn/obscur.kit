[`gallery`]: the component gallery the themes are judged against — the page
head over a grid of nine cards (buttons, selection controls, text fields,
menus, card + list, navigation, badges, sliders, the data table and the
feedback row).

Every builder below is one card of that page and has no meaning outside it, so
they stay together (spec §3.1); what makes the gallery an entity of its own is
that its whole interface to the scene is [`gallery`] returning one node.
