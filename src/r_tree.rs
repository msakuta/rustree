use crate::{bounding_box::BoundingBox, point::Point};
use std::{fmt::Debug, io::Write};

const M: usize = 4;

#[derive(Debug)]
pub enum RTreeNode<T> {
    Node(Vec<usize>),
    Leaf(T),
}

#[derive(Debug)]
pub struct RTreeEntry<T> {
    bb: BoundingBox,
    parent: Option<usize>,
    node: RTreeNode<T>,
}

impl<T> RTreeEntry<T> {
    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bb
    }

    pub fn node(&self) -> &RTreeNode<T> {
        &self.node
    }
}

#[derive(Debug)]
pub struct RTree<T> {
    nodes: Vec<RTreeEntry<T>>,
    max_depth: usize,
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
            max_depth: 1,
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.nodes[0].bb
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    fn choose_leaf_rec(
        &self,
        node: usize,
        bounding_box: &BoundingBox,
        level: usize,
    ) -> (usize, usize) {
        if matches!(self.nodes[node].node, RTreeNode::Leaf(_)) {
            return (node, level);
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
            self.choose_leaf_rec(min, bounding_box, level + 1)
        } else {
            (node, level)
        }
    }

    /// Returnst (id, level)
    fn choose_leaf(&self, this: usize, bounding_box: &BoundingBox) -> (usize, usize) {
        self.choose_leaf_rec(this, bounding_box, 0)
    }

    fn find_rec(&self, this: usize, bounding_box: &BoundingBox) -> Option<&T> {
        // println!(
        //     "nodes[{this}].intersects({}, {bounding_box}) => {}",
        //     self.nodes[this].bb,
        //     self.nodes[this].bb.intersects(bounding_box)
        // );
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

    /// Finds an entry from this RTree that intersects with the given bounding box.
    /// It returns only the first found item.
    pub fn find(&self, bounding_box: &BoundingBox) -> Option<&T> {
        self.find_rec(0, bounding_box)
    }

    pub fn find_multi(&self, bounding_box: &BoundingBox) -> impl Iterator<Item = &T> {
        struct Finder<'a, T> {
            this: &'a RTree<T>,
            bb: BoundingBox,
            /// (Node id, child index)
            stack: Vec<(usize, usize)>,
        }

        impl<'a, T> Finder<'a, T> {
            fn new(this: &'a RTree<T>, bb: BoundingBox) -> Self {
                Self {
                    this,
                    bb,
                    stack: vec![(0, 0)],
                }
            }

            fn find_multi(&mut self) -> Option<&'a T> {
                loop {
                    let Some((node, child)) = self.stack.pop() else {
                        return None;
                    };
                    if self.this.nodes[node].bb.intersects(&self.bb) {
                        match self.this.nodes[node].node {
                            RTreeNode::Leaf(ref leaf) => return Some(leaf),
                            RTreeNode::Node(ref children) => {
                                if let Some(child_id) = children.get(child) {
                                    self.stack.push((node, child + 1));
                                    self.stack.push((*child_id, 0));
                                }
                            }
                        };
                    }
                }
            }
        }

        impl<'a, T> Iterator for Finder<'a, T> {
            type Item = &'a T;
            fn next(&mut self) -> Option<Self::Item> {
                self.find_multi()
            }
        }

        Finder::new(self, *bounding_box)
    }
}

pub struct WalkCallbackPayload<'a, T>(pub usize, pub usize, pub &'a RTreeEntry<T>);

impl<T: Debug> RTree<T> {
    fn walk_rec(&self, node: usize, level: usize, f: &mut impl FnMut(&WalkCallbackPayload<T>)) {
        f(&WalkCallbackPayload(node, level, &self.nodes[node]));
        match self.nodes[node].node {
            RTreeNode::Leaf(_) => {}
            RTreeNode::Node(ref children) => {
                for child in children {
                    self.walk_rec(*child, level + 1, f);
                }
            }
        }
    }

