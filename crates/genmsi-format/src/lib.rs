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
pub mod properties;
pub mod edit;
pub mod layout;
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
pub use layout::{compose_incomplete, is_composed, relayout};
pub use error::{FormatError, Location};
pub use properties::{UndeclaredProperty, property_warnings, undeclared_properties};
pub use model::{
    Block, ContainerHeader, CustomClass, GmdlModel, Layout, Link, Port, PortLayout, Property,
    SectionKind, SourcePos, percent_decode, percent_encode,
};
pub use parser::{parse_file, parse_str};
pub use validate::{validate, validate_runnable};
pub use writer::{to_string, write_file};

use std::path::Path;

/// The skeleton of a model: the smallest `.gmdl` document the format reads.
///
/// It is the root container the original writes for every model, and nothing
/// else: the blocks a model needs are added to it, one `<module>` element
/// each, with the links between them inside the root's `<links>` section.
///
/// The catalogue has the template of every block class
/// ([`ClassSpec::template`]), so a model can be written out of the skeleton
/// and the templates without guessing an element name or an attribute.
pub const SKELETON_MODEL: &str = "<gmodel versioncode=\"1.00.2008.03.23\"><gclasses/><modules>\
<module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\" \
genprefer=\"1,1,1,1,1,0,0,0,0,0,0,0,0\" exehead=\"\" nonexehead=\"\" exefoot=\"\" nonexefoot=\"\">\
<links/><inputs/><outputs/></module></modules></gmodel>";

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
