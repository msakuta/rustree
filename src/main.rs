mod bounding_box;
mod point;
mod r_tree;

use crate::{bounding_box::BoundingBox, point::Point, r_tree::RTree};

fn main() -> std::io::Result<()> {
    let mut rtree = RTree::new();
    let mut try_add = |x, y| {
        let pt = Point { x, y };
        rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
        println!("inserted: {rtree:?}");
        println!("now bb: {:?}", rtree.bounding_box());
    };
    try_add(2., 0.);
    try_add(-2., 1.);
    try_add(1., 7.);
    try_add(0., 5.);
    try_add(-1., -5.);

    let bbox = BoundingBox::from_center_size(Point::new(0.5, 7.1), Point::new(0.5, 0.5));
    let found = rtree.find(&bbox);
    println!("Found single: {found:?}");

    if let Ok(f) = std::fs::File::create("graph.dot") {
        rtree.dot(true, &mut std::io::BufWriter::new(f))?;
    }

    let bbox = BoundingBox::from_center_size(Point::new(0., 0.5), Point::new(2., 2.));
    for (i, found) in rtree.find_multi(&bbox).enumerate() {
        println!("Found multi[{i}]: {found:?}");
    }

    Ok(())
}
