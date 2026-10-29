//! The pairs of activity blocks the links of a container carry.
//!
//! The compiler resolves the borders of a container while it wires the
//! program, and the model document publishes, for every link, the pairs of
//! activity blocks the link carries. The two are the same truth said twice:
//! the program sends the flow to a block, and the document says which block
//! that is on the link the flow travels.
//!
//! The models pinned here are the ones with the awkward shapes:
//!
//! - a container inside a container, so the flow crosses two borders;
//! - a container whose exit port is fed by several blocks of its inside, so
//!   one link carries several pairs;
//! - an entry port with no internal link, so the flow starts at the first
//!   child of the container.

use genmsi_core::documents::{ModelDocument, model_document};
use genmsi_core::program::{Program, compile};
use genmsi_format::{GmdlModel, parse_str};
use std::path::Path;

/// A container inside a container.
///
////// The container `Caja1` (id 19) lives inside the container `FreeShop`
/// (id 13), and the container `Buffet` (id 4) lives in the scope above.
/// The flow enters `Buffet` at the block `Queue2` (id 9) and leaves it at
/// the block `Transfer11` (id 11).
const AEROPUERTO: &str = "../../fixtures/models/TP3__aeropuerto.gmdl";

/// A model with a container that is entered and left, and a container inside
/// it.
///
/// The flow goes from the generator (id 1) into the container (id 2), through
/// the blocks it holds (ids 5 and 6), out of it and into the other container
/// (id 3), through the block it holds (id 7) and out to the terminate block
/// (id 4). The link that leaves the first container and enters the second
/// crosses two borders.
const NESTED: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links>
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>
<link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/>
<link frommoduleid="3" fromioid="2" tomoduleid="4" toioid="1"/>
</links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Contenedor"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links>
<link frommoduleid="2" fromioid="1" tomoduleid="5" toioid="1"/>
<link frommoduleid="5" fromioid="2" tomoduleid="6" toioid="1"/>
<link frommoduleid="6" fromioid="2" tomoduleid="2" toioid="2"/>
</links>
</module>
<module moduleid="3" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links>
<link frommoduleid="3" fromioid="1" tomoduleid="7" toioid="1"/>
<link frommoduleid="7" fromioid="2" tomoduleid="3" toioid="2"/>
</links>
</module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs>
<props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="5" moduleclass="Advance" owner="2" caption="a" name="Advance" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="6" moduleclass="Advance" owner="2" caption="a" name="Advance2" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="7" moduleclass="Advance" owner="3" caption="a" name="Advance3" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

/// The airport of the reference models.
fn airport() -> GmdlModel {
    genmsi_format::load(Path::new(AEROPUERTO)).expect("the reference model loads")
}

/// The nested model.
fn nested() -> GmdlModel {
    parse_str(NESTED).expect("the model parses")
}

/// The block of the program a flow reaches, when the program sends it there.
///
/// The program wires the last statement of a block with the targets of its
/// output ports, so the target of a port is the `next` of that statement.
fn target_of(program: &Program, module_id: u32, port: usize) -> Option<u32> {
    let last = program
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.module_id == module_id)
        .map(|(index, _)| index)
        .next_back()?;
    program.blocks[last]
        .next
        .get(port)
        .copied()
        .flatten()
        .map(|target| program.blocks[target].module_id)
}

/// The pairs the links of the document publish, by the id of the link.
fn pairs_of(document: &ModelDocument) -> Vec<(String, Vec<(u32, u32)>)> {
    document
        .edges
        .iter()
        .map(|edge| (edge.id.clone(), edge.hops.clone()))
        .collect()
}

#[test]
fn the_document_and_the_program_say_the_same_thing_about_the_airport() {
    let model = airport();
    let program = compile(&model, "aeropuerto").expect("the airport compiles");
    let document = model_document(&model, AEROPUERTO);
    let pairs = pairs_of(&document);
    let mut checked = 0;
    for block in model.activity_blocks() {
        let owner = model.block(block.owner).expect("the owner exists");
        for (index, port) in block.outputs.iter().enumerate() {
            let Some(target) = target_of(&program, block.id, index) else {
                continue;
            };
            let Some(link) = owner
                .links
                .iter()
                .find(|link| link.from_module == block.id && link.from_ioid == port.ioid)
            else {
                continue;
            };
            let id = format!(
                "{}:{}:{}->{}:{}",
                owner.id, link.from_module, link.from_ioid, link.to_module, link.to_ioid
            );
            let hops = pairs
                .iter()
                .find(|(edge, _)| *edge == id)
                .map(|(_, hops)| hops.clone())
                .unwrap_or_default();
            assert!(
                hops.contains(&(block.id, target)),
                "the link {id} does not carry the pair ({}, {target})",
                block.id
            );
            checked += 1;
        }
    }
    assert!(checked > 50, "the airport has {checked} links to check");
}

#[test]
fn the_document_and_the_program_say_the_same_thing_about_a_nested_container() {
    let model = nested();
    let program = compile(&model, "nested").expect("the model compiles");
    let document = model_document(&model, "nested.gmdl");
    let pairs = pairs_of(&document);
    // The flow enters the outer container at the block inside it, and the
    // link that enters the container carries the pair (outside, inside).
    let entering = pairs
        .iter()
        .find(|(id, _)| id == "0:1:1->2:1")
        .map(|(_, hops)| hops.clone())
        .expect("the link that enters the container");
    assert_eq!(entering, vec![(1, 5)], "the pair of the move that enters");
    // The internal link of the container carries the same pair.
    let internal = pairs
        .iter()
        .find(|(id, _)| id == "2:2:1->5:1")
        .map(|(_, hops)| hops.clone())
        .expect("the internal link of the entry port");
    assert_eq!(internal, vec![(1, 5)], "the pair of the same move");
    // The container inside the container: the flow that leaves the first
    // container and enters the second crosses two borders, and the link that
    // does it carries the pair (the block inside the first, the block inside
    // the second).
    let nested_entering = pairs
        .iter()
        .find(|(id, _)| id == "0:2:2->3:1")
        .map(|(_, hops)| hops.clone())
        .expect("the link that leaves one container and enters the other");
    assert_eq!(
        nested_entering,
        vec![(6, 7)],
        "the pair across two borders"
    );
    // And the program sends the flow to the same block.
    assert_eq!(target_of(&program, 6, 0), Some(7), "the program agrees");
}

#[test]
fn a_link_that_leaves_a_container_carries_every_pair_that_leaves_it() {
    let model = airport();
    let document = model_document(&model, AEROPUERTO);
    // The container `Buffet` (id 4) is left by the block `Depart2` (id 8),
    // and the flow continues in the scope above at the block `Transfer11`
    // (id 11).
    let leaving = document
        .edges
        .iter()
        .find(|edge| edge.id == "0:4:2->11:1")
        .expect("the link that leaves the container");
    assert_eq!(leaving.hops, vec![(8, 11)], "the pair of the move that leaves");
}
