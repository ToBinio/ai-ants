use std::{fs, io, time::Instant};

use chrono::Local;
use console::Term;
use fancy_duration::AsFancyDuration;
use neural_network::NeuralNetwork;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use simulation::Simulation;

use crate::STEPS_PER_SIMULATION;

pub struct Trainer {
    simulations: Vec<(Simulation, usize)>,
    simulation_count: usize,
}

impl Trainer {
    pub fn new(simulation_count: usize) -> Trainer {
        let mut simulations = vec![];

        for _ in 0..simulation_count {
            simulations.push((Simulation::default(), 0))
        }

        Trainer {
            simulations,
            simulation_count,
        }
    }

    pub fn train(&mut self) -> io::Result<()> {
        let term = Term::stdout();
        term.write_line("starting Training")?;

        let start_time = Instant::now();

        let mut gen_count = 0;

        loop {
            gen_count += 1;
            self.run();

            for sim in &mut self.simulations {
                sim.1 = Self::eval(&sim.0)
            }

            self.simulations.sort_by(|a, b| b.1.cmp(&a.1));
            self.simulations
                .dedup_by(|a, b| a.0.neural_network() == b.0.neural_network());

            Self::save_network(
                gen_count,
                self.simulations[0].1,
                self.simulations[0].0.neural_network(),
            );

            term.clear_line()?;
            term.write_line(&format!(
                "gen({}) score: {} avg({}) - {}",
                gen_count,
                self.simulations[0].1,
                self.simulations
                    .iter()
                    .map(|(_, score)| *score)
                    .sum::<usize>()
                    / self.simulations.len(),
                start_time.elapsed().fancy_duration().truncate(2)
            ))?;
            term.move_cursor_up(1)?;

            let mut new_simulations = vec![];

            let top_30 = (self.simulation_count as f32 * 0.3).ceil() as usize;

            // keep top 30% as is
            for i in 0..top_30.min(self.simulations.len()) {
                new_simulations.push((
                    Simulation::new(self.simulations[i].0.neural_network().clone()),
                    0,
                ));
            }

            let mut network_chances = vec![];
            let mut last_chance = 0;

            for i in 0..self.simulations.len() {
                last_chance += self.simulations[i].1;
                network_chances.push((last_chance, self.simulations[i].0.neural_network().clone()));
            }

            let mut rng = thread_rng();

            'outer: while new_simulations.len() != self.simulation_count {
                let random = rng.gen_range(0..last_chance);

                for (chance, network) in &network_chances {
                    if &random <= chance {
                        let mut neural_network = network.clone();

                        for _ in 0..10 {
                            neural_network.mutate(0.4, 0.2);
                        }

                        new_simulations.push((Simulation::new(neural_network), 0));

                        continue 'outer;
                    }
                }
            }

            self.simulations = new_simulations;
        }
    }

    fn run(&mut self) {
        self.simulations.par_iter_mut().for_each(|simulation| {
            for _ in 0..STEPS_PER_SIMULATION {
                simulation.0.step();
            }
        });
    }

    fn eval(simulation: &Simulation) -> usize {
        simulation.stats().dropped_of_food * 5 + simulation.stats().picked_up_food
    }

    fn save_network(gen: usize, score: usize, best_network: &NeuralNetwork) {
        let path_string = format!(
            "./training/{}-{}-{}.json",
            gen,
            Local::now().format("%Y-%m-%d_%H-%M-%S"),
            score
        );
        let path = std::path::Path::new(&path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        fs::write(path, serde_json::to_string(best_network).unwrap()).unwrap();
    }
}
