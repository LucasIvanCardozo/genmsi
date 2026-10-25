//! The flow of a model: how deep in the flow every block is.
//!
//! The flow of a model is not the flow of each container: a link that crosses
//! the border of a container says where the flow goes, and the blocks on both
//! sides have to know it. So the flow is computed over the whole model, with
//! the borders of the containers spliced: the flow that enters a container
//! goes to the block its entry port names, and the flow that leaves it comes
//! from the block its exit port names.
//!
//! The depth of a block (its **rank**) is the length of the longest path that
//! reaches it: a block with no incoming link is at the start of the flow
//! (rank 0), and a block that follows another one is one rank deeper. A link
//! that closes a cycle is taken out of the path: the flow of a model with a
//! cycle is still a flow, and the block the cycle comes back to is its
//! start.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Block, GmdlModel};

/// The flow of a model.
pub struct Flow {
    /// How deep in the flow every block is.
    ranks: BTreeMap<u32, usize>,
    /// The children of every container.
    children: BTreeMap<u32, Vec<u32>>,
    /// The links every container owns.
    links: BTreeMap<u32, Vec<(u32, u32)>>,
    /// The blocks every block is joined to.
    neighbours: BTreeMap<u32, Vec<u32>>,
    /// The blocks that simulate under every container.
    leaves: BTreeMap<u32, Vec<u32>>,
}

impl Flow {
    /// The flow of a model.
    pub fn of(model: &GmdlModel) -> Flow {
        let blocks: BTreeMap<u32, &Block> =
            model.blocks.iter().map(|block| (block.id, block)).collect();
        let root = model.root().map(|block| block.id);
        // The children of every container. The root is not one of them: it is
        // the model, not a block of it.
        let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for block in &model.blocks {
            if Some(block.id) == root {
                continue;
            }
            children.entry(block.owner).or_default().push(block.id);
        }
        for list in children.values_mut() {
            list.sort_unstable();
        }
        let edges = flow_edges(model, &blocks, &children);
        let mut ranks = rank_of(&blocks, &edges);
        // A container sits where its content begins: the frame is around the
        // flow, not before it.
        rank_containers(&blocks, &children, &mut ranks);
        // The links every container owns.
        let mut links: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
        // The blocks every block is joined to.
        let mut neighbours: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for block in &model.blocks {
            let owned = links.entry(block.id).or_default();
            for link in &block.links {
                if link.from_module != link.to_module {
                    owned.push((link.from_module, link.to_module));
                    neighbours
                        .entry(link.from_module)
                        .or_default()
                        .push(link.to_module);
                    neighbours
                        .entry(link.to_module)
                        .or_default()
                        .push(link.from_module);
                }
            }
            owned.sort_unstable();
            owned.dedup();
        }
        for list in neighbours.values_mut() {
            list.sort_unstable();
            list.dedup();
        }
        // The blocks that simulate under every container.
        let mut leaves: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for block in &model.blocks {
            if block.is_container() {
                leaves.insert(block.id, leaves_under(block.id, &children, &blocks));
            }
        }
        Flow {
            ranks,
            children,
            links,
            neighbours,
            leaves,
        }
    }

    /// Whether no link reaches a block.
    ///
    /// A block that no link reaches is not part of the flow: it does not say
    /// where anything comes from or where it goes. A container that holds
    /// blocks with links is part of the flow even when the links do not
    /// touch the container itself.
    pub fn is_isolated(&self, id: u32) -> bool {
        if !self
            .neighbours
            .get(&id)
            .map(|list| list.is_empty())
            .unwrap_or(true)
        {
            return false;
        }
        self.leaves
            .get(&id)
            .map(|leaves| {
                leaves.iter().all(|leaf| {
                    self.neighbours
                        .get(leaf)
                        .map(|list| list.is_empty())
                        .unwrap_or(true)
                })
            })
            .unwrap_or(true)
    }

    /// How deep in the flow a block is.
    pub fn rank(&self, id: u32) -> usize {
        self.ranks.get(&id).copied().unwrap_or(0)
    }

    /// The children of a container, by id.
    pub fn children_of(&self, container: u32) -> Vec<u32> {
        self.children.get(&container).cloned().unwrap_or_default()
    }

    /// The layers of a container: its children grouped by the depth they have
    /// in the flow.
    ///
    /// The depth is the depth of the whole model, so the children of a
    /// container may be far apart; the layers are the groups, in the order of
    /// the flow, without the holes between them.
    pub fn layers_of(&self, container: u32) -> Vec<Vec<u32>> {
        let mut by_rank: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
        for child in self.children_of(container) {
            by_rank.entry(self.rank(child)).or_default().push(child);
        }
        by_rank.into_values().collect()
    }

