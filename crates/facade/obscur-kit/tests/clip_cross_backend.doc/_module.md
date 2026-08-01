The two render backends must agree about clipping (spec §5.2).

§5.2 draws the determinism boundary explicitly. "Must match across backends"
names **clipping**, **paint order**, and **scene geometry**; "allowed
differences" names **rasterization**, **anti-aliasing**, and backend-specific
precision. So the property under test here is the *clip decision* — for a
given scene, which points does drawing reach? — and not the pixels each
rasterizer produces for it.

There was no cross-backend test in this repo at all, which is why `Clip::Mask`
was able to mean opposite things on the two backends: the CPU renderer painted
masked content, Vello clipped it entirely away, and nothing compared them.

# How each backend is asked

- **CPU** — the real [`CpuRenderer`] rasterizes the scene into a bitmap and a
  probe pixel is read back. This is the production path, end to end.
- **Vello** — the real [`ClipConverter`] resolves the clip to its outline and
  the probe point is tested against it. Vello's *rasterizer* needs a GPU, but
  the clip decision does not: the adapter is deliberately built so that "only
  rasterization is device-bound" (`vello_device.rs`), which puts the whole of
  §5.2's must-match clip geometry on this GPU-free path.

# What this does NOT cover — read before trusting it

- **No pixels from Vello.** Vello's clip outline is compared against the CPU's
  *rasterized coverage*, so this proves the two backends make the same clip
  decision, not that Vello's rasterizer honors it. A true golden-image
  comparison needs a GPU; this workspace has no GPU test target at all (every
  GPU proof in `obscur-kit` is an `[[example]]` run by hand), so a
  `#[test]` doing readback would silently no-op on any CI without an adapter —
  a worse lie than this stated gap.
- **Nesting and paint order are not compared**, only a single clip bracket.
- The **layer-clip** comparison shares the Vello limitation above. It does
  drive Vello's real `push_layer`: `a_layer_clip_reaches_vellos_push_layer`
  converts through the public `SceneConverter` and reads the clip back out of
  `Scene::encoding()`, which is GPU-free. What that cannot show is the
  *rasterized* result, which is the same gap as above.
- **Mask coverage is not tested because neither backend implements it.** Both
  now *ignore* mask coverage identically (the §5.8 fallback). Agreeing to
  ignore it is what this test pins; it is not evidence that masks work.
- Antialiased clip edges are avoided by probing well inside/outside a clip
  rather than on its boundary, since §5.2 lets AA differ.
- **Agreement cannot catch a shared bug.** Both backends resolve a
  `ClipShape::RoundedRect` through the *same* `Path::rounded_rect`, so a defect
  in that one builder makes both backends wrong *together* and the
  backends-agree assertion still passes. This was measured, not assumed:
  replacing the elliptical radii below with a single scalar leaves every
  `cpu == vello` assertion green. Only the `expected` column catches it, which
  is why the ellipse cases' expected values are chosen to sit in the region
  where a scalar radius and a true ellipse actually disagree. Treat `expected`
  as the real oracle here and agreement as the weaker, secondary claim.
