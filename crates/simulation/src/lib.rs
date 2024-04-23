use crate::ant::Ant;
use crate::pheromone::Pheromone;
use rayon::prelude::*;

use std::time::{Duration, Instant};

mod ant;
mod pheromone;

const TICKS_UNTIL_PHEROMONE: usize = 10;
pub struct Simulation {
    ants: Vec<Ant>,
    pheromones: Vec<Pheromone>,
    ticks_until_pheromone: usize,
    timings: Timings,
}

pub struct Timings {
    pub ant_updates: Duration,
    pub pheromone_updates: Duration,
    pub pheromone_spawn: Duration,
    pub pheromone_remove: Duration,
}

impl Default for Simulation {
    fn default() -> Self {
        let mut ants = vec![];

        for _ in 0..100 {
            ants.push(Ant::random());
        }

        Simulation {
            ants,
            pheromones: vec![],
            ticks_until_pheromone: TICKS_UNTIL_PHEROMONE,
            timings: Timings {
                ant_updates: Default::default(),
                pheromone_updates: Default::default(),
                pheromone_spawn: Default::default(),
                pheromone_remove: Default::default(),
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

    pub fn step(&mut self) {
        let instant = Instant::now();
        self.ants.par_iter_mut().for_each(|ant| ant.step());
        self.timings.ant_updates = instant.elapsed();

        if self.ticks_until_pheromone == 0 {
            let instant = Instant::now();
            self.ticks_until_pheromone = TICKS_UNTIL_PHEROMONE;

            self.pheromones.reserve(self.ants.len());
            for ant in &mut self.ants {
                self.pheromones.push(ant.new_pheromone())
            }
            self.timings.pheromone_spawn = instant.elapsed();
        } else {
            self.ticks_until_pheromone -= 1;
        }

        let instant = Instant::now();
        self.pheromones
            .par_iter_mut()
            .for_each(|pheromone| pheromone.step());
        self.timings.pheromone_updates = instant.elapsed();

        let instant = Instant::now();
        self.pheromones
            .retain(|pheromone| !pheromone.should_be_removed());
        self.timings.pheromone_remove = instant.elapsed();
    }
}
