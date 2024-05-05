use crate::benchmark::benchmark;
use clap::{Parser, Subcommand};
use neural_network::NeuralNetwork;
use simulation::{Simulation, NEURAL_NETWORK_INPUT_SIZE, NEURAL_NETWORK_OUTPUT_SIZE};
use std::time::Instant;

use crate::train::Trainer;

mod benchmark;
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
    Learn {
        #[arg(short, long, default_value_t = 10)]
        count: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Learn { count } => Trainer::new(count).train().unwrap(),
        Commands::Benchmark => benchmark().unwrap(),
    }
}
