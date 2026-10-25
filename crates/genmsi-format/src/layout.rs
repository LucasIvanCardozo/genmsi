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
//! crosses the fewest links (`layout::order`). The layers are columns, one
//! after the other, and inside a column every block goes to the place its
//! neighbours ask for, so the links are short and straight. The blocks that
//! no link reaches are not part of the flow: they are put aside, in a grid,
//! so they do not stretch the picture of the flow. The containers grow to
//! hold the blocks they own, and the model root (which the editor does not
//! draw) is left alone.

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
/// Gap between the flow and the blocks that are in no flow.
pub const GAP_ORPHANS: i32 = 120;
/// Gap between two bands of the flow.
pub const BAND_GAP: i32 = 80;
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
    let mut composition = Composition::of(model);
    let (origin_x, origin_y) = origin(model);
    let mut positions = Vec::new();
    composition.place(root.id, origin_x, origin_y, &mut positions);
    // The composition starts where the author put things, and never before
    // it: the picture has no negative coordinates.
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

/// The content of a container: where its children go, relative to it.
#[derive(Debug, Clone, Default)]
struct Content {
    /// The children: their id, their `x` and their `y`.
    children: Vec<(u32, i32, i32)>,
    /// The width the children take.
    width: i32,
    /// The height the children take.
    height: i32,
}

/// The composition of a model: the layers of every container, the size of
/// every block and the place of every child.
struct Composition<'a> {
    /// The blocks of the model, by id.
    blocks: HashMap<u32, &'a Block>,
    /// The flow of the model: how deep in it every block is.
    flow: Flow,
    /// The layers of every container, in order.
    layers: HashMap<u32, Vec<Vec<u32>>>,
    /// The blocks of every container that no link reaches.
    orphans: HashMap<u32, Vec<u32>>,
    /// The size of every block.
    sizes: HashMap<u32, Size>,
    /// The content of every container.
    contents: HashMap<u32, Content>,
}

