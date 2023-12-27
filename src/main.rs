mod bounding_box;
mod point;
mod r_tree;

use crate::{bounding_box::BoundingBox, point::Point, r_tree::RTree};

fn main() {
    let mut rtree = RTree::new();
    let mut try_add = |x, y| {
        let pt = Point { x, y };
        rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
        println!("inserted: {rtree:?}");
        println!("now bb: {:?}", rtree.bounding_box());
    };
    try_add(2., 0.);
    try_add(-2., 0.);
    try_add(1., 7.);
    try_add(1., 5.);
    try_add(-1., -5.);

    let pt = Point { x: 0.5, y: 7.1 };
    let min = Point {
        x: pt.x - 0.5,
        y: pt.y - 0.5,
    };
    let max = Point {
        x: pt.x + 0.5,
        y: pt.y + 0.5,
    };
    let found = rtree.find(&BoundingBox { min, max });
    println!("Found: {found:?}");
}
