use console::Term;
use simulation::Simulation;
use std::io;
use std::time::Instant;

pub fn benchmark() -> io::Result<()> {
    let term = Term::stdout();
    term.write_line("starting Benchmark!")?;

    let mut simulation = Simulation::default();

    let mut last_print_time = Instant::now();
    let start_time = Instant::now();

    loop {
        if last_print_time.elapsed().as_secs_f64() > 0.5 {
            last_print_time = Instant::now();

            term.clear_line()?;
            term.write_line(&format!(
                "steps: {} - {} steps/second",
                simulation.stats().step_count,
                simulation.stats().step_count as f32 / start_time.elapsed().as_secs_f32()
            ))?;
            term.move_cursor_up(1)?;
        }

        simulation.step();
    }
}
