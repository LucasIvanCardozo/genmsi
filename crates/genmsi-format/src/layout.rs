//! The composition of the model's canvas.
//!
//! Where a block sits on the canvas and how big it is is the engine's
//! business: a model that does not carry that geometry is still a model, and
//! the engine composes it so that the editor can draw it. The composition is
//! deterministic (the same model always produces the same positions), it never
//! touches the semantics (the geometry does not enter the simulation) and it
//! leaves a model that is already composed exactly as its author left it.
//!
//! The composition is **dagre** (the Rust port of dagre.js), the Sugiyama
//! pipeline: the flow of the model is cut into layers (*network simplex*),
//! the layers are ordered to cross the fewest links (barycenter) and the
//! blocks are placed (Brandes–Köpf). The direction is **top to bottom**
//! (`TB`): the ports of a block are its input at the top and its output at
//! the bottom, so the flow a reader follows goes down.
//!
//! The containers are the **compound** nodes of the graph: dagre computes
//! their frame from the blocks they hold, and the flow that crosses a
//! container's border is spliced before the layout (`layout::flow`), so a link
//! that goes from a block of one container to a block of another says where
//! the flow goes.
//!
//! There are **two** compositions, and both are of the engine. `relayout`
//! composes the whole tree (the one the CLI and the MCP ask for, with its
//! committed baseline), and `relayout_scope` composes **one scope**: the level
//! the reader is looking at, the root of the model or the content of a
//! container. A scope is a canvas of its own: the blocks of a container are
//! composed among themselves, a container nested in them is an atomic box, and
//! nothing outside the scope moves.

pub mod flow;

use dagre::graph::{Graph, GraphOptions};
use dagre::{EdgeLabel, LayoutOptions, NodeLabel, RankDir, Ranker};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::edit::{PortPosition, Position};
use crate::model::{GmdlModel, Layout};

/// Width of a block that has no geometry of its own.
pub const DEFAULT_WIDTH: i32 = 120;
/// Height of a block that has no geometry of its own.
pub const DEFAULT_HEIGHT: i32 = 48;
/// Margin between a container and the blocks it holds.
pub const PAD: i32 = 24;
/// Vertical gap between two blocks of the same layer.
pub const GAP_Y: i32 = 30;
/// Horizontal gap between two blocks side by side.
pub const GAP_X: i32 = 30;
/// Vertical gap between two layers of the flow.
pub const RANK_SEP: i32 = 60;
/// The smallest vertical gap between two layers: a link the reader cannot
/// see is worse than a picture that is not square.
pub const MIN_RANK_SEP: i32 = 20;
/// The smallest horizontal gap between two blocks of a layer.
pub const MIN_NODE_SEP: i32 = 10;

/// The positions that compose the model, one per block except the root.
///
/// The positions are sorted by block id, so the same model always produces the
/// same list in the same order.
pub fn relayout(model: &GmdlModel) -> Vec<Position> {
    // The network simplex cuts the rows and orders them to cross the fewest
    // links; the tight tree cuts the same rows (they are the depth of the
    // flow) but crosses 76 links more over the corpus.
    //
    // The port's network simplex also panics on some compound graphs (the
    // cycles of the container borders): the composition must never panic, so
    // the answer comes from the tight tree when the simplex fails. Both
    // answers are the same every time, so the composition stays
    // deterministic.
    let ranker = Ranker::NetworkSimplex;
    let Some(root) = model.root().map(|block| block.id) else {
        return Vec::new();
    };
    let first = match compose(model, root, ranker, GAP_X, RANK_SEP) {
        Some(positions) => positions,
        None => compose(model, root, Ranker::TightTree, GAP_X, RANK_SEP).unwrap_or_default(),
    };
    // The picture is as square as the model allows: the gaps are the lever.
    // A picture that is taller than it is wide comes together vertically, and
    // a picture that is wider than it is tall comes together horizontally.
    let Some((nodesep, ranksep)) = square_gaps(&first) else {
        return first;
    };
    if (nodesep, ranksep) == (GAP_X, RANK_SEP) {
        return first;
    }
    let second = match compose(model, root, ranker, nodesep, ranksep) {
        Some(positions) => positions,
        None => return first,
    };
    // The picture that is squarer wins; if none of them is, the first one
    // stays: the composition is the same every time.
    if largest_side(&second) < largest_side(&first) {
        second
    } else {
        first
    }
}

/// The width of a door of a container.
///
/// It is the box the app draws a door in (`DOOR_BOX` in
/// `crates/genmsi-app/src/graph/frames.ts`): the composition reserves the
/// room the reader is going to see.
pub const DOOR_WIDTH: i32 = 140;
/// The height of a door of a container.
///
/// It is the box the app draws a door in: see [`DOOR_WIDTH`].
pub const DOOR_HEIGHT: i32 = 44;
/// Where a scope with no blocks of its own starts its picture.
const EMPTY_SPOT: i32 = 80;
/// The air between the lowest block of a scope and the free spot below it.
///
/// It is the same air the app leaves: the door that the engine places and
/// the door the app draws in the meantime are in the same place.
const FREE_SPOT_AIR: i32 = 40;

/// The composition of one scope: the place of its blocks and of its doors.
///
/// A scope is a canvas of its own, and everything it holds is placed in it:
/// the blocks it owns and the doors that open and close its flow.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScopeComposition {
    /// The place of every block the scope owns.
    pub positions: Vec<Position>,
    /// The place of every door of the scope.
    pub ports: Vec<PortPosition>,
}

