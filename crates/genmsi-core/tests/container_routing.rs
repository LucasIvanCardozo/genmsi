//! How a transaction crosses the boundary of a container.
//!
//! A container is a frame: the links it owns decide what happens inside it,
//! and the links its owner owns decide what happens after it. The rules
//! pinned here are:
//!
//! - entering a container follows the container's own link that starts at the
//!   entry port;
//! - reaching a container from inside leaves it, and the owner decides where
//!   the transaction continues;
//! - an entry port with no internal link does not let the transaction reach
//!   the container's exit port.
//!
//! The direction of the link that reached the container tells an entry from an
//! exit, not the port id: the editor gives a container the same port id for
//! its entry and its exit port (`AddBlock` writes `inputs: [ioid 1]` and
//! `outputs: [ioid 1]`), so a container may use one id for both.

mod support;

use genmsi_core::{Program, program::compile};
use genmsi_format::{GmdlModel, parse_str};
use support::{block, run};

/// A container whose entry port has no internal link.
///
/// The container (id 2) declares an entry port (ioid 1) and an exit port
/// (ioid 2). Nothing inside it starts at the entry port: the block it holds
/// (id 3) is not linked to the entry port.
const UNCONNECTED_ENTRY: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="4" toioid="1"/></links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Contenedor"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="2" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links/></module>
<module moduleid="3" moduleclass="Advance" owner="2" caption="a" name="Advance" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs>
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="5" moduleclass="Clock" owner="0" caption="c" name="Clock" kind="A">
<props><property name="Frecuencia" hint="" value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

/// A container that is entered and left through its own ports.
///
/// The container (id 2) declares an entry port (ioid 1) and an exit port
/// (ioid 2). The entry port has an internal link to the block it holds
/// (id 3), and the block leaves the container through the exit port.
const CONNECTED_ENTRY: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="4" toioid="1"/></links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Contenedor"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links><link frommoduleid="2" fromioid="1" tomoduleid="3" toioid="1"/><link frommoduleid="3" fromioid="2" tomoduleid="2" toioid="2"/></links></module>
<module moduleid="3" moduleclass="Advance" owner="2" caption="a" name="Advance" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs>
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="5" moduleclass="Clock" owner="0" caption="c" name="Clock" kind="A">
<props><property name="Frecuencia" hint="" value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

fn compile_model(xml: &str) -> Program {
    compile(&parse_str(xml).expect("test model parses"), "container").expect("model compiles")
}

/// The container of `model` uses the same port id for its entry and its exit.
///
/// The editor builds a container that way (`AddBlock` writes an entry port and
/// an exit port with the id 1), so the engine must tell an entry from an exit
/// by the direction of the link that reached the container, not by the port id.
///
/// The file parser rejects a block with two ports of the same id, so the shape
/// is built after the model is parsed.
fn share_the_port_id(model: &mut GmdlModel) {
    let container = model.block_mut(2).expect("the container exists");
    container.outputs[0].ioid = 1;
    for link in container.links.iter_mut() {
        if link.to_module == 2 {
            link.to_ioid = 1;
        }
    }
    let root = model.block_mut(0).expect("the root exists");
    for link in root.links.iter_mut() {
        if link.from_module == 2 {
            link.from_ioid = 1;
        }
    }
}

/// An entry port with no internal link does not reach the exit port.
///
/// The container is entered, but nothing inside it starts at the entry port,
/// so the transaction never leaves the container through its exit port.
#[test]
fn an_entry_port_without_an_internal_link_does_not_reach_the_exit() {
    let mut model = parse_str(UNCONNECTED_ENTRY).expect("test model parses");
    share_the_port_id(&mut model);
    let program = compile(&model, "container").expect("model compiles");
    let result = run(&program, 100.0, 7);
    let generate = block(&program, "Generate");
    let terminate = block(&program, "A_Terminate");
    assert_eq!(
        result.blocks[generate].entries, 2,
        "the model generates transactions"
    );
    assert_eq!(
        result.blocks[terminate].entries, 0,
        "the transaction that enters the container does not reach its exit port"
    );
}

/// A container that is entered and left through its own ports routes the
/// transaction inside it and then out of it.
#[test]
fn a_connected_container_routes_the_transaction_inside_and_out() {
    let program = compile_model(CONNECTED_ENTRY);
    let result = run(&program, 100.0, 7);
    let interior = block(&program, "A_Interior");
    let terminate = block(&program, "A_Terminate");
    assert_eq!(
        result.blocks[interior].entries, 1,
        "the transaction runs the block inside the container"
    );
    assert_eq!(
        result.blocks[terminate].entries, 1,
        "the transaction leaves the container and reaches the block after it"
    );
}

/// The direction of the link decides, not the port id.
///
/// A container whose entry and exit ports share the id 1 is still entered
/// through its entry port and left through its exit port: the transaction
/// runs the block inside the container before it leaves.
#[test]
fn a_container_whose_ports_share_the_id_is_entered_and_left() {
    let mut model = parse_str(CONNECTED_ENTRY).expect("test model parses");
    share_the_port_id(&mut model);
    let program = compile(&model, "container").expect("model compiles");
    let result = run(&program, 100.0, 7);
    let interior = block(&program, "A_Interior");
    let terminate = block(&program, "A_Terminate");
    assert_eq!(
        result.blocks[interior].entries, 1,
        "the transaction runs the block inside the container"
    );
    assert_eq!(
        result.blocks[terminate].entries, 1,
        "the transaction leaves the container and reaches the block after it"
    );
}
