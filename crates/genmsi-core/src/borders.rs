//! The borders of a model: the rules that say which block the flow reaches
//! when it crosses the border of a container.
//!
//! A container is transparent to the flow. The engine resolves its borders
//! while it compiles (`program.rs`), so the program of a model with
//! containers is the program of its activity blocks and the trace records the
//! move from one activity block to another (`73 -> 1`), never the crossing of
//! the border that happened on the way.
//!
//! Those rules live here, once, and two callers use them:
//!
//! - the compiler, to wire the program (the `next` of every statement);
//! - the model document, to publish, for every link, the pairs of activity
//!   blocks the link carries (`ModelEdge.hops`).
//!
//! A client that resolved the borders on its own would carry a second
//! version of the rules, and the two versions would drift apart: the pairs
//! the document publishes are exactly the moves the trace records, because
//! both come from this resolver.
//!
//! # The rules
//!
//! Containers are transparent pass-throughs with two port kinds:
//!
//! - an **entry** port appears as a link starting *at* the container
//!   (`(container, ioid) -> (child, ioid)`), so entering continues inside;
//! - an **exit** port appears as a link ending *at* the container
//!   (`(child, ioid) -> (container, ioid)`), so leaving continues in the
//!   container's parent, where a link starts at `(container, ioid)`.
//!
//! The direction of the link that reached the container tells an entry from an
//! exit, not the port id: the editor gives a container the same id for its
//! entry and its exit port, so a container may use one id for both.
//!
//! An entry port with no internal link starts at the container's first child
//! (the editor allows it, e.g. TP4's `EleccionProductos`). Nested containers
//! are followed until an activity block is reached, and a cycle of containers
//! stops instead of looping forever.

use std::collections::HashMap;

use crate::error::EngineError;
use genmsi_format::{GmdlModel, Link};

/// How deep the resolver follows the borders before it gives up.
///
/// The compiler has always refused a model whose containers form a cycle
/// after this many steps, and the refusal is part of the contract of the
/// engine: the same limit applies here, so a model that compiles still
/// compiles.
const MAX_DEPTH: usize = 64;

/// The identity of one link of the model: the scope that owns it and the two
/// ends it joins.
///
/// A link only has a meaning inside its scope: the same two ends joined by
/// another link of another scope are another link. The key names both, so a
/// client can find the pairs a link carries without guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinkKey {
    /// Id of the container that owns the link.
    pub scope: u32,
    /// Source module id.
    pub from_module: u32,
    /// Source port id.
    pub from_ioid: u32,
    /// Target module id.
    pub to_module: u32,
    /// Target port id.
    pub to_ioid: u32,
}

impl LinkKey {
    /// The key of a link owned by `scope`.
    pub fn new(scope: u32, link: &Link) -> Self {
        LinkKey {
            scope,
            from_module: link.from_module,
            from_ioid: link.from_ioid,
            to_module: link.to_module,
            to_ioid: link.to_ioid,
        }
    }
}

/// The flow that leaves one port: the activity block it reaches and the links
/// it travels to get there.
#[derive(Debug, Clone)]
pub struct Flow {
    /// The activity block the flow reaches.
    pub target: u32,
    /// The links the flow travels, in the order it travels them.
    pub links: Vec<LinkKey>,
}

/// The resolver of the borders of a model.
///
/// The resolver indexes the model once, so following a border is a lookup and
/// not a walk: the document builder runs on every edit of the app and cannot
/// afford to scan the model for every port.
pub struct Borders<'a> {
    /// The model the borders belong to.
    model: &'a GmdlModel,
    /// Index of every block by id.
    blocks: HashMap<u32, usize>,
    /// The first link of every scope that starts at a given port.
    ///
    /// The compiler takes the first link it finds, so the index keeps the
    /// first one and the two agree on which link the flow travels.
    links: HashMap<(u32, u32, u32), usize>,
    /// The first child of every container, by id.
    first_child: HashMap<u32, u32>,
}