/// The positions that compose **one scope** of the model.
///
/// A scope is the level the reader is looking at: the root of the model, or
/// the content of a container. The reader sees one level at a time, so the
/// composition of a scope moves the blocks of that scope and nothing else: a
/// "rearrange" inside a container does not move the blocks of the root.
///
/// The rules of the composition:
///
/// - the blocks the scope composes are the ones it owns (`block.owner ==
///   scope`);
/// - a container **nested** in the scope is an *atomic box*: a node of the
///   scope's graph with the size the model gives it (the one the composition
///   of its own scope gave it), whose content is neither looked at nor moved;
/// - the links the scope composes are the ones it **owns**, with the borders
///   of the nested containers spliced: a link that enters a nested container
///   ends at the container (which is an atomic box) and a link that leaves it
///   starts there;
/// - the **ports of the scope** are the ends of the flow: they take part in
///   the composition as the ends of the flow, so the block the flow enters
///   through is placed first and the block it leaves through is placed last;
/// - what is written back is the scope's own: the positions of its children.
///   **Not** its frame: the container is a box of its own, and what it holds
///   does not change it — its size and its place are the ones the model
///   gives it, and nothing here touches them.
///
/// A scope is a canvas of its own: the positions of its children live in the
/// canvas of the scope, which is where the author put them. The composition
/// starts there, and never before it.
///
/// Nothing outside the scope is touched: a block that is not a child of the
/// scope, and is not the scope itself, keeps the position it has.
///
/// The composition is deterministic: the same model and the same scope always
/// produce the same positions, in the same order (sorted by block id).
pub fn relayout_scope(model: &GmdlModel, scope: u32) -> ScopeComposition {
    // A scope that is not in the model has no blocks to compose.
    if model.block(scope).is_none() {
        return ScopeComposition { positions: Vec::new(), ports: Vec::new() };
    }
    // The blocks the scope owns: the ones it composes.
    let mut children: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.owner == scope && block.id != scope)
        .map(|block| block.id)
        .collect();
    children.sort_unstable();
    // A scope with no blocks of its own has no picture of blocks, but it
    // can still have doors: the composition places them.
    if children.is_empty() && model.block(scope).is_some_and(|block| block.inputs.is_empty() && block.outputs.is_empty()) {
        return ScopeComposition { positions: Vec::new(), ports: Vec::new() };
    }
    // The same ranker as the whole model, with the same fallback: the
    // composition must never panic.
    let ranker = Ranker::NetworkSimplex;
    match guarded(|| compose_scope(model, scope, &children, ranker, GAP_X, RANK_SEP)) {
        Some(positions) => positions,
        None => {
            guarded(|| compose_scope(model, scope, &children, Ranker::TightTree, GAP_X, RANK_SEP))
                .unwrap_or_default()
        }
    }
}

/// The largest side of a picture: the one the reader feels.
fn largest_side(positions: &[Position]) -> i32 {
    let Some((left, top, right, bottom)) = box_of(positions.iter()) else {
        return 0;
    };
    (right - left).max(bottom - top)
}

/// The gaps that leave the picture as square as possible.
///
/// The picture is tall when the flow has many layers, and wide when a layer
/// holds many blocks. The gaps are what make it bigger: the layers come
/// closer when the picture is too tall, and the blocks of a layer come closer
/// when it is too wide.
///
/// The gaps never go below the minimum: a link the reader cannot see is worse
/// than a picture that is not square.
fn square_gaps(positions: &[Position]) -> Option<(i32, i32)> {
    let (left, top, right, bottom) = box_of(positions.iter())?;
    let (width, height) = (right - left, bottom - top);
    if width <= 0 || height <= 0 {
        return None;
    }
    // The layers of the picture, and the blocks of the widest one.
    let mut layers: BTreeSet<i32> = BTreeSet::new();
    for position in positions {
        layers.insert(position.y);
    }
    let layers = layers.len().max(1) as i32;
    if height > width {
        // The layers come closer: every layer is `RANK_SEP` taller than it
        // needs to be.
        let ranksep = RANK_SEP - (height - width) / layers;
        if ranksep < MIN_RANK_SEP {
            return None;
        }
        return Some((GAP_X, ranksep));
    }
    if width > height {
        // The blocks of a layer come closer: the widest layer holds the most
        // blocks.
        let mut per_layer: BTreeMap<i32, usize> = BTreeMap::new();
        for position in positions {
            *per_layer.entry(position.y).or_default() += 1;
        }
        let widest = per_layer.values().copied().max().unwrap_or(1) as i32;
        let nodesep = GAP_X - (width - height) / widest;
        if nodesep < MIN_NODE_SEP {
            return None;
        }
        return Some((nodesep, RANK_SEP));
    }
    None
}

/// The composition of a model with a ranker, when the ranker can do it.
///
/// `None` means the ranker failed: the port's network simplex panics on some
/// compound graphs, and the panic is caught so that the composition falls
/// back to the tight tree.
fn compose(
    model: &GmdlModel,
    root: u32,
    ranker: Ranker,
    nodesep: i32,
    ranksep: i32,
) -> Option<Vec<Position>> {
    guarded(|| compose_with(model, root, ranker, nodesep, ranksep))
}

/// The composition, when the ranker does not panic.
///
/// The port's network simplex panics on some compound graphs (the cycles of
/// the container borders): the composition must never panic, so the answer
/// comes from the tight tree when the simplex fails. Both answers are the
/// same every time, so the composition stays deterministic.
fn guarded<F, T>(compose: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    let hook = std::panic::take_hook();
    if std::env::var("DAGRE_VERBOSE").is_err() {
        std::panic::set_hook(Box::new(|_| {}));
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(compose));
    std::panic::set_hook(hook);
    result.ok()
}

