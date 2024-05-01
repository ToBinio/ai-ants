use crate::ant::{Ant, ANT_PICK_UP_DISTANCE};
use crate::pheromone::Pheromone;
use rayon::prelude::*;

use crate::food::Food;
use glam::vec2;
use neural_network::NeuralNetwork;
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

mod ant;
mod food;
mod pheromone;

const TICKS_UNTIL_PHEROMONE: usize = 10;
pub const ANT_HILL_RADIUS: f32 = 50.;
pub const GAME_SIZE: f32 = 500.;
pub struct Simulation {
    ants: Vec<Ant>,
    pheromones: Vec<Pheromone>,
    foods: Vec<Food>,

    ticks_until_pheromone: usize,
    timings: Timings,
    stats: Stats,

    neural_network: NeuralNetwork,
}

pub struct Stats {
    pub step_count: usize,
    pub picked_up_food: usize,
    pub dropped_of_food: usize,
}

pub struct Timings {
    pub ant_updates: Duration,
    pub keep_ants: Duration,
    pub neural_network_updates: Duration,
    pub pheromone_updates: Duration,
    pub pheromone_spawn: Duration,
    pub pheromone_remove: Duration,
    pub pick_up_food: Duration,
    pub drop_of_food: Duration,
}

impl Simulation {
    pub fn new(neural_network: NeuralNetwork) -> Simulation {
        assert_eq!(
            neural_network.get_input_size(),
            5,
            "Neural-network has wrong input size"
        );
        assert_eq!(
            neural_network.get_output_size(),
            1,
            "Neural-network has wrong output size"
        );

        let mut ants = vec![];

        for _ in 0..200 {
            ants.push(Ant::random());
        }

        let mut foods = vec![];
        let mut rng = thread_rng();

        for _ in 0..1000 {
            foods.push(Food::new(vec2(
                rng.gen_range(100.0..200.0),
                rng.gen_range(100.0..200.0),
            )))
        }

        Simulation {
            ants,
            pheromones: vec![],
            foods,
            ticks_until_pheromone: TICKS_UNTIL_PHEROMONE,
            timings: Timings {
                ant_updates: Default::default(),
                keep_ants: Default::default(),
                neural_network_updates: Default::default(),
                pheromone_updates: Default::default(),
                pheromone_spawn: Default::default(),
                pheromone_remove: Default::default(),
                pick_up_food: Default::default(),
                drop_of_food: Default::default(),
            },
            stats: Stats {
                step_count: 0,
                picked_up_food: 0,
                dropped_of_food: 0,
            },
            neural_network,
        }
    }
    pub fn timings(&self) -> &Timings {
        &self.timings
    }

    pub fn ants(&self) -> &Vec<Ant> {
        &self.ants
    }

    pub fn pheromones(&self) -> &Vec<Pheromone> {
        &self.pheromones
    }
    pub fn neural_network(&self) -> &NeuralNetwork {
        &self.neural_network
    }
    pub fn foods(&self) -> &Vec<Food> {
        &self.foods
    }
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    pub fn step(&mut self) {
        self.stats.step_count += 1;

        Simulation::update_network(&mut self.ants, &self.neural_network, &mut self.timings);
        Simulation::update_ants(&mut self.ants, &mut self.timings);
        Simulation::keep_ants(&mut self.ants, &mut self.timings);

        if self.ticks_until_pheromone == 0 {
            self.ticks_until_pheromone = TICKS_UNTIL_PHEROMONE;
            Simulation::spawn_pheromones(&mut self.pheromones, &self.ants, &mut self.timings);
        } else {
            self.ticks_until_pheromone -= 1;
        }

        Simulation::update_pheromones(&mut self.pheromones, &mut self.timings);
        Simulation::pick_up_food(
            &mut self.ants,
            &mut self.foods,
            &mut self.timings,
            &mut self.stats,
        );
        Simulation::drop_of_food(&mut self.ants, &mut self.timings, &mut self.stats);
    }

    fn update_network(ants: &mut Vec<Ant>, neural_network: &NeuralNetwork, timings: &mut Timings) {
        let instant = Instant::now();
        ants.par_iter_mut().for_each(|ant| {
            let values = neural_network.run(ant.get_neural_network_values());
            ant.set_neural_network_values(values);
        });
        timings.neural_network_updates = instant.elapsed();
    }

    fn update_pheromones(pheromones: &mut Vec<Pheromone>, timings: &mut Timings) {
        let instant = Instant::now();
        pheromones
            .par_iter_mut()
            .for_each(|pheromone| pheromone.step());
        timings.pheromone_updates = instant.elapsed();

        let instant = Instant::now();
        pheromones.retain(|pheromone| !pheromone.should_be_removed());
        timings.pheromone_remove = instant.elapsed();
    }

    fn update_ants(ants: &mut Vec<Ant>, timings: &mut Timings) {
        let instant = Instant::now();
        ants.par_iter_mut().for_each(|ant| ant.step());
        timings.ant_updates = instant.elapsed();
    }

    fn keep_ants(ants: &mut Vec<Ant>, timings: &mut Timings) {
        let instant = Instant::now();
        ants.par_iter_mut().for_each(|ant| {
            if ant.pos().x > GAME_SIZE
                || ant.pos().x < -GAME_SIZE
                || ant.pos().y > GAME_SIZE
                || ant.pos().y < -GAME_SIZE
            {
                ant.flip()
            }
        });
        timings.keep_ants = instant.elapsed();
    }

    fn spawn_pheromones(pheromones: &mut Vec<Pheromone>, ants: &Vec<Ant>, timings: &mut Timings) {
        let instant = Instant::now();

        pheromones.reserve(ants.len());
        for ant in ants {
            pheromones.push(ant.new_pheromone())
        }
        timings.pheromone_spawn = instant.elapsed();
    }

    fn pick_up_food(
        ants: &mut [Ant],
        foods: &mut Vec<Food>,
        timings: &mut Timings,
        stats: &mut Stats,
    ) {
        let instant = Instant::now();

        for ant in ants.iter_mut().filter(|ant| !ant.carries_food()) {
            let mut picked_up_food = None;

            for (index, food) in foods.iter().enumerate() {
                let distance =
                    vec2(food.pos().x - ant.pos().x, food.pos().y - ant.pos().y).length();
                if distance < ANT_PICK_UP_DISTANCE {
                    stats.picked_up_food += 1;
                    picked_up_food = Some(index);
                    break;
                }
            }

            if let Some(index) = picked_up_food {
                ant.set_carries_food(true);
                foods.remove(index);
            }
        }

        timings.pick_up_food = instant.elapsed();
    }

    fn drop_of_food(ants: &mut [Ant], timings: &mut Timings, stats: &mut Stats) {
        let instant = Instant::now();

        ants.iter_mut()
            .filter(|ant| ant.carries_food())
            .filter(|ant| ant.pos().length() < ANT_HILL_RADIUS)
            .for_each(|ant| {
                stats.dropped_of_food += 1;
                ant.set_carries_food(false)
            });

        timings.drop_of_food = instant.elapsed();
    }
}
