use obscur_geometry::Rect;

/// Port for a single host window to query operating-system gesture regions and request exclusions. Rectangles are in logical window coordinates.
pub trait SystemGestureRegions {
    /// Reports the system gesture regions the host can enumerate. An empty result may indicate unavailable geometry.
    fn reserved_regions(&self) -> Vec<Rect>;

    /// Submits the complete set of requested exclusions. An empty slice clears the request. True means the host accepted the request for processing. System policy determines the applied regions.
    fn set_exclusion_regions(&self, regions: &[Rect]) -> bool;
}
