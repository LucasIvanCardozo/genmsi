//! Reader from `.gmdl` XML to [`GmdlModel`].
//!
//! Parsing is strict: unknown elements, duplicated attributes, missing
//! required attributes and dangling links are reported instead of ignored.
//! The original editor silently "repaired" broken models, which is how its
//! structural bugs spread; this parser reports instead.

use std::path::Path;

use roxmltree::{Document, Node, ParsingOptions};

use crate::error::{FormatError, Location};
use crate::model::{
    Block, ContainerHeader, CustomClass, GmdlModel, Layout, Link, Port, PortLayout, Property,
    SectionKind, SourcePos,
};

/// Parse a model from text.
pub fn parse_str(text: &str) -> Result<GmdlModel, FormatError> {
    let options = ParsingOptions {
        allow_dtd: false,
        ..ParsingOptions::default()
    };
    let doc = Document::parse_with_options(text, options).map_err(|err| {
        let pos = err.pos();
        FormatError::Xml {
            location: Location::default().with_line(pos.row as usize),
            message: err.to_string(),
        }
    })?;

    let root = doc.root_element();
    if root.tag_name().name() != "gmodel" {
        return Err(structural(
            text,
            root,
            "root element must be <gmodel>",
        ));
    }

    let version = attr(text, root, "versioncode")?;
    // Attribute values in .gmdl files never contain a raw `>` (the editor
    // escapes or percent-encodes them), so the first `/>` in the text is a tag
    // close and reveals the file's self-closing spacing style.
    let spaced_self_closing = match text.find("/>") {
        Some(index) => text[..index].ends_with(' '),
        None => false,
    };
    let mut model = GmdlModel {
        version,
        has_classes: false,
        classes_self_closing: false,
        spaced_self_closing,
        classes: Vec::new(),
        blocks: Vec::new(),
        trailing: trailing_text(text),
    };

    let mut modules_node = None;
    for child in root.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "gclasses" => {
                model.has_classes = true;
                let slice = &text[child.range()];
                model.classes_self_closing = slice.trim_end().ends_with("/>");
                for class in child.children().filter(Node::is_element) {
                    if class.tag_name().name() != "moduleclass" {
                        return Err(structural(
                            text,
                            class,
                            format!(
                                "unexpected element <{}> inside <gclasses>",
                                class.tag_name().name()
                            ),
                        ));
                    }
                    model.classes.push(CustomClass {
                        attrs: class
                            .attributes()
                            .map(|a| (a.name().to_string(), a.value().to_string()))
                            .collect(),
                        children: class
                            .children()
                            .filter(Node::is_element)
                            .map(|n| n.tag_name().name().to_string())
                            .collect(),
                        raw: text[class.range()].to_string(),
                    });
                }
            }
            "modules" => {
                if modules_node.is_some() {
                    return Err(structural(text, child, "duplicate <modules> element"));
                }
                modules_node = Some(child);
            }
            other => {
                return Err(structural(
                    text,
                    child,
                    format!("unexpected element <{other}> inside <gmodel>"),
                ));
            }
        }
    }

    let modules_node =
        modules_node.ok_or_else(|| structural(text, root, "missing <modules> element"))?;

    for node in modules_node.children().filter(Node::is_element) {
        if node.tag_name().name() != "module" {
            return Err(structural(
                text,
                node,
                format!(
                    "unexpected element <{}> inside <modules>",
                    node.tag_name().name()
                ),
            ));
        }
        model.blocks.push(parse_block(text, node)?);
    }

    if model.blocks.is_empty() {
        return Err(structural(text, modules_node, "model has no blocks"));
    }

    Ok(model)
}

/// Read and parse a model file.
pub fn parse_file(path: impl AsRef<Path>) -> Result<GmdlModel, FormatError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|err| FormatError::Structural {
        location: Location::file(path.display().to_string()),
        message: format!("cannot read model: {err}"),
    })?;
    let file = path.display().to_string();
    parse_str(&text).map_err(|err| err.with_file(file))
}

