//! The composition of the model's canvas.
//!
//! Where a block sits on the canvas and how big it is is the engine's
//! business: a model that does not carry that geometry is still a model, and
//! the engine composes it so that the editor can draw it. The composition is
//! deterministic (the same model always produces the same positions), it never
//! touches the semantics (the geometry does not enter the simulation) and it
//! leaves a model that is already composed exactly as its author left it.
//!
//! The blocks of every container are placed in layers: a block with no
//! incoming link is in the first layer, and a block that follows another one is
//! in the layer after it. Inside a layer the blocks keep their order, so the
//! composition reorders the flow and not the reading order. The containers grow
//! to hold the blocks they own, and the model root (which the editor does not
//! draw) is left alone.

use std::collections::{HashMap, HashSet};

use crate::edit::Position;
use crate::model::{Block, GmdlModel, Layout};

/// Horizontal gap between two layers.
pub const GAP_X: i32 = 60;
/// Vertical gap between two blocks of the same layer.
pub const GAP_Y: i32 = 30;
/// Margin between a container and the blocks it holds.
pub const PAD: i32 = 24;
/// Width of a block that has no geometry of its own.
pub const DEFAULT_WIDTH: i32 = 120;
/// Height of a block that has no geometry of its own.
pub const DEFAULT_HEIGHT: i32 = 48;

/// The size of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Size {
    width: i32,
    height: i32,
}

/// The size a block that carries no geometry of its own gets.
const DEFAULT_SIZE: Size = Size {
    width: DEFAULT_WIDTH,
    height: DEFAULT_HEIGHT,
};

