use std::f32::consts::PI;

use glam::Vec2;

pub const ANT_SPEED: f32 = 100.;
pub const ANT_PICK_UP_DISTANCE: f32 = 10.;

pub const ANT_RAY_COUNT: usize = 7;
//see 90° evenly
pub const ANT_RAY_ANGLE: f32 = ((PI * 2.) / 4.) / ANT_RAY_COUNT as f32;
pub const ANT_SEE_DISTANCE: f32 = 50.;

pub struct Ants {
    //todo dont all pub
    pub positions: Vec<Vec2>,
    pub dirs: Vec<f32>,
    pub target_dirs: Vec<f32>,
    pub caries_foods: Vec<bool>,
    pub pheromone_colors: Vec<(f32, f32, f32)>,
    pub rays: Vec<Vec<f32>>,
}

impl Ants {
    pub fn get_ray_directions(dir: f32) -> impl Iterator<Item = Vec2> {
        const BASE_ANGLE: f32 = (ANT_RAY_COUNT / 2) as f32 * -ANT_RAY_ANGLE;
        (0..ANT_RAY_COUNT)
            .map(move |i| Vec2::from_angle(BASE_ANGLE + ANT_RAY_ANGLE * i as f32 + dir))
    }
}
