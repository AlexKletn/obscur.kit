End-to-end verification that authored `position` / inset CSS reaches the
layout engine and offsets a `position: relative` element (and its subtree)
without moving its siblings (spec §4.1).

Like the sibling `layout_css` suite, every test drives the genuine pipeline —
author CSS text, compile it through the real `obscur-css` frontend, and render
one frame on a [`UiRuntime`] via [`render_now`](UiRuntime::render_now)
(`reconcile -> match -> cascade -> resolve -> layout`). The assertions read the
actual absolute [`LayoutBox`](obscur_kit::layout::model::LayoutBox) coordinates
the pipeline produced, so a regression anywhere from the CSS lowering through
the computed→layout bridge to `arrange` fails the run.
