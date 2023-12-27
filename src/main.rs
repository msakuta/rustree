mod bounding_box;
mod point;
mod r_tree;

use crate::{bounding_box::BoundingBox, point::Point, r_tree::RTree};

fn main() {
    let mut rtree = RTree::new();
    let pt = Point { x: 2., y: 0. };
    rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
    println!("inserted: {rtree:?}");
    let pt = Point { x: -2., y: 0. };
    rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
    println!("inserted {pt:?}: {rtree:?}");
    let pt = Point { x: 1., y: 3. };
    rtree.insert_entry(pt, BoundingBox { min: pt, max: pt });
    println!("inserted {pt:?}: {rtree:?}");
}
