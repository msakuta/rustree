use crate::{bounding_box::BoundingBox, point::Point};
use std::{fmt::Debug, io::Write};

const M: usize = 4;

#[derive(Debug)]
enum RTreeNode<T> {
    Node(Vec<usize>),
    Leaf(T),
}

#[derive(Debug)]
struct RTreeEntry<T> {
    bb: BoundingBox,
    parent: Option<usize>,
    node: RTreeNode<T>,
}

#[derive(Debug)]
pub struct RTree<T> {
    nodes: Vec<RTreeEntry<T>>,
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

    pub fn bounding_box(&self) -> BoundingBox {
        self.nodes[0].bb
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

    fn choose_leaf(&self, this: usize, bounding_box: &BoundingBox) -> usize {
        self.choose_leaf_rec(this, bounding_box)
    }

    fn find_rec(&self, this: usize, bounding_box: &BoundingBox) -> Option<&T> {
        println!(
            "nodes[{this}].intersects({}, {bounding_box}) => {}",
            self.nodes[this].bb,
            self.nodes[this].bb.intersects(bounding_box)
        );
        if self.nodes[this].bb.intersects(bounding_box) {
            match self.nodes[this].node {
                RTreeNode::Leaf(ref leaf) => Some(leaf),
                RTreeNode::Node(ref children) => children
                    .iter()
                    .find_map(|c| self.find_rec(*c, bounding_box)),
            }
        } else {
            None
        }
    }

    pub fn find(&self, bounding_box: &BoundingBox) -> Option<&T> {
        self.find_rec(0, bounding_box)
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
                        todo!()
                    }
                }
                self.adjust_tree(p, nodes_to_add);
            }
        }
    }

    pub fn insert_entry(&mut self, value: T, bounding_box: BoundingBox) {
        let chosen_leaf_i = self.choose_leaf(0, &bounding_box);

        let node_to_add = RTreeEntry {
            bb: bounding_box,
            parent: Some(chosen_leaf_i),
            node: RTreeNode::Leaf(value),
        };

        let idx = self.nodes.len();
        self.nodes.push(node_to_add);
        let chosen_leaf = &mut self.nodes[chosen_leaf_i];
        let RTreeEntry { node, bb, .. } = chosen_leaf;

        match node {
            RTreeNode::Leaf(_) => panic!("Adding to leaf!!"),
            RTreeNode::Node(children) => {
                if children.len() < M {
                    *bb = bb.get_union(&bounding_box);
                    children.push(idx);
                } else {
                    // TODO: Split the node more smartly
                    let new_child = RTreeEntry {
                        bb: *bb,
                        parent: Some(chosen_leaf_i),
                        node: RTreeNode::Node(std::mem::take(children)),
                    };
                    *bb = bb.get_union(&bounding_box);
                    children.push(idx);
                    children.push(idx + 1);
                    self.nodes.push(new_child);
                }
            }
        }
    }

    pub fn dot(&self, vertical: bool, f: &mut impl Write) -> std::io::Result<()> {
        writeln!(
            f,
            "digraph G {{\nrankdir=\"{}\";
            newrank=true;",
            if vertical { "TB" } else { "LR" }
        )?;
        for (i, node) in self.nodes.iter().enumerate() {
            let color = if matches!(node.node, RTreeNode::Node(_)) {
                "style=filled fillcolor=\"#ffff7f\""
            } else {
                "style=filled fillcolor=\"#7fff7f\""
            };
            writeln!(
                f,
                "a{} [label=\"#{} {}\" shape=rect {color}];",
                i, i, node.bb
            )?;
            if let RTreeNode::Node(children) = &node.node {
                for child in children {
                    writeln!(f, "a{} -> a{};", i, *child)?;
                }
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
