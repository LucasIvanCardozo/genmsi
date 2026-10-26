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

use std::collections::HashMap;

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
    // The containers are the parents of the blocks they hold.
    for block in &model.blocks {
        if block.id == root || block.owner == block.id {
            continue;
        }
        graph.set_parent(&block.id.to_string(), Some(&block.owner.to_string()));
    }
    // The flow of the model: the links between the blocks that simulate, with
    // the borders of the containers spliced, and the links that carry the
    // flow into each container.
    for (from, to) in flow::flow_edges_for(model) {
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
    // The blocks that no link reaches have no place in the flow: dagre
    // leaves them without a rank and puts them in the last row, at the
    // bottom of the picture. They go to the first row instead, at the top:
    // the picture starts with them and the flow goes down from there.
    let edges = flow::flow_edges_for(model);
    let orphans: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.id != root && !block.is_container())
        .filter(|block| block.owner == root)
        .filter(|block| !edges.iter().any(|(from, to)| *from == block.id || *to == block.id))
        .map(|block| block.id)
        .collect();
    if !orphans.is_empty() {
        let flow_blocks: Vec<&Position> = positions
            .iter()
            .filter(|position| !orphans.contains(&position.id))
            .collect();
        let first = flow_blocks.iter().map(|position| position.y).min().unwrap_or(0);
        // The row of the blocks that are in no flow: the tallest of them, so
        // the row above the flow is free of them.
        let height = positions
            .iter()
            .filter(|position| orphans.contains(&position.id))
            .map(|position| position.height.unwrap_or(DEFAULT_HEIGHT))
            .max()
            .unwrap_or(DEFAULT_HEIGHT)
            + GAP_Y;
        for position in &mut positions {
            if orphans.contains(&position.id) {
                position.y = first - height - GAP_Y;
            }
        }
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
