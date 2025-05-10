use console::Term;
use simulation::Simulation;
use std::io;
use std::time::Instant;

pub fn benchmark() -> io::Result<()> {
    let term = Term::stdout();
    term.write_line("starting Benchmark!")?;

    let mut steps = 0;
    let mut elapesed = 0.;

    const ITERATIONS: usize = 10;
    const STEPS: usize = 50_000;

    for _ in 0..ITERATIONS {
        let mut simulation = Simulation::default();

        let start_time = Instant::now();

        for _ in 0..STEPS {
            simulation.step();
        }

        steps += simulation.stats().step_count;
        elapesed += start_time.elapsed().as_secs_f32();

        term.move_cursor_up(1)?;
        term.clear_line()?;
        term.write_line(&format!(
            "steps: {}% in {} seconds {} steps/second",
            steps / STEPS * ITERATIONS,
            elapesed,
            steps as f32 / elapesed
        ))?;
    }

    Ok(())
}
