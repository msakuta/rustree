use crate::bounding_box::BoundingBox;
use crate::point::Point;
use std::fmt::Debug;

const M: usize = 4;

#[derive(Debug)]
pub enum RTreeNode<T> {
    Node(Vec<usize>),
    Leaf(T),
}

#[derive(Debug)]
struct RTreeEntry<T> {
    pub bb: BoundingBox,
    pub parent: Option<usize>,
    pub node: RTreeNode<T>,
}

#[derive(Debug)]
pub struct RTree<T> {
    pub nodes: Vec<RTreeEntry<T>>,
}

impl<T: Debug> RTree<T> {
    pub fn new() -> Self {
        Self {
            nodes: vec![RTreeEntry {
                bb: BoundingBox {
                    min: Point { x: -1., y: -1. },
                    max: Point { x: 1., y: 1. },
                },
                parent: None,
                node: RTreeNode::Node(vec![]),
            }],
        }
    }

    fn choose_leaf_rec(&self, node: usize, bounding_box: &BoundingBox) -> usize {
        if matches!(self.nodes[node].node, RTreeNode::Leaf(_)) {
            return node;
        }
        let min = if let RTreeNode::Node(children) = &self.nodes[node].node {
            children
                .iter()
                .filter(|child| matches!(self.nodes[**child].node, RTreeNode::Node(_)))
                .min_by(|a, b| {
                    let area_a = self.nodes[**a].bb.get_union(bounding_box).get_area();
                    let area_b = self.nodes[**b].bb.get_union(bounding_box).get_area();
                    area_a.partial_cmp(&area_b).unwrap()
                })
                .cloned()
        } else {
            None
        };
        if let Some(min) = min {
            self.choose_leaf_rec(min, bounding_box)
        } else {
            node
        }
    }
    pub fn choose_leaf(&self, this: usize, bounding_box: &BoundingBox) -> usize {
        self.choose_leaf_rec(this, bounding_box)
    }

    pub fn adjust_tree(&mut self, node: usize, nodes_to_add: &mut Vec<RTreeEntry<T>>) {
        let node_bb = self.nodes[node].bb;
        match self.nodes[node].parent {
            None => (),
            Some(p) => {
                let mut bb = node_bb;
                let size = match &self.nodes[node].node {
                    RTreeNode::Leaf(_) => 0,
                    RTreeNode::Node(children) => {
                        for child in children {
                            bb = bb.get_union(&self.nodes[*child].bb);
                        }
                        children.len()
                    }
                };

                let parent = &mut self.nodes[p];
                parent.bb = bb;

                if !nodes_to_add.is_empty() {
                    if size < M {
                        let parent = &mut self.nodes[p];
                        let mut node_to_add = nodes_to_add.remove(0);
                        node_to_add.parent = Some(p);
                        parent.bb = parent.bb.get_union(&node_to_add.bb);
                        let idx = self.nodes.len();
                        self.nodes.push(node_to_add);
                        match &mut self.nodes[node].node {
                            RTreeNode::Leaf(_) => (),
                            RTreeNode::Node(children) => {
                                children.push(idx);
                            }
                        }
                    } else {
                        // Split the node
                    }
                }
                self.adjust_tree(p, nodes_to_add);
            }
        }
    }

    pub fn insert_entry(&mut self, value: T, bounding_box: BoundingBox) {
        let chosen_leaf = self.choose_leaf(0, &bounding_box);

        let node_to_add = RTreeEntry {
            bb: bounding_box,
            parent: Some(chosen_leaf),
            node: RTreeNode::Leaf(value),
        };

        let idx = self.nodes.len();
        self.nodes.push(node_to_add);
        let chosen_leaf = &mut self.nodes[chosen_leaf];
        let RTreeEntry { node, bb, .. } = &mut *chosen_leaf;

        match node {
            RTreeNode::Leaf(_) => println!("Adding to leaf!!"),
            RTreeNode::Node(children) => {
                if children.len() < M {
                    *bb = bb.get_union(&bounding_box);
                    children.push(idx);
                } else {
                    // Split the node
                }
            }
        }
    }
}
