//! The composition of one scope of the model.
//!
//! The reader sees one level of the model at a time: the root, or the content
//! of a container. The composition of a scope moves the blocks of that scope
//! and nothing else, so pressing "rearrange" inside a container does not move
//! the blocks of the root.
//!
//! The rules these tests pin:
//!
//! 1. composing a scope touches the blocks of the scope and the scope's own
//!    frame, and nothing else;
//! 2. a container nested in the scope is an atomic box: its content is
//!    neither looked at nor moved;
//! 3. the root scope composes the children of the root, with the containers
//!    among them as atomic boxes;
//! 4. the composition of a scope is deterministic;
//! 5. a scope with nothing in it composes nothing, and does not panic.

use std::path::{Path, PathBuf};

use genmsi_format::layout::relayout_scope;
use genmsi_format::{GmdlModel, Layout, Position};

/// The reference model with that name.
fn model(name: &str) -> GmdlModel {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models")
        .join(name);
    let text = std::fs::read_to_string(&path).expect("the reference model is there");
    genmsi_format::parse_str(&text).expect("the reference model parses")
}

/// The geometry of a block: its corner and its size.
fn layout_of(model: &GmdlModel, id: u32) -> Option<(i32, i32, i32, i32)> {
    model
        .block(id)
        .and_then(|block| block.layout)
        .map(|layout| (layout.x, layout.y, layout.width, layout.height))
}

/// The model with the positions applied.
fn composed(model: &GmdlModel, positions: &[Position]) -> GmdlModel {
    let mut model = model.clone();
    for position in positions {
        let Some(block) = model.block_mut(position.id) else {
            continue;
        };
        let current = block.layout;
        block.layout = Some(Layout {
            width: position
                .width
                .unwrap_or_else(|| current.map(|layout| layout.width).unwrap_or(120)),
            height: position
                .height
                .unwrap_or_else(|| current.map(|layout| layout.height).unwrap_or(48)),
            x: position.x,
            y: position.y,
        });
    }
    model
}

/// The children of a scope: the blocks it owns.
fn children_of(model: &GmdlModel, scope: u32) -> Vec<u32> {
    let mut children: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.owner == scope && block.id != scope)
        .map(|block| block.id)
        .collect();
    children.sort_unstable();
    children
}

/// The blocks whose geometry the composition changed.
fn moved(model: &GmdlModel, positions: &[Position]) -> Vec<u32> {
    let composed = composed(model, positions);
    model
        .blocks
        .iter()
        .filter(|block| layout_of(model, block.id) != layout_of(&composed, block.id))
        .map(|block| block.id)
        .collect()
}

#[test]
fn composing_a_scope_moves_only_the_blocks_of_the_scope() {
    // The container 1 of the fishing contest and the container 13 of the
    // airport: both hold blocks, and both have blocks of their own.
    for (name, scope) in [("TP10__tp10.gmdl", 1u32), ("TP3__aeropuerto.gmdl", 13)] {
        let model = model(name);
        let positions = relayout_scope(&model, scope);
        assert!(
            !positions.is_empty(),
            "{name}: the scope {scope} has nothing to compose"
        );
        // The blocks of the scope and the scope's own frame are the ones the
        // composition writes.
        let children = children_of(&model, scope);
        for position in &positions {
            assert!(
                children.contains(&position.id) || position.id == scope,
                "{name}: the composition writes the block {}, which the scope {scope} does not own",
                position.id
            );
        }
        // Every other block keeps the place it had.
        let composed = composed(&model, &positions);
        for block in &model.blocks {
            if children.contains(&block.id) || block.id == scope {
                continue;
            }
            assert_eq!(
                layout_of(&model, block.id),
                layout_of(&composed, block.id),
                "{name}: the block {}, which is outside the scope {scope}, moved",
                block.id
            );
        }
    }
}

