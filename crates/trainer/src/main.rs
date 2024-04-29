use simulation::Simulation;
use std::time::Instant;

fn main() {
    println!("Starting training!");

    let mut simulation = Simulation::default();

    let mut last_print_time = Instant::now();
    let start_time = Instant::now();

    let mut steps = 0;

    loop {
        if last_print_time.elapsed().as_secs_f64() > 1. {
            last_print_time = Instant::now();
            println!(
                "steps: {} - {} steps/second",
                steps,
                steps as f32 / start_time.elapsed().as_secs_f32()
            );
        }

        simulation.step();
        steps += 1;
    }
}
