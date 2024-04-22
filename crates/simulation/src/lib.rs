use crate::ant::Ant;
use std::ops::{Add, Mul};

mod ant;
pub struct Simulation {
    ants: Vec<Ant>,
}

impl Simulation {
    pub fn new() -> Simulation {
        let mut ants = vec![];

        for _ in 0..10 {
            ants.push(Ant::random());
        }

        Simulation { ants }
    }

    pub fn ants(&self) -> &Vec<Ant> {
        &self.ants
    }

    pub fn step(&mut self) {
        for ant in &mut self.ants {
            ant.step()
        }
    }
}
