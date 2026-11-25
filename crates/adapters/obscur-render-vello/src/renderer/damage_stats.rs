//! Compare consecutive fully rendered frames for damage diagnostics.

mod frame_capture;
#[cfg(feature = "lyon")]
mod scene_damage_baseline;
mod tile_coverage;

use frame_capture::FrameCapture;
#[cfg(feature = "lyon")]
use scene_damage_baseline::SceneDamageBaseline;
use tile_coverage::TileCoverage;

use std::collections::VecDeque;
use std::sync::mpsc;

use super::{VelloDevice, VelloFrame};

const TILE_SIZES: [u32; 4] = [16, 32, 64, 128];
const SLOTS: usize = 4;

#[derive(Default)]
pub(super) struct DamageStats {
    extent: (u32, u32),
    format: Option<wgpu::TextureFormat>,
    pending: VecDeque<FrameCapture>,
    free: Vec<wgpu::Buffer>,
    previous: Vec<u8>,
    previous_id: Option<u64>,
    #[cfg(feature = "lyon")]
    scene: Option<SceneDamageBaseline>,
    #[cfg(feature = "lyon")]
    scene_plan: Option<Vec<[u32; 4]>>,
}

impl DamageStats {
    pub(super) fn enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var_os("OBSCUR_DAMAGE_STATS").is_some_and(|value| value == "1")
        })
    }

    #[cfg(feature = "lyon")]
    pub(super) fn scene_enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED
            .get_or_init(|| std::env::var_os("OBSCUR_SCENE_DAMAGE_STATS").is_some_and(|v| v == "1"))
    }

    #[cfg(feature = "lyon")]
    pub(super) fn invalidate_scene(&mut self) {
        self.scene = None;
        self.scene_plan = None;
    }

    #[cfg(feature = "lyon")]
    pub(super) fn scene_regions(
        &mut self,
        snapshot: &obscur_scene::scene::SceneSnapshot,
        extent: (u32, u32),
        format: wgpu::TextureFormat,
        text_bounds: Vec<[u32; 4]>,
    ) -> Option<Vec<[u32; 4]>> {
        let previous = self.scene.replace(SceneDamageBaseline {
            list: std::sync::Arc::clone(snapshot.shared_display_list()),
            extent,
            format,
            text_bounds,
        })?;
        if previous.extent != extent || previous.format != format {
            return None;
        }
        if std::sync::Arc::ptr_eq(&previous.list, snapshot.shared_display_list()) {
            return Some(Vec::new());
        }
        super::lyon::partial_redraw::surface_regions(
            &previous.list,
            snapshot.display_list(),
            extent,
            [&previous.text_bounds, &self.scene.as_ref()?.text_bounds],
        )
    }

    #[cfg(feature = "lyon")]
    pub(super) fn record_scene(
        &mut self,
        snapshot: &obscur_scene::scene::SceneSnapshot,
        (width, height): (u32, u32),
        format: wgpu::TextureFormat,
        id: u64,
        text_bounds: Vec<[u32; 4]>,
    ) {
        self.scene_plan = self.scene_regions(snapshot, (width, height), format, text_bounds);
        if let Some(regions) = &self.scene_plan {
            let area: u64 = regions
                .iter()
                .map(|r| u64::from(r[2]) * u64::from(r[3]))
                .sum();
            println!(
                "[scene-damage] frame={id} regions={} pixels={area} extent={width}x{height}",
                regions.len()
            );
        } else {
            println!("[scene-damage] frame={id} full=true extent={width}x{height}");
        }
    }

    pub(super) fn capture(&mut self, gpu: &VelloDevice, frame: &VelloFrame, id: u64) {
        let (width, height) = frame.size();
        // Coverage counters are u32 and never exceed the number of pixels.
        if width == 0 || height == 0 || width.checked_mul(height).is_none() {
            return;
        }
        let format = frame.texture().format();
        if self.extent != (width, height) || self.format != Some(format) {
            *self = Self {
                extent: (width, height),
                format: Some(format),
                #[cfg(feature = "lyon")]
                scene: self.scene.take(),
                #[cfg(feature = "lyon")]
                scene_plan: self.scene_plan.take(),
                ..Default::default()
            };
        }
        let Some((_, stride)) = super::readback_layout::readback_row_bytes(width) else {
            return;
        };
        let bytes = u64::from(stride) * u64::from(height);
        if bytes > gpu.device().limits().max_buffer_size
            || usize::try_from(bytes).is_err()
            || gpu.device().poll(wgpu::PollType::Poll).is_err()
        {
            return;
        }
        self.collect(stride, width, height);
        if self.pending.len() >= SLOTS {
            return;
        }
        let buffer = self.free.pop().unwrap_or_else(|| {
            gpu.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("obscur-damage-readback"),
                size: bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("obscur-damage-readback"),
            });
        encoder.copy_texture_to_buffer(
            frame.texture().as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            frame.texture().size(),
        );
        gpu.queue().submit([encoder.finish()]);
        let (sender, ready) = mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        self.pending.push_back(FrameCapture {
            buffer,
            ready,
            id,
            #[cfg(feature = "lyon")]
            scene_plan: self.scene_plan.take(),
        });
    }

    fn collect(&mut self, stride: u32, width: u32, height: u32) {
        while let Some(next) = self.pending.front() {
            let mapped = match next.ready.try_recv() {
                Ok(result) => result.is_ok(),
                Err(mpsc::TryRecvError::Disconnected) => false,
                Err(mpsc::TryRecvError::Empty) => break,
            };
            let capture = self.pending.pop_front().expect("queued capture");
            let FrameCapture { ref buffer, id, .. } = capture;
            if !mapped {
                self.previous_id = None;
                continue;
            }
            {
                let pixels = buffer.get_mapped_range(..);
                if let Some((previous, gap)) = self
                    .previous_id
                    .and_then(|previous| id.checked_sub(previous).map(|gap| (previous, gap)))
                {
                    #[cfg(feature = "lyon")]
                    if let Some(regions) = capture.scene_plan.as_ref().filter(|_| gap == 1) {
                        let missed = uncovered_changes(
                            &self.previous,
                            &pixels,
                            width,
                            height,
                            stride,
                            regions,
                        );
                        println!("[scene-damage-check] frame={id} missed_pixels={missed}");
                    }
                    let (changed, grids) = compare(&self.previous, &pixels, width, height, stride);
                    let TileCoverage {
                        tiles: dirty_tiles,
                        pixels: tile_pixels,
                        ..
                    } = grids[1];
                    let count = f64::from(width) * f64::from(height);
                    println!(
                        "[damage] frame={id} previous={previous} gap={} extent={width}x{height} changed_pct={:.2} tile_pct={:.2} dirty_tiles={dirty_tiles}",
                        gap,
                        100.0 * f64::from(changed) / count,
                        100.0 * f64::from(tile_pixels) / count,
                    );
                    for (size, coverage) in TILE_SIZES.into_iter().zip(grids) {
                        println!(
                            "[damage-tiles] frame={id} gap={} size={size} dirty_tiles={} row_runs={} tile_pct={:.2}",
                            gap,
                            coverage.tiles,
                            coverage.row_runs,
                            100.0 * f64::from(coverage.pixels) / count,
                        );
                    }
                }
                self.previous.clear();
                // A diagnostic capture may be dropped under memory pressure.
                self.previous_id = None;
                if self.previous.try_reserve_exact(pixels.len()).is_ok() {
                    self.previous.extend_from_slice(&pixels);
                    self.previous_id = Some(id);
                }
            }
            buffer.unmap();
            self.free.push(capture.buffer);
        }
    }
}

