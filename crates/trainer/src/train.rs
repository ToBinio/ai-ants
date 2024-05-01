use std::fs::{self};

use chrono::Local;
use itertools::Itertools;
use neural_network::NeuralNetwork;
use rayon::prelude::*;
use simulation::{Simulation, NEURAL_NETWORK_INPUT_SIZE, NEURAL_NETWORK_OUTPUT_SIZE};

use crate::STEPS_PER_SIMULATION;

pub struct Trainer {
    simulations: Vec<Simulation>,
}

impl Trainer {
    pub fn new(simulation_count: usize) -> Trainer {
        let mut simulations = vec![];

        for _ in 0..simulation_count {
            simulations.push(Simulation::new(NeuralNetwork::new(vec![
                NEURAL_NETWORK_INPUT_SIZE,
                5,
                5,
                NEURAL_NETWORK_OUTPUT_SIZE,
            ])))
        }

        Trainer { simulations }
    }

    pub fn train(&mut self) {
        loop {
            self.run();
            let (best, score) = self.get_best();
            let best_network = best.neural_network().clone();

            Self::save_network(score, &best_network);

            println!("best score: {}", score);

            self.simulations.clear();

            for _ in 1..10 {
                let mut neural_network = best_network.clone();
                neural_network.mutate(0.1, -0.5..0.5);
                self.simulations.push(Simulation::new(neural_network))
            }
            self.simulations.push(Simulation::new(best_network));
        }
    }

    fn run(&mut self) {
        self.simulations.par_iter_mut().for_each(|simulation| {
            for _ in 0..STEPS_PER_SIMULATION {
                simulation.step();
            }
        });
    }

    fn get_best(&self) -> (&Simulation, usize) {
        self.simulations
            .iter()
            .map(|simulation| (simulation, Self::eval(simulation)))
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .next()
            .unwrap()
    }

    fn eval(simulation: &Simulation) -> usize {
        simulation.stats().dropped_of_food * 5 + simulation.stats().picked_up_food
    }

    fn save_network(score: usize, best_network: &NeuralNetwork) {
        let path_string = format!(
            "./training/{}-{}.json",
            Local::now().format("%Y-%m-%d#%H:%M:%S"),
            score
        );
        let path = std::path::Path::new(&path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        fs::write(path, serde_json::to_string(best_network).unwrap()).unwrap();
    }
}
