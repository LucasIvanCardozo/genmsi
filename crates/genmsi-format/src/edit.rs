//! The additive editing API: mutations over a model and the undo stack.
//!
//! An [`EditSession`] owns a [`GmdlModel`] and a command stack. Every applied
//! [`EditCommand`] pushes exactly one undo entry, even when it changed many
//! things (a paste, a multi-block `SetLayouts`): a drag is one `MoveBlock`
//! sent by the caller with the final position, not one per pixel. The steps
//! are reversible ([`UndoStep`]) so the session stores small changes instead
//! of whole copies of the model. The stack is not persisted.
//!
//! Labels are the connection points: a link is a pair of ports that share a
//! label, and connecting two ports gives both the same label. A pasted block
//! starts every port with an empty label and every remapped link gets a fresh
//! label, so a paste never shares labels with its original.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::catalogue::build_block;
use crate::error::FormatError;
use crate::model::{Block, GmdlModel, Layout, Link, Port};

/// Default size of a freshly created block (the original editor's small box).
const DEFAULT_WIDTH: i32 = 120;
const DEFAULT_HEIGHT: i32 = 48;

/// One requested block position (the payload of [`EditCommand::SetLayouts`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Position {
    /// Block id.
    pub id: u32,
    /// Horizontal position.
    pub x: i32,
    /// Vertical position.
    pub y: i32,
    /// The new width, when the block changes its size.
    #[serde(default)]
    pub width: Option<i32>,
    /// The new height, when the block changes its size.
    #[serde(default)]
    pub height: Option<i32>,
}

/// One editing operation over the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum EditCommand {
    /// Create a block of a catalogue class inside a container.
    AddBlock {
        /// Catalogue class (`Generate`, `Queue`, ...).
        class: String,
        /// Owning container id.
        container: u32,
        /// Horizontal position.
        x: i32,
        /// Vertical position.
        y: i32,
    },
    /// Remove a block, its descendants and every link that touches them.
    RemoveBlock {
        /// Block id (the model root is refused).
        id: u32,
    },
    /// Remove several blocks as ONE operation.
    ///
    /// The blocks that one of them owns are removed with it, and a block
    /// that is already gone when its turn comes is skipped.
    RemoveBlocks {
        /// Block ids.
        ids: Vec<u32>,
    },
    /// Move a block, keeping its size.
    MoveBlock {
        /// Block id.
        id: u32,
        /// Horizontal position.
        x: i32,
        /// Vertical position.
        y: i32,
    },
    /// Set the position of many blocks as ONE operation (the re-layout).
    SetLayouts {
        /// One position per block.
        positions: Vec<Position>,
    },
    /// Connect an output port to an input port of a sibling block.
    Connect {
        /// Source block id.
        from: u32,
        /// Source output port id.
        #[serde(rename = "fromIoid")]
        from_ioid: u32,
        /// Target block id.
        to: u32,
        /// Target input port id.
        #[serde(rename = "toIoid")]
        to_ioid: u32,
    },
    /// Remove the link that starts at a port of a container's child.
    Disconnect {
        /// Owning container id.
        container: u32,
        /// Source block id.
        from: u32,
        /// Source output port id.
        #[serde(rename = "fromIoid")]
        from_ioid: u32,
    },
    /// Set the value of a declared property.
    SetProperty {
        /// Block id.
        id: u32,
        /// Property name.
        property: String,
        /// Decoded value; the file's encoding convention is kept.
        value: String,
    },
    /// Set the block's caption.
    SetCaption {
        /// Block id.
        id: u32,
        /// New caption.
        caption: String,
    },
    /// Copy blocks with fresh ids, remapped links and fresh labels.
    Paste {
        /// Ids of the blocks to copy, in request order.
        ids: Vec<u32>,
        /// Horizontal offset for the copies.
        dx: i32,
        /// Vertical offset for the copies.
        dy: i32,
    },
}

/// One reversible change. Undo applies it backwards, redo forwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoStep {
    /// A block was appended (its data travels with the step).
    BlockAdded {
        /// Index in the model's block list.
        index: usize,
        /// The block as it was added.
        block: Box<Block>,
    },
    /// Blocks were removed (data plus original index, ascending).
    BlocksRemoved {
        /// The removed blocks with their original list indices.
        blocks: Vec<(usize, Block)>,
    },
    /// Links of one container were removed (data plus index, ascending).
    LinksRemoved {
        /// Owning container id.
        container: u32,
        /// The removed links with their original list indices.
        links: Vec<(usize, Link)>,
    },
    /// A link was appended to a container.
    LinkAdded {
        /// Owning container id.
        container: u32,
        /// The added link.
        link: Link,
    },
    /// A block's layout changed.
    LayoutSet {
        /// Block id.
        id: u32,
        /// Layout before.
        old: Option<Layout>,
        /// Layout after.
        new: Option<Layout>,
    },
    /// A property's raw value changed.
    PropertyChanged {
        /// Block id.
        id: u32,
        /// Property name.
        property: String,
        /// Raw value before.
        old_raw: String,
        /// Raw value after.
        new_raw: String,
    },
    /// A block's caption changed.
    CaptionSet {
        /// Block id.
        id: u32,
        /// Caption before.
        old: String,
        /// Caption after.
        new: String,
    },
    /// A port's connection label changed.
    PortLabelSet {
        /// Block id.
        block: u32,
        /// Port id.
        ioid: u32,
        /// Whether the port is an input port.
        input: bool,
        /// Label before.
        old: String,
        /// Label after.
        new: String,
    },
}

/// An owned model with the command stack of undo/redo.
pub struct EditSession {
    model: GmdlModel,
    undo_stack: Vec<Vec<UndoStep>>,
    redo_stack: Vec<Vec<UndoStep>>,
    /// The text of the last checkpoint: the state the session was created
    /// with, or the state of the last [`EditSession::mark_saved`].
    checkpoint: String,
}

impl EditSession {
    /// Take ownership of a model with a clean, empty stack.
    pub fn new(model: GmdlModel) -> Self {
        let checkpoint = crate::to_string(&model);
        Self {
            model,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            checkpoint,
        }
    }

    /// The current model.
    pub fn model(&self) -> &GmdlModel {
        &self.model
    }

    /// Whether the model differs from the last checkpoint: the state the
    /// session was created with, or the state of the last
    /// [`EditSession::mark_saved`].
    ///
    /// The flag follows the checkpoint instead of counting the commands:
    /// undoing an edit that came after a save leaves the model exactly as it
    /// was saved, and the model is clean again.
    pub fn dirty(&self) -> bool {
        crate::to_string(&self.model) != self.checkpoint
    }

    /// Mark the current state as saved (the caller just wrote the file).
    pub fn mark_saved(&mut self) {
        self.checkpoint = crate::to_string(&self.model);
    }