/// The composition of a model: dagre's Sugiyama pipeline, top to bottom.
///
/// The graph dagre composes has the blocks that **simulate** and the flow
/// between them, with the borders of the containers spliced: the flow that
/// enters a container goes to the block its entry port names, and the flow
/// that leaves it comes from the block its exit port names.
///
/// The containers are **not** part of that graph: dagre's compound graphs
/// leave blocks without a place when the model has a cycle, and the frames of
/// the containers are computed here, around the blocks they hold.
fn compose_with(
    model: &GmdlModel,
    root: u32,
    ranker: Ranker,
    nodesep: i32,
    ranksep: i32,
) -> Vec<Position> {
    // The margin of the frames: it is never wider than the gap between two
    // layers, or the frames of two containers that meet would overlap.
    let margin = PAD.min(ranksep / 2);
    // The flow of the model: the links between the blocks that simulate,
    // with the borders of the containers spliced.
    let edges = flow::flow_edges_for(model);
    // The containers that hold blocks: their frame is drawn around them, so
    // dagre gives them no size of their own. A container that holds nothing
    // is a block like any other: the composition places it.
    let holders: HashSet<u32> = model
        .blocks
        .iter()
        .filter(|block| block.id != root && block.owner != block.id)
        .map(|block| block.owner)
        .collect();
    // The blocks that are drawn in a grid beside the flow: the ones no link
    // reaches and that live in the root of the model.
    let grid = grid_blocks(model, root, &edges, &holders);
    let in_grid: HashSet<u32> = grid.iter().copied().collect();
    let mut graph = Graph::with_options(GraphOptions {
        directed: true,
        multigraph: true,
        compound: true,
    });
    for block in &model.blocks {
        if block.id == root {
            continue;
        }
        if holders.contains(&block.id) {
            // A container that holds blocks has no size of its own: its
            // frame is computed around the blocks it holds.
            graph.set_node(block.id.to_string(), None);
            continue;
        }
        // The blocks that are drawn in the grid take no room in the flow:
        // they are not drawn there. They stay in the graph all the same: the
        // order the crate gives the flow depends on them, and the flow
        // crosses fewer links when they are there.
        let (width, height) = if in_grid.contains(&block.id) {
            (0, 0)
        } else {
            size_of(model, block.id)
        };
        let label = NodeLabel {
            width: f64::from(width),
            height: f64::from(height),
            ..NodeLabel::default()
        };
        graph.set_node(block.id.to_string(), Some(label));
    }
    // The containers are the parents of the blocks they hold. The root of
    // the model is not one of them: it is not drawn, so the blocks it holds
    // are at the top level of the graph.
    for block in &model.blocks {
        if block.id == root || block.owner == block.id {
            continue;
        }
        graph.set_parent(&block.id.to_string(), Some(&block.owner.to_string()));
    }
    // The flow of the model: the links between the blocks that simulate, with
    // the borders of the containers spliced, and the links that carry the
    // flow into each container.
    for (from, to) in &edges {
        graph.set_edge(
            from.to_string(),
            to.to_string(),
            Some(EdgeLabel::default()),
            None,
        );
    }
    let options = LayoutOptions {
        // The flow goes down: the input of a block is at its top and its
        // output at its bottom.
        rankdir: RankDir::TB,
        ranker,
        // The gap between two blocks of the same layer, and the gap between
        // two layers of the flow. They are what makes the picture big: the
        // flow goes down, so the picture is tall and the gap between the
        // layers is what makes it taller.
        nodesep: f64::from(nodesep),
        ranksep: f64::from(ranksep),
        edgesep: 20.0,
        ..LayoutOptions::default()
    };
    dagre::layout(&mut graph, Some(options));

    // dagre places a block by its middle; the file carries its corner.
    let mut positions: Vec<Position> = Vec::new();
    for block in &model.blocks {
        if block.id == root {
            continue;
        }
        if holders.contains(&block.id) {
            // The frame of a container is computed around the blocks it
            // holds, not read from dagre.
            continue;
        }
        if in_grid.contains(&block.id) {
            // The blocks that are drawn in the grid are placed in the grid,
            // not where dagre puts them.
            continue;
        }
        let Some(node) = graph.node(&block.id.to_string()) else {
            continue;
        };
        let (Some(x), Some(y)) = (node.x, node.y) else {
            continue;
        };
        let width = node.width.round() as i32;
        let height = node.height.round() as i32;
        positions.push(Position {
            id: block.id,
            x: (x - f64::from(width) / 2.0).round() as i32,
            y: (y - f64::from(height) / 2.0).round() as i32,
            width: Some(width),
            height: Some(height),
        });
    }
    // The containers: the frame of each one is the box of the blocks it
    // holds, with the margin that separates them. The frames are computed
    // from the innermost out.
    place_frames(model, root, &mut positions, margin);
    // The components of the flow start together at the top: the sources of
    // the flow are the first row of the picture.
    align_components(model, root, &mut positions, &edges);
    // The components that share the canvas move apart: the picture grows
    // only where two chains of the flow meet nowhere and overlap.
    separate_components(model, root, &mut positions, &edges, margin);
    // The blocks that are drawn in the grid get a place in it: the grid is
    // laid out once the picture of the flow is known.
    for id in &grid {
        let (width, height) = size_of(model, *id);
        positions.push(Position {
            id: *id,
            x: 0,
            y: 0,
            width: Some(width),
            height: Some(height),
        });
    }
    // The blocks that are drawn in the grid go in a grid beside the flow, in
    // the shape that leaves the picture as square as possible.
    place_free(&mut positions, model, &grid, nodesep);
    // The containers: the frame of each one is the box of the blocks it
    // holds, with the margin that separates them. The frames are computed
    // from the innermost out, and they follow the blocks they hold.
    place_frames(model, root, &mut positions, margin);
    // The blocks nobody placed get a place: a block without one is a block
    // the reader cannot see. The port's tight tree leaves the blocks of a
    // cycle without a place, and they go where their container is.
    let missing: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.id != root)
        .filter(|block| !positions.iter().any(|position| position.id == block.id))
        .map(|block| block.id)
        .collect();
    if !missing.is_empty() {
        let frames = frames_of(model, root, &positions, margin);
        let mut stacked: HashMap<u32, i32> = HashMap::new();
        for block in missing {
            let owner = model.block(block).map(|block| block.owner).unwrap_or(root);
            let (x, y, _, _) = match frames.get(&owner) {
                Some((x, y, width, height)) => (*x, *y, *width, *height),
                None => (0, 0, DEFAULT_WIDTH, DEFAULT_HEIGHT),
            };
            let width = DEFAULT_WIDTH;
            let height = DEFAULT_HEIGHT;
            // The blocks that go in the same place are stacked, not piled.
            let offset = stacked.entry(owner).or_default();
            positions.push(Position {
                id: block,
                x: x + PAD + *offset,
                y: y + PAD + *offset,
                width: Some(width),
                height: Some(height),
            });
            *offset += DEFAULT_HEIGHT + GAP_Y;
        }
        // The frames of the containers grow to hold the blocks that were
        // placed in them.
        place_frames(model, root, &mut positions, margin);
    }
    // The composition starts where the author put things, and never before
    // it: the picture has no negative coordinates.
    let (origin_x, origin_y) = origin(model);
    let left = positions.iter().map(|position| position.x).min().unwrap_or(0);
    let top = positions.iter().map(|position| position.y).min().unwrap_or(0);
    let (shift_x, shift_y) = (origin_x - left.min(origin_x), origin_y - top.min(origin_y));
    for position in &mut positions {
        position.x += shift_x;
        position.y += shift_y;
    }
    positions.sort_by_key(|position| position.id);
    positions
}

