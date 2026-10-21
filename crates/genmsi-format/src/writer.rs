//! Writer from [`GmdlModel`] back to `.gmdl` XML.
//!
//! The serializer emits the original's compact style (no whitespace between
//! elements, self-closing empty tags, the same attribute order) so rewriting a
//! model that was only read produces a byte-identical file. Canvas geometry
//! lives in [`Layout`](crate::model::Layout) values but is written as the
//! original attributes, which keeps positions intact across a rewrite.

use std::fmt::Write as _;
use std::path::Path;

use crate::error::FormatError;
use crate::model::{
    Block, ContainerHeader, GmdlModel, Layout, Link, Port, PortLayout, Property, SectionKind,
};

/// Serialize a model to XML text.
pub fn to_string(model: &GmdlModel) -> String {
    let mut out = String::with_capacity(64 * 1024);
    // The original editor writes self-closing tags either as `/>` or ` />`,
    // consistently within one file.
    let sp = if model.spaced_self_closing { " " } else { "" };
    let _ = write!(out, "<gmodel versioncode=\"{}\">", escape(&model.version));
    if model.has_classes {
        if model.classes.is_empty() {
            out.push_str(if model.classes_self_closing {
                if model.spaced_self_closing {
                    "<gclasses />"
                } else {
                    "<gclasses/>"
                }
            } else {
                "<gclasses></gclasses>"
            });
        } else {
            out.push_str("<gclasses>");
            for class in &model.classes {
                out.push_str(&class.raw);
            }
            out.push_str("</gclasses>");
        }
    }
    out.push_str("<modules>");
    for block in &model.blocks {
        write_block(&mut out, block, sp);
    }
    out.push_str("</modules></gmodel>");
    out.push_str(&model.trailing);
    out
}

/// Write a model to a file.
pub fn write_file(model: &GmdlModel, path: impl AsRef<Path>) -> Result<(), FormatError> {
    let path = path.as_ref();
    let text = to_string(model);
    std::fs::write(path, text).map_err(|err| FormatError::Structural {
        location: crate::error::Location::file(path.display().to_string()),
        message: format!("cannot write model: {err}"),
    })
}

fn write_block(out: &mut String, block: &Block, sp: &str) {
    let _ = write!(
        out,
        "<module moduleid=\"{}\" moduleclass=\"{}\" owner=\"{}\" caption=\"{}\" name=\"{}\" kind=\"{}\"",
        block.id,
        escape(&block.class),
        block.owner,
        escape(&block.caption),
        escape(&block.name),
        block.kind
    );
    if let Some(header) = &block.header {
        write_header(out, header);
    }
    if let Some(layout) = &block.layout {
        write_layout(out, layout);
    }
    if block.sections.is_empty() {
        // The original editor writes section-less blocks (e.g. notes) as
        // self-closing tags; keep that style for byte-identical rewrites.
        out.push_str(sp);
        out.push_str("/>");
        return;
    }
    out.push('>');

    for section in &block.sections {
        match section {
            SectionKind::Inputs => write_ports(out, "inputs", "input", &block.inputs, sp),
            SectionKind::Outputs => write_ports(out, "outputs", "output", &block.outputs, sp),
            SectionKind::Links => write_links(out, block, sp),
            SectionKind::Props => write_props(out, &block.props, sp),
        }
    }
    out.push_str("</module>");
}

fn write_header(out: &mut String, header: &ContainerHeader) {
    let _ = write!(
        out,
        " genprefer=\"{}\" exehead=\"{}\" nonexehead=\"{}\" exefoot=\"{}\" nonexefoot=\"{}\"",
        escape(&header.genprefer),
        escape(&header.exehead),
        escape(&header.nonexehead),
        escape(&header.exefoot),
        escape(&header.nonexefoot)
    );
}

fn write_layout(out: &mut String, layout: &Layout) {
    let _ = write!(
        out,
        " width=\"{}\" height=\"{}\" x=\"{}\" y=\"{}\"",
        layout.width, layout.height, layout.x, layout.y
    );
}

fn write_ports(out: &mut String, section: &str, tag: &str, ports: &[Port], sp: &str) {
    if ports.is_empty() {
        let _ = write!(out, "<{section}{sp}/>");
        return;
    }
    let _ = write!(out, "<{section}>");
    for port in ports {
        let _ = write!(
            out,
            "<{tag} ioid=\"{}\" name=\"{}\" caption=\"{}\" mylabel=\"{}\"",
            port.ioid,
            escape(&port.name),
            escape(&port.caption),
            escape(&port.mylabel)
        );
        if let Some(PortLayout {
            width,
            height,
            x,
            y,
        }) = &port.layout
        {
            let _ = write!(
                out,
                " width=\"{width}\" height=\"{height}\" x=\"{x}\" y=\"{y}\""
            );
        }
        out.push_str(sp);
        out.push_str("/>");
    }
    let _ = write!(out, "</{section}>");
}

