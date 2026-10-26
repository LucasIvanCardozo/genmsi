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

/// Whether a place comes before another one in the reading order: the flow
/// goes down, so the row comes first and, inside a row, the column.
fn before(first: (i32, i32), second: (i32, i32)) -> bool {
    first.1 < second.1 || (first.1 == second.1 && first.0 < second.0)
}

#[test]
fn the_blocks_of_the_flow_are_placed_one_after_the_other() {
    let mut model = model("");
    compose_incomplete(&mut model).expect("the model is composed");
    let (x1, y1, _, _) = layout_of(&model, 1).expect("placed");
    let (x2, y2, _, _) = layout_of(&model, 2).expect("placed");
    // The link goes from the clock to the terminate: the terminate comes
    // after the clock, down a column or in the next one.
    assert!(
        before((x1, y1), (x2, y2)),
        "the flow does not go forward: the clock at ({x1}, {y1}), the terminate at ({x2}, {y2})"
    );
    // And the two do not sit on each other.
    assert!(
        !(x1 < x2 && x2 < x1 || y1 < y2 && y2 < y1),
        "the blocks of the flow are on each other"
    );
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

#[test]
fn the_blocks_follow_the_flow_across_the_border_of_a_container() {
    // Two containers, and a link that goes from the block of one to the
    // block of the other: the flow crosses the border, and the containers
    // have to be in the order the flow says.
    let text = "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"\" owner=\"0\" caption=\"A\" name=\"A\" kind=\"C\">\
         <links><link frommoduleid=\"4\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/></links>\
         <inputs/><outputs/></module>\
         <module moduleid=\"2\" moduleclass=\"Clock\" owner=\"1\" caption=\"Clock\" name=\"Clock\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"3\" moduleclass=\"\" owner=\"0\" caption=\"B\" name=\"B\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"4\" moduleclass=\"Terminate\" owner=\"3\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         </modules></gmodel>";
    let mut model = genmsi_format::parse_str(text).expect("the model parses");
    compose_incomplete(&mut model).expect("composed");
    // The flow goes from the block of B to the block of A: B comes first.
    let (bx, by, _, _) = layout_of(&model, 3).expect("the container B is placed");
    let (ax, ay, _, _) = layout_of(&model, 1).expect("the container A is placed");
    assert!(
        before((bx, by), (ax, ay)),
        "the container of the flow's start comes first: B at ({bx}, {by}), A at ({ax}, {ay})"
    );
}

/// Whether the two segments cross.
fn crosses(
    first: (i32, i32),
    second: (i32, i32),
    third: (i32, i32),
    fourth: (i32, i32),
) -> bool {
    fn side(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i64 {
        let value = i64::from(b.0 - a.0) * i64::from(c.1 - a.1)
            - i64::from(b.1 - a.1) * i64::from(c.0 - a.0);
        match value.cmp(&0) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
        }
    }
    let opposite = |first: i64, second: i64| {
        (first > 0 && second < 0) || (first < 0 && second > 0)
    };
    opposite(side(third, fourth, first), side(third, fourth, second))
        && opposite(side(first, second, third), side(first, second, fourth))
}

#[test]
fn the_blocks_of_a_layer_are_in_the_order_that_does_not_cross_the_links() {
    // Two blocks in a layer and two in the next one, with the links
    // crossed: the composition puts the blocks in the order that does not
    // cross them.
    let text = "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"\" owner=\"0\" caption=\"Grupo\" name=\"Grupo1\" kind=\"C\">\
         <links><link frommoduleid=\"2\" fromioid=\"1\" tomoduleid=\"5\" toioid=\"1\"/>\
         <link frommoduleid=\"3\" fromioid=\"1\" tomoduleid=\"4\" toioid=\"1\"/></links>\
         <inputs/><outputs/></module>\
         <module moduleid=\"2\" moduleclass=\"Clock\" owner=\"1\" caption=\"Clock\" name=\"Clock\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"3\" moduleclass=\"Terminate\" owner=\"1\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"4\" moduleclass=\"Terminate\" owner=\"1\" caption=\"Terminate2\" name=\"Terminate2\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"5\" moduleclass=\"Terminate\" owner=\"1\" caption=\"Terminate3\" name=\"Terminate3\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         </modules></gmodel>";
    let mut model = genmsi_format::parse_str(text).expect("the model parses");
    compose_incomplete(&mut model).expect("composed");
    let place = |id: u32| -> (i32, i32) {
        let (x, y, _, _) = layout_of(&model, id).expect("placed");
        (x, y)
    };
    // The link from the first block to the last, and the one from the second
    // to the third: they would cross if the blocks kept the order of their
    // ids.
    assert!(
        !crosses(place(2), place(5), place(3), place(4)),
        "the links of the model cross: {:?} {:?} {:?} {:?}",
        place(2),
        place(5),
        place(3),
        place(4)
    );
}

#[test]
fn a_container_inside_a_container_holds_its_own_blocks() {
    let text = "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"1\" moduleclass=\"main\" owner=\"0\" caption=\"Grupo\" name=\"Grupo1\" kind=\"C\">\
         <links/><inputs/><outputs/></module>\
         <module moduleid=\"2\" moduleclass=\"main\" owner=\"1\" caption=\"Subgrupo\" name=\"Subgrupo1\" kind=\"C\">\
         <links><link frommoduleid=\"3\" fromioid=\"1\" tomoduleid=\"4\" toioid=\"1\"/></links>\
         <inputs/><outputs/></module>\
         <module moduleid=\"3\" moduleclass=\"Clock\" owner=\"2\" caption=\"Clock\" name=\"Clock\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         <module moduleid=\"4\" moduleclass=\"Terminate\" owner=\"2\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
         <inputs/><outputs/><props/></module>\
         </modules></gmodel>";
    let mut model = genmsi_format::parse_str(text).expect("the model parses");
    compose_incomplete(&mut model).expect("composed");
    // The blocks of the inner container are inside it.
    let (cx, cy, cw, ch) = layout_of(&model, 2).expect("the inner container is placed");
    let (x1, y1, w1, h1) = layout_of(&model, 3).expect("placed");
    let (x2, y2, w2, h2) = layout_of(&model, 4).expect("placed");
    for (x, y, w, h) in [(x1, y1, w1, h1), (x2, y2, w2, h2)] {
        assert!(x >= cx && y >= cy, "the block is inside the container");
        assert!(x + w <= cx + cw, "the block is inside the container");
        assert!(y + h <= cy + ch, "the block is inside the container");
    }
    // And the inner container is inside the outer one.
    let (ox, oy, ow, oh) = layout_of(&model, 1).expect("the outer container is placed");
    assert!(cx >= ox && cy >= oy, "the inner container is inside the outer one");
    assert!(cx + cw <= ox + ow, "the inner container is inside the outer one");
    assert!(cy + ch <= oy + oh, "the inner container is inside the outer one");
}