fn parse_block(text: &str, node: Node<'_, '_>) -> Result<Block, FormatError> {
    let id = required_u32(text, node, "moduleid")?;
    let kind_text = attr(text, node, "kind")?;
    let mut chars = kind_text.chars();
    let kind = match (chars.next(), chars.next()) {
        (Some(kind @ ('A' | 'C')), None) => kind,
        _ => {
            return Err(structural(
                text,
                node,
                format!("node {id}: attribute kind must be A or C, found {kind_text:?}"),
            ));
        }
    };

    let layout = optional_layout(text, node, id)?;
    let header = if kind == 'C' {
        Some(ContainerHeader {
            genprefer: attr_or_default(node, "genprefer"),
            exehead: attr_or_default(node, "exehead"),
            nonexehead: attr_or_default(node, "nonexehead"),
            exefoot: attr_or_default(node, "exefoot"),
            nonexefoot: attr_or_default(node, "nonexefoot"),
        })
    } else {
        None
    };

    let mut block = Block {
        id,
        class: attr_or_default(node, "moduleclass"),
        kind,
        owner: required_u32(text, node, "owner")?,
        name: attr_or_default(node, "name"),
        caption: attr_or_default(node, "caption"),
        layout,
        header,
        inputs: Vec::new(),
        outputs: Vec::new(),
        links: Vec::new(),
        props: Vec::new(),
        sections: Vec::new(),
        source: Some(position_of(text, node.range().start)),
    };

    for section in node.children().filter(Node::is_element) {
        match section.tag_name().name() {
            "inputs" => {
                push_section(text, section, &mut block.sections, SectionKind::Inputs)?;
                block.inputs = parse_ports(text, section, id, true)?;
            }
            "outputs" => {
                push_section(text, section, &mut block.sections, SectionKind::Outputs)?;
                block.outputs = parse_ports(text, section, id, false)?;
            }
            "links" => {
                push_section(text, section, &mut block.sections, SectionKind::Links)?;
                block.links = parse_links(text, section, id)?;
            }
            "props" => {
                push_section(text, section, &mut block.sections, SectionKind::Props)?;
                block.props = parse_props(text, section, id)?;
            }
            other => {
                return Err(structural(
                    text,
                    section,
                    format!("node {id}: unknown element <{other}> inside <module>"),
                ));
            }
        }
    }

    validate_ports(text, node, id, &block)?;
    Ok(block)
}

fn push_section(
    text: &str,
    node: Node<'_, '_>,
    sections: &mut Vec<SectionKind>,
    kind: SectionKind,
) -> Result<(), FormatError> {
    if sections.contains(&kind) {
        return Err(structural(
            text,
            node,
            format!("duplicate <{}> section", section_name(kind)),
        ));
    }
    sections.push(kind);
    Ok(())
}

fn section_name(kind: SectionKind) -> &'static str {
    match kind {
        SectionKind::Inputs => "inputs",
        SectionKind::Links => "links",
        SectionKind::Outputs => "outputs",
        SectionKind::Props => "props",
    }
}

fn parse_ports(
    text: &str,
    section: Node<'_, '_>,
    block_id: u32,
    is_input: bool,
) -> Result<Vec<Port>, FormatError> {
    let expected = if is_input { "input" } else { "output" };
    let mut ports = Vec::new();
    for node in section.children().filter(Node::is_element) {
        if node.tag_name().name() != expected {
            return Err(structural(
                text,
                node,
                format!(
                    "node {block_id}: unexpected element <{}> inside <{}>",
                    node.tag_name().name(),
                    section.tag_name().name()
                ),
            ));
        }
        ports.push(Port {
            ioid: required_u32(text, node, "ioid")?,
            name: attr_or_default(node, "name"),
            caption: attr_or_default(node, "caption"),
            mylabel: attr_or_default(node, "mylabel"),
            layout: optional_port_layout(text, node, block_id)?,
        });
    }
    Ok(ports)
}

fn parse_links(
    text: &str,
    section: Node<'_, '_>,
    block_id: u32,
) -> Result<Vec<Link>, FormatError> {
    let mut links = Vec::new();
    for node in section.children().filter(Node::is_element) {
        if node.tag_name().name() != "link" {
            return Err(structural(
                text,
                node,
                format!(
                    "node {block_id}: unexpected element <{}> inside <links>",
                    node.tag_name().name()
                ),
            ));
        }
        links.push(Link {
            from_module: required_u32(text, node, "frommoduleid")?,
            from_ioid: required_u32(text, node, "fromioid")?,
            to_module: required_u32(text, node, "tomoduleid")?,
            to_ioid: required_u32(text, node, "toioid")?,
            breakpoints: node.attribute("breakpoints").map(str::to_string),
        });
    }
    Ok(links)
}

fn parse_props(
    text: &str,
    section: Node<'_, '_>,
    block_id: u32,
) -> Result<Vec<Property>, FormatError> {
    let mut props = Vec::new();
    for node in section.children().filter(Node::is_element) {
        if node.tag_name().name() != "property" {
            return Err(structural(
                text,
                node,
                format!(
                    "node {block_id}: unexpected element <{}> inside <props>",
                    node.tag_name().name()
                ),
            ));
        }
        props.push(Property {
            name: attr(text, node, "name")?,
            hint: attr_or_default(node, "hint"),
            raw_value: attr_or_default(node, "value"),
            ty: attr_or_default(node, "type"),
            subtype: attr_or_default(node, "subType"),
            visible: attr_or_default(node, "visible") != "0",
            unique: attr_or_default(node, "unique") != "0",
        });
    }
    Ok(props)
}

