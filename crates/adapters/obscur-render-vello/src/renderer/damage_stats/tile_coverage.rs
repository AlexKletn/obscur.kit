#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TileCoverage {
    pub(super) tiles: u32,
    pub(super) row_runs: u32,
    pub(super) pixels: u32,
}
