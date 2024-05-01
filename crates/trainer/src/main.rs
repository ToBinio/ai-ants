use clap::{Parser, Subcommand};
use neural_network::NeuralNetwork;
use simulation::{Simulation, NEURAL_NETWORK_INPUT_SIZE, NEURAL_NETWORK_OUTPUT_SIZE};
use std::time::Instant;

use crate::train::Trainer;

mod train;

const STEPS_PER_SIMULATION: usize = 5000;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Benchmark,
    Train {
        #[arg(short, long, default_value_t = 10)]
        count: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Train { count } => {
            let mut trainer = Trainer::new(count);
            trainer.train().unwrap()
        }
        Commands::Benchmark => {
            println!("Starting Benchmark!");

            let mut simulation = Simulation::new(NeuralNetwork::new(vec![
                NEURAL_NETWORK_INPUT_SIZE,
                10,
                7,
                5,
                NEURAL_NETWORK_OUTPUT_SIZE,
            ]));
            benchmark(&mut simulation);
        }
    }
}

fn benchmark(simulation: &mut Simulation) {
    let mut last_print_time = Instant::now();
    let start_time = Instant::now();

    loop {
        if last_print_time.elapsed().as_secs_f64() > 1. {
            last_print_time = Instant::now();
            println!(
                "steps: {} - {} steps/second",
                simulation.stats().step_count,
                simulation.stats().step_count as f32 / start_time.elapsed().as_secs_f32()
            );
        }

        simulation.step();
    }
}
