//! The pairs of activity blocks every link carries.
//!
//! The engine resolves the borders of a container while it compiles, so the
//! trace records the move from one activity block to another (`73 -> 1`) and
//! never the crossing of the border. The model document publishes, for every
//! link, the pairs of activity blocks that link carries, resolved through the
//! borders, so a client can draw the entity that crosses a link without
//! resolving the borders on its own.
//!
//! The rules pinned here:
//!
//! - a link that enters a container carries the pair (block outside, block
//!   inside), and the container's own internal link that feeds that entry
//!   port carries the same pair;
//! - a link that leaves a container carries the pair (block inside, block
//!   outside), and so does the internal link that reaches the exit port;
//! - a link between two activity blocks carries exactly that pair;
//! - a nested container is followed until an activity block is reached;
//! - an entry port with no internal link starts at the container's first
//!   child (the editor allows it);
//! - a cycle of containers stops instead of looping forever.

use genmsi_core::documents::{ModelDocument, model_document};
use genmsi_core::program::borders::{Borders, LinkKey};
use genmsi_core::program::compile;
use genmsi_core::sim::{SimConfig, Simulation};
use genmsi_format::{GmdlModel, parse_str};
use std::collections::HashSet;
use std::path::Path;

/// The airport of the reference models: it has containers of its own, one
/// inside another, and links that cross their borders.
const AEROPUERTO: &str = "../../fixtures/models/TP3__aeropuerto.gmdl";

/// The airport, loaded from the file.
fn airport() -> GmdlModel {
    genmsi_format::load(Path::new(AEROPUERTO)).expect("the reference model loads")
}

/// The document of the airport.
fn document() -> ModelDocument {
    model_document(&airport(), AEROPUERTO)
}

/// The edge of the document with that id.
fn edge<'a>(document: &'a ModelDocument, id: &str) -> &'a genmsi_core::documents::ModelEdge {
    document
        .edges
        .iter()
        .find(|edge| edge.id == id)
        .unwrap_or_else(|| panic!("the document has the edge {id}"))
}

#[test]
fn the_link_that_enters_a_container_carries_the_pair_of_the_move() {
    let document = document();
    // The container `Buffet` (id 4) is entered from the block `Transfer1`
    // (id 3) of the scope above, and the flow continues inside it at the
    // block `Queue2` (id 9).
    let entering = edge(&document, "0:3:2->4:1");
    assert_eq!(entering.hops, vec![(3, 9)], "the pair of the move that enters");

    // The internal link that feeds the entry port carries the same pair: the
    // one outside and the one inside.
    let internal = edge(&document, "4:4:1->9:1");
    assert_eq!(internal.hops, vec![(3, 9)], "the pair of the same move");
}

#[test]
fn the_link_that_leaves_a_container_carries_the_pair_of_the_move() {
    let document = document();
    // The container `Buffet` (id 4) is left by the block `Depart2` (id 8),
    // and the flow continues in the scope above at the block `Transfer11`
    // (id 11).
    let internal = edge(&document, "4:8:2->4:2");
    assert_eq!(internal.hops, vec![(8, 11)], "the pair of the move that leaves");

    // The link of the scope above that leaves the container carries the same
    // pair: the one inside and the one outside.
    let leaving = edge(&document, "0:4:2->11:1");
    assert_eq!(leaving.hops, vec![(8, 11)], "the pair of the same move");
}

#[test]
fn a_link_between_two_activity_blocks_carries_exactly_that_pair() {
    let document = document();
    let link = edge(&document, "0:11:2->12:1");
    assert_eq!(link.hops, vec![(11, 12)], "the pair of the link itself");
    // A link that a fork opens carries the pair of the branch it opens.
    let branch = edge(&document, "0:3:3->11:1");
    assert_eq!(branch.hops, vec![(3, 11)], "the pair of the branch");
}

#[test]
fn a_nested_container_is_followed_until_an_activity_block_is_reached() {
    let document = document();
    // The container `Caja1` (id 19) lives inside the container `FreeShop`
    // (id 13). The flow that enters it from `Transfer3` (id 45) reaches the
    // block `Queue` (id 24) inside it.
    let entering = edge(&document, "13:45:3->19:2");
    assert_eq!(entering.hops, vec![(45, 24)], "the pair across two borders");
    // The internal link of the inner container carries the same pair.
    let internal = edge(&document, "19:19:2->24:1");
    assert_eq!(internal.hops, vec![(45, 24)], "the pair of the same move");
}

#[test]
fn an_entry_port_with_no_internal_link_starts_at_the_first_child() {
    // The container (id 2) declares an entry port (ioid 1) and an exit port
    // (ioid 2), and nothing inside it starts at the entry port: the flow
    // starts at the first child of the container (id 3).
    let model = parse_str(UNCONNECTED_ENTRY).expect("the model parses");
    let document = model_document(&model, "m.gmdl");
    let entering = document
        .edges
        .iter()
        .find(|edge| edge.id == "0:1:1->2:1")
        .expect("the link that enters the container");
    assert_eq!(entering.hops, vec![(1, 3)], "the pair of the move that enters");
}

