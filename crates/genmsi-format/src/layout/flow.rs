//! The flow of a model.
//!
//! The flow is the graph the blocks form: the links say which block follows
//! which, and the layout needs that flow to say which block goes below
//! which.
//!
//! The flow of a model is not the flow of each container: a link that crosses
//! the border of a container says where the flow goes, and the blocks on both
//! sides have to know it. So the flow is counted over the whole model, with
//! the borders of the containers spliced: the flow that enters a container
//! goes to the block its entry port names, and the flow that leaves it comes
//! from the block its exit port names.
//!
//! A **scope** —the root of the model, or the content of a container— has a
//! flow of its own: the links it owns, with the borders of the containers
//! nested in it spliced. A container nested in a scope is an atomic box, so
//! the splice stops at the container: the flow that enters it ends there, and
//! the flow that leaves it starts there. The ports of the scope are the ends
//! of that flow.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Block, GmdlModel};

/// The links of the flow: the blocks that follow each other.
///
/// The links that close a cycle are taken out: the flow of a model with a
/// cycle is still a flow, and the block the cycle comes back to is the start
/// of it. The picture draws the links as the model declares them (the app
/// draws them, not the layout), so the cycle is broken only for the layout.
pub fn flow_edges_for(model: &GmdlModel) -> Vec<(u32, u32)> {
    let blocks: BTreeMap<u32, &Block> = model.blocks.iter().map(|block| (block.id, block)).collect();
    let root = model.root().map(|block| block.id);
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
    without_cycles(&spliced_edges(model, &blocks, &children))
}

/// The flow of one scope: the links the scope owns.
///
/// A scope is the level the reader is looking at: the root of the model, or
/// the content of a container. Its flow is made of the links it owns, and
/// the borders of the containers nested in it are spliced: a container nested
/// in the scope is an atomic box, so the flow that enters it ends at the
/// container and the flow that leaves it starts there.
///
/// The ports of the scope are the ends of the flow. The direction of the link
/// says which end a port is, not the port id: a link that **starts** at the
/// scope is its entry port (the flow enters the scope there), and a link that
/// **ends** at the scope is its exit port (the flow leaves the scope there).
/// The editor gives a container the same id for its entry and its exit port,
/// so a container may use one id for both.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeFlow {
    /// The links between the blocks of the scope.
    pub edges: Vec<(u32, u32)>,
    /// The entry ports of the scope: the port, and the block the flow enters.
    pub entries: Vec<(u32, u32)>,
    /// The exit ports of the scope: the block the flow leaves from, and the
    /// port it leaves through.
    pub exits: Vec<(u32, u32)>,
}

/// The flow of one scope: the links the scope owns, with the borders of the
/// containers nested in it spliced.
///
/// The links that close a cycle are taken out, as in the flow of the whole
/// model: the flow of a scope is still a flow, and the block the cycle comes
/// back to is the start of it.
pub fn scope_flow(model: &GmdlModel, scope: u32) -> ScopeFlow {
    let Some(block) = model.block(scope) else {
        return ScopeFlow::default();
    };
    // The blocks the scope owns: the ones it composes.
    let children: BTreeSet<u32> = model
        .blocks
        .iter()
        .filter(|child| child.owner == scope && child.id != scope)
        .map(|child| child.id)
        .collect();
    let mut flow = ScopeFlow::default();
    for link in &block.links {
        let (from, to) = (link.from_module, link.to_module);
        if from == to {
            continue;
        }
        if from == scope {
            // The flow enters the scope through its entry port.
            if children.contains(&to) {
                flow.entries.push((link.from_ioid, to));
            }
            continue;
        }
        if to == scope {
            // The flow leaves the scope through its exit port.
            if children.contains(&from) {
                flow.exits.push((from, link.to_ioid));
            }
            continue;
        }
        // A link between two blocks of the scope. A container nested in the
        // scope is one of them, and it is an atomic box: the splice stops
        // there.
        if children.contains(&from) && children.contains(&to) {
            flow.edges.push((from, to));
        }
    }
    flow.edges = without_cycles(&flow.edges);
    flow.edges.sort_unstable();
    flow.edges.dedup();
    flow.entries.sort_unstable();
    flow.entries.dedup();
    flow.exits.sort_unstable();
    flow.exits.dedup();
    flow
}

/// The links between the blocks that simulate, with the borders of the
/// containers spliced.
fn spliced_edges(
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

/// The links that do not close a cycle.
fn without_cycles(edges: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut outgoing: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (from, to) in edges {
        outgoing.entry(*from).or_default().push(*to);
    }
    let mut inside: BTreeSet<u32> = BTreeSet::new();
    let mut done: BTreeSet<u32> = BTreeSet::new();
    let mut back: BTreeSet<(u32, u32)> = BTreeSet::new();
    for from in outgoing.keys() {
        if done.contains(from) {
            continue;
        }
        walk(*from, &outgoing, &mut inside, &mut done, &mut back);
    }
    edges
        .iter()
        .filter(|(from, to)| !back.contains(&(*from, *to)))
        .copied()
        .collect()
}

/// The walk over the flow, looking for the links that close a cycle.
fn walk(
    id: u32,
    outgoing: &BTreeMap<u32, Vec<u32>>,
    inside: &mut BTreeSet<u32>,
    done: &mut BTreeSet<u32>,
    back: &mut BTreeSet<(u32, u32)>,
) {
    if !inside.insert(id) {
        return;
    }
    for next in outgoing.get(&id).map(Vec::as_slice).unwrap_or(&[]) {
        if inside.contains(next) {
            // The link comes back to a block the walk is on: it closes a
            // cycle, and it is the one that is taken out.
            back.insert((id, *next));
        } else if !done.contains(next) {
            walk(*next, outgoing, inside, done, back);
        }
    }
    inside.remove(&id);
    done.insert(id);
}