impl<'a> Borders<'a> {
    /// The resolver of the borders of `model`.
    pub fn new(model: &'a GmdlModel) -> Self {
        let blocks: HashMap<u32, usize> = model
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.id, index))
            .collect();
        let mut links: HashMap<(u32, u32, u32), usize> = HashMap::new();
        for block in model.blocks.iter().filter(|block| block.is_container()) {
            for (index, link) in block.links.iter().enumerate() {
                links
                    .entry((block.id, link.from_module, link.from_ioid))
                    .or_insert(index);
            }
        }
        let mut first_child: HashMap<u32, u32> = HashMap::new();
        for block in &model.blocks {
            if block.id == block.owner {
                continue;
            }
            let child = first_child.entry(block.owner).or_insert(block.id);
            *child = (*child).min(block.id);
        }
        Borders {
            model,
            blocks,
            links,
            first_child,
        }
    }

    /// The activity block the flow reaches when it leaves the port
    /// `(module_id, ioid)` of an activity block.
    ///
    /// `None` when the flow ends there: the port has no link, or the
    /// container it reaches has nothing to continue with.
    pub fn target_from(&self, module_id: u32, ioid: u32) -> Result<Option<u32>, EngineError> {
        let mut path = Vec::new();
        self.resolve_in_owner(module_id, ioid, 0, &mut path)
    }

    /// The activity block the flow reaches when it leaves the port
    /// `(module_id, ioid)`, and the links it travels to get there.
    ///
    /// The links are the border crossings: the link that leaves the port, the
    /// internal link of a container that continues the flow inside it, and the
    /// link that leaves the container in the scope above. A flow that enters
    /// a container travels two links: the one that enters it and the one that
    /// continues inside.
    pub fn flow_from(&self, module_id: u32, ioid: u32) -> Result<Option<Flow>, EngineError> {
        let mut path = Vec::new();
        let target = self.resolve_in_owner(module_id, ioid, 0, &mut path)?;
        Ok(target.map(|target| Flow { target, links: path }))
    }

    /// The activity block the flow reaches after it leaves the port
    /// `(module_id, ioid)` of a module of the model.
    ///
    /// The port belongs to the scope of the module's owner: the link that
    /// leaves the port is a link of that scope, and the flow continues with
    /// the sibling it points at.
    fn resolve_in_owner(
        &self,
        module_id: u32,
        ioid: u32,
        depth: usize,
        path: &mut Vec<LinkKey>,
    ) -> Result<Option<u32>, EngineError> {
        if depth > MAX_DEPTH {
            return Err(EngineError::Graph {
                message: format!(
                    "container cycle while resolving module {module_id} port {ioid}"
                ),
            });
        }
        let module = self.block(module_id)?;
        let owner = self.block_of(module.owner).ok_or_else(|| EngineError::Graph {
            message: format!("module {module_id} has missing owner {}", module.owner),
        })?;
        let Some(index) = self.links.get(&(owner.id, module_id, ioid)).copied() else {
            return Ok(None);
        };
        let link = &owner.links[index];
        path.push(LinkKey::new(owner.id, link));
        // The link belongs to the owner's scope, so the flow leaves the module
        // (or the container) and continues with a sibling of the owner.
        self.follow(link.to_module, link.to_ioid, owner.id, depth, path)
    }

    /// Continue from `(module_id, ioid)` after a link owned by `scope`
    /// reached it.
    ///
    /// `scope` is the container that owns the link the flow just travelled.
    /// It tells an entry from an exit, because the port id cannot: the editor
    /// gives a container the same id for its entry and its exit port, so a
    /// container may use one id for both. A link owned by the container itself
    /// ends at one of the container's output ports (the flow leaves), while a
    /// link owned by the container's owner ends at one of its input ports (the
    /// flow enters).
    fn follow(
        &self,
        module_id: u32,
        ioid: u32,
        scope: u32,
        depth: usize,
        path: &mut Vec<LinkKey>,
    ) -> Result<Option<u32>, EngineError> {
        if depth > MAX_DEPTH {
            return Err(EngineError::Graph {
                message: format!(
                    "container cycle while resolving module {module_id} port {ioid}"
                ),
            });
        }
        let module = self.block(module_id)?;
        if !module.is_container() {
            return Ok(Some(module_id));
        }
        // Exiting the container: the owner owns the continuation.
        if scope == module_id {
            return self.resolve_in_owner(module_id, ioid, depth + 1, path);
        }
        // Entering the container: an internal link may start at the port.
        if let Some(index) = self.links.get(&(module_id, module_id, ioid)).copied() {
            let inner = &module.links[index];
            path.push(LinkKey::new(module_id, inner));
            return self.follow(inner.to_module, inner.to_ioid, module_id, depth + 1, path);
        }
        // The entry port has no internal link: the editor allows it (TP4's
        // `EleccionProductos`), and the flow then starts at the container's
        // first child (lowest module id).
        let Some(child) = self.first_child.get(&module_id).copied() else {
            return Ok(None);
        };
        let child = self.block(child)?;
        if !child.is_container() {
            return Ok(Some(child.id));
        }
        // A container child is entered through its first input port.
        let Some(port) = child.inputs.first() else {
            return Ok(None);
        };
        self.follow(child.id, port.ioid, module_id, depth + 1, path)
    }

    /// The block of the model with that id.
    fn block(&self, id: u32) -> Result<&'a genmsi_format::Block, EngineError> {
        self.block_of(id).ok_or_else(|| EngineError::Graph {
            message: format!("link points at missing module {id}"),
        })
    }

    /// The block of the model with that id, when the model has it.
    fn block_of(&self, id: u32) -> Option<&'a genmsi_format::Block> {
        let index = self.blocks.get(&id).copied()?;
        Some(&self.model.blocks[index])
    }
}