/// The composition of **one scope**: the blocks the scope owns, and nothing
/// else.
///
/// The graph dagre composes is the scope's own: its blocks and the flow
/// between them. A container nested in the scope is an *atomic box*: a node
/// of the graph with the size the model gives it, whose content is neither
/// looked at nor moved. The ports of the scope are the ends of the flow:
/// they take part in the composition as the nodes the flow starts at and ends
/// at, so the block the flow enters through is placed first and the block it
/// leaves through is placed last.
///
/// The gaps are the ones of the whole model, so a scope and the whole model
/// agree on how far apart two blocks of a layer are. The tuning that leaves
/// the whole picture square is not applied here: a scope is a canvas of its
/// own, and its picture is not the picture of the model.
fn compose_scope(
    model: &GmdlModel,
    scope: u32,
    children: &[u32],
    ranker: Ranker,
    nodesep: i32,
    ranksep: i32,
) -> ScopeComposition {
    // The margin of the frames: it is never wider than the gap between two
    // layers.
    let margin = PAD.min(ranksep / 2);
    // The flow of the scope: the links it owns, with the borders of the
    // nested containers spliced, and the ports of the scope at its ends.
    let flow = flow::scope_flow(model, scope);
    // The blocks of the scope that no link of the scope reaches: they are
    // drawn in a grid beside the flow.
    let grid = grid_of_scope(&flow, children);
    let in_grid: HashSet<u32> = grid.iter().copied().collect();
    let mut graph = Graph::with_options(GraphOptions {
        directed: true,
        multigraph: true,
        compound: false,
    });
    for id in children {
        // The blocks that are drawn in the grid take no room in the flow:
        // they are not drawn there. They stay in the graph all the same: the
        // order the crate gives the flow depends on them, and the flow
        // crosses fewer links when they are there.
        let (width, height) = if in_grid.contains(id) {
            (0, 0)
        } else {
            size_of(model, *id)
        };
        let label = NodeLabel {
            width: f64::from(width),
            height: f64::from(height),
            ..NodeLabel::default()
        };
        graph.set_node(id.to_string(), Some(label));
    }
    // The ports of the scope: the ends of the flow. They are blocks of the
    // picture in their own right: the composition places them like it
    // places any other block, and their place is written back.
    for (ioid, _) in &flow.entries {
        graph.set_node(entry_node(*ioid), Some(door_label()));
    }
    for (_, ioid) in &flow.exits {
        graph.set_node(exit_node(*ioid), Some(door_label()));
    }
    // The doors that talk to each other: the flow of a submodule whose only
    // content is its doors. They are nodes of the composition like the
    // others, and the link between them says which goes first.
    for (from_ioid, to_ioid) in &flow.port_links {
        graph.set_node(door_node(model, scope, *from_ioid).0, Some(door_label()));
        graph.set_node(door_node(model, scope, *to_ioid).0, Some(door_label()));
    }
    for (from_ioid, to_ioid) in &flow.port_links {
        graph.set_edge(
            door_node(model, scope, *from_ioid).0,
            door_node(model, scope, *to_ioid).0,
            Some(EdgeLabel::default()),
            None,
        );
    }
    // The flow of the scope: the links between the blocks it owns, with the
    // borders of the nested containers spliced, and the links that carry the
    // flow into the scope and out of it.
    for (from, to) in &flow.edges {
        graph.set_edge(
            from.to_string(),
            to.to_string(),
            Some(EdgeLabel::default()),
            None,
        );
    }
    for (ioid, to) in &flow.entries {
        graph.set_edge(
            entry_node(*ioid),
            to.to_string(),
            Some(EdgeLabel::default()),
            None,
        );
    }
    for (from, ioid) in &flow.exits {
        graph.set_edge(
            from.to_string(),
            exit_node(*ioid),
            Some(EdgeLabel::default()),
            None,
        );
    }
    let options = LayoutOptions {
        // The flow goes down: the input of a block is at its top and its
        // output at its bottom.
        rankdir: RankDir::TB,
        ranker,
        // The gap between two blocks of the same layer, and the gap between
        // two layers of the flow.
        nodesep: f64::from(nodesep),
        ranksep: f64::from(ranksep),
        edgesep: 20.0,
        ..LayoutOptions::default()
    };
    dagre::layout(&mut graph, Some(options));

    // dagre places a block by its middle; the file carries its corner.
    let mut positions: Vec<Position> = Vec::new();
    for id in children {
        if in_grid.contains(id) {
            // The blocks that are drawn in the grid are placed in the grid,
            // not where dagre puts them.
            continue;
        }
        let Some(node) = graph.node(&id.to_string()) else {
            continue;
        };
        let (Some(x), Some(y)) = (node.x, node.y) else {
            continue;
        };
        let width = node.width.round() as i32;
        let height = node.height.round() as i32;
        positions.push(Position {
            id: *id,
            x: (x - f64::from(width) / 2.0).round() as i32,
            y: (y - f64::from(height) / 2.0).round() as i32,
            width: Some(width),
            height: Some(height),
        });
    }
    // The chains of the flow start together: the reader sees every start of
    // the flow in the first row.
    align_components(model, scope, &mut positions, &flow.edges);
    // The chains that share the canvas move apart: the picture grows only
    // where two chains of the flow meet nowhere and overlap.
    separate_components(model, scope, &mut positions, &flow.edges, margin);
    // The blocks that are drawn in the grid get a place in it: the grid is
    // laid out once the picture of the flow is known.
    for id in &grid {
        let (width, height) = size_of(model, *id);
        positions.push(Position {
            id: *id,
            x: 0,
            y: 0,
            width: Some(width),
            height: Some(height),
        });
    }
    // The blocks that are drawn in the grid go in a grid beside the flow, in
    // the shape that leaves the picture as square as possible.
    place_free(&mut positions, model, &grid, nodesep);
    // The frame of the scope is **not** composed: the container is a box of
    // its own, and what it holds does not change it. Its size and its place
    // are the ones the model gives it, and nothing here touches them.
    // The doors of the scope: the place the composition gave them. A door
    // is a block of the picture, and its place travels in the model. A door
    // that carries the flow to several blocks is still one door: it has one
    // place, not one per link.
    let mut ports: Vec<PortPosition> = Vec::new();
    let entries: BTreeSet<u32> = flow.entries.iter().map(|(ioid, _)| *ioid).collect();
    for ioid in &entries {
        if let Some(port) = door_position(&graph, model, scope, *ioid) {
            ports.push(port);
        }
    }
    let exits: BTreeSet<u32> = flow.exits.iter().map(|(_, ioid)| *ioid).collect();
    for ioid in &exits {
        if let Some(port) = door_position(&graph, model, scope, *ioid) {
            ports.push(port);
        }
    }
    // The doors that talk to each other: they are placed by the composition
    // like any other block of the picture.
    for (from_ioid, to_ioid) in &flow.port_links {
        for ioid in [from_ioid, to_ioid] {
            if ports.iter().any(|port| port.ioid == *ioid) {
                continue;
            }
            if let Some(port) = door_position(&graph, model, scope, *ioid) {
                ports.push(port);
            }
        }
    }
    // The doors that carry no flow yet —the reader just added them— go
    // with the doors of their own kind, one under the other: the picture
    // shows which door is which.
    if let Some(container) = model.block(scope) {
        // The doors that the flow joins to each other are placed by the
        // composition, not by the free spot.
        let joined: BTreeSet<u32> = flow
            .port_links
            .iter()
            .flat_map(|(from, to)| [*from, *to])
            .collect();
        let mut pending: Vec<(u32, bool)> = Vec::new();
        for port in &container.inputs {
            if !entries.contains(&port.ioid) && !joined.contains(&port.ioid) {
                pending.push((port.ioid, true));
            }
        }
        for port in &container.outputs {
            if !exits.contains(&port.ioid) && !joined.contains(&port.ioid) {
                pending.push((port.ioid, false));
            }
        }
        // The doors that go to the free spot are stacked by their kind: the
        // entry doors under the entry doors, the exit doors under the exit
        // doors.
        let mut stacked_inputs = 0;
        let mut stacked_outputs = 0;
        for (ioid, input) in pending {
            let same_kind: Vec<&PortPosition> =
                ports.iter().filter(|port| port.input == input).collect();
            let (x, y) = match same_kind
                .iter()
                .map(|port| port.y + port.height.unwrap_or(DOOR_HEIGHT))
                .max()
            {
                Some(bottom) => (same_kind[0].x, bottom + RANK_SEP),
                None => {
                    // There is no door of its kind to sit under: the door
                    // goes to the free spot of the scope, under the doors of
                    // its kind that are already there.
                    let (left, top) = free_spot_of_scope(model, children);
                    let placed = if input {
                        stacked_inputs += 1;
                        stacked_inputs - 1
                    } else {
                        stacked_outputs += 1;
                        stacked_outputs - 1
                    };
                    (left, top + placed * (DOOR_HEIGHT + GAP_Y))
                }
            };
            ports.push(PortPosition {
                block: scope,
                ioid,
                input,
                x,
                y,
                width: Some(DOOR_WIDTH),
                height: Some(DOOR_HEIGHT),
            });
        }
    }
    // The composition starts where the author put things, and never before
    // it: the picture has no negative coordinates.
    let (origin_x, origin_y) = scope_origin(model, children);
    let left = positions.iter().map(|position| position.x).min().unwrap_or(0);
    let top = positions.iter().map(|position| position.y).min().unwrap_or(0);
    let (shift_x, shift_y) = (origin_x - left.min(origin_x), origin_y - top.min(origin_y));
    for position in &mut positions {
        position.x += shift_x;
        position.y += shift_y;
    }
    for port in &mut ports {
        port.x += shift_x;
        port.y += shift_y;
    }
    positions.sort_by_key(|position| position.id);
    ports.sort_by_key(|port| (port.ioid, port.input));
    ScopeComposition { positions, ports }
}

