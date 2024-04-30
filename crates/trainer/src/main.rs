use chrono::Local;
use itertools::Itertools;
use neural_network::NeuralNetwork;
use rayon::prelude::*;
use simulation::Simulation;
use std::fs;
use std::time::Instant;

const STEPS_PER_SIMULATION: usize = 5000;

fn main() {
    println!("Starting training!");

    // let mut simulation = Simulation::new(NeuralNetwork::new(vec![5, 5, 5, 1]));
    // benchmark(&mut simulation);

    let mut simulations = vec![];

    for _ in 0..10 {
        simulations.push(Simulation::new(NeuralNetwork::new(vec![5, 5, 5, 1])))
    }

    loop {
        run(&mut simulations);
        let (best, score) = get_best(&simulations);
        let best_network = best.neural_network().clone();

        fs::write(
            format!(
                "./training/{}-{}.json",
                Local::now().format("%Y-%m-%d|%H:%M:%S"),
                score
            ),
            serde_json::to_string(&best_network).unwrap(),
        )
        .unwrap();

        println!("best score: {}", score);

        simulations.clear();

        simulations.push(Simulation::new(best_network));
        for _ in 1..10 {
            simulations.push(Simulation::new(NeuralNetwork::new(vec![5, 5, 5, 1])))
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

fn run(simulations: &mut Vec<Simulation>) {
    simulations.par_iter_mut().for_each(|simulation| {
        for _ in 0..STEPS_PER_SIMULATION {
            simulation.step();
        }
    });
}

fn get_best(simulations: &Vec<Simulation>) -> (&Simulation, usize) {
    simulations
        .iter()
        .map(|simulation| (simulation, eval(simulation)))
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .next()
        .unwrap()
}

fn eval(simulation: &Simulation) -> usize {
    simulation.stats().dropped_of_food * 5 + simulation.stats().picked_up_food
}
