//! The composition of the model's canvas.
//!
//! Where a block sits on the canvas and how big it is is the engine's
//! business: a model that does not carry that geometry is still a model, and
//! the engine composes it so that the editor can draw it. The composition is
//! deterministic (the same model always produces the same positions), it never
//! touches the semantics (the geometry does not enter the simulation) and it
//! leaves a model that is already composed exactly as its author left it.
//!
//! The blocks of every container are placed in layers: the depth of a block
//! in the flow is its layer, and the depth is the depth of the whole model,
//! so a link that crosses the border of a container says where the flow goes
//! (`layout::flow`). Inside a layer the blocks are put in the order that
//! crosses the fewest links (`layout::order`), and the layers are placed one
//! after the other, each one in the column of its depth. The containers grow
//! to hold the blocks they own, and the model root (which the editor does
//! not draw) is left alone.

mod flow;
mod order;

use std::collections::HashMap;

use crate::edit::Position;
use crate::model::{Block, GmdlModel, Layout};

use flow::Flow;

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
    let mut layout = Composition::of(model);
    let (origin_x, origin_y) = origin(model);
    let mut positions = Vec::new();
    layout.place(root.id, origin_x, origin_y, &mut positions);
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

/// The composition of a model: the layers of every container and the size of
/// every block.
struct Composition<'a> {
    /// The blocks of the model, by id.
    blocks: HashMap<u32, &'a Block>,
    /// The flow of the model: how deep in it every block is.
    flow: Flow,
    /// The layers of every container, in order.
    layers: HashMap<u32, Vec<Vec<u32>>>,
    /// The size of every block.
    sizes: HashMap<u32, Size>,
}

impl<'a> Composition<'a> {
    /// The composition of a model, with the size of every block.
    fn of(model: &'a GmdlModel) -> Composition<'a> {
        let blocks: HashMap<u32, &Block> = model
            .blocks
            .iter()
            .map(|block| (block.id, block))
            .collect();
        let mut layout = Composition {
            blocks,
            flow: Flow::of(model),
            layers: HashMap::new(),
            sizes: HashMap::new(),
        };
        // The size of every block, from the leaves up: a container is as big
        // as the blocks it holds, plus the margin that separates them.
        let ids: Vec<u32> = model
            .blocks
            .iter()
            .filter(|block| !block.is_root())
            .map(|block| block.id)
            .collect();
        for id in ids {
            layout.size_of(id);
        }
        layout
    }

    /// The layers of a container: the blocks it owns, in the layers the flow
    /// puts them in, and in the order that crosses the fewest links.
    fn layers_of(&mut self, container: u32) -> Vec<Vec<u32>> {
        if let Some(layers) = self.layers.get(&container) {
            return layers.clone();
        }
        let mut layers = self.flow.layers_of(container);
        let edges = self.flow.inner_edges(container);
        order::order(&mut layers, &edges);
        self.layers.insert(container, layers.clone());
        layers
    }

    /// The size of a block: its own, or the size that holds what it owns.
    fn size_of(&mut self, id: u32) -> Size {
        if let Some(size) = self.sizes.get(&id) {
            return *size;
        }
        let size = match self.blocks.get(&id) {
            Some(block) if block.is_container() => {
                let (width, height) = self.content_size(id);
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
        self.sizes.insert(id, size);
        size
    }

    /// The space the blocks of a container take, arranged in layers.
    fn content_size(&mut self, container: u32) -> (i32, i32) {
        let layers = self.layers_of(container);
        let mut width = 0;
        let mut height = 0;
        for (index, layer) in layers.iter().enumerate() {
            if index > 0 {
                width += GAP_X;
            }
            let mut layer_width = 0;
            let mut layer_height = 0;
            for (position, child) in layer.iter().enumerate() {
                let size = self.size_of(*child);
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

    /// Place the blocks of a container, and the blocks of the containers it
    /// owns.
    fn place(
        &mut self,
        container: u32,
        origin_x: i32,
        origin_y: i32,
        out: &mut Vec<Position>,
    ) {
        let layers = self.layers_of(container);
        let mut x = origin_x;
        let mut placed: HashMap<u32, (i32, i32)> = HashMap::new();
        for layer in &layers {
            let mut y = origin_y;
            let mut layer_width = 0;
            for child in layer {
                let size = self.size_of(*child);
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
        for child in self.flow.children_of(container) {
            let is_container = self
                .blocks
                .get(&child)
                .map(|block| block.is_container())
                .unwrap_or(false);
            if !is_container {
                continue;
            }
            let Some((x, y)) = placed.get(&child).copied() else {
                continue;
            };
            self.place(child, x + PAD, y + PAD, out);
        }
    }
}