    /// Whether there is something to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there is something to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Apply a command. Returns the ids the command produced (the new block
    /// for `AddBlock`, the copies for `Paste`, empty for the rest).
    ///
    /// A failed command changes nothing and clears no redo entry.
    pub fn apply(&mut self, command: EditCommand) -> Result<Vec<u32>, FormatError> {
        let mut steps = Vec::new();
        // The command runs on a copy of the model: a command that fails
        // changes nothing, and the promise the session makes to the app
        // holds for every command, not only for the ones that change one
        // block.
        let mut trial = EditSession {
            model: self.model.clone(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            checkpoint: self.checkpoint.clone(),
        };
        let produced = trial.run(command, &mut steps)?;
        self.model = trial.model;
        self.undo_stack.push(steps);
        self.redo_stack.clear();
        Ok(produced)
    }

    /// Run one command over the model, collecting the steps it takes.
    fn run(
        &mut self,
        command: EditCommand,
        steps: &mut Vec<UndoStep>,
    ) -> Result<Vec<u32>, FormatError> {
        let produced = match command {
            EditCommand::AddBlock {
                class,
                container,
                x,
                y,
            } => {
                let id = self.add_block(&class, container, x, y, &mut *steps)?;
                vec![id]
            }
            EditCommand::RemoveBlock { id } => {
                self.remove_block(id, &mut *steps)?;
                Vec::new()
            }
            EditCommand::RemoveBlocks { ids } => {
                let mut ordered = ids.clone();
                ordered.sort_unstable();
                ordered.dedup();
                for id in ordered {
                    // A block that another removal took with it is already
                    // gone: there is nothing left to remove.
                    if self.model.block(id).is_some() {
                        self.remove_block(id, &mut *steps)?;
                    }
                }
                Vec::new()
            }
            EditCommand::MoveBlock { id, x, y } => {
                self.move_block(id, x, y, &mut *steps)?;
                Vec::new()
            }
            EditCommand::SetLayouts { positions } => {
                self.set_layouts(&positions, &mut *steps)?;
                Vec::new()
            }
            EditCommand::Connect {
                from,
                from_ioid,
                to,
                to_ioid,
            } => {
                self.connect(from, from_ioid, to, to_ioid, &mut *steps)?;
                Vec::new()
            }
            EditCommand::Disconnect {
                container,
                from,
                from_ioid,
            } => {
                self.disconnect(container, from, from_ioid, &mut *steps)?;
                Vec::new()
            }
            EditCommand::SetProperty {
                id,
                property,
                value,
            } => {
                self.set_property(id, &property, &value, &mut *steps)?;
                Vec::new()
            }
            EditCommand::SetCaption { id, caption } => {
                self.set_caption(id, &caption, &mut *steps)?;
                Vec::new()
            }
            EditCommand::Paste { ids, dx, dy } => self.paste(&ids, dx, dy, &mut *steps)?,
        };
        Ok(produced)
    }

    /// Restore the state before the last command. Returns whether there was
    /// something to undo.
    pub fn undo(&mut self) -> Result<bool, FormatError> {
        let Some(steps) = self.undo_stack.pop() else {
            return Ok(false);
        };
        apply_steps(&mut self.model, &steps, false)?;
        self.redo_stack.push(steps);
        Ok(true)
    }

    /// Re-apply the last undone command. Returns whether there was something
    /// to redo.
    pub fn redo(&mut self) -> Result<bool, FormatError> {
        let Some(steps) = self.redo_stack.pop() else {
            return Ok(false);
        };
        apply_steps(&mut self.model, &steps, true)?;
        self.undo_stack.push(steps);
        Ok(true)
    }

    fn add_block(
        &mut self,
        class: &str,
        container: u32,
        x: i32,
        y: i32,
        steps: &mut Vec<UndoStep>,
    ) -> Result<u32, FormatError> {
        let owner = self.model.block(container).ok_or_else(|| {
            FormatError::structural(container, format!("no existe el contenedor {container}"))
        })?;
        if !owner.is_container() {
            return Err(FormatError::structural(
                container,
                format!("el bloque {container} no es un contenedor"),
            ));
        }
        let mut block = build_block(
            class,
            container,
            Some(Layout {
                width: DEFAULT_WIDTH,
                height: DEFAULT_HEIGHT,
                x,
                y,
            }),
        )?;
        block.id = fresh_id(&self.model);
        let index = self.model.blocks.len();
        self.model.blocks.push(block.clone());
        steps.push(UndoStep::BlockAdded { index, block: Box::new(block) });
        Ok(self.model.blocks[index].id)
    }

    fn remove_block(&mut self, id: u32, steps: &mut Vec<UndoStep>) -> Result<(), FormatError> {
        let block = self.model.block(id).ok_or_else(|| {
            FormatError::structural(id, format!("no existe el bloque {id}"))
        })?;
        if block.is_root() {
            return Err(FormatError::structural(
                id,
                "no se puede eliminar la raíz del modelo",
            ));
        }
        let removed: HashSet<u32> = {
            let mut set = HashSet::new();
            let mut pending = vec![id];
            while let Some(current) = pending.pop() {
                if set.insert(current) {
                    pending.extend(
                        self.model
                            .blocks
                            .iter()
                            .filter(|b| b.owner == current)
                            .map(|b| b.id),
                    );
                }
            }
            set
        };
        // Links first (reversed order on undo re-inserts them after the
        // blocks exist again), then the blocks.
        let containers: Vec<u32> = self
            .model
            .blocks
            .iter()
            .filter(|b| b.is_container() && !removed.contains(&b.id))
            .map(|b| b.id)
            .collect();
        for container in containers {
            let removed_links: Vec<(usize, Link)> = self
                .model
                .block(container)
                .expect("container exists")
                .links
                .iter()
                .enumerate()
                .filter(|(_, link)| {
                    removed.contains(&link.from_module) || removed.contains(&link.to_module)
                })
                .map(|(index, link)| (index, link.clone()))
                .collect();
            if removed_links.is_empty() {
                continue;
            }
            remove_links(&mut self.model, container, &removed_links);
            steps.push(UndoStep::LinksRemoved {
                container,
                links: removed_links,
            });
        }
        let removed_blocks: Vec<(usize, Block)> = self
            .model
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| removed.contains(&b.id))
            .map(|(index, b)| (index, b.clone()))
            .collect();
        self.model
            .blocks
            .retain(|b| !removed.contains(&b.id));
        steps.push(UndoStep::BlocksRemoved {
            blocks: removed_blocks,
        });
        Ok(())
    }

    fn move_block(&mut self, id: u32, x: i32, y: i32, steps: &mut Vec<UndoStep>) -> Result<(), FormatError> {
        self.require_block(id)?;
        let old = self.model.block(id).expect("checked").layout;
        let new = self.layout_at(id, x, y);
        self.model.block_mut(id).expect("checked").layout = Some(new);
        steps.push(UndoStep::LayoutSet {
            id,
            old,
            new: Some(new),
        });
        Ok(())
    }

