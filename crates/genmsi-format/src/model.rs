//! Semantic model of a `.gmdl` file.
//!
//! The file format is a flat list of `module` elements: activity blocks, and
//! `kind="C"` container blocks that group other modules and own their links.
//! Canvas geometry (position and size of blocks and ports) is kept in
//! [`Layout`] values attached to blocks and ports; it never influences
//! semantics and is preserved on rewrite.
//!
//! Attribute values inside a model may be percent-encoded (the original editor
//! writes hints, and in some models entity names, as URL-encoded CP1252 text).
//! The raw string is kept verbatim for byte-identical rewriting and
//! [`Property::value`] returns the decoded value used for semantics.

use std::collections::HashMap;

/// Order in which the child sections of a `module` element appeared in the
/// source. The data itself lives in typed fields; only the order is metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// `<inputs>`
    Inputs,
    /// `<links>`
    Links,
    /// `<outputs>`
    Outputs,
    /// `<props>`
    Props,
}

/// Geometry of a block on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Block width in canvas units.
    pub width: i32,
    /// Block height in canvas units.
    pub height: i32,
    /// Horizontal position.
    pub x: i32,
    /// Vertical position.
    pub y: i32,
}

/// Geometry of a port glyph on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortLayout {
    /// Glyph width.
    pub width: i32,
    /// Glyph height.
    pub height: i32,
    /// Horizontal position.
    pub x: i32,
    /// Vertical position.
    pub y: i32,
}

/// One connection point of a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    /// Port id, unique inside the block.
    pub ioid: u32,
    /// Port name (`Entrada1`, `Salida1`, ...).
    pub name: String,
    /// Port caption shown by the editor.
    pub caption: String,
    /// Connection label shared by the two ends of a link.
    pub mylabel: String,
    /// Optional glyph geometry.
    pub layout: Option<PortLayout>,
}

impl Port {
    /// Connection label, when the port declares one.
    pub fn label(&self) -> Option<&str> {
        let label = self.mylabel.trim();
        (!label.is_empty()).then_some(label)
    }
}

/// A link between two blocks, owned by their common container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Source module id, or the container itself for an entry port.
    pub from_module: u32,
    /// Source output port id.
    pub from_ioid: u32,
    /// Target module id, or the container itself for an exit port.
    pub to_module: u32,
    /// Target input port id.
    pub to_ioid: u32,
    /// Editor breakpoints (`x,y;x,y`), pure layout.
    pub breakpoints: Option<String>,
}

/// One named property of a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    /// Property name as declared by the block schema (`Media`, `Queue`, ...).
    pub name: String,
    /// Tooltip shown by the editor (percent-encoded).
    pub hint: String,
    /// Raw value exactly as stored in the file.
    pub raw_value: String,
    /// Declared type (`TSNA`, `EntityUse`, `Listed`, `PosInteger`, ...).
    pub ty: String,
    /// Declared subtype (`SNA`, `Facility`, `Queue`, ...).
    pub subtype: String,
    /// Whether the editor shows the property.
    pub visible: bool,
    /// Whether the property defines a new entity.
    pub unique: bool,
}

impl Property {
    /// Decoded semantic value.
    pub fn value(&self) -> String {
        percent_decode(&self.raw_value)
    }

    /// Whether the property holds a value.
    pub fn is_used(&self) -> bool {
        !self.raw_value.trim().is_empty()
    }

    /// Integer value, when the property holds an integer.
    pub fn int(&self) -> Option<i64> {
        self.raw_value.trim().parse::<i64>().ok()
    }

    /// Replace the value, keeping the file's encoding convention.
    pub fn set_value(&mut self, value: &str) {
        self.raw_value = if self.raw_value.contains('%') {
            percent_encode(value)
        } else {
            value.to_string()
        };
    }
}

/// Header fields that only container blocks use (GPSS code fragments and
/// per-container generation preferences).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContainerHeader {
    /// `genprefer` attribute.
    pub genprefer: String,
    /// `exehead` attribute.
    pub exehead: String,
    /// `nonexehead` attribute.
    pub nonexehead: String,
    /// `exefoot` attribute.
    pub exefoot: String,
    /// `nonexefoot` attribute.
    pub nonexefoot: String,
}