#[test]
fn every_move_of_the_run_is_published_by_some_link() {
    // The pairs the document publishes are the moves the trace records: the
    // trace says that a transaction went from one activity block to another,
    // and every one of those moves crosses a link that carries the pair.
    let model = airport();
    let program = compile(&model, "aeropuerto").expect("the airport compiles");
    let config = SimConfig {
        trace: true,
        trace_limit: 200_000,
        ..SimConfig::default()
    };
    let result = Simulation::new(&program, config).run().expect("the airport runs");
    // The block of the model every statement of the program belongs to.
    let node_of = |block: usize| program.blocks.get(block).map(|block| block.module_id);
    // The pairs every link of the document publishes.
    let published: HashSet<(u32, u32)> = document()
        .edges
        .iter()
        .flat_map(|edge| edge.hops.iter().copied())
        .collect();
    let mut moves = 0;
    let mut shortcuts = 0;
    for event in &result.trace {
        if event.event != "leave" {
            continue;
        }
        let Some(target) = target_of(&event.detail) else {
            continue;
        };
        let (Some(from), Some(to)) = (node_of(event.block), node_of(target)) else {
            continue;
        };
        // A move that stays inside the same block of the model is invisible
        // to the reader: a compound block is several statements of one block.
        if from == to {
            continue;
        }
        moves += 1;
        if published.contains(&(from, to)) {
            continue;
        }
        // The trace tells a move in the block the transaction is counted in,
        // and a transaction that backed up is counted in the block it came
        // from: the move then skips the blocks it went through, and the links
        // it crossed are the ones of the chain.
        shortcuts += 1;
        assert!(
            crosses(&published, from, to),
            "the move {from} -> {to} crosses no link"
        );
    }
    assert!(moves > 0, "the run has moves");
    assert!(shortcuts < moves / 10, "the run has {shortcuts} shortcuts");
}

/// Whether the pairs published carry a flow from one block to another.
///
/// The pairs are the edges of the graph of the blocks: the flow from `from`
/// to `to` crosses a link when there is a path of pairs between them.
fn crosses(published: &HashSet<(u32, u32)>, from: u32, to: u32) -> bool {
    let mut seen = HashSet::from([from]);
    let mut pending = vec![from];
    while let Some(block) = pending.pop() {
        for (left, right) in published {
            if *left != block {
                continue;
            }
            if *right == to {
                return true;
            }
            if seen.insert(*right) {
                pending.push(*right);
            }
        }
    }
    false
}

/// The block of the program a trace detail names (`-> 13`).
fn target_of(detail: &str) -> Option<usize> {
    let text = detail.trim();
    let target = text.strip_prefix("->")?.trim();
    target.parse::<usize>().ok()
}

#[test]
fn a_cycle_of_containers_stops_instead_of_looping_forever() {
    // Two containers that enter each other: the flow that enters one of them
    // never reaches an activity block.
    let model = parse_str(CYCLE).expect("the model parses");
    let borders = Borders::new(&model);
    let error = borders
        .flow_from(1, 1)
        .expect_err("the cycle is refused");
    assert!(
        error.to_string().contains("container cycle"),
        "the error names the cycle: {error}"
    );
}

/// A container whose entry port has no internal link.
const UNCONNECTED_ENTRY: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="4" toioid="1"/></links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Contenedor"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links/>
</module>
<module moduleid="3" moduleclass="Advance" owner="2" caption="a" name="Advance" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs>
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

/// Two containers that enter each other.
const CYCLE: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/></links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Contenedor"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links><link frommoduleid="2" fromioid="1" tomoduleid="3" toioid="1"/></links>
</module>
<module moduleid="3" moduleclass="" owner="2" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links><link frommoduleid="3" fromioid="1" tomoduleid="2" toioid="1"/></links>
</module>
</modules></gmodel>"#;

/// The key of a link, for the tests that look for one.
fn key(scope: u32, from: (u32, u32), to: (u32, u32)) -> LinkKey {
    LinkKey {
        scope,
        from_module: from.0,
        from_ioid: from.1,
        to_module: to.0,
        to_ioid: to.1,
    }
}

#[test]
fn the_key_of_a_link_names_its_scope_and_its_two_ends() {
    let document = document();
    let entering = edge(&document, "0:3:2->4:1");
    assert_eq!(
        key(
            entering.container,
            (entering.from_module, entering.from_ioid),
            (entering.to_module, entering.to_ioid)
        ),
        key(0, (3, 2), (4, 1))
    );
}

#[test]
fn the_resolution_of_the_borders_is_cheap() {
    // The model document is built on every edit of the app, so the
    // resolution of the borders cannot cost more than a blink: it must not
    // compile the program to know where the flow goes. The biggest model of
    // the reference set is built here, and the whole document has to be
    // built in a fraction of a second.
    let path = Path::new("../../fixtures/models/Parcial 6-11-23__Veron - Parcial.gmdl");
    let model = genmsi_format::load(path).expect("the model loads");
    let start = std::time::Instant::now();
    let document = model_document(&model, "parcial.gmdl");
    let elapsed = start.elapsed();
    assert!(!document.edges.is_empty());
    assert!(
        elapsed < std::time::Duration::from_millis(200),
        "the document took {elapsed:?}"
    );
}
