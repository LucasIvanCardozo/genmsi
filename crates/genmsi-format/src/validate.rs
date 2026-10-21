//! Structural and runnable validation of a parsed model.
//!
//! Two layers, mirroring the original tool:
//!
//! - [`validate`] runs at load time and rejects structurally broken models
//!   (dangling links, unknown owners, duplicated ids, invalid entity names).
//!   The original reports this class as `Error en linea N -> Bloque inválido`.
//! - [`validate_runnable`] runs before a simulation starts and rejects models
//!   the engine cannot terminate or whose required operands are missing. The
//!   original reports this class as `No hay bloques TERMINATE con parámetro A`
//!   and `El parámetro B no debe ser nulo`.
//!
//! Rules that were inferred from the oracle fixtures are marked "oracle" and
//! cite the baseline that proves them.

use std::collections::{HashMap, HashSet};

use crate::error::FormatError;
use crate::model::{Block, GmdlModel};

/// Validate the structural invariants of a model.
pub fn validate(model: &GmdlModel) -> Result<(), FormatError> {
    check_unique_ids(model)?;
    let root = check_root(model)?;
    check_ownership(model, root)?;
    check_links(model)?;
    check_entity_names(model)?;
    Ok(())
}

/// Validate that the model can actually run to completion.
///
/// Oracle: `fixtures/outputs/Problema_2__puenteLevadizo.report.txt` and
/// `TP6__Clase_6_0_TestGate.report.txt` show
/// `Advertencia -> No hay bloques TERMINATE con parámetro A` followed by a
/// refusal to start; every runnable baseline has either a `Clock` block or a
/// `TERMINATE` with a positive `A` operand.
pub fn validate_runnable(model: &GmdlModel) -> Result<(), FormatError> {
    // Oracle: `TP9__tp9.report.txt` fails with
    // `Error en linea 35 -> El parámetro B no debe ser nulo` on an UNLINK whose
    // "unlinked transactions" output carries no label. The other two outputs
    // may be unconnected (TP10's UNLINK blocks prove it).
    for block in model.activity_blocks().filter(|b| b.class == "Unlink") {
        let target = block
            .outputs
            .iter()
            .find(|p| p.name == "SalidaAlt" || p.ioid == 3);
        let labelled = target.and_then(|p| p.label()).is_some();
        if !labelled {
            return Err(FormatError::structural(
                block.id,
                format!(
                    "{}: the operand B must not be null; UNLINK needs a label for the \
                     unlinked transactions",
                    block.name
                ),
            ));
        }
    }

    let has_clock = model
        .activity_blocks()
        .any(|b| b.class == "Clock" && b.prop_int("Frecuencia").is_some_and(|f| f > 0));
    let has_terminate_a = model.activity_blocks().any(|b| {
        b.class == "Terminate" && b.prop_int("Unidades").is_some_and(|units| units > 0)
    });
    if !has_clock && !has_terminate_a {
        let node = model
            .activity_blocks()
            .find(|b| b.class == "Terminate")
            .map(|b| b.id)
            .unwrap_or(root_id(model));
        return Err(FormatError::structural(
            node,
            "no TERMINATE block with operand A and no CLOCK block: the simulation \
             would never finish",
        ));
    }

    Ok(())
}

fn root_id(model: &GmdlModel) -> u32 {
    model.root().map(|b| b.id).unwrap_or(0)
}

fn check_unique_ids(model: &GmdlModel) -> Result<(), FormatError> {
    let mut seen = HashSet::new();
    for block in &model.blocks {
        if !seen.insert(block.id) {
            return Err(FormatError::structural(
                block.id,
                format!("duplicate module id {}", block.id),
            ));
        }
    }
    Ok(())
}

fn check_root(model: &GmdlModel) -> Result<u32, FormatError> {
    let roots: Vec<&Block> = model.blocks.iter().filter(|b| b.is_root()).collect();
    match roots.as_slice() {
        [root] => {
            if root.kind != 'C' {
                return Err(FormatError::structural(
                    root.id,
                    "the model root must be a container (kind C)",
                ));
            }
            Ok(root.id)
        }
        [] => Err(FormatError::Structural {
            location: Default::default(),
            message: "model has no main block (moduleclass=\"main\")".to_string(),
        }),
        _ => {
            let ids: Vec<String> = roots.iter().map(|b| b.id.to_string()).collect();
            Err(FormatError::structural(
                roots[0].id,
                format!("model has more than one main block: {}", ids.join(", ")),
            ))
        }
    }
}