    /// Passes (index, level, bounding_box) to the callback
    pub fn walk(&self, f: &mut impl FnMut(&WalkCallbackPayload<T>)) {
        self.walk_rec(0, 0, f);
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

    fn append_entry(&mut self, node: RTreeEntry<T>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    fn update_bbox(&mut self, idx: usize, bounding_box: BoundingBox) {
        let mut parent = Some(idx);
        let mut bb = bounding_box;
        while let Some(p) = parent {
            let parent_node = &mut self.nodes[p];
            bb = parent_node.bb.get_union(&bb);
            parent_node.bb = bb;
            parent = parent_node.parent;
        }
    }

    /// Update cache of max depth from a subtree. You can start from the middle of a tree. You need to give the correct starting level.
    ///
    /// This crate does not cache the depth of a node inside each node payload to save space and also remove maintenance cost
    /// when the node is split into children, so we need to update the cached max depth every time we may have changed the tree.
    fn update_max_depth(max_depth: &mut usize, nodes: &[RTreeEntry<T>], id: usize, level: usize) {
        *max_depth = (*max_depth).max(level);
        match &nodes[id].node {
            RTreeNode::Node(node) => {
                for child in node {
                    Self::update_max_depth(max_depth, nodes, *child, level + 1);
                }
            }
            _ => {}
        }
    }

    /// Insert an entry object of type T in this RTree with an associated bounding box.
    ///
    /// There is no built-in mechanism to ensure `bounding_box` is actually bounding `value`.
    /// It is the caller's responsibility to hold that precondition.
    pub fn insert_entry(&mut self, value: T, bounding_box: BoundingBox) {
        let (chosen_leaf_i, level) = self.choose_leaf(0, &bounding_box);

        let node_to_add = RTreeEntry {
            bb: bounding_box,
            parent: Some(chosen_leaf_i),
            node: RTreeNode::Leaf(value),
        };

        let idx = self.nodes.len();
        self.nodes.push(node_to_add);
        let chosen_leaf = &mut self.nodes[chosen_leaf_i];
        let node = &mut chosen_leaf.node;

        let children = match node {
            RTreeNode::Leaf(_) => panic!("Adding to leaf!!"),
            RTreeNode::Node(children) => children,
        };

        children.push(idx);
        if children.len() <= M {
            self.update_bbox(chosen_leaf_i, bounding_box);
            Self::update_max_depth(&mut self.max_depth, &self.nodes, chosen_leaf_i, level);
            return;
        }

        // Our goal here is to split the collection of nodes into 2 groups that minimizes each bounding box.
        // I don't know if there is any fast way to find the solution, so I will just scan all possible combinations.
        let next_children = std::mem::take(children);
        let all_combi = 1 << next_children.len();
        // combi represents the combinations of node sets in a bitfield. If a bit is on, it is right side.
        // We skip 0 and all_combi-1 because it means all nodes are on one side.
        // Technically, it counts the same combination twice if you flip all bits.
        let combi = (1..all_combi - 1)
            .map(|combi| {
                let left_bb = next_children
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| combi & (1 << *i) == 0)
                    .map(|(_, id)| self.nodes[*id].bb)
                    .reduce(|a, b| a.get_union(&b));
                let right_bb = next_children
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| combi & (1 << *i) != 0)
                    .map(|(_, id)| self.nodes[*id].bb)
                    .reduce(|a, b| a.get_union(&b));
                (
                    combi,
                    left_bb.map(|bb| bb.get_area()).unwrap_or(0.)
                        + right_bb.map(|bb| bb.get_area()).unwrap_or(0.),
                )
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        let Some((combi, _)) = combi else {
            panic!("No combination is found")
        };
        let mut build_child = |f| {
            let children: Vec<_> = next_children
                .iter()
                .enumerate()
                .filter(|(i, _)| f ^ (combi & (1 << *i) == 0))
                .map(|(_, id)| *id)
                .collect();
            for child in &children {
                self.nodes[*child].parent = Some(self.nodes.len());
            }
            let bb = children
                .iter()
                .map(|id| self.nodes[*id].bb)
                .reduce(|a, b| a.get_union(&b))
                .unwrap();
            self.append_entry(RTreeEntry {
                bb,
                parent: Some(chosen_leaf_i),
                node: RTreeNode::Node(children),
            })
        };
        let left_child = build_child(true);
        let right_child = build_child(false);
        self.update_bbox(chosen_leaf_i, bounding_box);
        let node = &mut self.nodes[chosen_leaf_i].node;
        *node = RTreeNode::Node(vec![left_child, right_child]);
        Self::update_max_depth(&mut self.max_depth, &self.nodes, chosen_leaf_i, level);
    }

    /// Outputs a dot file for graphviz visualization.
    ///
    /// You would use it for debugging, but for actual data real space visualization should be better.
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
