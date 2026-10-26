//! The measure of a composed canvas.
//!
//! What a composition produces can be measured: how many blocks sit on top
//! of each other, how many links cross, how wide the canvas ends up. The
//! measure is what tells whether a change to the algorithm is an improvement
//! or only a different picture.
//!
//! The measure never judges the model: it looks at the canvas the engine
//! composed and counts. A model that does not compile can still be measured:
//! the geometry does not enter the simulation.

use std::collections::{BTreeMap, BTreeSet};

use crate::edit::Position;
use crate::layout::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::model::GmdlModel;

/// What a composed canvas measures.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Composition {
    /// The blocks the composition placed.
    pub blocks: usize,
    /// The links between the placed blocks.
    pub edges: usize,
    /// The pairs of blocks that share part of the canvas.
    pub overlaps: usize,
    /// The blocks that are not inside the container that owns them.
    pub outside: usize,
    /// The pairs of links that cross.
    pub crossings: usize,
    /// The blocks that no link reaches: they belong to no flow.
    pub isolated: usize,
    /// The links that pass over a block: the reader has to look past them.
    pub over_blocks: usize,
    /// The average length of the links, in canvas units.
    pub edge_length: f64,
    /// The width of the canvas.
    pub width: i32,
    /// The height of the canvas.
    pub height: i32,
}

impl Composition {
    /// The proportion of the canvas: its width over its height.
    pub fn ratio(&self) -> f64 {
        if self.height <= 0 {
            return 0.0;
        }
        f64::from(self.width) / f64::from(self.height)
    }
}

/// A rectangle of the canvas.
#[derive(Debug, Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    /// The right edge of the rectangle.
    fn right(&self) -> i32 {
        self.x + self.width
    }

    /// The bottom edge of the rectangle.
    fn bottom(&self) -> i32 {
        self.y + self.height
    }

    /// Whether the rectangle holds the other one.
    fn contains(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Whether the two rectangles share part of the canvas.
    fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    /// The middle of the rectangle, where an edge leaves from.
    fn center(&self) -> (f64, f64) {
        (
            f64::from(self.x) + f64::from(self.width) / 2.0,
            f64::from(self.y) + f64::from(self.height) / 2.0,
        )
    }
}

/// Measure a composed canvas.
///
/// The positions are the ones the composition produced; the model says who
/// owns whom and which links there are.
pub fn measure(model: &GmdlModel, positions: &[Position]) -> Composition {
    let mut rects: BTreeMap<u32, Rect> = BTreeMap::new();
    for position in positions {
        rects.insert(
            position.id,
            Rect {
                x: position.x,
                y: position.y,
                width: position.width.unwrap_or(DEFAULT_WIDTH),
                height: position.height.unwrap_or(DEFAULT_HEIGHT),
            },
        );
    }
    let owners: BTreeMap<u32, u32> = model
        .blocks
        .iter()
        .map(|block| (block.id, block.owner))
        .collect();
    let mut measure = Composition {
        blocks: rects.len(),
        ..Composition::default()
    };
    // The canvas: the box that holds every block.
    if let Some(first) = rects.values().next() {
        let mut left = first.x;
        let mut top = first.y;
        let mut right = first.right();
        let mut bottom = first.bottom();
        for rect in rects.values() {
            left = left.min(rect.x);
            top = top.min(rect.y);
            right = right.max(rect.right());
            bottom = bottom.max(rect.bottom());
        }
        measure.width = right - left;
        measure.height = bottom - top;
    }
    // The blocks that sit on each other. A block and the container that owns
    // it share the canvas by design: the container is the frame around it.
    let ids: Vec<u32> = rects.keys().copied().collect();
    for (index, first) in ids.iter().enumerate() {
        for second in ids.iter().skip(index + 1) {
            if ancestors(&owners, *first).contains(second)
                || ancestors(&owners, *second).contains(first)
            {
                continue;
            }
            if rects[first].intersects(&rects[second]) {
                if std::env::var("GENMSI_DEBUG_OVERLAPS").is_ok() {
                    eprintln!("SOLAPE {first} {:?} y {second} {:?}", rects[first], rects[second]);
                }
                measure.overlaps += 1;
            }
        }
    }
    // The blocks that fell out of the container that owns them.
    for (id, rect) in &rects {
        let Some(owner) = owners.get(id) else {
            continue;
        };
        let Some(frame) = rects.get(owner) else {
            // The root is not drawn: its children have no frame to be in.
            continue;
        };
        if !frame.contains(rect) {
            measure.outside += 1;
        }
    }
    // The links between the placed blocks, and the pairs that cross.
    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for block in &model.blocks {
        for link in &block.links {
            if link.from_module == link.to_module {
                continue;
            }
            if rects.contains_key(&link.from_module) && rects.contains_key(&link.to_module) {
                edges.insert((link.from_module, link.to_module));
            }
        }
    }
    measure.edges = edges.len();
    // The links that pass over a block. A link that goes through the frame
    // of a container is not one of them: the frame is around the flow, not
    // in the way of it.
    for (from, to) in &edges {
        let a = rects[from].center();
        let b = rects[to].center();
        for (id, rect) in &rects {
            if id == from || id == to {
                continue;
            }
            if ancestors(&owners, *from).contains(id) || ancestors(&owners, *to).contains(id) {
                continue;
            }
            if crosses_block(a, b, rect) {
                measure.over_blocks += 1;
                break;
            }
        }
    }
    // The blocks that no link reaches.
    let mut reached: BTreeSet<u32> = BTreeSet::new();
    for (from, to) in &edges {
        reached.insert(*from);
        reached.insert(*to);
    }
    measure.isolated = rects.keys().filter(|id| !reached.contains(id)).count();
    // The average length of the links: the shorter they are, the more the
    // picture reads as a flow.
    if !edges.is_empty() {
        let mut total = 0.0;
        for (from, to) in &edges {
            let a = rects[from].center();
            let b = rects[to].center();
            total += (b.0 - a.0).hypot(b.1 - a.1);
        }
        measure.edge_length = total / edges.len() as f64;
    }
    let pairs: Vec<(u32, u32)> = edges.iter().copied().collect();
    for (index, first) in pairs.iter().enumerate() {
        for second in pairs.iter().skip(index + 1) {
            if first.0 == second.0 || first.1 == second.1 {
                continue;
            }
            if first.0 == second.1 || first.1 == second.0 {
                continue;
            }
            let a = rects[&first.0].center();
            let b = rects[&first.1].center();
            let c = rects[&second.0].center();
            let d = rects[&second.1].center();
            if crosses(a, b, c, d) {
                measure.crossings += 1;
            }
        }
    }
    measure
}

