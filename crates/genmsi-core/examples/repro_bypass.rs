// ¿En qué forma del contenedor la instancia ATRAVIESA y sale por la salida?
use genmsi_core::program::compile;
use genmsi_core::sim::{SimConfig, Simulation};
use genmsi_format::{GmdlModel, parse_str};

fn base(container_ports: &str) -> String {
    format!(r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/></links>
<inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="A_C"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="2" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Advance" owner="0" caption="a" name="Advance" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_C"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="A_X"/></outputs>
<props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="3" moduleclass="" owner="0" caption="c" name="Contenedor" kind="C">
{container_ports}
<links/></module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="A_X"/></inputs>
<props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="5" moduleclass="Clock" owner="0" caption="c" name="Clock" kind="A">
<props><property name="Frecuencia" hint="" value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#)
}

fn probe(name: &str, xml: &str) {
    let model = parse_str(xml).expect("parses");
    let program = compile(&model, "repro").expect("compiles");
    let cfg = SimConfig { trace: true, ..SimConfig::default() };
    let result = Simulation::new(&program, cfg).run().expect("runs");
    println!("\n### {name}");
    for e in result.trace.iter() {
        if e.event == "enter" || e.event == "terminate" || e.event == "generate" {
            println!("   t={} {:9} bloque={} label={:?}", e.time, e.event, e.block, e.label);
        }
    }
    let term = program.blocks.iter().position(|b| b.class == "Terminate");
    let term_entries = term.map(|i| result.blocks[i].entries).unwrap_or(0);
    println!("   => Terminate recibió {term_entries} transacciones  (0 = la instancia NO atravesó el contenedor)");
}

fn main() {
    // Forma A: entrada y salida con el MISMO id (lo que escribía la app antes del arreglo).
    let a = base("<inputs><input ioid=\"1\" name=\"Entrada1\" caption=\"\" mylabel=\"A_C\"/></inputs><outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"A_X\"/></outputs>");
    probe("A: entrada=1 salida=1, sin link interno", &a);

    // Forma B: entrada=1 salida=2 (la forma actual de la app).
    let b = base("<inputs><input ioid=\"1\" name=\"Entrada1\" caption=\"\" mylabel=\"A_C\"/></inputs><outputs><output ioid=\"2\" name=\"Salida1\" caption=\"\" mylabel=\"A_X\"/></outputs>");
    probe("B: entrada=1 salida=2, sin link interno", &b);
}
