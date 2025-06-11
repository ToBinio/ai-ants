use std::{fs, io, time::Instant};

use chrono::Local;
use console::Term;
use fancy_duration::AsFancyDuration;
use glam::vec2;
use itertools::{izip, Itertools};
use neural_network::NeuralNetwork;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use simulation::Simulation;

use crate::STEPS_PER_SIMULATION;

pub struct Trainer {
    simulations: Vec<SimulationData>,
    simulation_count: usize,
    perturbed_count: usize,
}

struct SimulationData {
    base: Simulation,
    perturbed: Vec<Simulation>,
    reward: f32,
}

impl Trainer {
    pub fn new(simulation_count: usize, perturbed_count: usize) -> Trainer {
        let simulations = (0..simulation_count)
            .map(|_| SimulationData {
                base: Simulation::default(),
                perturbed: vec![],
                reward: 0.,
            })
            .collect_vec();

        Trainer {
            simulations,
            simulation_count,
            perturbed_count,
        }
    }

    pub fn train(&mut self) -> io::Result<()> {
        let term = Term::stdout();
        term.write_line("starting Training")?;

        let start_time = Instant::now();

        let mut gen_count = 0;

        loop {
            gen_count += 1;

            //gradient ascent
            for _ in 0..5 {
                //create pertubed
                for data in &mut self.simulations {
                    data.perturbed = (0..self.perturbed_count)
                        .map(|_| {
                            let mut network = data.base.neural_network().clone();
                            network.randomize_weights(0.05, 1.);
                            Simulation::new(network)
                        })
                        .collect_vec();
                }

                //run perturbed networks
                Self::run(
                    self.simulations
                        .iter_mut()
                        .map(|data| &mut data.perturbed)
                        .flat_map(|permuted| permuted)
                        .collect(),
                );

                //gradient ascent
                for data in &mut self.simulations {
                    let perturbed = data
                        .perturbed
                        .iter()
                        .map(|simulation| (simulation.neural_network(), Self::eval(simulation)))
                        .collect_vec();

                    data.base
                        .neural_network_mut()
                        .gradient_ascent(0.5, perturbed);
                }
            }

            // run base simulations
            Self::run(
                self.simulations
                    .iter_mut()
                    .map(|data| &mut data.base)
                    .collect(),
            );

            for sim in &mut self.simulations {
                sim.reward = Self::eval(&sim.base)
            }

            self.simulations
                .sort_by(|a, b| b.reward.total_cmp(&a.reward));
            self.simulations
                .dedup_by(|a, b| a.base.neural_network() == b.base.neural_network());

            Self::save_network(
                gen_count,
                self.simulations[0].reward,
                self.simulations[0].base.neural_network(),
            );

            term.clear_line()?;
            term.write_line(&format!(
                "gen({}) score: {} avg({}) - {}",
                gen_count,
                self.simulations[0].reward,
                self.simulations.iter().map(|data| data.reward).sum::<f32>()
                    / self.simulations.len() as f32,
                start_time.elapsed().fancy_duration().truncate(2)
            ))?;
            term.move_cursor_up(1)?;

            let mut new_simulations = vec![];

            let top_30 = (self.simulation_count as f32 * 0.3).ceil() as usize;

            // keep top 30% as is
            for i in 0..top_30.min(self.simulations.len()) {
                new_simulations.push(SimulationData {
                    base: Simulation::new(self.simulations[i].base.neural_network().clone()),
                    perturbed: vec![],
                    reward: 0.,
                });
            }

            let mut network_chances = vec![];
            let mut last_chance = 1.;

            for i in 0..self.simulations.len() {
                last_chance += self.simulations[i].reward;
                network_chances.push((
                    last_chance,
                    self.simulations[i].base.neural_network().clone(),
                ));
            }

            let mut rng = thread_rng();

            'outer: while new_simulations.len() != self.simulation_count {
                let random = rng.gen_range(0. ..last_chance);

                for (chance, network) in &network_chances {
                    if &random <= chance {
                        let mut neural_network = network.clone();

                        for _ in 0..rng.gen_range(0..5) {
                            neural_network.mutate_strucutre();
                        }
                        neural_network.randomize_weights(0.2, 0.5);

                        new_simulations.push(SimulationData {
                            base: Simulation::new(neural_network),
                            perturbed: vec![],
                            reward: 0.,
                        });

                        continue 'outer;
                    }
                }
            }

            self.simulations = new_simulations;
        }
    }

    fn run(networks: Vec<&mut Simulation>) {
        networks.into_par_iter().for_each(|simulation| {
            for _ in 0..STEPS_PER_SIMULATION {
                simulation.step();
            }
        });
    }

    fn eval(simulation: &Simulation) -> f32 {
        let mut score = 0.;

        for (carries, position) in izip!(
            &simulation.ants().caries_foods,
            &simulation.ants().positions
        ) {
            if *carries {
                score += 1. - (position.distance(vec2(0., 0.)) / 1000.);
            } else {
                score += 1. - (position.distance(vec2(325., 325.)) / 1000.);
            }
        }

        score += simulation.stats().dropped_of_food as f32 * 5.
            + simulation.stats().picked_up_food as f32;

        score
    }

    fn save_network(gen: usize, score: f32, best_network: &NeuralNetwork) {
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