/// The positions that compose the model, one per block except the root.
///
/// The positions are sorted by block id, so the same model always produces the
/// same list in the same order.
pub fn relayout(model: &GmdlModel) -> Vec<Position> {
    let Some(root) = model.root() else {
        return Vec::new();
    };
    let blocks: HashMap<u32, &Block> = model
        .blocks
        .iter()
        .map(|block| (block.id, block))
        .collect();
    // The size of every block, from the leaves up: a container is as big as
    // the blocks it holds, plus the margin that separates them.
    let mut sizes: HashMap<u32, Size> = HashMap::new();
    for block in &model.blocks {
        if block.id != root.id {
            block_size(&blocks, block.id, &mut sizes);
        }
    }
    let (origin_x, origin_y) = origin(model);
    let mut positions = Vec::new();
    place(
        &blocks,
        root.id,
        origin_x,
        origin_y,
        &sizes,
        &mut positions,
    );
    positions.sort_by_key(|position| position.id);
    positions
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

/// The blocks a container owns, in the order they are composed.
fn children_of(blocks: &HashMap<u32, &Block>, container: u32) -> Vec<u32> {
    let mut children: Vec<u32> = blocks
        .values()
        .filter(|block| block.owner == container && block.id != container)
        .map(|block| block.id)
        .collect();
    children.sort_unstable();
    children
}

/// The links a container owns, as the pairs of blocks they join.
fn edges_of(blocks: &HashMap<u32, &Block>, container: u32) -> Vec<(u32, u32)> {
    blocks
        .get(&container)
        .map(|block| {
            block
                .links
                .iter()
                .map(|link| (link.from_module, link.to_module))
                .collect()
        })
        .unwrap_or_default()
}

/// The size of a block: its own, or the size that holds what it owns.
fn block_size(
    blocks: &HashMap<u32, &Block>,
    id: u32,
    memo: &mut HashMap<u32, Size>,
) -> Size {
    if let Some(size) = memo.get(&id) {
        return *size;
    }
    let size = match blocks.get(&id) {
        Some(block) if block.is_container() => {
            let (width, height) = content_size(blocks, id, memo);
            Size {
                width: width + 2 * PAD,
                height: height + 2 * PAD,
            }
        }
        Some(block) => match block.layout {
            Some(layout) => Size {
                width: layout.width,
                height: layout.height,
            },
            None => DEFAULT_SIZE,
        },
        None => DEFAULT_SIZE,
    };
    memo.insert(id, size);
    size
}

/// The space the blocks of a container take, arranged in layers.
fn content_size(
    blocks: &HashMap<u32, &Block>,
    container: u32,
    memo: &mut HashMap<u32, Size>,
) -> (i32, i32) {
    let children = children_of(blocks, container);
    let layers = layers_of(&children, &edges_of(blocks, container));
    let mut width = 0;
    let mut height = 0;
    for (index, layer) in layers.iter().enumerate() {
        if index > 0 {
            width += GAP_X;
        }
        let mut layer_width = 0;
        let mut layer_height = 0;
        for (position, child) in layer.iter().enumerate() {
            let size = block_size(blocks, *child, memo);
            layer_width = layer_width.max(size.width);
            layer_height += size.height;
            if position > 0 {
                layer_height += GAP_Y;
            }
        }
        width += layer_width;
        height = height.max(layer_height);
    }
    (width, height)
}

/// Place the blocks of a container, and the blocks of the containers it owns.
fn place(
    blocks: &HashMap<u32, &Block>,
    container: u32,
    origin_x: i32,
    origin_y: i32,
    sizes: &HashMap<u32, Size>,
    out: &mut Vec<Position>,
) {
    let children = children_of(blocks, container);
    let layers = layers_of(&children, &edges_of(blocks, container));
    let mut x = origin_x;
    let mut placed: HashMap<u32, (i32, i32)> = HashMap::new();
    for layer in &layers {
        let mut y = origin_y;
        let mut layer_width = 0;
        for child in layer {
            let size = sizes.get(child).copied().unwrap_or(DEFAULT_SIZE);
            out.push(Position {
                id: *child,
                x,
                y,
                width: Some(size.width),
                height: Some(size.height),
            });
            placed.insert(*child, (x, y));
            layer_width = layer_width.max(size.width);
            y += size.height + GAP_Y;
        }
        x += layer_width + GAP_X;
    }
    // The blocks of a container that is itself a container.
    for child in children {
        let is_container = blocks
            .get(&child)
            .map(|block| block.is_container())
            .unwrap_or(false);
        if !is_container {
            continue;
        }
        let Some((x, y)) = placed.get(&child).copied() else {
            continue;
        };
        place(blocks, child, x + PAD, y + PAD, sizes, out);
    }
}

/// The layers of the blocks of one container.
///
/// A block with no incoming link between the blocks of the container is in the
/// first layer; a block that follows another one is one layer deeper. A cycle
/// (the editor allows one) is broken at the block the walk meets again, so the
/// result is still deterministic.
fn layers_of(children: &[u32], edges: &[(u32, u32)]) -> Vec<Vec<u32>> {
    let ids: HashSet<u32> = children.iter().copied().collect();
    let mut incoming: HashMap<u32, Vec<u32>> = HashMap::new();
    for (from, to) in edges {
        if !ids.contains(from) || !ids.contains(to) || from == to {
            continue;
        }
        incoming.entry(*to).or_default().push(*from);
    }
    let mut layer_of: HashMap<u32, usize> = HashMap::new();
    let mut visiting: HashSet<u32> = HashSet::new();
    for child in children {
        depth(
            *child,
            &incoming,
            &mut layer_of,
            &mut visiting,
        );
    }
    let deepest = children
        .iter()
        .filter_map(|child| layer_of.get(child).copied())
        .max()
        .unwrap_or(0);
    let mut layers: Vec<Vec<u32>> = vec![Vec::new(); deepest + 1];
    for child in children {
        let layer = layer_of.get(child).copied().unwrap_or(0);
        layers[layer].push(*child);
    }
    layers
}

/// How deep a block is: the layer it belongs to.
fn depth(
    id: u32,
    incoming: &HashMap<u32, Vec<u32>>,
    layer_of: &mut HashMap<u32, usize>,
    visiting: &mut HashSet<u32>,
) -> usize {
    if let Some(known) = layer_of.get(&id) {
        return *known;
    }
    // The walk meets the block again: the links have a cycle, and the block
    // stays in the first layer so the result is still deterministic.
    if !visiting.insert(id) {
        return 0;
    }
    let sources = incoming.get(&id).cloned().unwrap_or_default();
    let value = if sources.is_empty() {
        0
    } else {
        1 + sources
            .iter()
            .map(|source| depth(*source, incoming, layer_of, visiting))
            .max()
            .unwrap_or(0)
    };
    visiting.remove(&id);
    layer_of.insert(id, value);
    value
}