/// Where a block sits in the source file.
///
/// The position is read from the file and never written back: the writer does
/// not look at it, so a model that is only read and rewritten stays
/// byte-identical. It exists so a client (the MCP server, an editor) can point
/// at the exact place of a block that fails to compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePos {
    /// 1-based line of the block's opening tag.
    pub line: usize,
    /// 1-based column of the block's opening tag.
    pub column: usize,
}

impl SourcePos {
    /// A position on the given line and column.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// One block (activity) or container of the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// `moduleid`, unique in the model.
    pub id: u32,
    /// `moduleclass`; empty for plain containers.
    pub class: String,
    /// `kind`: `A` for an activity block, `C` for a container.
    pub kind: char,
    /// Owning container id (`0` for the model root's children).
    pub owner: u32,
    /// Block name as shown by the editor.
    pub name: String,
    /// Block caption.
    pub caption: String,
    /// Canvas geometry, absent for the model root.
    pub layout: Option<Layout>,
    /// Container header fields, absent for activity blocks.
    pub header: Option<ContainerHeader>,
    /// Input ports in file order.
    pub inputs: Vec<Port>,
    /// Output ports in file order.
    pub outputs: Vec<Port>,
    /// Links owned by this container.
    pub links: Vec<Link>,
    /// Properties in schema order.
    pub props: Vec<Property>,
    /// Section order recorded from the source.
    pub sections: Vec<SectionKind>,
    /// Position of the block in the source file, when it was read from one.
    pub source: Option<SourcePos>,
}

impl Block {
    /// Whether this module groups other modules instead of simulating.
    pub fn is_container(&self) -> bool {
        self.kind == 'C'
    }

    /// Whether this module is the model root.
    pub fn is_root(&self) -> bool {
        self.class == "main"
    }

    /// Property by name.
    pub fn prop(&self, name: &str) -> Option<&Property> {
        self.props.iter().find(|p| p.name == name)
    }

    /// Mutable property by name.
    pub fn prop_mut(&mut self, name: &str) -> Option<&mut Property> {
        self.props.iter_mut().find(|p| p.name == name)
    }

    /// Decoded value of a property, or `None` when missing or blank.
    pub fn prop_value(&self, name: &str) -> Option<String> {
        self.prop(name)
            .filter(|p| p.is_used())
            .map(Property::value)
    }

    /// Integer property value.
    pub fn prop_int(&self, name: &str) -> Option<i64> {
        self.prop(name).and_then(Property::int)
    }

    /// Input port by id.
    pub fn input(&self, ioid: u32) -> Option<&Port> {
        self.inputs.iter().find(|p| p.ioid == ioid)
    }

    /// Output port by id.
    pub fn output(&self, ioid: u32) -> Option<&Port> {
        self.outputs.iter().find(|p| p.ioid == ioid)
    }

    /// One port by id, of either kind.
    ///
    /// The entry ports and the exit ports of a block share one numbering, so
    /// the kind is needed to tell them apart.
    pub fn port_of(&self, ioid: u32, input: bool) -> Option<&Port> {
        if input {
            self.input(ioid)
        } else {
            self.output(ioid)
        }
    }

    /// Label of the block: the connection label of its first labelled input
    /// port. Blocks without inputs (generators, definitions) have none.
    pub fn label(&self) -> Option<&str> {
        self.inputs.iter().find_map(Port::label)
    }
}

/// A custom module class declared inside `<gclasses>` (macro code).
///
/// The body is macro code the engine does not interpret yet, so the exact
/// source slice is preserved and re-emitted verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CustomClass {
    /// Attributes of the `moduleclass` element, in file order.
    pub attrs: Vec<(String, String)>,
    /// Child element names, for introspection.
    pub children: Vec<String>,
    /// Exact source text of the element, from `<moduleclass` through its `>`.
    pub raw: String,
}

impl CustomClass {
    /// `className` attribute.
    pub fn class_name(&self) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(name, _)| name == "className")
            .map(|(_, value)| value.as_str())
    }
}