fn write_links(out: &mut String, block: &Block, sp: &str) {
    if block.links.is_empty() {
        let _ = write!(out, "<links{sp}/>");
        return;
    }
    out.push_str("<links>");
    for link in &block.links {
        write_link(out, link, sp);
    }
    out.push_str("</links>");
}

fn write_link(out: &mut String, link: &Link, sp: &str) {
    let _ = write!(
        out,
        "<link frommoduleid=\"{}\" fromioid=\"{}\" tomoduleid=\"{}\" toioid=\"{}\"",
        link.from_module, link.from_ioid, link.to_module, link.to_ioid
    );
    if let Some(breakpoints) = &link.breakpoints {
        let _ = write!(out, " breakpoints=\"{}\"", escape(breakpoints));
    }
    out.push_str(sp);
    out.push_str("/>");
}

fn write_props(out: &mut String, props: &[Property], sp: &str) {
    if props.is_empty() {
        let _ = write!(out, "<props{sp}/>");
        return;
    }
    out.push_str("<props>");
    for prop in props {
        let _ = write!(
            out,
            "<property name=\"{}\" hint=\"{}\" value=\"{}\" type=\"{}\" subType=\"{}\" visible=\"{}\" unique=\"{}\"",
            escape(&prop.name),
            escape(&prop.hint),
            escape(&prop.raw_value),
            escape(&prop.ty),
            escape(&prop.subtype),
            if prop.visible { 1 } else { 0 },
            if prop.unique { 1 } else { 0 }
        );
        out.push_str(sp);
        out.push_str("/>");
    }
    out.push_str("</props>");
}

fn escape(value: &str) -> String {
    if !value.contains(['&', '<', '>', '"']) {
        return value.to_string();
    }
    let mut escaped = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;

    const SOURCE: &str = r#"<gmodel versioncode="1.00.2008.03.23"><gclasses/><modules><module moduleid="0" moduleclass="main" owner="0" caption="Modelo" name="Modelo1" kind="C" genprefer="1,1,1,1,1,0,0,0,0,0,0,0,0" exehead="" nonexehead="" exefoot="" nonexefoot=""><links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1" breakpoints="10,20;30,40"/></links><inputs/><outputs/></module><module moduleid="1" moduleclass="Advance" owner="0" caption="Advance" name="Advance" kind="A" width="80" height="32" x="236" y="104"><inputs><input ioid="1" name="Entrada1" caption="Ingresa" mylabel="A_Advance"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="A_Terminate" width="35" height="35" x="10" y="20"/></outputs><props><property name="Media" hint="Tiempo%20Medio" value="60" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module><module moduleid="2" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A" width="84" height="64" x="315" y="377"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel="A_Terminate"/></inputs><props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module></modules></gmodel>"#;

    #[test]
    fn round_trip_is_byte_identical() {
        let model = parse_str(SOURCE).expect("parses");
        assert_eq!(to_string(&model), SOURCE);
    }

    #[test]
    fn round_trip_preserves_self_closing_style() {
        let source = SOURCE.replace("<gclasses/>", "<gclasses></gclasses>");
        let model = parse_str(&source).expect("parses");
        assert_eq!(to_string(&model), source);
    }

    #[test]
    fn round_trip_preserves_spaced_self_closing_style() {
        // Some reference models (e.g. entregaComida) write every self-closing
        // tag with a space before `/>`; the rewrite must keep that spelling.
        let source = SOURCE.replace("/>", " />");
        let model = parse_str(&source).expect("parses");
        assert!(model.spaced_self_closing);
        assert_eq!(to_string(&model), source);
    }

    #[test]
    fn round_trip_appends_original_trailing_text() {
        let source = format!("{SOURCE}\r\n");
        let model = parse_str(&source).expect("parses");
        assert_eq!(to_string(&model), source);
    }

    #[test]
    fn writer_escapes_special_characters() {
        let mut model = parse_str(SOURCE).expect("parses");
        model.block_mut(1).unwrap().caption = "a & b <c>".to_string();
        let text = to_string(&model);
        assert!(text.contains("caption=\"a &amp; b &lt;c&gt;\""), "{text}");
        assert_eq!(parse_str(&text).unwrap().block(1).unwrap().caption, "a & b <c>");
    }

    #[test]
    fn rewriting_preserves_layout_after_semantic_edit() {
        let mut model = parse_str(SOURCE).expect("parses");
        model.block_mut(1).unwrap().prop_mut("Media").unwrap().set_value("90");
        let text = to_string(&model);
        let reparsed = parse_str(&text).expect("parses");
        let advance = reparsed.block(1).unwrap();
        assert_eq!(advance.prop_value("Media").as_deref(), Some("90"));
        assert_eq!(advance.layout.unwrap().x, 236);
        assert_eq!(advance.outputs[0].layout.unwrap().y, 20);
    }
}