fn compare(
    previous: &[u8],
    pixels: &[u8],
    width: u32,
    height: u32,
    stride: u32,
) -> (u32, [TileCoverage; TILE_SIZES.len()]) {
    let smallest = TILE_SIZES[0];
    let columns = width.div_ceil(smallest);
    let rows = height.div_ceil(smallest);
    let mut dirty = vec![false; (columns * rows) as usize];
    let mut changed = 0;

    for ty in 0..rows {
        let top = ty * smallest;
        let bottom = top.saturating_add(smallest).min(height);
        for tx in 0..columns {
            let left = tx * smallest;
            let right = left.saturating_add(smallest).min(width);
            let mut tile_changed = 0;
            for y in top..bottom {
                let row = y as usize * stride as usize;
                let range = row + left as usize * 4..row + right as usize * 4;
                let from = &previous[range.clone()];
                let to = &pixels[range];
                if from != to {
                    tile_changed += from
                        .chunks_exact(4)
                        .zip(to.chunks_exact(4))
                        .filter(|(old, new)| old != new)
                        .count() as u32;
                }
            }
            if tile_changed != 0 {
                changed += tile_changed;
                dirty[(ty * columns + tx) as usize] = true;
            }
        }
    }

    let coverage = TILE_SIZES.map(|size| {
        let scale = size / smallest;
        let mut coverage = TileCoverage::default();
        for row in 0..height.div_ceil(size) {
            let mut previous_dirty = false;
            for column in 0..width.div_ceil(size) {
                let tile_dirty = (row * scale..((row + 1) * scale).min(rows)).any(|y| {
                    let left = (y * columns + column * scale) as usize;
                    let right = (y * columns + ((column + 1) * scale).min(columns)) as usize;
                    dirty[left..right].contains(&true)
                });
                if tile_dirty {
                    coverage.tiles += 1;
                    coverage.row_runs += u32::from(!previous_dirty);
                    coverage.pixels +=
                        (width - column * size).min(size) * (height - row * size).min(size);
                }
                previous_dirty = tile_dirty;
            }
        }
        coverage
    });
    (changed, coverage)
}

