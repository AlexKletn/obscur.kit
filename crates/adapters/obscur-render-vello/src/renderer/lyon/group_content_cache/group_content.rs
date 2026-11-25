use std::borrow::Cow;

use super::super::gradient::GRADIENT_ROWS;
use super::super::mesh::{MAX_DRAW_PARAMETERS, Mesh, Operation};
use super::{ContentOperation, DrawContent, GroupInput};

const MAX_GEOMETRY_BYTES: usize = 2 * 1024 * 1024;

#[derive(PartialEq)]
pub(super) struct GroupContent<'a> {
    operations: Vec<ContentOperation<'a>>,
    translated_extent: Option<[u32; 2]>,
    transforms: Vec<(usize, [u32; 8])>,
    gradients: Vec<(usize, Cow<'a, [u8]>)>,
}

impl<'a> GroupContent<'a> {
    pub(super) fn into_owned(self) -> GroupContent<'static> {
        GroupContent {
            operations: self
                .operations
                .into_iter()
                .map(|operation| match operation {
                    ContentOperation::Draw(draw) => ContentOperation::Draw(DrawContent {
                        operation: draw.operation,
                        depth: draw.depth,
                        image: draw.image,
                        base_vertex: draw.base_vertex,
                        indices: Cow::Owned(draw.indices.into_owned()),
                        vertices: Cow::Owned(draw.vertices.into_owned()),
                    }),
                    ContentOperation::Composite(composite) => {
                        ContentOperation::Composite(composite)
                    }
                })
                .collect(),
            translated_extent: self.translated_extent,
            transforms: self.transforms,
            gradients: self
                .gradients
                .into_iter()
                .map(|(slot, bytes)| (slot, Cow::Owned(bytes.into_owned())))
                .collect(),
        }
    }

    pub(super) fn from_inputs(
        inputs: Vec<GroupInput<'_>>,
        mesh: &'a Mesh,
        transforms: &[[[f32; 4]; 2]],
        bounds: Option<[u32; 4]>,
    ) -> Option<(Self, Option<[u32; 2]>)> {
        let mut operations = Vec::with_capacity(inputs.len());
        let mut transform_slots = [false; MAX_DRAW_PARAMETERS];
        let mut gradient_slots = [false; GRADIENT_ROWS];
        let mut remaining = MAX_GEOMETRY_BYTES;
        for input in inputs {
            let draw = match input {
                GroupInput::Draw(draw) => draw,
                GroupInput::Composite(composite) => {
                    operations.push(ContentOperation::Composite(composite));
                    continue;
                }
            };
            let image = match draw.operation {
                Operation::Paint | Operation::Push | Operation::Pop => None,
                Operation::Image(slot) => Some(mesh.images.get(slot)?.clone()),
                _ => return None,
            };
            let indices = mesh
                .indices
                .get(draw.indices.start as usize..draw.indices.end as usize)?;
            let first = *indices.iter().min()?;
            let last = *indices.iter().max()?;
            let vertices = mesh.vertices.get(first as usize..=last as usize)?;
            let bytes = bytemuck::cast_slice::<_, u8>(vertices);
            remaining = remaining.checked_sub(bytes.len() + std::mem::size_of_val(indices))?;
            for vertex in vertices {
                *transform_slots.get_mut(vertex.transform as usize)? = true;
                *gradient_slots.get_mut(vertex.gradient as usize)? = true;
            }
            operations.push(ContentOperation::Draw(DrawContent {
                operation: draw.operation,
                depth: draw.depth,
                image,
                base_vertex: first,
                indices: Cow::Borrowed(indices),
                vertices: Cow::Borrowed(bytes),
            }));
        }

        let bounds = bounds.filter(|_| {
            transform_slots.iter().enumerate().all(|(slot, &used)| {
                !used
                    || mesh.parameters.get(slot).is_some_and(|parameter| {
                        parameter.transform.coeffs[4..]
                            .iter()
                            .all(|&offset| offset.abs() < 4_194_304.0 && offset.fract() == 0.0)
                    })
            })
        });
        let mut used_transforms = Vec::new();
        for (slot, _) in transform_slots
            .into_iter()
            .enumerate()
            .filter(|(_, used)| *used)
        {
            let [a, b] = *transforms.get(slot)?;
            let mut rows = [0; 8];
            rows[..4].copy_from_slice(&a.map(f32::to_bits));
            rows[4..].copy_from_slice(&b.map(f32::to_bits));
            if let Some([x, y, _, _]) = bounds {
                let [_, _, _, _, tx, ty] = mesh.parameters[slot].transform.coeffs;
                rows[2] = (tx as i32 - x as i32) as u32;
                rows[6] = (ty as i32 - y as i32) as u32;
            }
            used_transforms.push((slot, rows));
        }
        let mut gradients = Vec::new();
        for (slot, _) in gradient_slots
            .into_iter()
            .enumerate()
            .skip(1)
            .filter(|(_, used)| *used)
        {
            gradients.push((
                slot,
                Cow::Borrowed(bytemuck::cast_slice(mesh.gradients.encoded(slot)?)),
            ));
        }
        Some((
            Self {
                operations,
                translated_extent: bounds.map(|[_, _, w, h]| [w, h]),
                transforms: used_transforms,
                gradients,
            },
            bounds.map(|[x, y, _, _]| [x, y]),
        ))
    }
}