#[test]
fn the_composition_of_a_scope_moves_the_children_and_the_frame_of_the_scope() {
    // The measure of the rule: composing the scope of a container moves the
    // children of the container and the frame of the container, and nothing
    // else. The fishing contest has 87 blocks; 9 of them are the children of
    // the container 1.
    let model = model("TP10__tp10.gmdl");
    let scope = 1;
    let positions = relayout_scope(&model, scope);
    let mut moved = moved(&model, &positions);
    let mut expected = children_of(&model, scope);
    expected.push(scope);
    expected.sort_unstable();
    moved.sort_unstable();
    assert_eq!(
        moved, expected,
        "the composition of the scope moved the blocks it should not move"
    );
    assert_eq!(
        moved.len(),
        10,
        "ten blocks move: the nine children and the frame"
    );
}

#[test]
fn a_nested_container_is_an_atomic_box() {
    // The container 13 of the airport holds three containers (19, 29 and 40):
    // they are atomic boxes in the composition of the scope 13.
    let model = model("TP3__aeropuerto.gmdl");
    let scope = 13;
    let nested = 19;
    let positions = relayout_scope(&model, scope);
    let composed = composed(&model, &positions);
    // The nested container keeps the size the model gives it: its frame is
    // not computed around its content, which is in another canvas.
    let before = layout_of(&model, nested).expect("the nested container has a frame");
    let after = layout_of(&composed, nested).expect("the nested container has a frame");
    assert_eq!(
        (after.2, after.3),
        (before.2, before.3),
        "the nested container changed its size"
    );
    // Its content does not move: it belongs to the scope of the container
    // itself.
    for child in children_of(&model, nested) {
        assert_eq!(
            layout_of(&model, child),
            layout_of(&composed, child),
            "the block {child}, inside the nested container, moved"
        );
    }
}

#[test]
fn the_root_scope_composes_the_children_of_the_root() {
    let model = model("TP10__tp10.gmdl");
    let positions = relayout_scope(&model, 0);
    let children = children_of(&model, 0);
    let ids: Vec<u32> = positions.iter().map(|position| position.id).collect();
    assert_eq!(ids, children, "the root scope composes its own children");
    // The root itself is not drawn: it has no frame.
    assert!(
        !positions.iter().any(|position| position.id == 0),
        "the root of the model is drawn"
    );
    // The containers among the children are atomic boxes: the size is the
    // one the model gives them.
    for id in &children {
        let Some(block) = model.block(*id) else {
            continue;
        };
        if !block.is_container() {
            continue;
        }
        let Some(layout) = block.layout else {
            continue;
        };
        let position = positions
            .iter()
            .find(|position| position.id == *id)
            .expect("the container is placed");
        assert_eq!(
            (position.width, position.height),
            (Some(layout.width), Some(layout.height)),
            "the container {id} changed its size"
        );
    }
}

#[test]
fn the_composition_of_a_scope_is_deterministic() {
    for (name, scope) in [
        ("TP10__tp10.gmdl", 1u32),
        ("TP3__aeropuerto.gmdl", 13),
        ("TP10__tp10.gmdl", 0),
    ] {
        let model = model(name);
        let first = relayout_scope(&model, scope);
        let second = relayout_scope(&model, scope);
        assert_eq!(
            first, second,
            "{name}: the composition of the scope {scope} is not deterministic"
        );
    }
}

#[test]
fn a_scope_with_nothing_in_it_composes_nothing() {
    let model = model("TP10__tp10.gmdl");
    // A block that is not a container has no children.
    assert!(
        relayout_scope(&model, 2).is_empty(),
        "a block that is not a container composes something"
    );
    // A scope that is not in the model has no children either.
    assert!(
        relayout_scope(&model, 9999).is_empty(),
        "a scope that is not in the model composes something"
    );
    // A container that holds nothing: there is nothing to place, and no
    // frame to draw around it.
    let text = "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"\" owner=\"0\" caption=\"Grupo\" name=\"Grupo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         </modules></gmodel>";
    let empty = genmsi_format::parse_str(text).expect("the model parses");
    assert!(
        relayout_scope(&empty, 1).is_empty(),
        "an empty container composes something"
    );
    // The root of that model holds the empty container, so the root scope
    // composes it: it is a block of the root, like any other.
    let root = relayout_scope(&empty, 0);
    assert_eq!(
        root.iter()
            .map(|position| position.id)
            .collect::<Vec<u32>>(),
        vec![1],
        "the root scope composes the blocks it owns"
    );
}

