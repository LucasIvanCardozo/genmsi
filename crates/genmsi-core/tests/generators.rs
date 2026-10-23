//! What a reader reported: three copies of a generator, all feeding the same
//! advance, and the transactions of the copies arriving at the same time.
//!
//! The engine gives every block its own random stream, so two generators with
//! different ids cannot draw the same numbers. What made the copies coincide was
//! the generator itself: a `Generate` without a spread has a constant
//! interarrival, so every copy of it produces the very same arrivals, and the
//! seed has nothing to say. The original gives a new generator a mean of 60 and
//! a spread of 30; ours gave it nothing, and a generator with no spread is a
//! clock, not a random source.

mod support;

use genmsi_core::{Program, SimConfig, Simulation, program::compile};
use genmsi_format::{EditCommand, EditSession, load_str};

/// The model a reader would build: a generator, an advance and a terminate, with
/// a clock that ends the run.
fn model_xml(media: &str, spread: &str) -> String {
    format!(
        r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>
<link frommoduleid="1" fromioid="2" tomoduleid="2" toioid="1"/>
<link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/>
</links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A" width="10" height="10" x="0" y="0"><inputs/><outputs><output ioid="2" name="Salida1" caption="" mylabel="T1"/></outputs><props><property name="Media" hint="" value="{media}" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="{spread}" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Advance" owner="0" caption="Advance" name="Advance" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="T1"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="T2"/></outputs><props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="3" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="T2"/></inputs><outputs/><props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="4" moduleclass="Clock" owner="0" caption="Clock" name="Clock" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#
    )
}

/// The model with the generator copied three times, every copy feeding the same
/// advance.
fn four_generators(media: &str, spread: &str) -> Program {
    let mut session = EditSession::new(load_str(&model_xml(media, spread)).expect("the model parses"));
    // Copy the generator and paste it three times, feeding every copy to the
    // same advance — exactly what a reader does with copy and paste.
    let mut copies = Vec::new();
    for _ in 0..3 {
        let pasted = session
            .apply(EditCommand::Paste {
                ids: vec![1],
                dx: 20,
                dy: 20,
            })
            .expect("the paste works");
        assert_eq!(pasted.len(), 1, "one copy per paste");
        copies.extend(pasted);
    }
    for copy in &copies {
        session
            .apply(EditCommand::Connect {
                from: *copy,
                from_ioid: 2,
                to: 2,
                to_ioid: 1,
            })
            .expect("the copy feeds the advance");
    }
    compile(session.model(), "copies").expect("the model compiles")
}

/// The times at which each generator produced a transaction.
fn arrivals(program: &Program, seed: u64) -> Vec<Vec<f64>> {
    let config = SimConfig {
        seed,
        stop_time: Some(3600.0),
        trace: true,
        max_events: 5_000_000,
        ..SimConfig::default()
    };
    let result = Simulation::new(program, config).run().expect("the run works");
    let mut times: Vec<Vec<f64>> = vec![Vec::new(); program.blocks.len()];
    for record in &result.trace {
        if record.event == "generate" {
            times[record.block].push(record.time);
        }
    }
    times
}

/// The arrival times of the generator and of its three copies.
fn arrival_sets(program: &Program, seed: u64) -> Vec<Vec<f64>> {
    let times = arrivals(program, seed);
    let generators: Vec<usize> = program
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.class == "Generate" && block.module_id != 4)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(generators.len(), 4, "the generator and its three copies");
    generators.iter().map(|index| times[*index].clone()).collect()
}

#[test]
fn the_copies_of_a_generator_draw_their_own_numbers() {
    // The original's generator has a mean of 60 and a spread of 30.
    let program = four_generators("60", "30");
    let each = arrival_sets(&program, 1);
    for (index, times) in each.iter().enumerate() {
        assert!(times.len() > 10, "generator {index} produced few transactions");
    }

    // From the second transaction on, every generator draws its own numbers.
    for (index, times) in each.iter().enumerate() {
        for other in each.iter().skip(index + 1) {
            assert_ne!(
                times[1],
                other[1],
                "two generators must not draw the same second arrival"
            );
        }
    }
}

#[test]
fn a_generator_without_a_spread_is_a_clock() {
    // With a constant interarrival there is nothing to draw: the seed has no
    // say, and every copy of the generator produces the very same arrivals.
    let program = four_generators("60", "0");
    let each = arrival_sets(&program, 1);
    for times in &each {
        assert_eq!(times[0], 60.0, "the first arrival is the mean");
        assert_eq!(times[1], 120.0, "a constant interarrival is a clock");
    }
    for (index, times) in each.iter().enumerate() {
        for other in each.iter().skip(index + 1) {
            assert_eq!(
                times[1],
                other[1],
                "without a spread every copy arrives at the same time"
            );
        }
    }
}