/// The containers above a block: its owner, the owner's owner, and so on.
fn ancestors(owners: &BTreeMap<u32, u32>, id: u32) -> BTreeSet<u32> {
    let mut above = BTreeSet::new();
    let mut current = owners.get(&id).copied();
    while let Some(owner) = current {
        if !above.insert(owner) {
            break;
        }
        current = owners.get(&owner).copied();
    }
    above
}

/// Whether the segment `a`-`b` passes over the rectangle.
fn crosses_block(a: (f64, f64), b: (f64, f64), rect: &Rect) -> bool {
    let corners = [
        (f64::from(rect.x), f64::from(rect.y)),
        (f64::from(rect.right()), f64::from(rect.y)),
        (f64::from(rect.right()), f64::from(rect.bottom())),
        (f64::from(rect.x), f64::from(rect.bottom())),
    ];
    for index in 0..4 {
        if crosses(a, b, corners[index], corners[(index + 1) % 4]) {
            return true;
        }
    }
    false
}

/// Whether the segment `a`-`b` crosses the segment `c`-`d`.
fn crosses(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    fn side(p: (f64, f64), q: (f64, f64), r: (f64, f64)) -> i64 {
        let value = (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
        if value > 0.0 {
            1
        } else if value < 0.0 {
            -1
        } else {
            0
        }
    }
    let d1 = side(c, d, a);
    let d2 = side(c, d, b);
    let d3 = side(a, b, c);
    let d4 = side(a, b, d);
    // Only a strict crossing counts: two links that touch at an endpoint, or
    // that run side by side, are not crossing.
    let opposite = |first: i64, second: i64| (first > 0 && second < 0) || (first < 0 && second > 0);
    opposite(d1, d2) && opposite(d3, d4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Layout;

    fn model_of(text: &str) -> GmdlModel {
        crate::parse_str(text).expect("the model parses")
    }

    /// A model with two blocks and the link that joins them.
    fn two_blocks() -> GmdlModel {
        model_of(
            "<gmodel versioncode=\"1\"><gclasses/><modules>\
             <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\">\
             <links/><inputs/><outputs/></module>\
             <module moduleid=\"1\" moduleclass=\"Clock\" owner=\"0\" caption=\"c\" name=\"c\" kind=\"A\">\
             <inputs/><outputs/><props/></module>\
             <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\">\
             <inputs/><outputs/><props/></module>\
             </modules></gmodel>",
        )
    }

    #[test]
    fn two_blocks_that_do_not_touch_measure_clean() {
        let mut model = two_blocks();
        model.blocks[1].layout = Some(Layout {
            width: 100,
            height: 40,
            x: 0,
            y: 0,
        });
        model.blocks[2].layout = Some(Layout {
            width: 100,
            height: 40,
            x: 300,
            y: 0,
        });
        let positions = vec![
            Position {
                id: 1,
                x: 0,
                y: 0,
                width: Some(100),
                height: Some(40),
            },
            Position {
                id: 2,
                x: 300,
                y: 0,
                width: Some(100),
                height: Some(40),
            },
        ];
        let measure = measure(&model, &positions);
        assert_eq!(measure.blocks, 2);
        assert_eq!(measure.overlaps, 0);
        assert_eq!(measure.outside, 0);
        assert_eq!(measure.crossings, 0);
        assert_eq!(measure.width, 400);
        assert_eq!(measure.height, 40);
    }

    #[test]
    fn two_blocks_that_share_the_canvas_are_one_overlap() {
        let mut model = two_blocks();
        model.blocks[1].layout = Some(Layout {
            width: 100,
            height: 40,
            x: 0,
            y: 0,
        });
        model.blocks[2].layout = Some(Layout {
            width: 100,
            height: 40,
            x: 50,
            y: 0,
        });
        let positions = vec![
            Position {
                id: 1,
                x: 0,
                y: 0,
                width: Some(100),
                height: Some(40),
            },
            Position {
                id: 2,
                x: 50,
                y: 0,
                width: Some(100),
                height: Some(40),
            },
        ];
        assert_eq!(measure(&model, &positions).overlaps, 1);
    }
}
