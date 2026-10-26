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

pub mod flow;

use dagre::graph::{Graph, GraphOptions};
use dagre::{EdgeLabel, LayoutOptions, NodeLabel, RankDir, Ranker};

use std::collections::{HashMap, HashSet};

use crate::edit::Position;
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
    match compose(model, Ranker::NetworkSimplex) {
        Some(positions) => positions,
        None => compose(model, Ranker::TightTree).unwrap_or_default(),
    }
}

/// The composition of a model with a ranker, when the ranker can do it.
///
/// `None` means the ranker failed: the port's network simplex panics on some
/// compound graphs, and the panic is caught so that the composition falls
/// back to the tight tree.
fn compose(model: &GmdlModel, ranker: Ranker) -> Option<Vec<Position>> {
    let root = model.root()?.id;
    let hook = std::panic::take_hook();
    if std::env::var("DAGRE_VERBOSE").is_err() {
        std::panic::set_hook(Box::new(|_| {}));
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compose_with(model, root, ranker)
    }));
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
fn compose_with(model: &GmdlModel, root: u32, ranker: Ranker) -> Vec<Position> {
    // The flow of the model: the links between the blocks that simulate,
    // with the borders of the containers spliced.
    let edges = flow::flow_edges_for(model);
    // The blocks that are in no flow: they are not part of the flow dagre
    // composes, they go in a grid beside it.
    let free = free_blocks(model, root, &edges);
    let in_flow: HashSet<u32> = free.iter().copied().collect();
    let mut graph = Graph::with_options(GraphOptions {
        directed: true,
        multigraph: true,
        compound: true,
    });
    for block in &model.blocks {
        if block.id == root {
            continue;
        }
        if block.is_container() {
            // A container has no size of its own: its frame is computed
            // around the blocks it holds.
            graph.set_node(block.id.to_string(), None);
            continue;
        }
        if in_flow.contains(&block.id) {
            // A block that is in no flow has no place in the flow: the
            // composition puts it in a grid beside the flow.
            continue;
        }
        let (width, height) = match block.layout {
            Some(layout) => (layout.width, layout.height),
            None => (DEFAULT_WIDTH, DEFAULT_HEIGHT),
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
        nodesep: 30.0,
        ranksep: 60.0,
        edgesep: 20.0,
        ..LayoutOptions::default()
    };
    dagre::layout(&mut graph, Some(options));

    // dagre places a block by its middle; the file carries its corner.
    let mut positions: Vec<Position> = Vec::new();
    for block in &model.blocks {
        if block.id == root || block.is_container() {
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
    // The components of the flow start together at the top: the sources of
    // the flow are the first row of the picture.
    align_components(model, root, &mut positions, &edges);
    // The blocks that are in no flow get a place in the grid: the grid is
    // laid out once the picture of the flow is known.
    for id in &free {
        let (width, height) = size_of(model, *id);
        positions.push(Position {
            id: *id,
            x: 0,
            y: 0,
            width: Some(width),
            height: Some(height),
        });
    }
    // The containers: the frame of each one is the box of the blocks it
    // holds, with the margin that separates them. The frames are computed
    // from the innermost out.
    let frames = frames_of(model, root, &positions);
    for block in &model.blocks {
        if !block.is_container() {
            continue;
        }
        let Some((x, y, width, height)) = frames.get(&block.id).copied() else {
            continue;
        };
        positions.push(Position {
            id: block.id,
            x,
            y,
            width: Some(width),
            height: Some(height),
        });
    }
    // The containers dagre did not place (its tight tree leaves the blocks of
    // a cycle without a place): their frame is computed around the blocks
    // they hold.
    let missing: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.is_container() && block.id != root)
        .filter(|block| !positions.iter().any(|position| position.id == block.id))
        .map(|block| block.id)
        .collect();
    if !missing.is_empty() {
        let frames = frames_of(model, root, &positions);
        for container in missing {
            let Some((x, y, width, height)) = frames.get(&container).copied() else {
                continue;
            };
            positions.push(Position {
                id: container,
                x,
                y,
                width: Some(width),
                height: Some(height),
            });
        }
    }
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
        let frames = frames_of(model, root, &positions);
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
        let frames = frames_of(model, root, &positions);
        for position in &mut positions {
            let Some(block) = model.block(position.id) else {
                continue;
            };
            if !block.is_container() {
                continue;
            }
            let Some((x, y, width, height)) = frames.get(&block.id).copied() else {
                continue;
            };
            position.x = x;
            position.y = y;
            position.width = Some(width);
            position.height = Some(height);
        }
    }
    // The blocks that are in no flow go in a grid beside the flow, in the
    // shape that leaves the picture as square as possible.
    place_free(&mut positions, model, &free);
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

/// The blocks that are in no flow: the ones no link reaches and that live
/// in the root of the model.
///
/// The blocks inside a container are not here: they stay with the container
/// that owns them, and the frame of the container grows to hold them.
fn free_blocks(model: &GmdlModel, root: u32, edges: &[(u32, u32)]) -> Vec<u32> {
    let mut free: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.id != root && !block.is_container() && block.owner == root)
        .filter(|block| !edges.iter().any(|(from, to)| *from == block.id || *to == block.id))
        .map(|block| block.id)
        .collect();
    free.sort_unstable();
    free
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

/// The component of every block of the flow.
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
    let entities = entities_of(model, root);
    // The flow between the entities: two blocks of the same container are
    // one entity, and they move together.
    let entity_edges: Vec<(u32, u32)> = edges
        .iter()
        .filter_map(|(from, to)| Some((entities.get(from)?, entities.get(to)?)))
        .map(|(from, to)| (*from, *to))
        .collect();
    let component = components_of(&entity_edges);
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
fn place_free(positions: &mut Vec<Position>, model: &GmdlModel, free: &[u32]) {
    if free.is_empty() {
        return;
    }
    // The size of the cells of the grid: the biggest of the free blocks.
    let mut cell_width = 0;
    let mut cell_height = 0;
    for id in free {
        let (width, height) = size_of(model, *id);
        cell_width = cell_width.max(width);
        cell_height = cell_height.max(height);
    }
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
        let width = columns as i32 * (cell_width + GAP_X) - GAP_X;
        let height = rows as i32 * (cell_height + GAP_Y) - GAP_Y;
        for side in [Side::Left, Side::Above] {
            let (total_width, total_height) = match (&flow, side) {
                (None, _) => (width, height),
                (Some((left, top, right, bottom)), Side::Left) => (
                    width + GAP_X + (right - left),
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
            if best.as_ref().map_or(true, |other| candidate.squarer_than(other)) {
                best = Some(candidate);
            }
        }
    }
    let Some(grid) = best else {
        return;
    };
    // The place of the grid: beside the flow or above it, at the top.
    let (origin_x, origin_y) = match (&flow, grid.side) {
        (Some((left, top, _, _)), Side::Left) => (*left - GAP_X - grid.width, *top),
        (Some((left, top, _, _)), Side::Above) => (*left, *top - GAP_Y - grid.height),
        (None, _) => (0, 0),
    };
    for (index, id) in free.iter().enumerate() {
        let column = index % grid.columns;
        let row = index / grid.columns;
        let Some(position) = positions.iter_mut().find(|position| position.id == *id) else {
            continue;
        };
        position.x = origin_x + column as i32 * (cell_width + GAP_X);
        position.y = origin_y + row as i32 * (cell_height + GAP_Y);
    }
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
        frame_of(block.id, model, &boxes, &mut frames);
    }
    frames
}

/// The frame of a container: the box of the blocks it holds, with the margin
/// that separates them.
///
/// A container that holds another container holds that container's frame, so
/// the frames are taken from the innermost out.
fn frame_of(
    container: u32,
    model: &GmdlModel,
    boxes: &HashMap<u32, (i32, i32, i32, i32)>,
    frames: &mut HashMap<u32, (i32, i32, i32, i32)>,
) -> Option<(i32, i32, i32, i32)> {
    let children: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.owner == container && block.id != container)
        .map(|block| block.id)
        .collect();
    let mut box_of: Option<(i32, i32, i32, i32)> = None;
    for child in children {
        let corners = match frames.get(&child) {
            // The frame is stored as its corner and its size; the box of the
            // children is their corners.
            Some((x, y, width, height)) => Some((*x, *y, x + width, y + height)),
            None => match model.block(child) {
                Some(block) if block.is_container() => {
                    // The walk returns the corners of the frame.
                    frame_of(child, model, boxes, frames)
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
    let frame = (x1 - PAD, y1 - PAD, x2 - x1 + 2 * PAD, y2 - y1 + 2 * PAD);
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
