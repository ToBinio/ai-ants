use crate::pheromone::Pheromone;
use crate::GAME_SIZE;
use glam::{vec2, Vec2};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

const ANT_SPEED: f32 = 100.;
pub const ANT_PICK_UP_DISTANCE: f32 = 10.;

pub struct Ant {
    pos: Vec2,
    dir: f32,
    target_dir: f32,

    carries_food: bool,

    pheromon_color: (f32, f32, f32),
}

impl Ant {
    pub fn random() -> Ant {
        let mut rng = thread_rng();

        Ant {
            pos: vec2(0.0, 0.0),
            dir: rng.gen_range(-(2. * PI)..(2. * PI)),
            target_dir: rng.gen_range(-(2. * PI)..(2. * PI)),
            carries_food: false,
            pheromon_color: (0.0, 0.0, 0.0),
        }
    }

    pub fn pos(&self) -> &Vec2 {
        &self.pos
    }

    pub fn dir(&self) -> f32 {
        self.dir
    }

    pub fn carries_food(&self) -> bool {
        self.carries_food
    }

    pub fn flip(&mut self) {
        self.dir += PI;
        self.target_dir += PI;
    }

    pub fn set_carries_food(&mut self, carries_food: bool) {
        self.carries_food = carries_food
    }

    pub fn set_neural_network_values(&mut self, values: Vec<f64>) {
        self.target_dir += values[0] as f32;
        self.pheromon_color = (values[1] as f32, values[2] as f32, values[3] as f32)
    }

    pub fn get_neural_network_values(&self) -> Vec<f64> {
        vec![
            (self.pos.x / GAME_SIZE) as f64,
            (self.pos.y / GAME_SIZE) as f64,
            (self.dir / PI * 2.) as f64,
            (self.target_dir / PI * 2.) as f64,
            if self.carries_food { 1. } else { -1. },
        ]
    }

    pub fn step(&mut self) {
        //rotate to target
        //todo dont do that... dont create vec`s...
        let angle_diff =
            Vec2::from_angle(self.target_dir).angle_between(Vec2::from_angle(self.dir));

        self.dir += angle_diff * 0.01;

        self.dir %= PI * 2.;
        self.target_dir %= PI * 2.;

        //move ant
        //calc how fast to move based on how strong the ant is turning
        let mov_speed = 1. - angle_diff.abs() / (PI * 2.);
        let mov_speed = ANT_SPEED * mov_speed;
        // 60 = frame rate
        let mov_speed = mov_speed / 60.;

        self.pos += Vec2::from_angle(self.dir) * mov_speed
    }

    pub fn new_pheromone(&self) -> Pheromone {
        Pheromone::new(self.pos, 5., self.pheromon_color)
    }
}
