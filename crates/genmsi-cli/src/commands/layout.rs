//! `genmsi layout <model>`: compose the canvas of a model and say what it
//! composed.
//!
//! Where a block sits on the canvas and how big it is is the engine's
//! business: a model that carries no geometry is a model the editor cannot
//! draw. The command composes it and prints where every block went, but it
//! writes nothing: the write is explicit (`--write`), so a reader sees the
//! composition before the file changes.
//!
//! A model that already carries its geometry is left exactly as its author
//! left it: the composition is only for the models the editor could not draw.

use serde::Serialize;

use crate::cli::LayoutArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, LAYOUT_SCHEMA};

/// The document the command prints.
#[derive(Debug, Serialize)]
struct LayoutDocument {
    /// Path of the model, as the user wrote it.
    path: String,
    /// `composed` when the engine composed the model, `unchanged` when the
    /// model already carried its geometry and was left alone.
    status: &'static str,
    /// The blocks of the model, the root included.
    blocks: usize,
    /// The blocks the engine placed.
    composed: usize,
    /// Whether the composed model was written back to the file.
    written: bool,
    /// Where every block was placed.
    positions: Vec<Placed>,
}

/// Where a block was placed.
#[derive(Debug, Serialize)]
struct Placed {
    /// Id of the block.
    id: u32,
    /// Horizontal position.
    x: i32,
    /// Vertical position.
    y: i32,
    /// Width of the block.
    width: i32,
    /// Height of the block.
    height: i32,
}

/// Human-readable form of a layout document.
fn render(document: &LayoutDocument) -> String {
    let mut out = String::new();
    out.push_str(&format!("model: {}\n", document.path));
    out.push_str(&format!("status: {}\n", document.status));
    out.push_str(&format!("blocks: {}\n", document.blocks));
    out.push_str(&format!("composed: {}\n", document.composed));
    out.push_str(&format!(
        "written: {}\n",
        if document.written { "yes" } else { "no" }
    ));
    if document.positions.is_empty() {
        out.push_str("positions: none\n");
    } else {
        out.push_str("positions:\n");
        for position in &document.positions {
            out.push_str(&format!(
                "  - block {} at ({}, {}) size {}x{}\n",
                position.id, position.x, position.y, position.width, position.height
            ));
        }
    }
    out
}

/// Where the engine placed one block.
fn placed_of(position: &genmsi_format::Position) -> Placed {
    Placed {
        id: position.id,
        x: position.x,
        y: position.y,
        width: position
            .width
            .unwrap_or(genmsi_format::layout::DEFAULT_WIDTH),
        height: position
            .height
            .unwrap_or(genmsi_format::layout::DEFAULT_HEIGHT),
    }
}

/// Run `genmsi layout`.
pub fn run(args: LayoutArgs) -> Result<ExitCode, CliError> {
    // The model is loaded without compiling it: the composition is a matter
    // of geometry, and a model the engine refuses can still be laid out.
    let mut loaded = model::load_geometry(&args.model.path)?;
    // The composition: the engine places the blocks of a model whose
    // geometry is incomplete, and leaves a model that already carries one
    // exactly as its author left it.
    let composed = genmsi_format::compose_incomplete(&mut loaded);
    let positions: Vec<Placed> = composed
        .as_ref()
        .map(|positions| positions.iter().map(placed_of).collect())
        .unwrap_or_default();
    if args.write {
        // Only a model the engine composed is written: a model that already
        // carries its geometry is left alone, byte for byte. A command that
        // lays the canvas out does not re-serialize a file that needs no
        // layout.
        if composed.is_some() {
            model::save(&loaded, &args.model.path)?;
        }
    }
    let document = LayoutDocument {
        path: args.model.path.display().to_string(),
        status: if composed.is_some() {
            "composed"
        } else {
            "unchanged"
        },
        blocks: loaded.blocks.len(),
        composed: positions.len(),
        written: args.write && composed.is_some(),
        positions,
    };
    if args.output.json {
        output::print_json(LAYOUT_SCHEMA, &document)?;
    } else {
        output::print(&render(&document))?;
    }
    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(status: &'static str, written: bool) -> LayoutDocument {
        LayoutDocument {
            path: "model.gmdl".to_string(),
            status,
            blocks: 3,
            composed: 2,
            written,
            positions: vec![Placed {
                id: 1,
                x: 0,
                y: 0,
                width: 120,
                height: 48,
            }],
        }
    }

    #[test]
    fn the_human_form_says_where_every_block_went() {
        let text = render(&document("composed", false));
        assert!(text.contains("model: model.gmdl"), "{text}");
        assert!(text.contains("status: composed"), "{text}");
        assert!(text.contains("composed: 2"), "{text}");
        assert!(text.contains("written: no"), "{text}");
        assert!(
            text.contains("  - block 1 at (0, 0) size 120x48"),
            "{text}"
        );
    }

    #[test]
    fn a_model_that_was_not_composed_has_no_positions() {
        let text = render(&document("unchanged", true));
        assert!(text.contains("status: unchanged"), "{text}");
        assert!(text.contains("written: yes"), "{text}");
    }
}
