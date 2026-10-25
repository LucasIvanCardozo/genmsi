//! The composition of the model's canvas.

use genmsi_format::{GmdlModel, compose_incomplete, is_composed, relayout};

/// A model with two blocks and the link that joins them.
///
/// The geometry of the blocks is what the caller asks for: with no `width`,
/// `height`, `x` and `y` the model carries no geometry at all.
fn model(geometry: &str) -> GmdlModel {
    let text = format!(
        "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links><link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/></links>\
         <inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"Clock\" owner=\"0\" caption=\"Clock\" name=\"Clock\" kind=\"A\"{geometry}>\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\"{geometry}>\
         <inputs/><outputs/><props/></module>\
         </modules></gmodel>",
        geometry = geometry
    );
    genmsi_format::parse_str(&text).expect("the model parses")
}

fn layout_of(model: &GmdlModel, id: u32) -> Option<(i32, i32, i32, i32)> {
    model
        .block(id)
        .and_then(|block| block.layout)
        .map(|layout| (layout.x, layout.y, layout.width, layout.height))
}

#[test]
fn a_model_without_geometry_is_composed() {
    let mut model = model("");
    assert!(!is_composed(&model));
    let positions = compose_incomplete(&mut model).expect("the model is composed");
    assert_eq!(positions.len(), 2);
    // The root is not drawn, so it keeps no geometry.
    assert_eq!(layout_of(&model, 0), None);
    for block in [1, 2] {
        assert!(layout_of(&model, block).is_some(), "block {block} is placed");
    }
    assert!(is_composed(&model));
}

#[test]
fn the_blocks_of_a_layer_are_stacked_and_the_layers_advance() {
    let mut model = model("");
    compose_incomplete(&mut model).expect("the model is composed");
    let (x1, y1, _, _) = layout_of(&model, 1).expect("placed");
    let (x2, y2, _, _) = layout_of(&model, 2).expect("placed");
    // The link goes from the clock to the terminate: they are in two layers.
    assert_ne!(x1, x2);
    assert_eq!(y1, y2);
}

#[test]
fn the_composition_is_deterministic() {
    let mut first_model = model("");
    let first = compose_incomplete(&mut first_model).expect("composed");
    let second = relayout(&first_model);
    assert_eq!(first, second);
    let mut other = model("");
    let again = compose_incomplete(&mut other).expect("composed");
    assert_eq!(first, again);}

#[test]
fn a_composed_model_keeps_its_disposition() {
    let mut model = model(" width=\"80\" height=\"32\" x=\"236\" y=\"104\"");
    assert!(is_composed(&model));
    assert_eq!(compose_incomplete(&mut model), None);
    // The model is exactly as it was.
    assert_eq!(layout_of(&model, 1), Some((236, 104, 80, 32)));
    assert_eq!(layout_of(&model, 2), Some((236, 104, 80, 32)));
}

#[test]
fn an_unplaced_block_is_placed_next_to_the_ones_that_were() {
    let text = "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links><link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/></links>\
         <inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"Clock\" owner=\"0\" caption=\"Clock\" name=\"Clock\" kind=\"A\" width=\"80\" height=\"32\" x=\"236\" y=\"104\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         </modules></gmodel>";
    let mut model = genmsi_format::parse_str(text).expect("the model parses");
    assert!(!is_composed(&model));
    compose_incomplete(&mut model).expect("composed");
    // The block that had no geometry gets one.
    assert!(layout_of(&model, 2).is_some());
    assert!(is_composed(&model));
}

#[test]
fn a_container_grows_to_hold_the_blocks_it_owns() {
    let text = "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"main\" owner=\"0\" caption=\"Grupo\" name=\"Grupo1\" kind=\"C\">\
         <links><link frommoduleid=\"2\" fromioid=\"1\" tomoduleid=\"3\" toioid=\"1\"/></links>\
         <inputs/><outputs/></module>\
         <module moduleid=\"2\" moduleclass=\"Clock\" owner=\"1\" caption=\"Clock\" name=\"Clock\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"3\" moduleclass=\"Terminate\" owner=\"1\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         </modules></gmodel>";
    let mut model = genmsi_format::parse_str(text).expect("the model parses");
    compose_incomplete(&mut model).expect("composed");
    let (cx, cy, cw, ch) = layout_of(&model, 1).expect("the container is placed");
    let (x1, y1, w1, h1) = layout_of(&model, 2).expect("placed");
    let (x2, y2, w2, h2) = layout_of(&model, 3).expect("placed");
    // The blocks of the container are inside it.
    assert!(x1 >= cx && y1 >= cy, "the block is inside the container");
    assert!(x1 + w1 <= cx + cw, "the block is inside the container");
    assert!(y1 + h1 <= cy + ch, "the block is inside the container");
    assert!(x2 >= cx && y2 >= cy, "the block is inside the container");
    assert!(x2 + w2 <= cx + cw, "the block is inside the container");
    assert!(y2 + h2 <= cy + ch, "the block is inside the container");
    // The blocks of the container do not sit on each other.
    assert!(!(x1 < x2 + w2 && x2 < x1 + w1 && y1 < y2 + h2 && y2 < y1 + h1));
}