/// The free spot of a scope: under its lowest block, at its left.
///
/// It is where a block that has no place of its own goes, and where a door
/// that carries no flow yet goes when there is no door of its kind to sit
/// under.
pub fn free_spot_of_scope(model: &GmdlModel, children: &[u32]) -> (i32, i32) {
    let mut left = i32::MAX;
    let mut bottom = i32::MIN;
    for id in children {
        let Some(layout) = model.block(*id).and_then(|block| block.layout) else {
            continue;
        };
        left = left.min(layout.x);
        bottom = bottom.max(layout.y + layout.height);
    }
    if left == i32::MAX {
        return (EMPTY_SPOT, EMPTY_SPOT);
    }
    (left, bottom + FREE_SPOT_AIR)
}

/// The box of a door of the scope.
///
/// It is the box the app draws a door in, and the one the composition
/// reserves for it: the numbers are the same ones the app uses, so the
/// picture the composition makes is the picture the reader sees.
fn door_label() -> NodeLabel {
    NodeLabel {
        width: f64::from(DOOR_WIDTH),
        height: f64::from(DOOR_HEIGHT),
        ..NodeLabel::default()
    }
}

/// The place of a door of the scope, as the composition gave it.
///
/// Dagre places a node by its middle and the model carries its corner.
fn door_position(
    graph: &Graph<NodeLabel, EdgeLabel>,
    model: &GmdlModel,
    scope: u32,
    ioid: u32,
) -> Option<PortPosition> {
    let (name, input) = door_node(model, scope, ioid);
    let node = graph.node(&name)?;
    let (Some(x), Some(y)) = (node.x, node.y) else {
        return None;
    };
    let width = node.width.round() as i32;
    let height = node.height.round() as i32;
    Some(PortPosition {
        block: scope,
        ioid,
        input,
        x: (x - f64::from(width) / 2.0).round() as i32,
        y: (y - f64::from(height) / 2.0).round() as i32,
        width: Some(width),
        height: Some(height),
    })
}

/// The node of the graph that stands for a port of the scope, and whether the
/// port is an entry one.
///
/// The ports of a scope share one numbering, so the kind of the port is what
/// tells them apart: a port that is an input of the container is the node the
/// flow enters the scope by, and a port that is an output is the node it
/// leaves through.
fn door_node(model: &GmdlModel, scope: u32, ioid: u32) -> (String, bool) {
    let is_input = model
        .block(scope)
        .is_some_and(|block| block.inputs.iter().any(|port| port.ioid == ioid));
    if is_input {
        (entry_node(ioid), true)
    } else {
        (exit_node(ioid), false)
    }
}

/// The blocks of a scope that no link of the scope reaches: they are drawn in
/// a grid beside the flow.
///
/// A block that the ports of the scope reach is in the flow: the entry port
/// carries the flow into the scope, and the exit port carries it out.
fn grid_of_scope(flow: &flow::ScopeFlow, children: &[u32]) -> Vec<u32> {
    let mut reached: HashSet<u32> = HashSet::new();
    for (from, to) in &flow.edges {
        reached.insert(*from);
        reached.insert(*to);
    }
    for (_, to) in &flow.entries {
        reached.insert(*to);
    }
    for (from, _) in &flow.exits {
        reached.insert(*from);
    }
    children
        .iter()
        .copied()
        .filter(|id| !reached.contains(id))
        .collect()
}

/// Where the picture of a scope starts: where the author put the blocks the
/// scope owns.
///
/// A scope is a canvas of its own: its origin is the corner of the blocks it
/// holds, not the origin of the whole model.
fn scope_origin(model: &GmdlModel, children: &[u32]) -> (i32, i32) {
    let mut x = i32::MAX;
    let mut y = i32::MAX;
    for id in children {
        let Some(layout) = model.block(*id).and_then(|block| block.layout) else {
            continue;
        };
        x = x.min(layout.x);
        y = y.min(layout.y);
    }
    if x == i32::MAX {
        (0, 0)
    } else {
        (x, y)
    }
}

/// The node of the graph that stands for the entry port of a scope.
///
/// The name cannot be the id of a block: the ids of the blocks are numbers,
/// and the ports of a scope are not blocks of the model.
fn entry_node(ioid: u32) -> String {
    format!("port-in-{ioid}")
}

/// The node of the graph that stands for the exit port of a scope.
fn exit_node(ioid: u32) -> String {
    format!("port-out-{ioid}")
}

/// The blocks that are drawn in a grid beside the flow: the ones no link
/// reaches and that live in the root of the model.
///
/// The blocks inside a container are not here: they stay with the container
/// that owns them, and the frame of the container grows to hold them. A
/// container that holds nothing is here all the same: it is a block with no
/// flow in it.
fn grid_blocks(
    model: &GmdlModel,
    root: u32,
    edges: &[(u32, u32)],
    holders: &HashSet<u32>,
) -> Vec<u32> {
    let mut grid: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.id != root && block.owner == root)
        .filter(|block| !holders.contains(&block.id))
        .filter(|block| !edges.iter().any(|(from, to)| *from == block.id || *to == block.id))
        .map(|block| block.id)
        .collect();
    grid.sort_unstable();
    grid
}