impl<'a> Composition<'a> {
    /// The composition of a model, with the size of every block.
    fn of(model: &'a GmdlModel) -> Composition<'a> {
        let blocks: HashMap<u32, &Block> = model
            .blocks
            .iter()
            .map(|block| (block.id, block))
            .collect();
        let mut composition = Composition {
            blocks,
            flow: Flow::of(model),
            layers: HashMap::new(),
            orphans: HashMap::new(),
            sizes: HashMap::new(),
            contents: HashMap::new(),
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
            composition.size_of(id);
        }
        composition
    }

    /// The layers of a container: the blocks it owns, in the layers the flow
    /// puts them in, and in the order that crosses the fewest links.
    ///
    /// The blocks that no link reaches are not part of the flow: they are
    /// apart, in their own place.
    fn layers_of(&mut self, container: u32) -> Vec<Vec<u32>> {
        if let Some(layers) = self.layers.get(&container) {
            return layers.clone();
        }
        let orphans = self.orphans_of(container);
        let mut layers = self.flow.layers_of(container);
        for layer in &mut layers {
            layer.retain(|block| !orphans.contains(block));
        }
        layers.retain(|layer| !layer.is_empty());
        let edges = self.flow.inner_edges(container);
        order::order(&mut layers, &edges);
        self.layers.insert(container, layers.clone());
        layers
    }

    /// The blocks of a container that no link reaches.
    fn orphans_of(&mut self, container: u32) -> Vec<u32> {
        if let Some(orphans) = self.orphans.get(&container) {
            return orphans.clone();
        }
        let orphans: Vec<u32> = self
            .flow
            .children_of(container)
            .into_iter()
            .filter(|block| self.flow.is_isolated(*block))
            .collect();
        self.orphans.insert(container, orphans.clone());
        orphans
    }

    /// The size of a block: its own, or the size that holds what it owns.
    fn size_of(&mut self, id: u32) -> Size {
        if let Some(size) = self.sizes.get(&id) {
            return *size;
        }
        let size = match self.blocks.get(&id) {
            Some(block) if block.is_container() => {
                let content = self.content_of(id);
                Size {
                    width: content.width + 2 * PAD,
                    height: content.height + 2 * PAD,
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

    /// The place of the children of a container, relative to the container.
    ///
    /// The layers of the flow are packed into **columns**: a layer that
    /// follows another one goes under it, and the columns go one after the
    /// other, from left to right. The flow is read the way it is written:
    /// down a column, and then the next column.
    ///
    /// The picture of a flow is read at a glance, so it has to be close to
    /// what a reader expects: a shape that is neither a very long line nor a
    /// very tall column. The number of columns is the one that makes the
    /// picture the closest to a square.
    fn content_of(&mut self, container: u32) -> Content {
        if let Some(content) = self.contents.get(&container) {
            return content.clone();
        }
        let layers = self.layers_of(container);
        let orphans = self.orphans_of(container);
        let mut children: Vec<(u32, i32, i32)> = Vec::new();
        let mut top = i32::MAX;
        let mut bottom = i32::MIN;
        let mut right = 0;
        // The flow, packed into columns.
        let mut x = 0;
        for column in self.columns_of(&layers) {
            let mut y = 0;
            let mut column_width = 0;
            for layer in &column {
                for block in layer {
                    let size = self.size_of(*block);
                    children.push((*block, x, y));
                    top = top.min(y);
                    bottom = bottom.max(y + size.height);
                    right = right.max(x + size.width);
                    column_width = column_width.max(size.width);
                    y += size.height + GAP_Y;
                }
            }
            x += column_width + GAP_X;
        }
        // The blocks that are in no flow go aside, in a grid, so that they do
        // not stretch the picture of the flow.
        if !orphans.is_empty() {
            let (cell_width, cell_height) = self.grid_of(&orphans);
            let grid_x = right + GAP_ORPHANS;
            let columns = grid_columns(orphans.len());
            for (index, block) in orphans.iter().enumerate() {
                let column = (index % columns) as i32;
                let row = (index / columns) as i32;
                let y = row * (cell_height + GAP_Y);
                children.push((
                    *block,
                    grid_x + column * (cell_width + GAP_X),
                    y,
                ));
                let size = self.size_of(*block);
                top = top.min(y);
                bottom = bottom.max(y + size.height);
                right = right.max(grid_x + column * (cell_width + GAP_X) + size.width);
            }
        }
        if children.is_empty() {
            top = 0;
            bottom = 0;
        }
        // The content starts at the top of its box.
        for (_, _, y) in &mut children {
            *y -= top;
        }
        let content = Content {
            children,
            width: right.max(0),
            height: (bottom - top).max(0),
        };
        self.contents.insert(container, content.clone());
        content
    }

    /// The columns of the flow: the layers, packed into columns.
    ///
    /// The picture of a flow is read at a glance, so it has to be close to
    /// what a reader expects: a shape that is neither a very long line nor a
    /// very tall column. The number of columns is the one that makes the
    /// picture the closest to a square.
    fn columns_of(&mut self, layers: &[Vec<u32>]) -> Vec<Vec<Vec<u32>>> {
        if layers.len() < 2 {
            return layers.iter().map(|layer| vec![layer.clone()]).collect();
        }
        let blocks: usize = layers.iter().map(|layer| layer.len()).sum();
        let mut width = 0;
        let mut height = 0;
        for layer in layers {
            for block in layer {
                let size = self.size_of(*block);
                width += size.width;
                height += size.height;
            }
        }
        let width = width as f64 / blocks.max(1) as f64 + f64::from(GAP_X);
        let height = height as f64 / blocks.max(1) as f64 + f64::from(GAP_Y);
        let columns = ((blocks as f64 * height / width).sqrt().round() as i32).max(1) as usize;
        let columns = columns.min(layers.len());
        // The layers, in order, spread over the columns.
        let per_column = blocks.div_ceil(columns);
        let mut result: Vec<Vec<Vec<u32>>> = Vec::new();
        let mut current: Vec<Vec<u32>> = Vec::new();
        let mut count = 0;
        for layer in layers {
            current.push(layer.clone());
            count += layer.len();
            if count >= per_column && result.len() + 1 < columns {
                result.push(std::mem::take(&mut current));
                count = 0;
            }
        }
        if !current.is_empty() {
            result.push(current);
        }
        result
    }

    /// The size of the cell of the grid of the blocks that are in no flow.
    fn grid_of(&mut self, orphans: &[u32]) -> (i32, i32) {
        let mut width = 0;
        let mut height = 0;
        for block in orphans {
            let size = self.size_of(*block);
            width = width.max(size.width);
            height = height.max(size.height);
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
        let content = self.content_of(container);
        for (block, x, y) in content.children {
            let size = self.size_of(block);
            out.push(Position {
                id: block,
                x: origin_x + x,
                y: origin_y + y,
                width: Some(size.width),
                height: Some(size.height),
            });
            let is_container = self
                .blocks
                .get(&block)
                .map(|block| block.is_container())
                .unwrap_or(false);
            if is_container {
                self.place(block, origin_x + x + PAD, origin_y + y + PAD, out);
            }
        }
    }
}

/// How many columns the grid of the blocks that are in no flow has.
fn grid_columns(count: usize) -> usize {
    ((count as f64).sqrt().ceil() as usize).max(1)
}

