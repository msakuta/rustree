use super::{RTree, RTreeEntry, RTreeNode};
use std::fmt::Debug;

#[non_exhaustive]
pub struct WalkCallbackPayload<'a, T> {
    pub id: usize,
    pub level: usize,
    pub entry: &'a RTreeEntry<T>,
}

impl<T: Debug> RTree<T> {
    fn walk_rec(&self, id: usize, level: usize, f: &mut impl FnMut(&WalkCallbackPayload<T>)) {
        f(&WalkCallbackPayload {
            id,
            level,
            entry: &self.nodes[id],
        });
        match self.nodes[id].node {
            RTreeNode::Leaf(_) => {}
            RTreeNode::Node(ref children) => {
                for child in children {
                    self.walk_rec(*child, level + 1, f);
                }
            }
        }
    }

    pub fn walk(&self, f: &mut impl FnMut(&WalkCallbackPayload<T>)) {
        self.walk_rec(0, 0, f);
    }
}