    fn set_layouts(&mut self, positions: &[Position], steps: &mut Vec<UndoStep>) -> Result<(), FormatError> {
        for position in positions {
            self.require_block(position.id)?;
            let old = self.model.block(position.id).expect("checked").layout;
            let placed = self.layout_at(position.id, position.x, position.y);
            // A position may also resize the block: the re-layout grows the
            // containers so they hold the blocks they own.
            let new = Layout {
                width: position.width.unwrap_or(placed.width),
                height: position.height.unwrap_or(placed.height),
                ..placed
            };
            self.model
                .block_mut(position.id)
                .expect("checked")
                .layout = Some(new);
            steps.push(UndoStep::LayoutSet {
                id: position.id,
                old,
                new: Some(new),
            });
        }
        Ok(())
    }

    /// The layout a block gets at a new position: its own size, or the
    /// default size when it had none.
    fn layout_at(&self, id: u32, x: i32, y: i32) -> Layout {
        let (width, height) = self
            .model
            .block(id)
            .and_then(|b| b.layout)
            .map(|l| (l.width, l.height))
            .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));
        Layout {
            width,
            height,
            x,
            y,
        }
    }

    /// Connect an output port to an input port of a sibling block.
    ///
    /// A port has a single successor, so the link that starts at the
    /// source port is replaced by the new one. An input port may receive
    /// from several blocks (as the reference models do), so the links that
    /// end at the target port are kept.
    fn connect(
        &mut self,
        from: u32,
        from_ioid: u32,
        to: u32,
        to_ioid: u32,
        steps: &mut Vec<UndoStep>,
    ) -> Result<(), FormatError> {
        self.require_block(from)?;
        self.require_block(to)?;
        if from == to {
            return Err(FormatError::structural(
                from,
                "no se puede conectar un bloque consigo mismo",
            ));
        }
        if self.model.block(from).expect("checked").owner
            != self.model.block(to).expect("checked").owner
        {
            return Err(FormatError::structural(
                from,
                "los bloques pertenecen a contenedores distintos",
            ));
        }
        let container = self.model.block(from).expect("checked").owner;
        if self.model.block(from).expect("checked").output(from_ioid).is_none() {
            return Err(FormatError::structural(
                from,
                format!("el bloque {from} no tiene el puerto de salida {from_ioid}"),
            ));
        }
        if self.model.block(to).expect("checked").input(to_ioid).is_none() {
            return Err(FormatError::structural(
                to,
                format!("el bloque {to} no tiene el puerto de entrada {to_ioid}"),
            ));
        }
        // The source port has a single successor: the link that starts
        // there is replaced by the new one. The links that end at the
        // target port stay: an input may receive from several blocks.
        let replaced: Vec<(usize, Link)> = self
            .model
            .block(container)
            .expect("same owner, so the container exists")
            .links
            .iter()
            .enumerate()
            .filter(|(_, link)| link.from_module == from && link.from_ioid == from_ioid)
            .map(|(index, link)| (index, link.clone()))
            .collect();
        remove_links(&mut self.model, container, &replaced);
        if !replaced.is_empty() {
            steps.push(UndoStep::LinksRemoved {
                container,
                links: replaced,
            });
        }

        // The two ends share one label: keep one that exists, else mint one.
        let source_label = self
            .model
            .block(from)
            .and_then(|b| b.output(from_ioid))
            .and_then(Port::label)
            .map(str::to_string);
        let target_label = self
            .model
            .block(to)
            .and_then(|b| b.input(to_ioid))
            .and_then(Port::label)
            .map(str::to_string);
        let label = match source_label.or(target_label) {
            Some(label) => label,
            None => {
                let target_class = &self.model.block(to).expect("checked").class;
                mint_label(&self.model, &format!("A_{target_class}"))
            }
        };
        self.set_port_label(from, from_ioid, false, &label, steps);
        self.set_port_label(to, to_ioid, true, &label, steps);

        let link = Link {
            from_module: from,
            from_ioid,
            to_module: to,
            to_ioid,
            breakpoints: None,
        };
        self.model
            .block_mut(container)
            .expect("checked")
            .links
            .push(link.clone());
        steps.push(UndoStep::LinkAdded { container, link });
        Ok(())
    }

    fn disconnect(
        &mut self,
        container: u32,
        from: u32,
        from_ioid: u32,
        steps: &mut Vec<UndoStep>,
    ) -> Result<(), FormatError> {
        let owner = self.model.block(container).ok_or_else(|| {
            FormatError::structural(container, format!("no existe el contenedor {container}"))
        })?;
        if !owner.is_container() {
            return Err(FormatError::structural(
                container,
                format!("el bloque {container} no es un contenedor"),
            ));
        }
        let removed: Vec<(usize, Link)> = owner
            .links
            .iter()
            .enumerate()
            .filter(|(_, link)| link.from_module == from && link.from_ioid == from_ioid)
            .map(|(index, link)| (index, link.clone()))
            .collect();
        if removed.is_empty() {
            return Err(FormatError::structural(
                container,
                format!("no hay un enlace que comience en ({from}, {from_ioid})"),
            ));
        }
        remove_links(&mut self.model, container, &removed);
        // The ports keep their labels: a block's own label is the label of
        // its first labelled input and must survive the disconnection.
        steps.push(UndoStep::LinksRemoved {
            container,
            links: removed,
        });
        Ok(())
    }

    fn set_property(
        &mut self,
        id: u32,
        property: &str,
        value: &str,
        steps: &mut Vec<UndoStep>,
    ) -> Result<(), FormatError> {
        self.require_block(id)?;
        let class = self.model.block(id).expect("checked").class.clone();
        let block = self.model.block_mut(id).expect("checked");
        let prop = block.prop_mut(property).ok_or_else(|| {
            FormatError::structural(
                id,
                format!("el bloque {id} ({class}) no declara la propiedad {property}"),
            )
        })?;
        let old_raw = prop.raw_value.clone();
        prop.set_value(value);
        let new_raw = prop.raw_value.clone();
        steps.push(UndoStep::PropertyChanged {
            id,
            property: property.to_string(),
            old_raw,
            new_raw,
        });
        Ok(())
    }

    fn set_caption(&mut self, id: u32, caption: &str, steps: &mut Vec<UndoStep>) -> Result<(), FormatError> {
        self.require_block(id)?;
        let block = self.model.block_mut(id).expect("checked");
        let old = block.caption.clone();
        block.caption = caption.to_string();
        steps.push(UndoStep::CaptionSet {
            id,
            old,
            new: caption.to_string(),
        });
        Ok(())
    }

    fn paste(&mut self, ids: &[u32], dx: i32, dy: i32, steps: &mut Vec<UndoStep>) -> Result<Vec<u32>, FormatError> {
        for id in ids {
            self.require_block(*id)?;
            // A container is not offered for pasting: a copy without its
            // children would be a silent data loss, and moving a whole
            // container with everything inside is a move, not a paste.
            if self.model.block(*id).expect("checked").is_container() {
                return Err(FormatError::structural(
                    *id,
                    format!("el bloque {id} es un contenedor: no se puede pegar"),
                ));
            }
        }
        // Deep copies in request order, with fresh sequential ids.
        let mut mapping: Vec<(u32, u32)> = Vec::with_capacity(ids.len());
        let mut copies: Vec<Block> = Vec::with_capacity(ids.len());
        for (next, id) in (fresh_id(&self.model)..).zip(ids.iter()) {
            let mut copy = self
                .model
                .block(*id)
                .expect("checked above")
                .clone();
            copy.id = next;
            mapping.push((*id, copy.id));
            if let Some(layout) = &mut copy.layout {
                layout.x += dx;
                layout.y += dy;
            }
            for port in copy.inputs.iter_mut().chain(&mut copy.outputs) {
                port.mylabel = String::new();
            }
            copy.links = Vec::new();
            copies.push(copy);
        }

        // Remap the links between the copies and give both ends fresh labels,
        // so a pasted link never shares a label with the original one. Links
        // are owned by containers, so every container is scanned.
        let copied: HashSet<u32> = mapping.iter().map(|(old, _)| *old).collect();
        let remapped: Vec<(u32, Link)> = self
            .model
            .blocks
            .iter()
            .filter(|b| b.is_container())
            .flat_map(|b| b.links.iter())
            .filter(|link| copied.contains(&link.from_module) && copied.contains(&link.to_module))
            .map(|link| {
                let new_from = mapping.iter().find(|(old, _)| *old == link.from_module).expect("copied").1;
                let new_to = mapping.iter().find(|(old, _)| *old == link.to_module).expect("copied").1;
                (new_from, Link {
                    from_module: new_from,
                    from_ioid: link.from_ioid,
                    to_module: new_to,
                    to_ioid: link.to_ioid,
                    breakpoints: link.breakpoints.clone(),
                })
            })
            .collect();

        // Record: blocks (without labels), then labels, then links.
        let mut added: Vec<UndoStep> = Vec::new();
        for copy in &copies {
            let index = self.model.blocks.len();
            self.model.blocks.push(copy.clone());
            added.push(UndoStep::BlockAdded { index, block: Box::new(copy.clone()) });
        }
        for (_, link) in &remapped {
            let target_class = &self
                .model
                .block(link.to_module)
                .expect("copy exists")
                .class;
            let label = mint_label(&self.model, &format!("A_{target_class}"));
            self.set_port_label(link.from_module, link.from_ioid, false, &label, &mut added);
            self.set_port_label(link.to_module, link.to_ioid, true, &label, &mut added);
            let container = self
                .model
                .block(link.from_module)
                .expect("copy exists")
                .owner;
            self.model
                .block_mut(container)
                .expect("the copies' container exists")
                .links
                .push(link.clone());
            added.push(UndoStep::LinkAdded { container, link: link.clone() });
        }
        steps.extend(added);
        Ok(mapping.into_iter().map(|(_, new)| new).collect())
    }

    fn require_block(&self, id: u32) -> Result<(), FormatError> {
        self.model.block(id).map(|_| ()).ok_or_else(|| {
            FormatError::structural(id, format!("no existe el bloque {id}"))
        })
    }

    fn set_port_label(&mut self, block: u32, ioid: u32, input: bool, label: &str, steps: &mut Vec<UndoStep>) {
        let Some(b) = self.model.block_mut(block) else {
            return;
        };
        let port = if input {
            b.inputs.iter_mut().find(|p| p.ioid == ioid)
        } else {
            b.outputs.iter_mut().find(|p| p.ioid == ioid)
        };
        let Some(port) = port else { return };
        if port.mylabel == label {
            return;
        }
        let old = std::mem::take(&mut port.mylabel);
        port.mylabel = label.to_string();
        steps.push(UndoStep::PortLabelSet {
            block,
            ioid,
            input,
            old,
            new: label.to_string(),
        });
    }
}

