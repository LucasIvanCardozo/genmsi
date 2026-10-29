// Reproduce el modelo tal como lo construye la app, y simula.
use genmsi_core::program::compile;
use genmsi_core::sim::{SimConfig, Simulation};
use genmsi_format::{EditCommand, EditSession, SKELETON_MODEL, parse_str};

fn main() {
    let mut s = EditSession::new(parse_str(SKELETON_MODEL).expect("skeleton"));
    let mut add = |s: &mut EditSession, class: &str, c: u32| -> u32 {
        s.apply(EditCommand::AddBlock { class: class.into(), container: c, x: 0, y: 0 })
            .expect("adds")[0]
    };
    let generator = add(&mut s, "Generate", 0);
    let adv = add(&mut s, "Advance", 0);
    let cont = add(&mut s, "Container", 0);
    let term = add(&mut s, "Terminate", 0);
    let inner = add(&mut s, "Advance", cont);

    let mut conn = |s: &mut EditSession, f: u32, fi: u32, t: u32, ti: u32| {
        if let Err(e) = s.apply(EditCommand::Connect { from: f, from_ioid: fi, to: t, to_ioid: ti }) {
            println!("  link {f}:{fi} -> {t}:{ti}  ERR {e}");
        }
    };
    conn(&mut s, generator, 1, adv, 1);
    conn(&mut s, adv, 2, cont, 1);        // raiz -> entrada del contenedor
    conn(&mut s, cont, 1, inner, 1);      // entrada -> Advance interior
    conn(&mut s, inner, 2, cont, 2);      // Advance interior -> salida
    conn(&mut s, cont, 2, term, 1);       // salida -> Terminate

    // Un Terminate necesita su operando, como en la app.
    s.apply(EditCommand::SetProperty { id: term, property: "Unidades".into(), value: "1".into() })
        .expect("units");
    add(&mut s, "Clock", 0);

    let program = compile(s.model(), "repro").expect("compiles");
    println!("=== PROGRAMA (bloques) ===");
    for (i, b) in program.blocks.iter().enumerate() {
        println!("  {i}: {} module={} label={:?} next={:?}", b.class, b.module_id, b.label, b.next);
    }
    let cfg = SimConfig { trace: true, ..SimConfig::default() };
    let result = Simulation::new(&program, cfg).run().expect("runs");
    println!("\n=== TRAZA (los movimientos de la instancia) ===");
    for e in result.trace.iter() {
        println!("  t={} xact={} {} block={} label={:?} detail={}",
            e.time, e.xact, e.event, e.block, e.label, e.detail);
    }
}