/// The size of a block: the one its model carries, or the default one.
fn size_of(model: &GmdlModel, id: u32) -> (i32, i32) {
    model
        .block(id)
        .map(|block| match block.layout {
            Some(layout) => (layout.width, layout.height),
            None => (DEFAULT_WIDTH, DEFAULT_HEIGHT),
        })
        .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT))
}

/// The entity of a block: the container it lives in, or the block itself
/// when it lives in the root of the model.
///
/// The blocks of a container move together: the frame of a container is the
/// box of the blocks it holds, and a frame that moves apart from its blocks
/// no longer holds them.
fn entity_of(model: &GmdlModel, root: u32, id: u32) -> u32 {
    let mut current = id;
    loop {
        let Some(block) = model.block(current) else {
            return id;
        };
        if block.owner == root || block.owner == current {
            return current;
        }
        current = block.owner;
    }
}

/// The component of every entity of the flow.
///
/// The flow of a model is often several chains that meet nowhere. Each one
/// of them is a component.
fn components_of(edges: &[(u32, u32)]) -> HashMap<u32, usize> {
    let mut neighbours: HashMap<u32, Vec<u32>> = HashMap::new();
    for (from, to) in edges {
        neighbours.entry(*from).or_default().push(*to);
        neighbours.entry(*to).or_default().push(*from);
    }
    let mut component: HashMap<u32, usize> = HashMap::new();
    let mut starts: Vec<u32> = neighbours.keys().copied().collect();
    starts.sort_unstable();
    let mut count = 0;
    for start in starts {
        if component.contains_key(&start) {
            continue;
        }
        let mut pending = vec![start];
        component.insert(start, count);
        while let Some(block) = pending.pop() {
            for next in neighbours.get(&block).into_iter().flatten() {
                if !component.contains_key(next) {
                    component.insert(*next, count);
                    pending.push(*next);
                }
            }
        }
        count += 1;
    }
    component
}

/// The entity of every block of the model: the container it lives in, or
/// the block itself when it lives in the root.
fn entities_of(model: &GmdlModel, root: u32) -> HashMap<u32, u32> {
    model
        .blocks
        .iter()
        .filter(|block| block.id != root)
        .map(|block| (block.id, entity_of(model, root, block.id)))
        .collect()
}

/// The component of every block of the flow.
///
/// The blocks of a container are one entity: they move together, and they
/// belong to the same chain of the flow.
fn block_components(
    model: &GmdlModel,
    root: u32,
    edges: &[(u32, u32)],
) -> (HashMap<u32, u32>, HashMap<u32, usize>) {
    let entities = entities_of(model, root);
    // The flow between the entities: two blocks of the same container are
    // one entity, and they move together.
    let entity_edges: Vec<(u32, u32)> = edges
        .iter()
        .filter_map(|(from, to)| Some((entities.get(from)?, entities.get(to)?)))
        .map(|(from, to)| (*from, *to))
        .collect();
    let components = components_of(&entity_edges);
    (entities, components)
}

/// Put the components of the flow at the same height.
///
/// Every source of the flow starts the picture: the reader sees all of them
/// in the first row, and the flow goes down from there. The crate the
/// composition uses ends the chains of the flow at the same row, so the
/// sources end up staggered: the reader has to look for the start of every
/// chain.
fn align_components(
    model: &GmdlModel,
    root: u32,
    positions: &mut [Position],
    edges: &[(u32, u32)],
) {
    let (entities, component) = block_components(model, root, edges);
    if component.is_empty() {
        return;
    }
    // The top of every component.
    let mut tops: HashMap<usize, i32> = HashMap::new();
    for position in positions.iter() {
        let Some(entity) = entities.get(&position.id) else {
            continue;
        };
        let Some(id) = component.get(entity) else {
            continue;
        };
        let top = tops.entry(*id).or_insert(position.y);
        *top = (*top).min(position.y);
    }
    let Some(first) = tops.values().min().copied() else {
        return;
    };
    // The shift every component needs to start where the first one starts.
    let shifts: HashMap<usize, i32> = tops
        .iter()
        .map(|(id, top)| (*id, first - top))
        .collect();
    for position in positions.iter_mut() {
        let Some(entity) = entities.get(&position.id) else {
            continue;
        };
        let Some(id) = component.get(entity) else {
            continue;
        };
        if let Some(shift) = shifts.get(id) {
            position.y += shift;
        }
    }
}

/// The box of every component of the flow: the blocks it holds and the
/// frames of the containers it holds.
///
/// A container is drawn around the blocks it holds, so the frame of the
/// container is what the component occupies on the canvas.
fn component_boxes(
    model: &GmdlModel,
    root: u32,
    positions: &[Position],
    entities: &HashMap<u32, u32>,
    components: &HashMap<u32, usize>,
    margin: i32,
) -> HashMap<usize, (i32, i32, i32, i32)> {
    let frames = frames_of(model, root, positions, margin);
    let mut boxes: HashMap<usize, (i32, i32, i32, i32)> = HashMap::new();
    for position in positions {
        let Some(entity) = entities.get(&position.id) else {
            continue;
        };
        let Some(component) = components.get(entity) else {
            continue;
        };
        let rect = match model.block(position.id) {
            // A container that holds blocks has a frame around them. A
            // container whose content lives in another canvas (a container
            // nested in the scope being composed) has no frame here: it is
            // the box it is.
            Some(block) if block.is_container() => frames.get(&position.id).copied().or(Some((
                position.x,
                position.y,
                position.width.unwrap_or(DEFAULT_WIDTH),
                position.height.unwrap_or(DEFAULT_HEIGHT),
            ))),
            _ => Some((
                position.x,
                position.y,
                position.width.unwrap_or(DEFAULT_WIDTH),
                position.height.unwrap_or(DEFAULT_HEIGHT),
            )),
        };
        let Some((x, y, width, height)) = rect else {
            continue;
        };
        boxes
            .entry(*component)
            .and_modify(|corners| {
                corners.0 = corners.0.min(x);
                corners.1 = corners.1.min(y);
                corners.2 = corners.2.max(x + width);
                corners.3 = corners.3.max(y + height);
            })
            .or_insert((x, y, x + width, y + height));
    }
    boxes
}