impl std::fmt::Debug for EditSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditSession")
            .field("dirty", &self.dirty())
            .field("undo_entries", &self.undo_stack.len())
            .field("redo_entries", &self.redo_stack.len())
            .field("blocks", &self.model.blocks.len())
            .finish()
    }
}

/// Apply a command's steps in one direction: forward re-applies the change,
/// backward reverts it.
fn apply_steps(model: &mut GmdlModel, steps: &[UndoStep], forward: bool) -> Result<(), FormatError> {
    let iter: Box<dyn Iterator<Item = &UndoStep>> = if forward {
        Box::new(steps.iter())
    } else {
        Box::new(steps.iter().rev())
    };
    for step in iter {
        match step {
            UndoStep::BlockAdded { index, block } => {
                if forward {
                    model.blocks.insert(*index, (**block).clone());
                } else {
                    model.blocks.retain(|b| b.id != block.id);
                }
            }
            UndoStep::BlocksRemoved { blocks } => {
                if forward {
                    let ids: HashSet<u32> = blocks.iter().map(|(_, b)| b.id).collect();
                    model.blocks.retain(|b| !ids.contains(&b.id));
                } else {
                    for (index, block) in blocks.iter() {
                        model.blocks.insert(*index, block.clone());
                    }
                }
            }
            UndoStep::LinksRemoved { container, links } => {
                if forward {
                    remove_links(model, *container, links);
                } else {
                    let Some(owner) = model.block_mut(*container) else {
                        return Err(FormatError::structural(
                            *container,
                            "no existe el contenedor del enlace eliminado",
                        ));
                    };
                    for (index, link) in links.iter() {
                        owner.links.insert(*index, link.clone());
                    }
                }
            }
            UndoStep::LinkAdded { container, link } => {
                let Some(owner) = model.block_mut(*container) else {
                    return Err(FormatError::structural(
                        *container,
                        "no existe el contenedor del enlace",
                    ));
                };
                if forward {
                    owner.links.push(link.clone());
                } else {
                    owner.links.retain(|l| !same_ends(l, link));
                }
            }
            UndoStep::LayoutSet { id, old, new } => {
                let layout = if forward { *new } else { *old };
                if let Some(block) = model.block_mut(*id) {
                    block.layout = layout;
                }
            }
            UndoStep::PropertyChanged {
                id,
                property,
                old_raw,
                new_raw,
            } => {
                let raw = if forward { new_raw } else { old_raw };
                if let Some(prop) = model.block_mut(*id).and_then(|b| b.prop_mut(property)) {
                    prop.raw_value = raw.clone();
                }
            }
            UndoStep::CaptionSet { id, old, new } => {
                let caption = if forward { new } else { old };
                if let Some(block) = model.block_mut(*id) {
                    block.caption = caption.clone();
                }
            }
            UndoStep::PortLabelSet {
                block,
                ioid,
                input,
                old,
                new,
            } => {
                let label = if forward { new } else { old };
                if let Some(b) = model.block_mut(*block) {
                    let port = if *input {
                        b.inputs.iter_mut().find(|p| p.ioid == *ioid)
                    } else {
                        b.outputs.iter_mut().find(|p| p.ioid == *ioid)
                    };
                    if let Some(port) = port {
                        port.mylabel = label.clone();
                    }
                }
            }
        }
    }
    Ok(())
}

