use crate::pheromone::Pheromone;
use glam::{vec2, Vec2};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

const ANT_SPEED: f32 = 100.;

pub struct Ant {
    pos: Vec2,
    dir: f32,
    target_dir: f32,
}

impl Ant {
    pub fn random() -> Ant {
        let mut rng = thread_rng();

        Ant {
            pos: vec2(0.0, 0.0),
            dir: rng.gen_range(-(2. * PI)..(2. * PI)),
            target_dir: rng.gen_range(-(2. * PI)..(2. * PI)),
        }
    }

    pub fn pos(&self) -> &Vec2 {
        &self.pos
    }

    pub fn dir(&self) -> f32 {
        self.dir
    }

    pub fn step(&mut self) {
        //update angles
        //temp - later based on neural network
        let mut rng = thread_rng();
        self.target_dir += rng.gen_range(-0.5..0.5);

        //rotate to target
        //todo dont do that... dont create vec`s...
        let angle_diff =
            Vec2::from_angle(self.target_dir).angle_between(Vec2::from_angle(self.dir));

        self.dir += angle_diff * 0.01;

        //move ant
        //calc how fast to move based on how strong the ant is turning
        let mov_speed = 1. - angle_diff.abs() / (PI * 2.);
        let mov_speed = ANT_SPEED * mov_speed;
        // 60 = frame rate
        let mov_speed = mov_speed / 60.;

        self.pos += Vec2::from_angle(self.dir) * mov_speed
    }

    pub fn new_pheromone(&self) -> Pheromone {
        //todo strength from ai
        Pheromone::new(self.pos, 5.)
    }
}