/// The pairs of activity blocks every link of the model carries.
///
/// The pairs are resolved through the borders: a link that enters a
/// container carries the pair (block outside, block inside), and the
/// container's own internal link that feeds that entry port carries the same
/// pair. A link between two activity blocks carries exactly that pair.
///
/// A link may carry several pairs: the exit of a container is fed by several
/// blocks of its inside, and an entry port may be fed by several blocks of
/// the scope above.
///
/// The pairs of a link are the moves the trace records across it: the
/// resolver is the one the compiler uses, so the two cannot disagree. A link
/// the flow never travels carries no pair, and a model that does not compile
/// (a cycle of containers, a link that points at nothing) carries the pairs
/// that could be resolved and no more: the document is built for a model the
/// reader can look at, not only for a model that runs.
pub fn hops_of(model: &GmdlModel) -> HashMap<LinkKey, Vec<(u32, u32)>> {
    let borders = Borders::new(model);
    let mut hops: HashMap<LinkKey, Vec<(u32, u32)>> = HashMap::new();
    for block in model.activity_blocks() {
        for port in &block.outputs {
            let Ok(Some(flow)) = borders.flow_from(block.id, port.ioid) else {
                continue;
            };
            for key in &flow.links {
                let pairs = hops.entry(*key).or_default();
                let pair = (block.id, flow.target);
                if !pairs.contains(&pair) {
                    pairs.push(pair);
                }
            }
        }
    }
    hops
}

#[cfg(test)]
mod tests {
    use super::*;
    use genmsi_format::parse_str;

    /// A model with a container that is entered and left.
    ///
    /// The container (id 2) holds one block (id 3). The flow enters the
    /// container through its entry port (ioid 1), continues inside to the
    /// block it holds, and leaves through its exit port (ioid 2) to the
    /// block of the scope above (id 4).
    const CONTAINER: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="4" toioid="1"/></links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Contenedor"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Contenedor"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaContenedor"/></outputs>
<links><link frommoduleid="2" fromioid="1" tomoduleid="3" toioid="1"/><link frommoduleid="3" fromioid="2" tomoduleid="2" toioid="2"/></links>
</module>
<module moduleid="3" moduleclass="Advance" owner="2" caption="a" name="Advance" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Interior"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_SalidaInterior"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs>
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

    fn model() -> GmdlModel {
        parse_str(CONTAINER).expect("the model parses")
    }

    #[test]
    fn the_flow_that_enters_a_container_reaches_the_block_inside() {
        let model = model();
        let borders = Borders::new(&model);
        let flow = borders
            .flow_from(1, 1)
            .expect("the flow resolves")
            .expect("the flow goes somewhere");
        assert_eq!(flow.target, 3, "the flow reaches the block inside");
        assert_eq!(flow.links.len(), 2, "{:?}", flow.links);
        // The link that enters the container, and the internal link that
        // continues the flow inside it.
        assert_eq!(flow.links[0].scope, 0);
        assert_eq!(flow.links[0].from_module, 1);
        assert_eq!(flow.links[0].to_module, 2);
        assert_eq!(flow.links[1].scope, 2);
        assert_eq!(flow.links[1].from_module, 2);
        assert_eq!(flow.links[1].to_module, 3);
    }

    #[test]
    fn the_flow_that_leaves_a_container_reaches_the_block_outside() {
        let model = model();
        let borders = Borders::new(&model);
        let flow = borders
            .flow_from(3, 2)
            .expect("the flow resolves")
            .expect("the flow goes somewhere");
        assert_eq!(flow.target, 4, "the flow reaches the block outside");
        assert_eq!(flow.links.len(), 2, "{:?}", flow.links);
        // The internal link that leaves the block for the exit port, and the
        // link of the scope above that continues the flow.
        assert_eq!(flow.links[0].scope, 2);
        assert_eq!(flow.links[1].scope, 0);
        assert_eq!(flow.links[1].from_module, 2);
        assert_eq!(flow.links[1].to_module, 4);
    }

    #[test]
    fn the_link_that_enters_a_container_carries_the_pair_of_the_move() {
        let model = model();
        let hops = hops_of(&model);
        // The link of the scope above that enters the container carries the
        // pair (outside, inside).
        let entering = hops
            .get(&LinkKey {
                scope: 0,
                from_module: 1,
                from_ioid: 1,
                to_module: 2,
                to_ioid: 1,
            })
            .expect("the link that enters the container");
        assert_eq!(entering, &vec![(1, 3)], "{entering:?}");
        // The internal link that feeds the entry port carries the same pair.
        let internal = hops
            .get(&LinkKey {
                scope: 2,
                from_module: 2,
                from_ioid: 1,
                to_module: 3,
                to_ioid: 1,
            })
            .expect("the internal link of the entry port");
        assert_eq!(internal, &vec![(1, 3)], "{internal:?}");
        // The link that leaves the container carries the pair (inside,
        // outside), and so does the internal link that reaches the exit port.
        let leaving = hops
            .get(&LinkKey {
                scope: 0,
                from_module: 2,
                from_ioid: 2,
                to_module: 4,
                to_ioid: 1,
            })
            .expect("the link that leaves the container");
        assert_eq!(leaving, &vec![(3, 4)], "{leaving:?}");
        let to_exit = hops
            .get(&LinkKey {
                scope: 2,
                from_module: 3,
                from_ioid: 2,
                to_module: 2,
                to_ioid: 2,
            })
            .expect("the internal link that reaches the exit port");
        assert_eq!(to_exit, &vec![(3, 4)], "{to_exit:?}");
    }
}
