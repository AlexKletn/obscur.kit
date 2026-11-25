//! Conservative tile damage for Lyon frames.

mod damage_scope;
mod draw_state;

use damage_scope::DamageScope;
use draw_state::DrawState;

use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::{Brush, Paint};
use obscur_scene::path::Path;

use super::mesh::path_rect;

fn tile_size() -> u32 {
    static SIZE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *SIZE.get_or_init(
        || match std::env::var("OBSCUR_LYON_DAMAGE_TILE").as_deref() {
            Ok("16") => 16,
            Ok("64") => 64,
            _ => 32,
        },
    )
}

type Region = [u32; 4];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Disabled,
    Resolved,
    Multisampled,
}

pub(super) fn mode() -> Mode {
    static MODE: std::sync::OnceLock<Mode> = std::sync::OnceLock::new();
    *MODE.get_or_init(
        || match std::env::var("OBSCUR_LYON_PARTIAL_REDRAW").as_deref() {
            Ok("resolved") => Mode::Resolved,
            Ok("1" | "msaa") => Mode::Multisampled,
            _ => Mode::Disabled,
        },
    )
}

pub(super) fn trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("OBSCUR_GPU_STATS").is_some_and(|value| value == "1"))
}

pub(super) fn regions(
    previous: &DisplayList,
    next: &DisplayList,
    extent: (u32, u32),
) -> Option<Vec<Region>> {
    plan(previous, next, extent, None)
}

/// Damage for presenting a newly rendered surface, including supported alpha groups.
pub(crate) fn surface_regions(
    previous: &DisplayList,
    next: &DisplayList,
    extent: (u32, u32),
    text_bounds: [&[Region]; 2],
) -> Option<Vec<Region>> {
    plan(previous, next, extent, Some(text_bounds))
}

