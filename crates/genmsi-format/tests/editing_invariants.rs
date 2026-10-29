//! The editing invariants over a real reference model (read-only): pasting
//! twice must leave every id unique, every link resolvable, the model valid
//! and byte-identical round-trippable.

use std::path::Path;

use genmsi_format::{parse_str, to_string, validate, EditCommand, EditSession, Position};

fn load_reference(name: &str) -> genmsi_format::GmdlModel {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    parse_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn pasting_twice_keeps_ids_links_validity_and_round_trip() {
    let model = load_reference("TP2__problema1.gmdl");
    let before = to_string(&model);
    let mut session = EditSession::new(model);

    // Pick two linked activity blocks to copy (a source and its target); a
    // container is not a paste source (see `Paste`'s refusal).
    let (source, target) = {
        let candidate = session
            .model()
            .blocks
            .iter()
            .filter(|b| b.is_container())
            .flat_map(|b| b.links.iter())
            .find(|link| {
                let from = session.model().block(link.from_module);
                let to = session.model().block(link.to_module);
                from.is_some_and(|b| !b.is_container()) && to.is_some_and(|b| !b.is_container())
            })
            .expect("TP2 has a link between activity blocks");
        (candidate.from_module, candidate.to_module)
    };
    session
        .apply(EditCommand::Paste {
            ids: vec![source, target],
            dx: 40,
            dy: 40,
        })
        .expect("first paste");
    let first = session
        .apply(EditCommand::Paste {
            ids: vec![source, target],
            dx: 80,
            dy: 80,
        })
        .expect("second paste");

    let model = session.model();
    // Every id of the model is unique.
    let mut ids: Vec<u32> = model.blocks.iter().map(|b| b.id).collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "every moduleid is unique");

    // Every link resolves: its two ends exist and the ports exist. An end
    // on the container itself is an entry/exit link and refers to the
    // container's own input/output port (the same rule validate.rs applies).
    for container in model.blocks.iter().filter(|b| b.is_container()) {
        for link in &container.links {
            let from = model
                .block(link.from_module)
                .unwrap_or_else(|| panic!("link source {} missing", link.from_module));
            let to = model
                .block(link.to_module)
                .unwrap_or_else(|| panic!("link target {} missing", link.to_module));
            if from.id == container.id {
                assert!(
                    from.input(link.from_ioid).is_some(),
                    "entry link {link:?}: container input port missing"
                );
            } else {
                assert!(
                    from.output(link.from_ioid).is_some(),
                    "link {link:?}: source port missing"
                );
                assert_eq!(
                    from.owner, container.id,
                    "link source is a child of its container"
                );
            }
            if to.id == container.id {
                assert!(
                    to.output(link.to_ioid).is_some(),
                    "exit link {link:?}: container output port missing"
                );
            } else {
                assert!(
                    to.input(link.to_ioid).is_some(),
                    "link {link:?}: target port missing"
                );
                assert_eq!(
                    to.owner, container.id,
                    "link target is a child of its container"
                );
            }
        }
    }

    // The pasted link never shares a label with the original one.
    let pasted_labels: Vec<String> = first
        .iter()
        .filter_map(|id| model.block(*id))
        .flat_map(|b| b.inputs.iter().chain(&b.outputs))
        .filter_map(|p| p.label().map(str::to_string))
        .collect();
    assert!(!pasted_labels.is_empty(), "the pasted pair carries labels");
    let original_labels: Vec<String> = [source, target]
        .iter()
        .filter_map(|id| model.block(*id))
        .flat_map(|b| b.inputs.iter().chain(&b.outputs))
        .filter_map(|p| p.label().map(str::to_string))
        .collect();
    for label in &pasted_labels {
        assert!(
            !original_labels.contains(label),
            "label {label} is shared with the original"
        );
    }

    // The model is still structurally valid.
    validate(model).expect("the edited model is still valid");

    // And it still round-trips: rewriting the reparsed model gives the same
    // bytes again (the recorded source positions are not written back, so
    // the models are compared through the writer, their public form).
    let rewritten = to_string(model);
    let reparsed = parse_str(&rewritten).expect("the edited model parses again");
    assert_eq!(
        to_string(&reparsed),
        rewritten,
        "the round trip is lossless"
    );
    // The untouched original text is unchanged by all of this.
    assert_ne!(rewritten, before, "the paste must be visible in the file");
}

#[test]
fn undo_restores_the_reference_model_byte_for_byte() {
    let model = load_reference("TP2__problema1.gmdl");
    let before = to_string(&model);
    let mut session = EditSession::new(model);
    let queue = session
        .model()
        .activity_blocks()
        .find(|b| b.class == "Queue")
        .expect("TP2 has a Queue block")
        .id;
    let layout = session.model().block(queue).expect("queue").layout;
    let (x, y) = (layout.map(|l| l.x).unwrap_or(0) + 11, layout.map(|l| l.y).unwrap_or(0) + 7);

    session
        .apply(EditCommand::AddBlock {
            class: "Advance".into(),
            container: session.model().root().expect("root").id,
            x,
            y,
        })
        .expect("adds");
    session
        .apply(EditCommand::SetProperty {
            id: queue,
            property: "Queue".into(),
            value: "otra_cola".into(),
        })
        .expect("sets");
    session
        .apply(EditCommand::SetLayouts {
            positions: vec![Position {
                id: queue,
                x: x + 5,
                y: y + 5,
                width: None,
                height: None,
            }],
            ports: Vec::new(),
        })
        .expect("moves");
    assert!(session.dirty());
    while session.undo().expect("undo ok") {}
    assert_eq!(
        to_string(session.model()),
        before,
        "undoing everything restores the reference model byte for byte"
    );
    assert!(!session.dirty());
}