fn validate_ports(text: &str, node: Node<'_, '_>, id: u32, block: &Block) -> Result<(), FormatError> {
    let mut seen = std::collections::HashSet::new();
    for port in block.inputs.iter().chain(&block.outputs) {
        if !seen.insert(port.ioid) {
            return Err(structural(
                text,
                node,
                format!("node {id}: duplicate port id {}", port.ioid),
            ));
        }
    }
    if block.is_container() && !block.links.is_empty() {
        // Container links are validated against the model in `validate.rs`.
    } else if !block.is_container() && !block.links.is_empty() {
        return Err(structural(
            text,
            node,
            format!("node {id}: activity block must not own <links>"),
        ));
    }
    Ok(())
}

fn optional_layout(
    text: &str,
    node: Node<'_, '_>,
    id: u32,
) -> Result<Option<Layout>, FormatError> {
    let width = node.attribute("width");
    let height = node.attribute("height");
    let x = node.attribute("x");
    let y = node.attribute("y");
    match (width, height, x, y) {
        (None, None, None, None) => Ok(None),
        (Some(w), Some(h), Some(x), Some(y)) => Ok(Some(Layout {
            width: parse_i32(text, node, id, "width", w)?,
            height: parse_i32(text, node, id, "height", h)?,
            x: parse_i32(text, node, id, "x", x)?,
            y: parse_i32(text, node, id, "y", y)?,
        })),
        _ => Err(structural(
            text,
            node,
            format!("node {id}: layout needs width, height, x and y together"),
        )),
    }
}

fn optional_port_layout(
    text: &str,
    node: Node<'_, '_>,
    block_id: u32,
) -> Result<Option<PortLayout>, FormatError> {
    let width = node.attribute("width");
    let height = node.attribute("height");
    let x = node.attribute("x");
    let y = node.attribute("y");
    match (width, height, x, y) {
        (None, None, None, None) => Ok(None),
        (Some(w), Some(h), Some(x), Some(y)) => Ok(Some(PortLayout {
            width: parse_i32(text, node, block_id, "width", w)?,
            height: parse_i32(text, node, block_id, "height", h)?,
            x: parse_i32(text, node, block_id, "x", x)?,
            y: parse_i32(text, node, block_id, "y", y)?,
        })),
        _ => Err(structural(
            text,
            node,
            format!("node {block_id}: port layout needs width, height, x and y together"),
        )),
    }
}

fn parse_i32(
    text: &str,
    node: Node<'_, '_>,
    id: u32,
    name: &str,
    raw: &str,
) -> Result<i32, FormatError> {
    raw.trim().parse::<i32>().map_err(|_| {
        structural(
            text,
            node,
            format!("node {id}: attribute {name} must be an integer, found {raw:?}"),
        )
    })
}

fn attr(text: &str, node: Node<'_, '_>, name: &str) -> Result<String, FormatError> {
    node.attribute(name)
        .map(str::to_string)
        .ok_or_else(|| structural(text, node, format!("missing required attribute {name}")))
}

fn attr_or_default(node: Node<'_, '_>, name: &str) -> String {
    node.attribute(name).unwrap_or_default().to_string()
}

fn required_u32(text: &str, node: Node<'_, '_>, name: &str) -> Result<u32, FormatError> {
    let raw = attr(text, node, name)?;
    raw.trim().parse::<u32>().map_err(|_| {
        structural(
            text,
            node,
            format!("attribute {name} must be a non-negative integer, found {raw:?}"),
        )
    })
}

fn structural(text: &str, node: Node<'_, '_>, message: impl Into<String>) -> FormatError {
    FormatError::Structural {
        location: Location::default().with_line(line_of(text, node.range().start)),
        message: message.into(),
    }
}

/// 1-based line of a byte offset.
pub(crate) fn line_of(text: &str, offset: usize) -> usize {
    position_of(text, offset).line
}

/// 1-based line and column of a byte offset.
///
/// The column counts characters, not bytes, so a model written with accented
/// characters still points at the right place.
fn position_of(text: &str, offset: usize) -> SourcePos {
    let end = offset.min(text.len());
    let before = text.get(..end).unwrap_or("");
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map(|last| last.chars().count() + 1)
        .unwrap_or(1);
    SourcePos::new(line, column)
}