fn plan(
    previous: &DisplayList,
    next: &DisplayList,
    extent: (u32, u32),
    text_bounds: Option<[&[Region]; 2]>,
) -> Option<Vec<Region>> {
    use PaintCommand::*;
    let surface = text_bounds.is_some();
    if surface
        && let Some(command) = previous
            .iter()
            .chain(next.iter())
            .find(|command| match command {
                BackdropFilter { .. } => true,
                PushLayer(layer) => {
                    layer.is_isolated()
                        || !layer.effects().is_empty()
                        || !layer.blend_mode().is_normal()
                        || !layer.composition().is_src_over()
                        || layer.clip().is_some()
                        || layer.transform() != AffineTransform::IDENTITY
                }
                PushClip(clip) => clip.is_mask(),
                _ => false,
            })
    {
        println!(
            "[scene-damage-fallback] effect={:?}",
            std::mem::discriminant(command)
        );
        return None;
    }
    let topology_changed = previous.len() != next.len();
    if topology_changed && !surface {
        return None;
    }
    let tile = tile_size();
    let columns = extent.0.div_ceil(tile);
    let rows = extent.1.div_ceil(tile);
    let mut dirty = vec![false; (columns * rows) as usize];
    if topology_changed {
        let text_bounds = text_bounds?;
        if topology_damage([previous, next], extent, text_bounds, |bounds| {
            mark(&mut dirty, tile, columns, bounds, extent);
        })
        .is_none()
        {
            println!("[scene-damage-fallback] command_count");
            return None;
        }
        let regions = merge_regions(&dirty, tile, extent, surface)?;
        println!("[scene-damage-topology] regions={}", regions.len());
        return Some(regions);
    }
    let mut transforms = vec![([AffineTransform::IDENTITY; 2], false, true)];
    let full = [0.0, 0.0, extent.0 as f32, extent.1 as f32];
    let mut clips = vec![([full; 2], Some(full))];
    for (index, (old, new)) in previous.iter().zip(next.iter()).enumerate() {
        let ([before, after], opacity_changed, opaque_group) = *transforms.last()?;
        let (parent, parent_interior) = *clips.last()?;
        if surface {
            match (old, new) {
                (PushClip(old), PushClip(new)) => {
                    let outer = [
                        intersect(parent[0], device_edges(old.bounds()?, before)?),
                        intersect(parent[1], device_edges(new.bounds()?, after)?),
                    ];
                    let shared = clip_interior(old, before)
                        .zip(clip_interior(new, after))
                        .map(|(a, b)| intersect(a, b));
                    if old != new || before != after {
                        for bounds in outer {
                            if let Some(shared) = shared {
                                for strip in difference(bounds, shared) {
                                    mark(&mut dirty, tile, columns, strip, extent);
                                }
                            } else {
                                mark(&mut dirty, tile, columns, bounds, extent);
                            }
                        }
                    }
                    clips.push((
                        outer,
                        shared.zip(parent_interior).map(|(a, b)| intersect(a, b)),
                    ));
                    continue;
                }
                (PopClip, PopClip) if clips.len() > 1 => {
                    clips.pop();
                    continue;
                }
                (PopClip, PopClip) => return None,
                _ => {}
            }
        }
        let clip = |edges, side: usize| {
            if surface {
                intersect(edges, parent[side])
            } else {
                edges
            }
        };
        match (old, new) {
            (PushTransform(old), PushTransform(new)) => {
                transforms.push((
                    [before.multiply(*old), after.multiply(*new)],
                    opacity_changed,
                    opaque_group,
                ));
            }
            (PopTransform, PopTransform) if transforms.len() > 1 => {
                transforms.pop();
            }
            (PushLayer(old), PushLayer(new)) if surface => {
                transforms.push((
                    [before, after],
                    opacity_changed || old.opacity() != new.opacity(),
                    opaque_group && old.opacity() == 1.0 && new.opacity() == 1.0,
                ));
            }
            (PopLayer, PopLayer) if surface && transforms.len() > 1 => {
                transforms.pop();
            }
            (
                DrawPath {
                    path,
                    paint: Paint::Fill(Brush::Solid(brush)),
                },
                _,
            ) if surface
                && opaque_group
                && old == new
                && before == after
                && brush.color().a == 1.0 =>
            {
                if let Some(coverage) = parent_interior
                    && before.coeffs[1] == 0.0
                    && before.coeffs[2] == 0.0
                    && let Some(rect) = path_rect(path)
                    && let Some(bounds) = device_edges(rect, before)
                {
                    clear_covered_tiles(
                        &mut dirty,
                        tile,
                        columns,
                        intersect(coverage, bounds),
                        extent,
                    );
                }
            }
            _ if old == new && before == after && !opacity_changed => {}
            (
                DrawPath {
                    path: old,
                    paint: Paint::Fill(_),
                },
                DrawPath {
                    path: new,
                    paint: Paint::Fill(_),
                },
            ) => {
                for (side, (bounds, transform)) in [(old.bounds(), before), (new.bounds(), after)]
                    .into_iter()
                    .enumerate()
                {
                    if let Some(bounds) = bounds {
                        mark(
                            &mut dirty,
                            tile,
                            columns,
                            clip(device_edges(bounds, transform)?, side),
                            extent,
                        );
                    }
                }
            }
            (
                DrawPath {
                    path: old,
                    paint: old_paint,
                },
                DrawPath {
                    path: new,
                    paint: new_paint,
                },
            ) if surface => {
                for (side, (path, paint, transform)) in
                    [(old, old_paint, before), (new, new_paint, after)]
                        .into_iter()
                        .enumerate()
                {
                    if let Some(bounds) = painted_path_bounds(path, paint) {
                        mark(
                            &mut dirty,
                            tile,
                            columns,
                            clip(device_edges(bounds, transform)?, side),
                            extent,
                        );
                    }
                }
            }
            (DrawText { .. }, DrawText { .. }) if surface => {
                for (side, bounds) in text_bounds?.into_iter().enumerate() {
                    let &[x, y, w, h] = bounds.get(index)?;
                    if w != 0 && h != 0 {
                        mark(
                            &mut dirty,
                            tile,
                            columns,
                            clip([x as f32, y as f32, (x + w) as f32, (y + h) as f32], side),
                            extent,
                        );
                    }
                }
            }
            (DrawShadow { .. }, DrawShadow { .. }) if surface => {
                mark(
                    &mut dirty,
                    tile,
                    columns,
                    clip(device_edges(shadow_bounds(old)?, before)?, 0),
                    extent,
                );
                mark(
                    &mut dirty,
                    tile,
                    columns,
                    clip(device_edges(shadow_bounds(new)?, after)?, 1),
                    extent,
                );
            }
            (DrawImage { region: old, .. }, DrawImage { region: new, .. }) => {
                mark(
                    &mut dirty,
                    tile,
                    columns,
                    clip(device_edges(*old, before)?, 0),
                    extent,
                );
                mark(
                    &mut dirty,
                    tile,
                    columns,
                    clip(device_edges(*new, after)?, 1),
                    extent,
                );
            }
            (
                PushClip(Clip::Shape(ClipShape::Rect(old))),
                PushClip(Clip::Shape(ClipShape::Rect(new))),
            ) => {
                let [a, b] = [device_edges(*old, before)?, device_edges(*new, after)?];
                let axis_aligned = |t: AffineTransform| t.coeffs[1] == 0.0 && t.coeffs[2] == 0.0;
                if before == after
                    && axis_aligned(before)
                    && before.coeffs[0] > 0.0
                    && before.coeffs[3] > 0.0
                    && old.origin == new.origin
                    && old.size.height == new.size.height
                    && old.size.width > 0.0
                    && old.size.height > 0.0
                    && new.size.width > 0.0
                    && new.size.height > 0.0
                {
                    for (base, other) in [(a, b), (b, a)] {
                        for strip in difference(base, other) {
                            mark(&mut dirty, tile, columns, strip, extent);
                        }
                    }
                } else {
                    mark(&mut dirty, tile, columns, a, extent);
                    mark(&mut dirty, tile, columns, b, extent);
                }
            }
            (PopClip, PopClip) => {}
            _ => {
                if surface {
                    println!(
                        "[scene-damage-fallback] pair={:?}/{:?} moved={}",
                        std::mem::discriminant(old),
                        std::mem::discriminant(new),
                        before != after
                    );
                }
                return None;
            }
        }
    }
    if transforms.len() != 1 || clips.len() != 1 {
        return None;
    }
    merge_regions(&dirty, tile, extent, surface)
}