    /// The links between the children of a container.
    ///
    /// The links that leave the container (to a block of another container)
    /// are not here: they do not order the blocks of this one.
    pub fn inner_edges(&self, container: u32) -> Vec<(u32, u32)> {
        let children = self.children_of(container);
        let owned = self.links.get(&container).cloned().unwrap_or_default();
        owned
            .into_iter()
            .filter(|(from, to)| {
                children.contains(from) && children.contains(to) && from != to
            })
            .collect()
    }
}

/// The flow of the model, over the blocks that simulate.
///
/// A link between two blocks is an edge of the flow. A link that touches the
/// border of a container is not: the container is a frame, and the flow goes
/// through it. The flow that enters a container comes from the block its
/// entry port names, and the flow that leaves it goes to the block its exit
/// port names.
fn flow_edges(
    model: &GmdlModel,
    blocks: &BTreeMap<u32, &Block>,
    children: &BTreeMap<u32, Vec<u32>>,
) -> Vec<(u32, u32)> {
    let is_leaf = |id: u32| blocks.get(&id).is_some_and(|block| !block.is_container());
    let owns = |container: u32, child: u32| {
        children
            .get(&container)
            .is_some_and(|list| list.contains(&child))
    };
    // The blocks the flow enters a container through, and the blocks it
    // leaves it through: the children its ports name.
    let mut entries: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    let mut exits: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for block in &model.blocks {
        if !block.is_container() {
            continue;
        }
        for link in &block.links {
            if link.from_module == block.id && is_leaf(link.to_module) {
                entries.entry(block.id).or_default().push(link.to_module);
            }
            if link.to_module == block.id && is_leaf(link.from_module) {
                exits.entry(block.id).or_default().push(link.from_module);
            }
        }
    }
    // The links that cross the border of a container: the flow arrives at the
    // container, or leaves it.
    let mut arrivals: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    let mut departures: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for block in &model.blocks {
        for link in &block.links {
            if link.from_module == link.to_module {
                continue;
            }
            let from = link.from_module;
            let to = link.to_module;
            if blocks.get(&to).is_some_and(|block| block.is_container()) && !owns(to, from) {
                arrivals.entry(to).or_default().push(from);
            }
            if blocks.get(&from).is_some_and(|block| block.is_container()) && !owns(from, to) {
                departures.entry(from).or_default().push(to);
            }
        }
    }
    let mut edges: Vec<(u32, u32)> = Vec::new();
    for block in &model.blocks {
        for link in &block.links {
            let from = link.from_module;
            let to = link.to_module;
            if from == to {
                continue;
            }
            match (is_leaf(from), is_leaf(to)) {
                // The flow goes from one block to another.
                (true, true) => edges.push((from, to)),
                // The flow leaves a container at this block: it goes on to
                // wherever the container's exit port leads.
                (true, false) => {
                    for target in departures.get(&to).into_iter().flatten() {
                        for source in leaves_of(*target, &entries, blocks) {
                            edges.push((from, source));
                        }
                    }
                }
                // The flow enters a container at this block: it comes from
                // wherever the container's entry port comes from.
                (false, true) => {
                    for source in arrivals.get(&from).into_iter().flatten() {
                        for target in leaves_of(*source, &exits, blocks) {
                            edges.push((target, to));
                        }
                    }
                }
                // A link between two containers: the flow crosses both, and
                // the links inside them say where it goes.
                (false, false) => {}
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    edges
}

/// The blocks the flow reaches through a block: the leaves under it that
/// the flow enters (or leaves) it at.
///
/// A container's ports name the blocks inside it; when the port names another
/// container, the answer is the blocks under that one.
fn leaves_of(
    id: u32,
    ports: &BTreeMap<u32, Vec<u32>>,
    blocks: &BTreeMap<u32, &Block>,
) -> Vec<u32> {
    let mut seen = BTreeSet::new();
    let mut leaves = Vec::new();
    leaves_of_walk(id, ports, blocks, &mut seen, &mut leaves);
    leaves.sort_unstable();
    leaves.dedup();
    leaves
}

/// The walk that collects the blocks the flow reaches through a block.
fn leaves_of_walk(
    id: u32,
    ports: &BTreeMap<u32, Vec<u32>>,
    blocks: &BTreeMap<u32, &Block>,
    seen: &mut BTreeSet<u32>,
    leaves: &mut Vec<u32>,
) {
    let Some(block) = blocks.get(&id) else {
        return;
    };
    if !seen.insert(id) {
        // The ports name each other: the walk stops there.
        return;
    }
    if !block.is_container() {
        leaves.push(id);
        return;
    }
    for child in ports.get(&id).into_iter().flatten() {
        leaves_of_walk(*child, ports, blocks, seen, leaves);
    }
}

/// How deep in the flow every block is.
///
/// The depth is the length of the longest path that reaches the block: the
/// links that close a cycle are taken out first, so the depth is always
/// defined and always the same.
fn rank_of(blocks: &BTreeMap<u32, &Block>, edges: &[(u32, u32)]) -> BTreeMap<u32, usize> {
    let mut outgoing: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (from, to) in edges {
        outgoing.entry(*from).or_default().push(*to);
    }
    // The walk that finds the links that close a cycle, and the order the
    // blocks finish in.
    let mut state: BTreeMap<u32, Visit> = BTreeMap::new();
    let mut back: BTreeSet<(u32, u32)> = BTreeSet::new();
    let mut finished: Vec<u32> = Vec::new();
    for id in blocks.keys() {
        if state.contains_key(id) {
            continue;
        }
        visit(*id, &outgoing, &mut state, &mut back, &mut finished);
    }
    // The longest path over the links that do not close a cycle. The blocks
    // are taken in the order the walk finished them, backwards: a block is
    // always taken before the blocks that follow it.
    let mut ranks: BTreeMap<u32, usize> = blocks.keys().map(|id| (*id, 0)).collect();
    for id in finished.iter().rev() {
        let rank = ranks[id];
        for next in outgoing.get(id).map(Vec::as_slice).unwrap_or(&[]) {
            if back.contains(&(*id, *next)) {
                continue;
            }
            let entry = ranks.entry(*next).or_insert(0);
            if *entry < rank + 1 {
                *entry = rank + 1;
            }
        }
    }
    ranks
}

/// How far the walk has gone with a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Visit {
    /// The walk is on the block: a link to it closes a cycle.
    Inside,
    /// The walk left the block.
    Done,
}

/// The walk over the flow, looking for the links that close a cycle.
fn visit(
    id: u32,
    outgoing: &BTreeMap<u32, Vec<u32>>,
    state: &mut BTreeMap<u32, Visit>,
    back: &mut BTreeSet<(u32, u32)>,
    finished: &mut Vec<u32>,
) {
    state.insert(id, Visit::Inside);
    for next in outgoing.get(&id).map(Vec::as_slice).unwrap_or(&[]) {
        match state.get(next) {
            Some(Visit::Inside) => {
                // The link comes back to a block the walk is on: it closes a
                // cycle, and it is the one that is taken out.
                back.insert((id, *next));
            }
            Some(Visit::Done) => {}
            None => visit(*next, outgoing, state, back, finished),
        }
    }
    state.insert(id, Visit::Done);
    finished.push(id);
}

/// A container sits where its content begins.
///
/// The frame of a container is around the flow that goes through it: the
/// container is placed in the layer of the first block of its content, so the
/// flow that enters the container and the flow that leaves it are on the
/// sides of the frame.
fn rank_containers(
    blocks: &BTreeMap<u32, &Block>,
    children: &BTreeMap<u32, Vec<u32>>,
    ranks: &mut BTreeMap<u32, usize>,
) {
    // The containers, from the innermost to the outermost.
    let mut containers: Vec<u32> = blocks
        .values()
        .filter(|block| block.is_container())
        .map(|block| block.id)
        .collect();
    containers.sort_by_key(|id| {
        let mut depth = 0;
        let mut current = *id;
        while let Some(block) = blocks.get(&current) {
            // The model root owns itself: the walk stops there.
            if block.owner == current {
                break;
            }
            depth += 1;
            current = block.owner;
        }
        std::cmp::Reverse(depth)
    });
    for container in containers {
        let leaves = leaves_under(container, children, blocks);
        let Some(first) = leaves
            .iter()
            .map(|leaf| ranks.get(leaf).copied().unwrap_or(0))
            .min()
        else {
            continue;
        };
        ranks.insert(container, first);
    }
}

/// The blocks that simulate under a container.
fn leaves_under(
    container: u32,
    children: &BTreeMap<u32, Vec<u32>>,
    blocks: &BTreeMap<u32, &Block>,
) -> Vec<u32> {
    let mut seen = BTreeSet::new();
    let mut leaves = Vec::new();
    leaves_under_walk(container, children, blocks, &mut seen, &mut leaves);
    leaves.sort_unstable();
    leaves.dedup();
    leaves
}

/// The walk that collects the blocks that simulate under a container.
fn leaves_under_walk(
    container: u32,
    children: &BTreeMap<u32, Vec<u32>>,
    blocks: &BTreeMap<u32, &Block>,
    seen: &mut BTreeSet<u32>,
    leaves: &mut Vec<u32>,
) {
    if !seen.insert(container) {
        return;
    }
    for child in children.get(&container).into_iter().flatten() {
        match blocks.get(child) {
            Some(block) if block.is_container() => {
                leaves_under_walk(*child, children, blocks, seen, leaves);
            }
            Some(_) => leaves.push(*child),
            None => {}
        }
    }
}