fn check_ownership(model: &GmdlModel, root: u32) -> Result<(), FormatError> {
    let ids: HashSet<u32> = model.blocks.iter().map(|b| b.id).collect();
    for block in &model.blocks {
        if block.id == root {
            continue;
        }
        if !ids.contains(&block.owner) {
            return Err(FormatError::structural(
                block.id,
                format!("owner {} does not exist", block.owner),
            ));
        }
    }
    // Detect ownership cycles: walk each chain up to the root.
    for block in &model.blocks {
        let mut current = block.id;
        let mut steps = 0;
        while let Some(node) = model.block(current) {
            if node.id == root {
                break;
            }
            if node.owner == node.id {
                return Err(FormatError::structural(
                    node.id,
                    "block owns itself outside the model root",
                ));
            }
            current = node.owner;
            steps += 1;
            if steps > model.blocks.len() {
                return Err(FormatError::structural(
                    block.id,
                    "ownership cycle detected",
                ));
            }
        }
    }
    Ok(())
}

fn check_links(model: &GmdlModel) -> Result<(), FormatError> {
    let by_id: HashMap<u32, &Block> = model.blocks.iter().map(|b| (b.id, b)).collect();
    for container in model.blocks.iter().filter(|b| b.is_container()) {
        for link in &container.links {
            let from = by_id.get(&link.from_module).ok_or_else(|| {
                FormatError::structural(
                    container.id,
                    format!("link references missing source module {}", link.from_module),
                )
            })?;
            let to = by_id.get(&link.to_module).ok_or_else(|| {
                FormatError::structural(
                    container.id,
                    format!("link references missing target module {}", link.to_module),
                )
            })?;
            if from.id == container.id && link.to_module == container.id {
                return Err(FormatError::structural(
                    container.id,
                    "link connects a container to itself",
                ));
            }
            if from.id != container.id {
                if from.output(link.from_ioid).is_none() {
                    return Err(FormatError::structural(
                        from.id,
                        format!("link starts at missing output port {}", link.from_ioid),
                    ));
                }
                if from.owner != container.id {
                    return Err(FormatError::structural(
                        container.id,
                        format!(
                            "link source {} is not a child of container {}",
                            from.id, container.id
                        ),
                    ));
                }
            }
            if to.id != container.id {
                if to.input(link.to_ioid).is_none() {
                    return Err(FormatError::structural(
                        to.id,
                        format!("link ends at missing input port {}", link.to_ioid),
                    ));
                }
                if to.owner != container.id {
                    return Err(FormatError::structural(
                        container.id,
                        format!(
                            "link target {} is not a child of container {}",
                            to.id, container.id
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Entity-definition names must be legal GPSS identifiers: no whitespace and
/// no `:` (the SNA separator).
///
/// Oracle: `Parcial 8-11-21__ParcialMikelModelos` is the only model of the 25
/// whose QTable definition name contains a space (`tiempo demora`) and it is
/// one of the five outputs where the original reports
/// `Error en linea 9 -> Bloque inválido`. Bracket placeholders such as
/// `[storage]` are legal (TP10 runs with one).
fn check_entity_names(model: &GmdlModel) -> Result<(), FormatError> {
    for block in &model.blocks {
        for prop in &block.props {
            if !prop.unique || !prop.is_used() {
                continue;
            }
            let value = prop.value();
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.chars().any(|ch| ch.is_whitespace() || ch == ':') {
                return Err(FormatError::structural(
                    block.id,
                    format!(
                        "{}: invalid block: definition name {:?} contains whitespace or ':'",
                        block.name, trimmed
                    ),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;

    fn model(blocks: &str) -> GmdlModel {
        let xml = format!(
            "<gmodel versioncode=\"1\"><gclasses/><modules>\
             <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\
             {blocks}</modules></gmodel>"
        );
        parse_str(&xml).expect("test model parses")
    }

    fn activity(id: u32, class: &str, props: &str) -> String {
        format!(
            "<module moduleid=\"{id}\" moduleclass=\"{class}\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">\
             <inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L{id}\"/></inputs>\
             <outputs><output ioid=\"2\" name=\"o\" caption=\"\" mylabel=\"O{id}\"/></outputs>\
             <props>{props}</props></module>"
        )
    }

    fn prop(name: &str, value: &str, unique: bool) -> String {
        format!(
            "<property name=\"{name}\" hint=\"\" value=\"{value}\" type=\"EntityDef\" subType=\"X\" visible=\"1\" unique=\"{}\"/>",
            if unique { 1 } else { 0 }
        )
    }

    #[test]
    fn accepts_a_minimal_runnable_model() {
        let model = model(&activity(
            1,
            "Clock",
            &prop("Frecuencia", "3600", false),
        ));
        validate(&model).expect("valid");
        validate_runnable(&model).expect("runnable");
    }

    #[test]
    fn rejects_duplicate_ids() {
        let model = model(&format!(
            "{}{}",
            activity(1, "Clock", &prop("Frecuencia", "3600", false)),
            activity(1, "Clock", &prop("Frecuencia", "3600", false))
        ));
        let err = validate(&model).expect_err("must fail");
        assert!(err.to_string().contains("duplicate module id"), "{err}");
    }

    #[test]
    fn rejects_missing_owner() {
        let model = model(
            "<module moduleid=\"5\" moduleclass=\"Advance\" owner=\"99\" caption=\"c\" name=\"n\" kind=\"A\"><props/></module>",
        );
        let err = validate(&model).expect_err("must fail");
        assert!(err.to_string().contains("owner 99"), "{err}");
    }

    #[test]
    fn rejects_dangling_link_port() {
        let xml = "<gmodel versioncode=\"1\"><gclasses/><modules>\
            <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\">\
            <links><link frommoduleid=\"1\" fromioid=\"9\" tomoduleid=\"2\" toioid=\"1\"/></links>\
            <inputs/><outputs/></module>\
            <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\"><outputs><output ioid=\"1\" name=\"o\" caption=\"\" mylabel=\"A\"/></outputs><props/></module>\
            <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"A\"/></inputs><props/></module>\
            </modules></gmodel>";
        let err = validate(&parse_str(xml).unwrap()).expect_err("must fail");
        assert!(err.to_string().contains("missing output port 9"), "{err}");
    }

    #[test]
    fn rejects_definition_names_with_whitespace_or_colon() {
        for bad in ["tiempo demora", "Q:cola"] {
            let model = model(&activity(1, "QTable", &prop("NombreTabla", bad, true)));
            let err = validate(&model).expect_err("must fail");
            assert!(err.to_string().contains("invalid block"), "{err}");
            assert_eq!(err.location().node, Some(1));
        }
        let ok = model(&activity(1, "StorageDef", &prop("Storage", "[storage]", true)));
        validate(&ok).expect("bracket placeholders are legal");
    }

    #[test]
    fn refuses_models_without_termination() {
        let model = model(&activity(1, "Generate", &prop("Media", "60", false)));
        let err = validate_runnable(&model).expect_err("must fail");
        assert!(err.to_string().contains("TERMINATE"), "{err}");
    }

    #[test]
    fn accepts_terminate_with_positive_operand_a() {
        let model = model(&activity(
            1,
            "Terminate",
            &prop("Unidades", "1", false),
        ));
        validate_runnable(&model).expect("runnable");
    }

    #[test]
    fn refuses_zero_terminate_without_clock() {
        let model = model(&activity(
            1,
            "Terminate",
            &prop("Unidades", "0", false),
        ));
        assert!(validate_runnable(&model).is_err());
    }

    fn unlink(label: &str) -> String {
        format!(
            "<module moduleid=\"2\" moduleclass=\"Unlink\" owner=\"0\" caption=\"c\" name=\"u\" kind=\"A\">\
             <inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L\"/></inputs>\
             <outputs>\
             <output ioid=\"2\" name=\"Salida1\" caption=\"\" mylabel=\"\"/>\
             <output ioid=\"3\" name=\"SalidaAlt\" caption=\"\" mylabel=\"{label}\"/>\
             <output ioid=\"4\" name=\"Salida2\" caption=\"\" mylabel=\"\"/>\
             </outputs>\
             <props><property name=\"Cadena\" hint=\"\" value=\"C\" type=\"EntityUse\" subType=\"UserChain\" visible=\"1\" unique=\"0\"/></props></module>"
        )
    }

    #[test]
    fn refuses_unlink_without_operand_b() {
        let clock = activity(1, "Clock", &prop("Frecuencia", "3600", false));
        let broken = model(&format!("{clock}{}", unlink("")));
        let err = validate_runnable(&broken).expect_err("must fail");
        assert!(err.to_string().contains("operand B"), "{err}");
        assert_eq!(err.location().node, Some(2));

        let fixed = model(&format!("{clock}{}", unlink("Salida")));
        validate_runnable(&fixed).expect("labelled unlink is runnable");
    }
}