fn merge_regions(
    dirty: &[bool],
    tile: u32,
    extent: (u32, u32),
    surface: bool,
) -> Option<Vec<Region>> {
    let (columns, rows) = (extent.0.div_ceil(tile), extent.1.div_ceil(tile));
    let mut merged: Vec<Region> = Vec::new();
    let mut area = 0_u64;
    for row in 0..rows {
        let mut column = 0;
        while column < columns {
            if !dirty[(row * columns + column) as usize] {
                column += 1;
                continue;
            }
            let start = column;
            while column < columns && dirty[(row * columns + column) as usize] {
                column += 1;
            }
            let (x, y) = (start * tile, row * tile);
            let w = (column * tile).min(extent.0) - x;
            let h = (extent.1 - y).min(tile);
            area += u64::from(w) * u64::from(h);
            if let Some(rect) = merged
                .iter_mut()
                .rev()
                .find(|r| r[0] == x && r[2] == w && r[1] + r[3] == y)
            {
                rect[3] += h;
            } else {
                merged.push([x, y, w, h]);
            }
        }
    }
    let full_area = u64::from(extent.0) * u64::from(extent.1);
    let acceptable = merged.len() <= 24 && area * 5 <= full_area * 3;
    if surface && !acceptable {
        println!(
            "[scene-damage-fallback] regions={} pixels={area} full_pixels={full_area}",
            merged.len()
        );
    }
    acceptable.then_some(merged)
}