/// Exact text that followed the root element.
fn trailing_text(text: &str) -> String {
    match text.rfind("</gmodel>") {
        Some(index) => text[index + "</gmodel>".len()..].to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"<gmodel versioncode="1.00.2008.03.23"><gclasses/><modules><module moduleid="0" moduleclass="main" owner="0" caption="Modelo" name="Modelo1" kind="C" genprefer="1,1,1,1,1,0,0,0,0,0,0,0,0" exehead="" nonexehead="" exefoot="" nonexefoot=""><links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/></links><inputs/><outputs/></module><module moduleid="1" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A" width="120" height="63" x="330" y="39"><outputs><output ioid="1" name="Salida1" caption="" mylabel="A_Queue"/></outputs><props><property name="Media" hint="media" value="60" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module><module moduleid="2" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A" width="84" height="64" x="315" y="377"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel="A_Queue"/></inputs><props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module></modules></gmodel>"#;

    #[test]
    fn parses_minimal_model() {
        let model = parse_str(MINIMAL).expect("model parses");
        assert_eq!(model.version, "1.00.2008.03.23");
        assert_eq!(model.blocks.len(), 3);
        assert!(model.has_classes);
        assert_eq!(model.root().map(|b| b.id), Some(0));

        let generate = model.block(1).unwrap();
        assert_eq!(generate.class, "Generate");
        assert_eq!(generate.kind, 'A');
        assert_eq!(generate.layout.unwrap().x, 330);
        assert!(!generate.is_container());
        assert_eq!(generate.prop_value("Media").as_deref(), Some("60"));
        assert!(generate.prop("Media").unwrap().is_used());
        assert_eq!(generate.prop_int("Media"), Some(60));

        let terminate = model.block(2).unwrap();
        assert_eq!(terminate.label(), Some("A_Queue"));
        assert_eq!(terminate.prop_int("Unidades"), Some(1));

        let root = model.root().unwrap();
        assert_eq!(root.links.len(), 1);
        assert_eq!(root.links[0].to_module, 2);
        assert_eq!(root.sections, vec![SectionKind::Links, SectionKind::Inputs, SectionKind::Outputs]);
    }

    #[test]
    fn rejects_missing_required_attribute() {
        let broken = MINIMAL.replacen("moduleid=\"2\"", "moduleid=\"x\"", 1);
        let err = parse_str(&broken).expect_err("must fail");
        let text = err.to_string();
        assert!(text.contains("moduleid"), "{text}");
        assert_eq!(err.location().line, Some(1));
    }

    #[test]
    fn rejects_unknown_element() {
        // The injected element must stay well-nested, otherwise the XML parser
        // rejects the document before the unknown-element check can run.
        let broken = MINIMAL
            .replacen("<props>", "<weird><props>", 1)
            .replacen("</props>", "</weird></props>", 1);
        let err = parse_str(&broken).expect_err("must fail");
        assert!(err.to_string().contains("weird"), "{err}");
    }

    #[test]
    fn rejects_invalid_xml_with_line() {
        let err = parse_str("<gmodel>\n<modules>\n</gmodel>").expect_err("must fail");
        match err {
            FormatError::Xml { location, message } => {
                // roxmltree reports the 1-based position of the mismatched
                // closing tag, which is `</gmodel>` on line 3.
                assert_eq!(location.line, Some(3));
                assert!(!message.is_empty());
            }
            other => panic!("expected XML error, got {other:?}"),
        }
    }

    #[test]
    fn keeps_trailing_crlf() {
        let with_crlf = format!("{MINIMAL}\r\n");
        assert_eq!(parse_str(&with_crlf).unwrap().trailing, "\r\n");
        assert_eq!(parse_str(MINIMAL).unwrap().trailing, "");
    }

    #[test]
    fn every_block_records_its_source_position() {
        let model = parse_str(MINIMAL).expect("model parses");
        let generate = model.block(1).expect("generate block");
        let offset = MINIMAL.find("<module moduleid=\"1\"").expect("tag in text");
        let expected_column = MINIMAL[..offset].chars().count() + 1;
        assert_eq!(
            generate.source,
            Some(SourcePos::new(1, expected_column)),
            "the position points at the opening tag"
        );
        // Every block of the model carries a position.
        assert!(model.blocks.iter().all(|block| block.source.is_some()));
    }

    #[test]
    fn the_position_follows_the_line_of_the_block() {
        let text = "<gmodel versioncode=\"1\">\n  <gclasses/>\n  <modules>\n    <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\n  </modules>\n</gmodel>";
        let model = parse_str(text).expect("model parses");
        let root = model.block(0).expect("root block");
        assert_eq!(root.source, Some(SourcePos::new(4, 5)));
    }
}
