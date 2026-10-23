//! Parsing, validation and writing of GenMSI model files (`.gmdl`).
//!
//! Design rules (see `docs/DECISIONS.md`):
//!
//! - The semantic model (blocks, ports, links, properties) is stored separately
//!   from canvas layout (positions, sizes, breakpoints). Layout never
//!   influences simulation and is preserved across a rewrite.
//! - Every value read from XML is validated on load; the parser rejects
//!   structurally invalid models with precise errors instead of "fixing" them
//!   silently (the original tool accumulated structural bugs this way).
//! - This crate is the only place that knows the `.gmdl` on-disk format.
//!
//! # Example
//!
//! ```
//! use genmsi_format::{load, save};
//!
//! # let dir = std::env::temp_dir().join("genmsi-format-doc-example");
//! # std::fs::create_dir_all(&dir).unwrap();
//! # let path = dir.join("model.gmdl");
//! # std::fs::write(&path, r#"<gmodel versioncode="1"><gclasses/><modules><module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links/><inputs/><outputs/></module><module moduleid="1" moduleclass="Clock" owner="0" caption="c" name="n" kind="A"><props><property name="Frecuencia" hint="" value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module></modules></gmodel>"#).unwrap();
//! let model = load(&path).expect("model loads");
//! assert_eq!(model.activity_blocks().count(), 1);
//! save(&model, &path).expect("model saves");
//! ```

pub mod catalogue;
pub mod declaration;
pub mod edit;
pub mod error;
pub mod model;
pub mod parser;
pub mod validate;
pub mod writer;

pub use catalogue::{ClassSpec, PortSpec, PropertySpec, build_block, class_spec, listed_options};
pub use declaration::{
    UndeclaredEntity, declaration_warnings, undeclared_entities, validate_declarations,
};
pub use edit::{EditCommand, EditSession, Position, UndoStep};
pub use error::{FormatError, Location};
pub use model::{
    Block, ContainerHeader, CustomClass, GmdlModel, Layout, Link, Port, PortLayout, Property,
    SectionKind, SourcePos, percent_decode, percent_encode,
};
pub use parser::{parse_file, parse_str};
pub use validate::{validate, validate_runnable};
pub use writer::{to_string, write_file};

use std::path::Path;

/// Read, parse and structurally validate a model file.
pub fn load(path: impl AsRef<Path>) -> Result<GmdlModel, FormatError> {
    let model = parse_file(path)?;
    validate(&model)?;
    Ok(model)
}

/// Parse and structurally validate model text.
pub fn load_str(text: &str) -> Result<GmdlModel, FormatError> {
    let model = parse_str(text)?;
    validate(&model)?;
    Ok(model)
}

/// Write a model back to a file.
pub fn save(model: &GmdlModel, path: impl AsRef<Path>) -> Result<(), FormatError> {
    write_file(model, path)
}