/// Move the components of the flow that share the canvas apart.
///
/// The crate the composition uses does not separate the chains of the flow
/// that meet nowhere: two of them can end up in the same columns. The chains
/// that start the picture together have to move apart, or one of them lands
/// on top of the other. The picture grows only where it has to.
fn separate_components(
    model: &GmdlModel,
    root: u32,
    positions: &mut [Position],
    edges: &[(u32, u32)],
    margin: i32,
) {
    let (entities, components) = block_components(model, root, edges);
    if components.is_empty() {
        return;
    }
    let boxes = component_boxes(model, root, positions, &entities, &components, margin);
    // The components, from left to right.
    let mut order: Vec<usize> = boxes.keys().copied().collect();
    order.sort_by_key(|id| (boxes[id].0, boxes[id].1, *id));
    // The shift of every component: the one that overlaps a component that
    // is already placed moves to the right of it.
    let mut shifts: HashMap<usize, i32> = HashMap::new();
    for (index, id) in order.iter().enumerate() {
        let (left, top, right, bottom) = boxes[id];
        let mut shift = 0;
        for other in order.iter().take(index) {
            let (other_left, other_top, other_right, other_bottom) = boxes[other];
            let other_shift = shifts.get(other).copied().unwrap_or(0);
            let (other_left, other_right) = (other_left + other_shift, other_right + other_shift);
            if left + shift >= other_right
                || other_left >= right + shift
                || top >= other_bottom
                || other_top >= bottom
            {
                continue;
            }
            shift = shift.max(other_right + GAP_X - (left + shift));
        }
        if shift != 0 {
            shifts.insert(*id, shift);
        }
    }
    if shifts.is_empty() {
        return;
    }
    for position in positions.iter_mut() {
        let Some(entity) = entities.get(&position.id) else {
            continue;
        };
        let Some(component) = components.get(entity) else {
            continue;
        };
        if let Some(shift) = shifts.get(component) {
            position.x += shift;
        }
    }
}

/// Put the frame of every container on the canvas.
///
/// The frame of a container is the box of the blocks it holds, with the
/// margin that separates them. The frame follows the blocks: a container
/// that moves apart from them no longer holds them.
fn place_frames(model: &GmdlModel, root: u32, positions: &mut Vec<Position>, margin: i32) {
    let frames = frames_of(model, root, positions, margin);
    for block in &model.blocks {
        if !block.is_container() || block.id == root {
            continue;
        }
        let Some((x, y, width, height)) = frames.get(&block.id).copied() else {
            continue;
        };
        match positions
            .iter_mut()
            .find(|position| position.id == block.id)
        {
            Some(position) => {
                position.x = x;
                position.y = y;
                position.width = Some(width);
                position.height = Some(height);
            }
            None => positions.push(Position {
                id: block.id,
                x,
                y,
                width: Some(width),
                height: Some(height),
            }),
        }
    }
}

/// The side of the flow the grid of the blocks that are in no flow goes to.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    /// Beside the flow.
    Left,
    /// Above the flow.
    Above,
}

/// The shape of the grid of the blocks that are in no flow.
struct FreeGrid {
    /// The columns of the grid.
    columns: usize,
    /// The side of the flow the grid goes to.
    side: Side,
    /// The width of the grid.
    width: i32,
    /// The height of the grid.
    height: i32,
    /// The width of the whole picture.
    total_width: i32,
    /// The height of the whole picture.
    total_height: i32,
}

impl FreeGrid {
    /// Whether this grid leaves a picture with a smaller largest side.
    ///
    /// The largest side is what the reader feels: a picture that is very
    /// wide and very short has to be read zoomed out, and the same happens
    /// with a picture that is very tall. The ties are broken by the narrower
    /// picture and then by the grid with fewer columns, so the composition
    /// is the same every time.
    fn squarer_than(&self, other: &FreeGrid) -> bool {
        (
            self.total_width.max(self.total_height),
            self.total_width,
            self.columns,
        ) < (
            other.total_width.max(other.total_height),
            other.total_width,
            other.columns,
        )
    }
}

/// Put the blocks that are in no flow in a grid beside the flow.
///
/// The blocks that no link reaches have no place in the flow: they go in a
/// grid, and the shape of the grid and the side it goes are the ones that
/// leave the picture as square as possible.
///
/// Every column of the grid is as wide as the widest block of it, and every
/// row as tall as the tallest block of it: a block that is small does not
/// make the grid bigger.
fn place_free(positions: &mut [Position], model: &GmdlModel, free: &[u32], nodesep: i32) {
    if free.is_empty() {
        return;
    }
    // The size of every block of the grid.
    let order: Vec<(u32, i32, i32)> = free
        .iter()
        .map(|id| (*id, size_of(model, *id).0, size_of(model, *id).1))
        .collect();
    let sizes: Vec<(i32, i32)> = order.iter().map(|(_, width, height)| (*width, *height)).collect();
    // The box of everything else: the flow and the frames around it.
    let flow = box_of(
        positions
            .iter()
            .filter(|position| !free.contains(&position.id)),
    );
    // The candidates: the grid beside the flow or above it, with as many
    // columns as it takes.
    let mut best: Option<FreeGrid> = None;
    for columns in 1..=free.len() {
        let rows = free.len().div_ceil(columns);
        let (width, height, _) = grid_of(&sizes, columns, rows, nodesep);
        for side in [Side::Left, Side::Above] {
            let (total_width, total_height) = match (&flow, side) {
                (None, _) => (width, height),
                (Some((left, top, right, bottom)), Side::Left) => (
                    width + nodesep + (right - left),
                    height.max(bottom - top),
                ),
                (Some((left, top, right, bottom)), Side::Above) => (
                    width.max(right - left),
                    height + GAP_Y + (bottom - top),
                ),
            };
            let candidate = FreeGrid {
                columns,
                side,
                width,
                height,
                total_width,
                total_height,
            };
            if best.as_ref().is_none_or(|other| candidate.squarer_than(other)) {
                best = Some(candidate);
            }
        }
    }
    let Some(grid) = best else {
        return;
    };
    // The place of the grid: beside the flow or above it, at the top.
    let (origin_x, origin_y) = match (&flow, grid.side) {
        (Some((left, top, _, _)), Side::Left) => (*left - nodesep - grid.width, *top),
        (Some((left, top, _, _)), Side::Above) => (*left, *top - GAP_Y - grid.height),
        (None, _) => (0, 0),
    };
    let rows = free.len().div_ceil(grid.columns);
    let (_, _, offsets) = grid_of(&sizes, grid.columns, rows, nodesep);
    for (index, (id, _, _)) in order.iter().enumerate() {
        let column = index % grid.columns;
        let row = index / grid.columns;
        let Some(position) = positions.iter_mut().find(|position| position.id == *id) else {
            continue;
        };
        position.x = origin_x + offsets.0[column];
        position.y = origin_y + offsets.1[row];
    }
}

