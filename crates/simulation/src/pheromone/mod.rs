use glam::{vec2, Vec2};
use rand::thread_rng;
use std::f32::consts::PI;

pub struct Pheromone {
    pos: Vec2,
    size: f32,
    strength: f32,
    color: (f32, f32, f32),
}

impl Pheromone {
    pub fn new(pos: Vec2, strength: f32) -> Pheromone {
        return Pheromone {
            pos,
            size: 1.0,
            strength,
            color: (0.8, 0., 1.),
        };
    }

    pub fn should_be_removed(&self) -> bool {
        self.density() < 0.01
    }

    pub fn pos(&self) -> &Vec2 {
        &self.pos
    }
    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn strength(&self) -> f32 {
        self.strength
    }

    pub fn density(&self) -> f32 {
        self.strength / (self.size * self.size * PI)
    }

    pub fn color(&self) -> &(f32, f32, f32) {
        &self.color
    }

    pub fn step(&mut self) {
        self.size *= 1.002;
    }
}
