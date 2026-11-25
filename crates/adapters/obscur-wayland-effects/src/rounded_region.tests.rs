use super::*;

#[test]
fn rectangles_follow_the_rounded_outline_and_merge_center_rows() {
    let shape = RoundedRegion::new([12.0, 10.0], [[3.0, 3.0]; 4]).unwrap();
    let mut rectangles = Vec::new();
    shape.rectangles(|x, y, width, height| rectangles.push((x, y, width, height)));

    assert_eq!(
        rectangles,
        [
            (3, 0, 6, 1),
            (1, 1, 10, 2),
            (0, 3, 12, 4),
            (1, 7, 10, 2),
            (3, 9, 6, 1)
        ]
    );
}
