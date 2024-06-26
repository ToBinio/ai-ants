use crate::GAME_SIZE;
use glam::{vec2, Vec2};
use std::f32::consts::PI;

pub const ANT_SPEED: f32 = 100.;
pub const ANT_PICK_UP_DISTANCE: f32 = 10.;

pub const ANT_RAY_COUNT: usize = 7;
//see 90° evenly
pub const ANT_RAY_ANGLE: f32 = ((PI * 2.) / 4.) / ANT_RAY_COUNT as f32;
pub const ANT_SEE_DISTANCE: f32 = 50.;

pub struct Ant {
    pos: Vec2,
    dir: f32,
    target_dir: f32,

    carries_food: bool,

    pheromone_color: (f32, f32, f32),

    rays: Vec<f32>,
}

impl Ant {
    pub fn from_direction(direction: f32) -> Ant {
        Ant {
            pos: vec2(0.0, 0.0),
            dir: direction,
            target_dir: direction,
            carries_food: false,
            pheromone_color: (0.0, 0.0, 0.0),
            rays: vec![0.; ANT_RAY_COUNT],
        }
    }

    pub fn pos(&self) -> &Vec2 {
        &self.pos
    }

    //todo maybe dont... just add some modification function
    pub fn pos_mut(&mut self) -> &mut Vec2 {
        &mut self.pos
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

    pub fn set_neural_network_values(&mut self, values: Vec<f32>) {
        self.target_dir += values[0] / 100.;
        self.pheromone_color = (values[1], values[2], values[3])
    }

    pub fn get_neural_network_values(&self) -> Vec<f32> {
        let mut values = vec![
            (self.pos.x / GAME_SIZE),
            (self.pos.y / GAME_SIZE),
            (self.dir / PI * 2.),
            (self.target_dir / PI * 2.),
            if self.carries_food { 1. } else { -1. },
        ];

        for ray in &self.rays {
            values.push(*ray);
        }

        values
    }

    pub fn set_rays(&mut self, value: Vec<f32>) {
        self.rays = value;
    }

    pub fn get_ray_directions(dir: f32) -> Vec<Vec2> {
        let mut current_angle = (ANT_RAY_COUNT as f32 / 2.).floor() * -ANT_RAY_ANGLE;

        let mut rays = Vec::with_capacity(ANT_RAY_COUNT);
        for _ in 0..ANT_RAY_COUNT {
            current_angle += ANT_RAY_ANGLE;
            rays.push(Vec2::from_angle(current_angle + dir))
        }

        rays
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

    pub fn pheromone_color(&self) -> (f32, f32, f32) {
        self.pheromone_color
    }
}