fn topology_damage(
    lists: [&DisplayList; 2],
    extent: (u32, u32),
    text_bounds: [&[Region]; 2],
    mut mark: impl FnMut([f32; 4]),
) -> Option<()> {
    let [before, after] = lists.map(DisplayList::commands);
    let mut prefix_len = before.iter().zip(after).take_while(|(a, b)| a == b).count();
    let mut state = DrawState {
        transform: AffineTransform::IDENTITY,
        clip: [0.0, 0.0, extent.0 as f32, extent.1 as f32],
    };
    let mut scopes = Vec::new();
    for (index, command) in before[..prefix_len].iter().enumerate() {
        update_scope(command, index, &mut state, &mut scopes, 0)?;
    }
    // An edit can change how Lyon composites an enclosing opacity group.
    // Include the whole group, even when some of its draws compare equal.
    if let Some(layer) = scopes
        .iter()
        .position(|(scope, _, _)| *scope == DamageScope::Layer)
    {
        let (_, entry, start) = scopes[layer];
        prefix_len = start;
        state = entry;
        scopes.truncate(layer);
    }
    let suffix_len = before[prefix_len..]
        .iter()
        .rev()
        .zip(after[prefix_len..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let inherited_depth = scopes.len();
    let mut suffix_skipped = None;
    for (commands, text_bounds) in [before, after].into_iter().zip(text_bounds) {
        let mut state = state;
        let mut scopes = scopes.clone();
        let middle_end = commands.len() - suffix_len;
        let mut end = prefix_len;
        while end < middle_end || scopes.len() > inherited_depth {
            let command = commands.get(end)?;
            update_scope(command, end, &mut state, &mut scopes, inherited_depth)?;
            if let Some(bounds) = draw_bounds(command, text_bounds, end, state.transform)? {
                mark(intersect(state.clip, bounds));
            }
            end += 1;
        }
        let remaining = commands.len() - end;
        if suffix_skipped.is_some_and(|previous| previous != remaining) {
            return None;
        }
        suffix_skipped = Some(remaining);
    }
    Some(())
}

fn update_scope(
    command: &PaintCommand,
    index: usize,
    state: &mut DrawState,
    scopes: &mut Vec<(DamageScope, DrawState, usize)>,
    inherited_depth: usize,
) -> Option<()> {
    use PaintCommand::*;
    let (scope, next) = match command {
        PushTransform(transform) => (
            DamageScope::Transform,
            DrawState {
                transform: state.transform.multiply(*transform),
                ..*state
            },
        ),
        PushClip(clip) => (
            DamageScope::Clip,
            DrawState {
                clip: intersect(state.clip, device_edges(clip.bounds()?, state.transform)?),
                ..*state
            },
        ),
        PushLayer(layer) => (
            DamageScope::Layer,
            DrawState {
                transform: state.transform.multiply(layer.transform()),
                ..*state
            },
        ),
        PopTransform => return pop_scope(DamageScope::Transform, state, scopes, inherited_depth),
        PopClip => return pop_scope(DamageScope::Clip, state, scopes, inherited_depth),
        PopLayer => return pop_scope(DamageScope::Layer, state, scopes, inherited_depth),
        _ => return Some(()),
    };
    scopes.push((scope, *state, index));
    *state = next;
    Some(())
}

fn pop_scope(
    expected: DamageScope,
    state: &mut DrawState,
    scopes: &mut Vec<(DamageScope, DrawState, usize)>,
    inherited_depth: usize,
) -> Option<()> {
    if scopes.len() <= inherited_depth {
        return None;
    }
    let (scope, entry, _) = scopes.pop()?;
    if scope != expected {
        return None;
    }
    *state = entry;
    Some(())
}

fn draw_bounds(
    command: &PaintCommand,
    text_bounds: &[Region],
    index: usize,
    transform: AffineTransform,
) -> Option<Option<[f32; 4]>> {
    use PaintCommand::*;
    let bounds = match command {
        DrawPath { path, paint } => painted_path_bounds(path, paint),
        DrawShadow { .. } => Some(shadow_bounds(command)?),
        DrawImage { region, .. } => Some(*region),
        DrawText { .. } => {
            let &[x, y, w, h] = text_bounds.get(index)?;
            return Some((w > 0 && h > 0).then_some([
                x as f32,
                y as f32,
                (x + w) as f32,
                (y + h) as f32,
            ]));
        }
        _ => None,
    };
    match bounds {
        Some(bounds) => Some(Some(device_edges(bounds, transform)?)),
        None => Some(None),
    }
}

fn painted_path_bounds(path: &Path, paint: &Paint) -> Option<Rect> {
    let bounds = path.bounds()?;
    let Paint::Stroke(stroke) = paint else {
        return Some(bounds);
    };
    let pad = stroke.width().abs() * stroke.miter_limit().max(1.0);
    Some(Rect::new(
        Point::new(bounds.origin.x - pad, bounds.origin.y - pad),
        obscur_geometry::Size::new(
            bounds.size.width + 2.0 * pad,
            bounds.size.height + 2.0 * pad,
        ),
    ))
}

fn shadow_bounds(command: &PaintCommand) -> Option<Rect> {
    let PaintCommand::DrawShadow {
        rect,
        blur,
        spread,
        offset,
        inset,
        ..
    } = command
    else {
        return None;
    };
    if *inset {
        return Some(*rect);
    }
    let pad = spread + blur.max(0.0) * 1.75;
    Some(Rect::new(
        Point::new(
            rect.origin.x + offset.0 - pad,
            rect.origin.y + offset.1 - pad,
        ),
        obscur_geometry::Size::new(
            (rect.size.width + 2.0 * pad).max(0.0),
            (rect.size.height + 2.0 * pad).max(0.0),
        ),
    ))
}

fn device_edges(rect: Rect, transform: AffineTransform) -> Option<[f32; 4]> {
    let mut edges = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for x in [rect.origin.x, rect.right()] {
        for y in [rect.origin.y, rect.bottom()] {
            let p = transform.apply(Point::new(x, y));
            if !p.x.is_finite() || !p.y.is_finite() {
                return None;
            }
            edges = [
                edges[0].min(p.x),
                edges[1].min(p.y),
                edges[2].max(p.x),
                edges[3].max(p.y),
            ];
        }
    }
    Some(edges)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn clear_covered_tiles(
    dirty: &mut [bool],
    tile: u32,
    columns: u32,
    [l, t, r, b]: [f32; 4],
    extent: (u32, u32),
) {
    // A tile must be inside the opaque fill and its stencils in both frames,
    // including a pixel of clearance from antialiased edges.
    let left = (l.ceil() as u32)
        .saturating_add(1)
        .min(extent.0)
        .div_ceil(tile);
    let top = (t.ceil() as u32)
        .saturating_add(1)
        .min(extent.1)
        .div_ceil(tile);
    let right = (r.floor() as u32).saturating_sub(1).min(extent.0) / tile;
    let bottom = (b.floor() as u32).saturating_sub(1).min(extent.1) / tile;
    if left >= right || top >= bottom {
        return;
    }
    for y in top..bottom {
        let offset = y * columns;
        dirty[(offset + left) as usize..(offset + right) as usize].fill(false);
    }
}

fn intersect([l, t, r, b]: [f32; 4], [a, c, d, e]: [f32; 4]) -> [f32; 4] {
    [l.max(a), t.max(c), r.min(d), b.min(e)]
}

fn clip_interior(clip: &Clip, transform: AffineTransform) -> Option<[f32; 4]> {
    let [sx, skew_y, skew_x, sy, ..] = transform.coeffs;
    if skew_x != 0.0 || skew_y != 0.0 || sx <= 0.0 || sy <= 0.0 {
        return None;
    }
    match clip {
        Clip::Shape(ClipShape::Rect(rect)) => device_edges(*rect, transform),
        Clip::Shape(ClipShape::RoundedRect { rect, radii }) => {
            let radii = radii.normalized(*rect);
            let top = radii.top_left.y.max(radii.top_right.y);
            let bottom = radii.bottom_left.y.max(radii.bottom_right.y);
            let inner = Rect::new(
                Point::new(rect.origin.x, rect.origin.y + top),
                obscur_geometry::Size::new(rect.size.width, rect.size.height - top - bottom),
            );
            (inner.size.height > 0.0)
                .then(|| device_edges(inner, transform))
                .flatten()
        }
        _ => None,
    }
}

fn difference([l, t, r, b]: [f32; 4], other: [f32; 4]) -> Vec<[f32; 4]> {
    let [il, it, ir, ib] = [
        l.max(other[0]),
        t.max(other[1]),
        r.min(other[2]),
        b.min(other[3]),
    ];
    if il >= ir || it >= ib {
        return vec![[l, t, r, b]];
    }
    [
        [l, t, r, it],
        [l, ib, r, b],
        [l, it, il, ib],
        [ir, it, r, ib],
    ]
    .into_iter()
    .filter(|p| p[0] < p[2] && p[1] < p[3])
    .collect()
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn mark(dirty: &mut [bool], tile: u32, columns: u32, [l, t, r, b]: [f32; 4], extent: (u32, u32)) {
    if l > r || t > b {
        return;
    }
    let left = (l.floor() as u32).saturating_sub(1).min(extent.0);
    let top = (t.floor() as u32).saturating_sub(1).min(extent.1);
    let right = (r.ceil() as u32).saturating_add(1).min(extent.0);
    let bottom = (b.ceil() as u32).saturating_add(1).min(extent.1);
    if left >= right || top >= bottom {
        return;
    }
    for y in top / tile..bottom.div_ceil(tile) {
        let start = (y * columns + left / tile) as usize;
        let end = (y * columns + right.div_ceil(tile)) as usize;
        dirty[start..end].fill(true);
    }
}

pub(super) fn clear_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("obscur-lyon-clear-damage"),
        source: wgpu::ShaderSource::Wgsl(
            "@vertex fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
                let corners = array(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
                return vec4(corners[i], 0.0, 1.0);
             }
             @fragment fn fragment() -> @location(0) vec4<f32> { return vec4(0.0); }"
                .into(),
        ),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("obscur-lyon-clear-damage"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: super::pipelines::STENCIL,
            depth_write_enabled: None,
            depth_compare: None,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: super::pipelines::SAMPLES,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Size;
    use obscur_scene::color::Color;
    use obscur_scene::path::Path;

    use super::*;

    #[test]
    fn rectangle_clip_width_marks_only_the_exposed_strip() {
        let scene = |width| {
            DisplayList::from_commands(vec![
                PaintCommand::PushClip(Clip::rect(Rect::new(
                    Point::new(0.0, 0.0),
                    Size::new(width, 224.0),
                ))),
                PaintCommand::DrawPath {
                    path: Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(320.0, 224.0))),
                    paint: Paint::solid_fill(Color::WHITE),
                },
                PaintCommand::PopClip,
            ])
        };
        let plan = regions(&scene(102.0), &scene(113.0), (320, 224)).unwrap();
        assert_eq!(plan, [[96, 0, 32, 224]]);
    }

    #[test]
    fn surface_damage_tracks_moving_shadow_in_a_plain_opacity_group() {
        use obscur_geometry::CornerRadii;
        use obscur_scene::layer::SceneLayer;
        let scene = |x, opacity| {
            DisplayList::from_commands(vec![
                PaintCommand::PushLayer(SceneLayer::new().with_opacity(opacity)),
                PaintCommand::DrawShadow {
                    rect: Rect::new(Point::new(x, 40.0), Size::new(8.0, 12.0)),
                    radii: CornerRadii::default(),
                    color: Color::BLACK,
                    blur: 4.0,
                    spread: 0.0,
                    offset: (0.0, 0.0),
                    inset: false,
                },
                PaintCommand::PopLayer,
            ])
        };
        assert_eq!(
            surface_regions(
                &scene(92.0, 0.8),
                &scene(100.0, 0.6),
                (320, 224),
                [&[], &[]]
            ),
            Some(vec![[64, 32, 64, 32]])
        );
    }

    #[test]
    fn surface_damage_uses_rendered_bounds_for_both_text_positions() {
        use obscur_scene::{paint::Brush, text::GlyphRun};

        let scene = |x| {
            DisplayList::from_commands(vec![PaintCommand::DrawText {
                origin: Point::new(x, 10.0),
                run: GlyphRun::new(Vec::new(), 12.0, "label"),
                brush: Brush::solid(Color::WHITE),
            }])
        };
        assert_eq!(
            surface_regions(
                &scene(10.0),
                &scene(68.0),
                (320, 224),
                [&[[10, 10, 10, 10]], &[[68, 10, 10, 10]]],
            ),
            Some(vec![[0, 0, 32, 32], [64, 0, 32, 32]])
        );
        assert!(surface_regions(&scene(10.0), &scene(68.0), (320, 224), [&[], &[]]).is_none());
    }

    #[test]
    fn surface_damage_does_not_include_hidden_parts_of_a_changed_fill() {
        let scene = |color| {
            DisplayList::from_commands(vec![
                PaintCommand::PushClip(Clip::rect(Rect::new(
                    Point::new(64.0, 0.0),
                    Size::new(32.0, 224.0),
                ))),
                PaintCommand::DrawPath {
                    path: Path::rect(Rect::new(Point::new(0.0, 0.0), Size::new(320.0, 224.0))),
                    paint: Paint::solid_fill(color),
                },
                PaintCommand::PopClip,
            ])
        };
        assert_eq!(
            surface_regions(
                &scene(Color::WHITE),
                &scene(Color::BLACK),
                (320, 224),
                [&[], &[]]
            ),
            Some(vec![[32, 0, 96, 224]])
        );
    }

    #[test]
    fn unchanged_opaque_fill_cancels_hidden_damage_but_translucent_fill_does_not() {
        let scene = |color, foreground| {
            let fill = |x, y, width, height, color| PaintCommand::DrawPath {
                path: Path::rect(Rect::new(Point::new(x, y), Size::new(width, height))),
                paint: Paint::solid_fill(color),
            };
            DisplayList::from_commands(vec![
                fill(70.0, 70.0, 8.0, 8.0, color),
                fill(32.0, 32.0, 96.0, 96.0, foreground),
            ])
        };
        let plan = |foreground| {
            surface_regions(
                &scene(Color::WHITE, foreground),
                &scene(Color::BLACK, foreground),
                (320, 224),
                [&[], &[]],
            )
        };
        assert_eq!(plan(Color::BLACK), Some(Vec::new()));
        assert_eq!(
            plan(Color::rgba(0.0, 0.0, 0.0, 0.9)),
            Some(vec![[64, 64, 32, 32]])
        );
    }

    #[test]
    fn surface_damage_tracks_insertion_and_removal_inside_a_clip() {
        use PaintCommand::*;
        let draw = |x, y, width, height| DrawPath {
            path: Path::rect(Rect::new(Point::new(x, y), Size::new(width, height))),
            paint: Paint::solid_fill(Color::WHITE),
        };
        let scene = |insert| {
            let mut commands = vec![
                draw(0.0, 0.0, 256.0, 256.0),
                PushClip(Clip::rect(Rect::new(
                    Point::new(70.0, 70.0),
                    Size::new(12.0, 12.0),
                ))),
            ];
            if insert {
                commands.push(draw(0.0, 0.0, 256.0, 256.0));
            }
            commands.extend([PopClip, draw(160.0, 160.0, 80.0, 80.0)]);
            DisplayList::from_commands(commands)
        };
        let empty = scene(false);
        let inserted = scene(true);
        for (before, after) in [(&empty, &inserted), (&inserted, &empty)] {
            assert_eq!(
                surface_regions(before, after, (256, 256), [&[], &[]]),
                Some(vec![[64, 64, 32, 32]])
            );
        }
    }

    #[test]
    fn surface_damage_includes_the_existing_fill_when_opacity_group_gains_a_draw() {
        use PaintCommand::*;
        let draw = |x| DrawPath {
            path: Path::rect(Rect::new(Point::new(x, 72.0), Size::new(12.0, 12.0))),
            paint: Paint::solid_fill(Color::WHITE),
        };
        let scene = |insert| {
            let mut commands = vec![
                PushLayer(obscur_scene::layer::SceneLayer::new().with_opacity(0.5)),
                draw(72.0),
            ];
            if insert {
                commands.push(draw(136.0));
            }
            commands.push(PopLayer);
            DisplayList::from_commands(commands)
        };
        assert_eq!(
            surface_regions(&scene(false), &scene(true), (256, 256), [&[], &[]]),
            Some(vec![[64, 64, 32, 32], [128, 64, 32, 32]])
        );
    }

    #[test]
    fn insertions_require_a_full_frame() {
        let paint = PaintCommand::DrawPath {
            path: Path::rect(Rect::new(Point::new(12.0, 24.0), Size::new(20.0, 20.0))),
            paint: Paint::solid_fill(Color::WHITE),
        };
        assert!(
            regions(
                &DisplayList::new(),
                &DisplayList::from_commands(vec![paint]),
                (128, 128)
            )
            .is_none()
        );
    }
}
