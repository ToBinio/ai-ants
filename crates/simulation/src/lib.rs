use crate::ant::{Ant, ANT_PICK_UP_DISTANCE};
use crate::pheromone::Pheromone;
use rayon::prelude::*;

use crate::food::Food;
use glam::vec2;
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

mod ant;
mod food;
mod pheromone;

const TICKS_UNTIL_PHEROMONE: usize = 10;
pub struct Simulation {
    ants: Vec<Ant>,
    pheromones: Vec<Pheromone>,
    foods: Vec<Food>,

    ticks_until_pheromone: usize,
    timings: Timings,
}

pub struct Timings {
    pub ant_updates: Duration,
    pub pheromone_updates: Duration,
    pub pheromone_spawn: Duration,
    pub pheromone_remove: Duration,
    pub pick_up_food: Duration,
}

impl Default for Simulation {
    fn default() -> Self {
        let mut ants = vec![];

        for _ in 0..100 {
            ants.push(Ant::random());
        }

        let mut foods = vec![];
        let mut rng = thread_rng();

        for _ in 0..100 {
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
                pheromone_updates: Default::default(),
                pheromone_spawn: Default::default(),
                pheromone_remove: Default::default(),
                pick_up_food: Default::default(),
            },
        }
    }
}

impl Simulation {
    pub fn timings(&self) -> &Timings {
        &self.timings
    }

    pub fn ants(&self) -> &Vec<Ant> {
        &self.ants
    }

    pub fn pheromones(&self) -> &Vec<Pheromone> {
        &self.pheromones
    }
    pub fn foods(&self) -> &Vec<Food> {
        &self.foods
    }

    pub fn step(&mut self) {
        Simulation::update_ants(&mut self.ants, &mut self.timings);

        if self.ticks_until_pheromone == 0 {
            self.ticks_until_pheromone = TICKS_UNTIL_PHEROMONE;
            Simulation::spawn_pheromones(&mut self.pheromones, &self.ants, &mut self.timings);
        } else {
            self.ticks_until_pheromone -= 1;
        }

        Simulation::update_pheromones(&mut self.pheromones, &mut self.timings);
        Simulation::pick_up_food(&mut self.ants, &mut self.foods, &mut self.timings);
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

    fn spawn_pheromones(pheromones: &mut Vec<Pheromone>, ants: &Vec<Ant>, timings: &mut Timings) {
        let instant = Instant::now();

        pheromones.reserve(ants.len());
        for ant in ants {
            pheromones.push(ant.new_pheromone())
        }
        timings.pheromone_spawn = instant.elapsed();
    }

    fn pick_up_food(ants: &mut Vec<Ant>, foods: &mut Vec<Food>, timings: &mut Timings) {
        let instant = Instant::now();

        for ant in ants.iter_mut().filter(|ant| !ant.carries_food()) {
            let mut picked_up_food = None;

            for (index, food) in foods.iter().enumerate() {
                let distance =
                    vec2(food.pos().x - ant.pos().x, food.pos().y - ant.pos().y).length();
                if distance < ANT_PICK_UP_DISTANCE {
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
}