/// A parsed `.gmdl` model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmdlModel {
    /// `versioncode` of the root element.
    pub version: String,
    /// Whether `<gclasses>` was present.
    pub has_classes: bool,
    /// Whether `<gclasses>` was written as a self-closing empty element.
    pub classes_self_closing: bool,
    /// Whether the source writes self-closing tags with a space before `/>`
    /// (`<link ... />`). The writer reproduces the style for byte-identical
    /// rewrites; the original editor uses one style per file.
    pub spaced_self_closing: bool,
    /// Custom module classes.
    pub classes: Vec<CustomClass>,
    /// Blocks in file order.
    pub blocks: Vec<Block>,
    /// Exact text after `</gmodel>` (`"\r\n"`, `""`, ...).
    pub trailing: String,
}

impl GmdlModel {
    /// Block index by id.
    pub fn index(&self) -> HashMap<u32, usize> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect()
    }

    /// Block by id.
    pub fn block(&self, id: u32) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Mutable block by id.
    pub fn block_mut(&mut self, id: u32) -> Option<&mut Block> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }

    /// The model root (`moduleclass="main"`).
    pub fn root(&self) -> Option<&Block> {
        self.blocks.iter().find(|b| b.is_root())
    }

    /// Activity blocks (those that simulate) in file order.
    pub fn activity_blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter().filter(|b| !b.is_container())
    }

    /// Direct children of a container, in file order.
    pub fn children_of(&self, container: u32) -> impl Iterator<Item = &Block> {
        let mut blocks: Vec<&Block> = self.blocks.iter().filter(|b| b.owner == container).collect();
        blocks.sort_by_key(|b| b.id);
        blocks.into_iter()
    }

    /// Link owned by `container` that starts at `(from_module, from_ioid)`.
    pub fn outgoing_link(&self, container: u32, from_module: u32, from_ioid: u32) -> Option<&Link> {
        self.block(container)?
            .links
            .iter()
            .find(|l| l.from_module == from_module && l.from_ioid == from_ioid)
    }
}

/// Decode percent-encoded text (`%20`, `%3A`, `%F3`, ...).
///
/// The original editor encodes Windows-1252 bytes, so a decode that is not
/// valid UTF-8 falls back to one character per byte (Latin-1). Valid UTF-8 is
/// returned unchanged apart from the escapes.
pub fn percent_decode(input: &str) -> String {
    if !input.contains('%') {
        return input.to_string();
    }
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let high = (bytes[i + 1] as char).to_digit(16);
            let low = (bytes[i + 2] as char).to_digit(16);
            if let (Some(high), Some(low)) = (high, low) {
                out.push((high * 16 + low) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    match String::from_utf8(out) {
        Ok(text) => text,
        Err(err) => err.into_bytes().iter().map(|b| *b as char).collect(),
    }
}

/// Encode a value the way the original editor stores entity names.
///
/// Only `[A-Za-z0-9_.~-]` stay literal; every other byte becomes `%XX`.
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        let literal = byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'~' | b'-');
        if literal {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_handles_ascii_escapes() {
        assert_eq!(percent_decode("tiempo%20demora"), "tiempo demora");
        assert_eq!(percent_decode("Q%3Aqpostvacunacion"), "Q:qpostvacunacion");
        assert_eq!(percent_decode("25%25"), "25%");
        assert_eq!(percent_decode("plain"), "plain");
    }

    #[test]
    fn percent_decode_falls_back_to_latin1_for_cp1252_bytes() {
        assert_eq!(percent_decode("interrupci%F3n"), "interrupción");
    }

    #[test]
    fn percent_encode_round_trips_aspects_of_entity_names() {
        let encoded = percent_encode("P:Peso Pescado");
        assert_eq!(encoded, "P%3APeso%20Pescado");
        assert_eq!(percent_decode(&encoded), "P:Peso Pescado");
    }

    #[test]
    fn property_set_value_keeps_encoding_convention() {
        let mut encoded = Property {
            name: "Queue".into(),
            hint: String::new(),
            raw_value: "qtiempototal%20x".into(),
            ty: "EntityUse".into(),
            subtype: "Queue".into(),
            visible: true,
            unique: false,
        };
        encoded.set_value("cola nueva");
        assert_eq!(encoded.raw_value, "cola%20nueva");
        encoded.set_value("cola");
        assert_eq!(encoded.raw_value, "cola");

        let mut plain = Property {
            raw_value: "centro".into(),
            ..encoded.clone()
        };
        plain.set_value("cola nueva");
        assert_eq!(plain.raw_value, "cola nueva");
    }
}
