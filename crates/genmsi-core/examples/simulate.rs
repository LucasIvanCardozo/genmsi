//! Run one model and print its GPSS report.
//!
//! `cargo run -p genmsi-core --example simulate -- fixtures/models/TP2__problema1.gmdl [seed]`

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: simulate <model.gmdl> [seed]");
        std::process::exit(2);
    };
    let seed = args.next().and_then(|value| value.parse::<u64>().ok()).unwrap_or(1);
    let model = genmsi_format::load(&path).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });
    let (program, result) = genmsi_core::simulate(
        &model,
        genmsi_core::SimConfig {
            seed,
            stop_time: Some(3600.0),
            ..genmsi_core::SimConfig::default()
        },
    )
    .unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });
    print!("{}", genmsi_core::render(&result, &program));
}