/// The shape of a grid of blocks: its width, its height, and the place of
/// every column and every row inside it.
///
/// A column is as wide as the widest block of it, and a row as tall as the
/// tallest block of it.
fn grid_of(
    sizes: &[(i32, i32)],
    columns: usize,
    rows: usize,
    gap: i32,
) -> (i32, i32, (Vec<i32>, Vec<i32>)) {
    let mut column_widths = vec![0; columns];
    let mut row_heights = vec![0; rows];
    for (index, (width, height)) in sizes.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        column_widths[column] = column_widths[column].max(*width);
        row_heights[row] = row_heights[row].max(*height);
    }
    let mut column_offsets = Vec::with_capacity(columns);
    let mut x = 0;
    for width in &column_widths {
        column_offsets.push(x);
        x += width + gap;
    }
    let mut row_offsets = Vec::with_capacity(rows);
    let mut y = 0;
    for height in &row_heights {
        row_offsets.push(y);
        y += height + GAP_Y;
    }
    let width: i32 = column_widths.iter().sum::<i32>() + (columns as i32 - 1) * gap;
    let height: i32 = row_heights.iter().sum::<i32>() + (rows as i32 - 1) * GAP_Y;
    (width, height, (column_offsets, row_offsets))
}

/// The box that holds a set of positions: its corners.
fn box_of<'a>(
    positions: impl Iterator<Item = &'a Position>,
) -> Option<(i32, i32, i32, i32)> {
    positions.fold(None, |corners, position| {
        let (x, y, width, height) = (
            position.x,
            position.y,
            position.width.unwrap_or(DEFAULT_WIDTH),
            position.height.unwrap_or(DEFAULT_HEIGHT),
        );
        Some(match corners {
            None => (x, y, x + width, y + height),
            Some((left, top, right, bottom)) => (
                left.min(x),
                top.min(y),
                right.max(x + width),
                bottom.max(y + height),
            ),
        })
    })
}

/// The frame of every container: the box of the blocks it holds.
///
/// The frames are computed from the innermost out: the frame of a container
/// is the box of its children, and a container that holds another container
/// holds that container's frame.
fn frames_of(
    model: &GmdlModel,
    root: u32,
    positions: &[Position],
    margin: i32,
) -> HashMap<u32, (i32, i32, i32, i32)> {
    let mut boxes: HashMap<u32, (i32, i32, i32, i32)> = HashMap::new();
    for position in positions {
        boxes.insert(
            position.id,
            (
                position.x,
                position.y,
                position.x + position.width.unwrap_or(0),
                position.y + position.height.unwrap_or(0),
            ),
        );
    }
    let mut frames: HashMap<u32, (i32, i32, i32, i32)> = HashMap::new();
    for block in &model.blocks {
        if !block.is_container() || block.id == root {
            continue;
        }
        frame_of(block.id, model, &boxes, &mut frames, margin);
    }
    frames
}

/// The frame of a container: the box of the blocks it holds, with the margin
/// that separates them.
///
/// A container that holds another container holds that container's frame, so
/// the frames are taken from the innermost out. A container that holds
/// nothing has no frame around it: the frame is the container itself.
fn frame_of(
    container: u32,
    model: &GmdlModel,
    boxes: &HashMap<u32, (i32, i32, i32, i32)>,
    frames: &mut HashMap<u32, (i32, i32, i32, i32)>,
    margin: i32,
) -> Option<(i32, i32, i32, i32)> {
    let children: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.owner == container && block.id != container)
        .map(|block| block.id)
        .collect();
    if children.is_empty() {
        // A container that holds nothing is a box of its own: the frame is
        // the container, without a margin around it.
        let (x1, y1, x2, y2) = boxes.get(&container).copied()?;
        let frame = (x1, y1, x2 - x1, y2 - y1);
        frames.insert(container, frame);
        return Some(frame);
    }
    let mut box_of: Option<(i32, i32, i32, i32)> = None;
    for child in children {
        let corners = match frames.get(&child) {
            // The frame is stored as its corner and its size; the box of the
            // children is their corners.
            Some((x, y, width, height)) => Some((*x, *y, x + width, y + height)),
            None => match model.block(child) {
                Some(block) if block.is_container() => {
                    // The walk returns the corners of the frame.
                    frame_of(child, model, boxes, frames, margin)
                        .map(|(x, y, width, height)| (x, y, x + width, y + height))
                }
                _ => boxes.get(&child).copied(),
            },
        };
        let Some((x1, y1, x2, y2)) = corners else {
            continue;
        };
        box_of = Some(match box_of {
            None => (x1, y1, x2, y2),
            Some((bx1, by1, bx2, by2)) => (bx1.min(x1), by1.min(y1), bx2.max(x2), by2.max(y2)),
        });
    }
    let (x1, y1, x2, y2) = box_of?;
    let frame = (
        x1 - margin,
        y1 - margin,
        x2 - x1 + 2 * margin,
        y2 - y1 + 2 * margin,
    );
    frames.insert(container, frame);
    Some(frame)
}

/// Whether every block of the model, the root aside, carries a geometry.
///
/// A composed model is one the editor can draw: every block of it has a place
/// on the canvas.
pub fn is_composed(model: &GmdlModel) -> bool {
    model
        .blocks
        .iter()
        .filter(|block| !block.is_root())
        .all(|block| block.layout.is_some())
}

/// Compose the model when its geometry is incomplete.
///
/// A model that is already composed keeps the disposition its author left: the
/// composition is only for the models the editor could not draw. The
/// positions that were applied come back; `None` means the model was left
/// alone.
pub fn compose_incomplete(model: &mut GmdlModel) -> Option<Vec<Position>> {
    if is_composed(model) {
        return None;
    }
    let positions = relayout(model);
    apply(model, &positions);
    Some(positions)
}

/// Put the positions on the model.
fn apply(model: &mut GmdlModel, positions: &[Position]) {
    for position in positions {
        let Some(block) = model.block_mut(position.id) else {
            continue;
        };
        let current = block.layout;
        block.layout = Some(Layout {
            width: position
                .width
                .unwrap_or_else(|| current.map(|layout| layout.width).unwrap_or(DEFAULT_WIDTH)),
            height: position
                .height
                .unwrap_or_else(|| current.map(|layout| layout.height).unwrap_or(DEFAULT_HEIGHT)),
            x: position.x,
            y: position.y,
        });
    }
}

/// Where the composition starts: where the author put things.
fn origin(model: &GmdlModel) -> (i32, i32) {
    let mut x = i32::MAX;
    let mut y = i32::MAX;
    for block in &model.blocks {
        if let Some(layout) = block.layout {
            x = x.min(layout.x);
            y = y.min(layout.y);
        }
    }
    if x == i32::MAX {
        (0, 0)
    } else {
        (x, y)
    }
}