#[cfg(feature = "lyon")]
fn uncovered_changes(
    previous: &[u8],
    next: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    regions: &[[u32; 4]],
) -> usize {
    (0..height)
        .map(|y| {
            let offset = y as usize * stride as usize;
            let row = offset..offset + width as usize * 4;
            let old = &previous[row.clone()];
            let new = &next[row];
            if old == new {
                return 0;
            }
            let row_regions: Vec<_> = regions
                .iter()
                .filter(|r| y.checked_sub(r[1]).is_some_and(|offset| offset < r[3]))
                .collect();
            old.chunks_exact(4)
                .zip(new.chunks_exact(4))
                .enumerate()
                .filter(|(x, (a, b))| {
                    a != b
                        && !row_regions.iter().any(|r| {
                            x.checked_sub(r[0] as usize)
                                .is_some_and(|offset| offset < r[2] as usize)
                        })
                })
                .count()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_changed_pixels_and_partial_edge_tiles_without_row_padding() {
        let old = vec![0; 256 * 33];
        let mut new = old.clone();
        new[255] = 1;
        assert_eq!(
            compare(&old, &new, 33, 33, 256),
            (0, [TileCoverage::default(); 4])
        );
        new[0] = 1;
        new[32 * 256 + 32 * 4 + 3] = 1;
        let (changed, grids) = compare(&old, &new, 33, 33, 256);
        assert_eq!(changed, 2);
        assert_eq!(grids.map(|grid| grid.tiles), [2, 2, 1, 1]);
        assert_eq!(grids.map(|grid| grid.pixels), [257, 1025, 1089, 1089]);
        assert_eq!(grids.map(|grid| grid.row_runs), [2, 2, 1, 1]);
    }

    #[cfg(feature = "lyon")]
    #[test]
    fn coverage_with_an_unrepresentable_far_edge_still_counts_missed_pixels() {
        let before = [0; 16];
        let mut after = before;
        after[0] = 255;
        after[12] = 255;
        let regions = [[1, 1, u32::MAX, u32::MAX]];

        assert_eq!(uncovered_changes(&before, &after, 2, 2, 8, &regions), 1);
    }

    #[test]
    fn merges_adjacent_dirty_tiles_into_horizontal_runs() {
        let previous = vec![0; 256 * 16];
        let mut next = previous.clone();
        for x in [0, 16, 48] {
            next[x * 4] = 255;
        }
        let (changed, grids) = compare(&previous, &next, 64, 16, 256);
        assert_eq!(changed, 3);
        assert_eq!(grids.map(|grid| grid.tiles), [3, 2, 1, 1]);
        assert_eq!(grids.map(|grid| grid.row_runs), [2, 1, 1, 1]);
    }
}
