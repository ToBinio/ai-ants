use console::Term;
use simulation::Simulation;
use std::io;
use std::time::Instant;

pub fn benchmark() -> io::Result<()> {
    let term = Term::stdout();
    term.write_line("starting Benchmark!")?;

    let mut simulation = Simulation::default();

    let start_time = Instant::now();

    for _ in 0..50_000 {
        simulation.step();
    }

    term.write_line(&format!(
        "steps: {} in {} seconds {} steps/second",
        simulation.stats().step_count,
        start_time.elapsed().as_secs_f32(),
        simulation.stats().step_count as f32 / start_time.elapsed().as_secs_f32()
    ))?;

    Ok(())
}