#[test]
fn the_ports_of_the_scope_are_the_ends_of_the_flow() {
    // The container 13 of the airport: its entry port (ioid 1) carries the
    // flow into the block 14, and its exit port (ioid 2) is fed by the two
    // containers it holds (19 and 29).
    let model = model("TP3__aeropuerto.gmdl");
    let scope = 13;
    let positions = relayout_scope(&model, scope);
    let row = |id: u32| -> i32 {
        positions
            .iter()
            .find(|position| position.id == id)
            .map(|position| position.y)
            .expect("the block is placed")
    };
    let children = children_of(&model, scope);
    // The block the flow enters through comes first.
    let first = row(14);
    for id in &children {
        if *id == 14 {
            continue;
        }
        assert!(
            row(*id) >= first,
            "the block {id} comes before the block the flow enters through"
        );
    }
    // And the blocks the flow leaves through come last.
    let last = row(19).max(row(29));
    for id in &children {
        if *id == 19 || *id == 29 {
            continue;
        }
        assert!(
            row(*id) <= last,
            "the block {id} comes after the block the flow leaves through"
        );
    }
}

#[test]
fn the_composition_of_a_scope_does_not_depend_on_the_blocks_outside_it() {
    // The scope is what the reader is looking at: what is outside it does not
    // take part in its composition.
    let model = model("TP10__tp10.gmdl");
    let scope = 1;
    let first = relayout_scope(&model, scope);
    // Every block outside the scope is moved somewhere else.
    let mut moved_model = model.clone();
    for block in &mut moved_model.blocks {
        if block.owner == scope || block.id == scope {
            continue;
        }
        if let Some(layout) = &mut block.layout {
            layout.x += 1000;
            layout.y += 1000;
        }
    }
    let second = relayout_scope(&moved_model, scope);
    assert_eq!(
        first, second,
        "the composition of the scope depends on the blocks outside it"
    );
}

#[test]
fn the_composition_of_a_scope_leaves_no_block_outside_its_frame() {
    // The measure of the house: a block that falls out of the frame of the
    // scope that owns it is a broken composition, and two blocks that sit on
    // each other are a picture the reader cannot read. The measure is taken
    // over every scope of every reference model.
    for path in corpus() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(model) = genmsi_format::parse_str(&text) else {
            continue;
        };
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        for scope in model
            .blocks
            .iter()
            .filter(|block| block.is_container())
            .map(|block| block.id)
        {
            let positions = relayout_scope(&model, scope);
            if positions.is_empty() {
                continue;
            }
            let composed = composed(&model, &positions);
            let children = children_of(&model, scope);
            let frame = layout_of(&composed, scope);
            for id in &children {
                let (x, y, width, height) = layout_of(&composed, *id).expect("the block is placed");
                if let Some((left, top, frame_width, frame_height)) = frame {
                    assert!(
                        x >= left
                            && y >= top
                            && x + width <= left + frame_width
                            && y + height <= top + frame_height,
                        "{name}: the block {id} fell out of the frame of the scope {scope}"
                    );
                }
                for other in &children {
                    if other >= id {
                        continue;
                    }
                    let (other_x, other_y, other_width, other_height) =
                        layout_of(&composed, *other).expect("the block is placed");
                    assert!(
                        !(x < other_x + other_width
                            && other_x < x + width
                            && y < other_y + other_height
                            && other_y < y + height),
                        "{name}: the blocks {id} and {other} sit on each other in the scope {scope}"
                    );
                }
            }
        }
    }
}

/// The corpus: the reference models of the repository.
fn corpus() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the reference models are there")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "gmdl")
        })
        .collect();
    paths.sort();
    paths
}