/// Remove the given links from a container's link list, by their two ends.
fn remove_links(model: &mut GmdlModel, container: u32, links: &[(usize, Link)]) {
    if let Some(owner) = model.block_mut(container) {
        owner
            .links
            .retain(|l| !links.iter().any(|(_, removed)| same_ends(l, removed)));
    }
}

/// Whether two links join the same pair of ports.
fn same_ends(a: &Link, b: &Link) -> bool {
    a.from_module == b.from_module
        && a.from_ioid == b.from_ioid
        && a.to_module == b.to_module
        && a.to_ioid == b.to_ioid
}

/// The lowest id no block uses: one more than the highest existing id.
fn fresh_id(model: &GmdlModel) -> u32 {
    model.blocks.iter().map(|b| b.id).max().unwrap_or(0) + 1
}

/// Mint a fresh connection label: the base when it is unused in the model,
/// else the base with the lowest free numeric suffix (`A_Queue1`).
fn mint_label(model: &GmdlModel, base: &str) -> String {
    let mut used: HashSet<&str> = HashSet::new();
    for block in &model.blocks {
        for port in block.inputs.iter().chain(&block.outputs) {
            if let Some(label) = port.label() {
                used.insert(label);
            }
        }
    }
    if !used.contains(base) {
        return base.to_string();
    }
    for n in 1.. {
        let candidate = format!("{base}{n}");
        if !used.contains(candidate.as_str()) {
            return candidate;
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_str;

    /// A minimal model: root + Generate + Terminate linked.
    fn model() -> GmdlModel {
        parse_str(
            r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C" genprefer="" exehead="" nonexehead="" exefoot="" nonexefoot=""><links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/></links><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A" width="120" height="63" x="10" y="20"><outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Queue"/></outputs><props><property name="Media" hint="Tiempo%20Medio" value="60" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="2" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A" width="84" height="64" x="100" y="200"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Queue"/></inputs><props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
</modules></gmodel>"#,
        )
        .expect("parses")
    }

    /// The two ends of a link, as `from:from_ioid->to:to_ioid`.
    fn ends(link: &Link) -> String {
        let (from, from_ioid) = (link.from_module, link.from_ioid);
        let (to, to_ioid) = (link.to_module, link.to_ioid);
        format!("{from}:{from_ioid}->{to}:{to_ioid}")
    }

    #[test]
    fn add_block_gives_a_fresh_id_and_lands_in_the_container() {
        let mut session = EditSession::new(model());
        let ids = session
            .apply(EditCommand::AddBlock {
                class: "Advance".into(),
                container: 0,
                x: 30,
                y: 40,
            })
            .expect("adds");
        assert_eq!(ids, vec![3], "max id 2, so the fresh id is 3");
        let block = session.model().block(3).expect("added");
        assert_eq!(block.class, "Advance");
        assert_eq!(block.owner, 0);
        assert_eq!(block.kind, 'A');
        assert_eq!(block.name, "Advance");
        assert_eq!(block.caption, "");
        assert_eq!(block.layout.map(|l| (l.width, l.height, l.x, l.y)), Some((120, 48, 30, 40)));
        assert_eq!(block.input(1).expect("input").name, "Entrada1");
        assert_eq!(block.input(1).expect("input").mylabel, "");
        assert_eq!(block.output(2).expect("output").name, "Salida1");
        assert_eq!(
            block.sections,
            vec![crate::SectionKind::Inputs, crate::SectionKind::Outputs, crate::SectionKind::Props]
        );
    }

    #[test]
    fn a_note_has_no_sections_at_all() {
        let mut session = EditSession::new(model());
        let ids = session
            .apply(EditCommand::AddBlock { class: "Note".into(), container: 0, x: 0, y: 0 })
            .expect("adds");
        let note = session.model().block(ids[0]).expect("added");
        assert!(note.sections.is_empty());
        assert!(note.inputs.is_empty() && note.outputs.is_empty() && note.props.is_empty());
    }

    #[test]
    fn add_block_refuses_unknown_classes_and_non_containers() {
        let mut session = EditSession::new(model());
        let err = session
            .apply(EditCommand::AddBlock { class: "MacroPropio".into(), container: 0, x: 0, y: 0 })
            .expect_err("custom macros are not in the catalogue");
        assert!(err.to_string().contains("MacroPropio"), "{err}");
        let err = session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 99, x: 0, y: 0 })
            .expect_err("missing container");
        assert!(err.to_string().contains("99"), "{err}");
        // The failed commands changed nothing and pushed no undo entry.
        assert!(!session.can_undo());
        assert!(!session.dirty());
    }

    #[test]
    fn remove_block_takes_its_subtree_and_links() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 0, y: 0 })
            .expect("adds Advance (id 3)");
        session.apply(EditCommand::RemoveBlock { id: 1 }).expect("removes Generate");
        assert!(session.model().block(1).is_none());
        // The link Generate->Terminate travelled with Generate; the
        // unconnected Advance survives.
        assert!(session.model().block(0).expect("root").links.is_empty());
        assert!(session.model().block(2).is_some(), "Terminate survives");
        assert!(session.model().block(3).is_some(), "Advance survives");
        // Undo brings everything back, including the link.
        session.undo().expect("undoes");
        assert!(session.model().block(1).is_some());
        assert_eq!(session.model().block(0).expect("root").links.len(), 1);
    }

    #[test]
    fn removing_a_container_takes_its_children_and_their_links() {
        // A nested container (id 1) holding Generate(2) -> Terminate(3),
        // plus a Terminate(4) outside that is linked to nothing of it.
        let model = parse_str(
            r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C" genprefer="" exehead="" nonexehead="" exefoot="" nonexefoot=""><links/><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="" owner="0" caption="c" name="c" kind="C" genprefer="" exehead="" nonexehead="" exefoot="" nonexefoot="" width="100" height="100" x="0" y="0"><links><link frommoduleid="2" fromioid="1" tomoduleid="3" toioid="1"/></links><inputs/><outputs/></module>\
<module moduleid="2" moduleclass="Generate" owner="1" caption="g" name="Generate" kind="A" width="120" height="48" x="0" y="0"><outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Terminate"/></outputs><props/></module>\
<module moduleid="3" moduleclass="Terminate" owner="1" caption="t" name="Terminate" kind="A" width="84" height="48" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_Terminate"/></inputs><props/></module>\
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate2" kind="A" width="84" height="48" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel=""/></inputs><props/></module>\
</modules></gmodel>"#,
        )
        .expect("parses");
        let mut session = EditSession::new(model);
        session.apply(EditCommand::RemoveBlock { id: 1 }).expect("removes the container");
        assert!(session.model().block(1).is_none());
        assert!(session.model().block(2).is_none(), "the children go with it");
        assert!(session.model().block(3).is_none());
        assert!(session.model().block(4).is_some(), "blocks outside survive");
        session.undo().expect("undoes");
        assert!(session.model().block(2).is_some());
        assert_eq!(
            session.model().block(1).expect("container").links.len(),
            1,
            "the container's links come back with it"
        );
    }

    #[test]
    fn removing_the_model_root_is_refused() {
        let mut session = EditSession::new(model());
        let err = session.apply(EditCommand::RemoveBlock { id: 0 }).expect_err("refused");
        assert!(err.to_string().contains("raíz"), "{err}");
    }

    #[test]
    fn move_block_keeps_the_size_and_fills_the_default() {
        let mut session = EditSession::new(model());
        session.apply(EditCommand::MoveBlock { id: 1, x: 5, y: 6 }).expect("moves");
        let layout = session.model().block(1).expect("block").layout.expect("layout");
        assert_eq!((layout.width, layout.height, layout.x, layout.y), (120, 63, 5, 6));
        // A block without layout gets the default size.
        let mut stripped = model();
        stripped.block_mut(1).expect("block").layout = None;
        let mut session = EditSession::new(stripped);
        session.apply(EditCommand::MoveBlock { id: 1, x: 7, y: 8 }).expect("moves");
        let layout = session.model().block(1).expect("block").layout.expect("layout");
        assert_eq!((layout.width, layout.height, layout.x, layout.y), (120, 48, 7, 8));
    }

    #[test]
    fn set_layouts_is_one_undo_entry() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::SetLayouts {
                positions: vec![
                    Position { id: 1, x: 1, y: 1, width: None, height: None },
                    Position { id: 2, x: 2, y: 2, width: None, height: None },
                ],
            })
            .expect("sets");
        assert_eq!(session.model().block(1).expect("b").layout.unwrap().x, 1);
        assert_eq!(session.model().block(2).expect("b").layout.unwrap().y, 2);
        assert!(session.can_undo());
        session.undo().expect("undoes");
        assert_eq!(session.model().block(1).expect("b").layout.unwrap().x, 10);
        assert_eq!(session.model().block(2).expect("b").layout.unwrap().x, 100);
        assert!(!session.can_undo());
    }

    #[test]
    fn connect_shares_one_label_and_lands_in_the_container() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 0, y: 0 })
            .expect("adds Advance (id 3)");
        session
            .apply(EditCommand::Connect { from: 3, from_ioid: 2, to: 2, to_ioid: 1 })
            .expect("connects");
        let model = session.model();
        let advance = model.block(3).expect("advance");
        let terminate = model.block(2).expect("terminate");
        // The target port already carries A_Queue; an existing label is kept.
        assert_eq!(advance.output(2).expect("out").mylabel, "A_Queue");
        assert_eq!(terminate.input(1).expect("in").mylabel, "A_Queue");
        let links = &model.block(0).expect("root").links;
        // The link that ended at that input is kept: an input may receive
        // from several blocks.
        assert_eq!(links.len(), 2);
        assert_eq!(
            ends(&links[0]),
            "1:1->2:1",
            "the link the Generate already had is kept"
        );
        let link = links
            .iter()
            .find(|l| l.from_module == 3)
            .expect("the new link");
        assert_eq!(ends(link), "3:2->2:1");
    }

    #[test]
    fn connect_keeps_an_existing_label_and_mints_a_taken_base_with_suffix() {
        let mut session = EditSession::new(model());
        // The Generate's output already carries A_Queue; connecting from it
        // keeps that label on both ends.
        session
            .apply(EditCommand::AddBlock { class: "Queue".into(), container: 0, x: 0, y: 0 })
            .expect("adds Queue (id 3)");
        session
            .apply(EditCommand::Connect { from: 1, from_ioid: 1, to: 3, to_ioid: 1 })
            .expect("connects");
        let model = session.model();
        assert_eq!(model.block(3).expect("queue").input(1).expect("in").mylabel, "A_Queue");
        // Two unlabelled fresh blocks: the label follows the original
        // editor's convention A_<ClassOfTargetBlock>.
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 0, y: 0 })
            .expect("adds Advance (id 4)");
        session
            .apply(EditCommand::AddBlock { class: "Terminate".into(), container: 0, x: 0, y: 0 })
            .expect("adds Terminate (id 5)");
        session
            .apply(EditCommand::Connect { from: 4, from_ioid: 2, to: 5, to_ioid: 1 })
            .expect("mints A_Terminate");
        assert_eq!(
            session.model().block(5).expect("t").input(1).expect("in").mylabel,
            "A_Terminate"
        );
        // Another pair of fresh blocks: the base A_Terminate is taken, so the
        // numeric suffix is 1.
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 0, y: 0 })
            .expect("adds Advance (id 6)");
        session
            .apply(EditCommand::AddBlock { class: "Terminate".into(), container: 0, x: 0, y: 0 })
            .expect("adds Terminate (id 7)");
        session
            .apply(EditCommand::Connect { from: 6, from_ioid: 2, to: 7, to_ioid: 1 })
            .expect("mints the suffix");
        assert_eq!(
            session.model().block(7).expect("t").input(1).expect("in").mylabel,
            "A_Terminate1"
        );
    }

    #[test]
    fn connecting_over_an_occupied_output_replaces_the_link() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 0, y: 0 })
            .expect("adds Advance (id 3)");
        session
            .apply(EditCommand::AddBlock { class: "Terminate".into(), container: 0, x: 0, y: 0 })
            .expect("adds Terminate (id 4)");
        session
            .apply(EditCommand::Connect { from: 3, from_ioid: 2, to: 4, to_ioid: 1 })
            .expect("links Advance to the fresh Terminate");
        assert_eq!(session.model().block(0).expect("root").links.len(), 2);
        session
            .apply(EditCommand::Connect { from: 1, from_ioid: 1, to: 4, to_ioid: 1 })
            .expect("the occupied output port is freed first");
        let links = &session.model().block(0).expect("root").links;
        // The source port's own previous link (1->2) is removed: one port,
        // one link. The link that ended at the target port (3->4) stays:
        // an input may receive from several blocks.
        assert_eq!(links.len(), 2);
        assert!(
            links.iter().any(|l| l.from_module == 3 && l.to_module == 4),
            "the link that ends at the target port is kept"
        );
        assert!(
            !links.iter().any(|l| l.from_module == 1 && l.to_module == 2),
            "the link that starts at the source port is replaced"
        );
        let new_link = links
            .iter()
            .find(|l| l.from_module == 1)
            .expect("the new link");
        assert_eq!(ends(new_link), "1:1->4:1");
    }

    #[test]
    fn connect_refuses_cross_container_and_missing_ports() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 1, x: 0, y: 0 })
            .expect_err("Generate is not a container");
        let err = session
            .apply(EditCommand::Connect { from: 1, from_ioid: 7, to: 2, to_ioid: 1 })
            .expect_err("missing output port");
        assert!(err.to_string().contains("puerto de salida 7"), "{err}");
        let err = session
            .apply(EditCommand::Connect { from: 1, from_ioid: 1, to: 2, to_ioid: 7 })
            .expect_err("missing input port");
        assert!(err.to_string().contains("puerto de entrada 7"), "{err}");
    }

    #[test]
    fn disconnect_keeps_the_labels() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::Disconnect { container: 0, from: 1, from_ioid: 1 })
            .expect("disconnects");
        assert!(session.model().block(0).expect("root").links.is_empty());
        assert_eq!(
            session.model().block(1).expect("g").output(1).expect("out").mylabel,
            "A_Queue",
            "the source port keeps its label"
        );
        assert_eq!(
            session.model().block(2).expect("t").label(),
            Some("A_Queue"),
            "the block's own label survives the disconnection"
        );
    }

    #[test]
    fn disconnect_without_a_link_is_refused() {
        let mut session = EditSession::new(model());
        let err = session
            .apply(EditCommand::Disconnect { container: 0, from: 2, from_ioid: 1 })
            .expect_err("no link starts there");
        assert!(err.to_string().contains("no hay un enlace"), "{err}");
    }

    #[test]
    fn set_property_keeps_the_encoding_convention_and_refuses_unknowns() {
        let mut session = EditSession::new(model());
        // Media's stored value has no `%`, so the plain convention is kept.
        session
            .apply(EditCommand::SetProperty { id: 1, property: "Media".into(), value: "90".into() })
            .expect("sets");
        assert_eq!(session.model().block(1).expect("b").prop("Media").expect("p").raw_value, "90");
        let err = session
            .apply(EditCommand::SetProperty { id: 1, property: "Inexistente".into(), value: "1".into() })
            .expect_err("undeclared property");
        assert!(err.to_string().contains("Inexistente"), "{err}");
    }

    #[test]
    fn set_property_on_an_encoded_value_stays_encoded() {
        let mut prepared = model();
        prepared
            .block_mut(1)
            .expect("b")
            .prop_mut("Media")
            .expect("p")
            .raw_value = "tiempo%20medio".into();
        let mut session = EditSession::new(prepared);
        session
            .apply(EditCommand::SetProperty { id: 1, property: "Media".into(), value: "nuevo valor".into() })
            .expect("sets");
        assert_eq!(
            session.model().block(1).expect("b").prop("Media").expect("p").raw_value,
            "nuevo%20valor",
            "an encoded value stays encoded"
        );
    }

    #[test]
    fn set_caption_changes_the_caption() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::SetCaption { id: 1, caption: "Llegadas".into() })
            .expect("sets");
        assert_eq!(session.model().block(1).expect("b").caption, "Llegadas");
    }

    #[test]
    fn paste_gives_fresh_ids_remapped_links_and_fresh_labels() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::AddBlock { class: "Queue".into(), container: 0, x: 0, y: 0 })
            .expect("adds Queue (id 3)");
        // Link Generate -> Queue: a real chain to copy.
        session
            .apply(EditCommand::Connect { from: 1, from_ioid: 1, to: 3, to_ioid: 1 })
            .expect("connects");
        let ids = session
            .apply(EditCommand::Paste { ids: vec![1, 3], dx: 15, dy: 25 })
            .expect("pastes");
        assert_eq!(ids, vec![4, 5], "fresh ids in request order");
        let copy_gen = session.model().block(4).expect("copy");
        let copy_queue = session.model().block(5).expect("copy");
        assert_eq!(copy_gen.class, "Generate");
        assert_eq!(copy_queue.class, "Queue");
        // The offset moves the copies.
        let original = session.model().block(1).expect("original").layout.unwrap();
        let copied = copy_gen.layout.unwrap();
        assert_eq!((copied.x, copied.y), (original.x + 15, original.y + 25));
        // Every port label of the copies is empty or fresh: none is A_Queue.
        for block in [copy_gen, copy_queue] {
            for port in block.inputs.iter().chain(&block.outputs) {
                assert_ne!(port.mylabel, "A_Queue", "a pasted label must be fresh");
            }
        }
        // The link between the copies exists, remapped, with fresh labels on
        // both ends that agree with each other.
        let links = &session.model().block(0).expect("root").links;
        let remapped = links
            .iter()
            .find(|l| l.from_module == 4)
            .expect("the copied link");
        assert_eq!((remapped.from_ioid, remapped.to_module, remapped.to_ioid), (1, 5, 1));
        let out_label = copy_gen.output(1).expect("out").mylabel.clone();
        assert!(!out_label.is_empty());
        assert_eq!(copy_queue.input(1).expect("in").mylabel, out_label);
        assert_eq!(out_label, "A_Queue1", "A_Queue is taken, so the suffix is 1");
        // The original link and labels are untouched.
        assert!(links.iter().any(|l| l.from_module == 1 && l.to_module == 3));
    }

    #[test]
    fn paste_of_two_copies_never_reuses_a_label() {
        let mut session = EditSession::new(model());
        let first = session
            .apply(EditCommand::Paste { ids: vec![1, 2], dx: 10, dy: 10 })
            .expect("first paste");
        let second = session
            .apply(EditCommand::Paste { ids: vec![1, 2], dx: 20, dy: 20 })
            .expect("second paste");
        // The two ends of one link share one label by design, so the label is
        // taken once per link: from the source end of every link that starts
        // at a pasted block.
        let mut labels: Vec<String> = Vec::new();
        for container in session.model().blocks.iter().filter(|b| b.is_container()) {
            for link in &container.links {
                if first.contains(&link.from_module) || second.contains(&link.from_module) {
                    let label = session
                        .model()
                        .block(link.from_module)
                        .and_then(|b| b.output(link.from_ioid))
                        .map(|p| p.mylabel.clone())
                        .expect("the link's source end carries the label");
                    assert!(!label.is_empty(), "a pasted link has a fresh label");
                    labels.push(label);
                }
            }
        }
        assert_eq!(labels.len(), 2, "one remapped link per paste");
        let original_labels: Vec<String> = [1u32, 2]
            .iter()
            .filter_map(|id| session.model().block(*id))
            .flat_map(|b| b.inputs.iter().chain(&b.outputs))
            .filter_map(|p| p.label().map(str::to_string))
            .collect();
        assert_eq!(original_labels, vec!["A_Queue".to_string(), "A_Queue".to_string()]);
        for label in &labels {
            assert_ne!(label, "A_Queue", "a pasted label must be fresh");
        }
        let mut unique = labels.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), labels.len(), "the two pastes never share a label");
    }

    #[test]
    fn undo_redo_restores_the_model_exactly() {
        let mut session = EditSession::new(model());
        let original = crate::to_string(session.model());
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 5, y: 5 })
            .expect("adds");
        session.apply(EditCommand::SetCaption { id: 3, caption: "x".into() }).expect("caption");
        session
            .apply(EditCommand::Connect { from: 3, from_ioid: 2, to: 2, to_ioid: 1 })
            .expect("connects");
        session
            .apply(EditCommand::SetProperty { id: 3, property: "Media".into(), value: "7".into() })
            .expect("property");
        session
            .apply(EditCommand::SetProperty { id: 1, property: "Media".into(), value: "a b".into() })
            .expect("property");
        session.apply(EditCommand::MoveBlock { id: 1, x: 42, y: 43 }).expect("moves");
        session.apply(EditCommand::RemoveBlock { id: 2 }).expect("removes");
        assert_ne!(crate::to_string(session.model()), original);
        while session.undo().expect("undo ok") {}
        assert_eq!(crate::to_string(session.model()), original, "undo restores the model exactly");
        assert!(!session.can_undo());
        while session.redo().expect("redo ok") {}
        let redone = crate::to_string(session.model());
        assert_ne!(redone, original, "redo re-applies every command");
        while session.undo().expect("undo ok") {}
        assert_eq!(crate::to_string(session.model()), original);
        while session.redo().expect("redo ok") {}
        assert_eq!(crate::to_string(session.model()), redone);
    }

    #[test]
    fn a_new_command_clears_the_redo_stack() {
        let mut session = EditSession::new(model());
        session.apply(EditCommand::MoveBlock { id: 1, x: 1, y: 1 }).expect("cmd");
        session.undo().expect("undo");
        assert!(session.can_redo());
        session.apply(EditCommand::MoveBlock { id: 1, x: 2, y: 2 }).expect("cmd");
        assert!(!session.can_redo(), "a new command clears the redo stack");
        assert!(!session.redo().expect("redo ok"));
    }

    #[test]
    fn the_dirty_flag_follows_the_last_checkpoint() {
        let mut session = EditSession::new(model());
        session.apply(EditCommand::MoveBlock { id: 1, x: 1, y: 1 }).expect("cmd");
        assert!(session.dirty());
        session.mark_saved();
        assert!(!session.dirty(), "the saved state is clean");
        // An edit after the save is a difference from the saved state.
        session.apply(EditCommand::MoveBlock { id: 1, x: 2, y: 2 }).expect("cmd");
        assert!(session.dirty());
        // Undoing it leaves the model exactly as it was saved: the model is
        // clean again, not "changed since the save".
        session.undo().expect("undo");
        assert!(!session.dirty(), "the model is the saved one again");
        // And so is redoing it back to the saved state.
        session.redo().expect("redo");
        assert!(session.dirty());
        session.undo().expect("undo");
        assert!(!session.dirty());
        // Without a save, undoing everything returns to the clean start.
        let mut session = EditSession::new(model());
        session.apply(EditCommand::MoveBlock { id: 1, x: 1, y: 1 }).expect("cmd");
        session.undo().expect("undo");
        assert!(!session.dirty());
    }

    #[test]
    fn a_failed_command_changes_nothing() {
        let mut session = EditSession::new(model());
        let before = crate::to_string(session.model());
        // A position of a block that does not exist: the command fails after
        // the first position was already applied.
        let error = session
            .apply(EditCommand::SetLayouts {
                positions: vec![
                    Position { id: 1, x: 1, y: 1, width: None, height: None },
                    Position { id: 999, x: 2, y: 2, width: None, height: None },
                ],
            })
            .expect_err("the unknown block is refused");
        assert!(error.to_string().contains("999"), "{error}");
        assert_eq!(
            crate::to_string(session.model()),
            before,
            "the failed command changed the model"
        );
        assert!(!session.can_undo(), "the failed command left an undo entry");
        assert!(!session.dirty(), "the failed command made the model dirty");
    }

    #[test]
    fn a_failed_removal_changes_nothing() {
        let mut session = EditSession::new(model());
        let before = crate::to_string(session.model());
        // The model root cannot be removed: the command fails and the model
        // keeps the block that was removed before the failure.
        let error = session
            .apply(EditCommand::RemoveBlocks { ids: vec![1, 0] })
            .expect_err("the root is refused");
        assert!(error.to_string().contains("raíz"), "{error}");
        assert_eq!(
            crate::to_string(session.model()),
            before,
            "the failed command changed the model"
        );
        assert!(!session.can_undo());
    }

    #[test]
    fn one_command_is_one_undo_entry() {
        let mut session = EditSession::new(model());
        session
            .apply(EditCommand::AddBlock { class: "Advance".into(), container: 0, x: 0, y: 0 })
            .expect("adds");
        session
            .apply(EditCommand::Paste { ids: vec![3], dx: 1, dy: 1 })
            .expect("pastes (one entry though it adds blocks, labels and links)");
        session
            .apply(EditCommand::SetLayouts {
                positions: vec![
                    Position { id: 1, x: 1, y: 1, width: None, height: None },
                    Position { id: 2, x: 2, y: 2, width: None, height: None },
                ],
            })
            .expect("sets");
        assert!(session.can_undo());
        session.undo().expect("undo 1");
        assert!(session.can_undo());
        session.undo().expect("undo 2");
        assert!(session.can_undo());
        session.undo().expect("undo 3");
        assert!(!session.can_undo(), "three commands, exactly three entries");
    }

    #[test]
    fn the_wire_form_uses_the_camel_case_tag() {
        let command = EditCommand::AddBlock {
            class: "Generate".into(),
            container: 0,
            x: 10,
            y: 20,
        };
        let json = serde_json::to_string(&command).expect("serializes");
        assert_eq!(json, r#"{"op":"addBlock","class":"Generate","container":0,"x":10,"y":20}"#);
        let back: EditCommand = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, command);
        let connect = EditCommand::Connect { from: 1, from_ioid: 2, to: 3, to_ioid: 4 };
        let json = serde_json::to_string(&connect).expect("serializes");
        assert_eq!(json, r#"{"op":"connect","from":1,"fromIoid":2,"to":3,"toIoid":4}"#);
    }
}
