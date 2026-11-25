#![doc = include_str!("../../docs/api/algorithm/grid_item_margins.doc/_module.md")]

use crate::model::Margin;

#[doc = include_str!("../../docs/api/algorithm/grid_item_margins.doc/absorbs_free_space.md")]
pub(super) const fn absorbs_free_space(lead: &Margin, trail: &Margin) -> bool {
    lead.is_auto() || trail.is_auto()
}

#[doc = include_str!("../../docs/api/algorithm/grid_item_margins.doc/auto_lead.md")]
pub(super) fn auto_lead(lead: &Margin, trail: &Margin, free: f32) -> f32 {
    let free = free.max(0.0);
    match (lead.is_auto(), trail.is_auto()) {
        (true, true) => free / 2.0,
        (true, false) => free,
        (false, _) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::{absorbs_free_space, auto_lead};
    use crate::model::Margin;

    #[test]
    fn a_length_pair_leaves_the_free_space_to_the_alignment() {
        assert!(!absorbs_free_space(&Margin::Px(2.0), &Margin::Px(2.0)));
        assert_eq!(auto_lead(&Margin::Px(2.0), &Margin::Px(2.0), 40.0), 0.0);
    }

    #[test]
    fn one_auto_edge_takes_all_of_it() {
        // `margin-left: auto` on a grid item pushes it to the end of its area.
        assert!(absorbs_free_space(&Margin::Auto, &Margin::Px(0.0)));
        assert_eq!(auto_lead(&Margin::Auto, &Margin::Px(0.0), 40.0), 40.0);
        // …and `margin-right: auto` leaves it at the start.
        assert!(absorbs_free_space(&Margin::Px(0.0), &Margin::Auto));
        assert_eq!(auto_lead(&Margin::Px(0.0), &Margin::Auto, 40.0), 0.0);
    }

    #[test]
    fn two_auto_edges_centre_the_item() {
        assert_eq!(auto_lead(&Margin::Auto, &Margin::Auto, 40.0), 20.0);
    }

    #[test]
    fn an_item_wider_than_its_area_gets_no_negative_offset() {
        // An overflowing item stays at the start of its area rather than being
        // pulled back out of it.
        assert_eq!(auto_lead(&Margin::Auto, &Margin::Auto, -40.0), 0.0);
    }
}
