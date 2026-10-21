//! Parse failures carry the full position: line, column and the `moduleid`
//! of the `<module>` the failure belongs to.

use genmsi_format::parse_str;

/// Two ports of node 1 share `ioid="1"`; the module starts on line 2,
/// indented two spaces.
const DUPLICATE_PORT: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\n  \
     <module moduleid=\"1\" moduleclass=\"Queue\" owner=\"0\" caption=\"q\" name=\"q\" kind=\"A\">\
     <inputs><input ioid=\"1\" name=\"a\" caption=\"\" mylabel=\"L\"/>\
     <input ioid=\"1\" name=\"b\" caption=\"\" mylabel=\"L\"/></inputs>\
     <outputs/><props/></module>\n</modules></gmodel>";

#[test]
fn a_duplicate_port_id_points_at_its_module() {
    let error = parse_str(DUPLICATE_PORT).expect_err("two ports share ioid 1");
    let text = error.to_string();
    assert!(text.contains("duplicate port id 1"), "{text}");
    let location = error.location();
    assert_eq!(location.node, Some(1), "the offending <module> is node 1");
    assert_eq!(location.line, Some(2));
    assert_eq!(
        location.column,
        Some(3),
        "the column points at the opening tag, indented two spaces"
    );
}

#[test]
fn a_failure_outside_every_module_has_a_position_but_no_node() {
    // The root element itself is wrong: no `<module>` owns the failure, so
    // `node` stays absent while the position still says where to look.
    let error = parse_str("<model/>").expect_err("must fail");
    let location = error.location();
    assert_eq!(location.node, None);
    assert_eq!(location.line, Some(1));
    assert_eq!(location.column, Some(1));
}
