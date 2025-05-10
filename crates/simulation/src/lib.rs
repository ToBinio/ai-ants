use std::{cell::OnceCell, f32::consts::PI};

use crate::food::Food;
use crate::grid::Grid;
use crate::timings::Timings;
use ants::{Ants, ANT_PICK_UP_DISTANCE, ANT_RAY_COUNT, ANT_SEE_DISTANCE};
use glam::{vec2, Vec2};
use itertools::Itertools;
use math::ray_inserect_circle;
use neural_network::NeuralNetwork;
use std::time::Instant;

mod food;
mod grid;
mod math;
pub mod timings;

pub mod ants;

const TICKS_UNTIL_PHEROMONE: usize = 10;
pub const ANT_HILL_RADIUS: f32 = 50.;
pub const GAME_SIZE: f32 = 500.;
pub const FOOD_SIZE: f32 = 7.;

pub const NEURAL_NETWORK_INPUT_SIZE: usize = 5 + ANT_RAY_COUNT;
pub const NEURAL_NETWORK_OUTPUT_SIZE: usize = 4;

pub struct Simulation {
    ants: Ants,
    pheromones: Pheromones,
    foods: Grid<Food>,

    ticks_until_pheromone: usize,
    timings: Timings,
    stats: Stats,

    neural_network: NeuralNetwork,
}

pub struct Pheromones {
    //todo dont all pub
    pub grid: Grid<usize>,
    pub positions: Vec<Vec2>,
    pub colors: Vec<(f32, f32, f32)>,

    // size per pheromone group, None marking a deleted section
    pub sizes: Vec<Option<f32>>,
}
impl Default for Simulation {
    fn default() -> Self {
        Simulation::new(NeuralNetwork::new(
            NEURAL_NETWORK_INPUT_SIZE,
            NEURAL_NETWORK_OUTPUT_SIZE,
        ))
    }
}

pub struct Stats {
    pub step_count: usize,
    pub picked_up_food: usize,
    pub dropped_of_food: usize,
}

impl Simulation {
    pub fn zero() -> Self {
        Simulation::new(NeuralNetwork::zero(
            NEURAL_NETWORK_INPUT_SIZE,
            NEURAL_NETWORK_OUTPUT_SIZE,
        ))
    }

    pub fn new(neural_network: NeuralNetwork) -> Simulation {
        assert_eq!(
            neural_network.get_input_size(),
            NEURAL_NETWORK_INPUT_SIZE,
            "Neural-network has wrong input size"
        );
        assert_eq!(
            neural_network.get_output_size(),
            NEURAL_NETWORK_OUTPUT_SIZE,
            "Neural-network has wrong output size"
        );

        let mut ants = Ants {
            positions: vec![],
            dirs: vec![],
            target_dirs: vec![],
            caries_foods: vec![],
            pheromone_colors: vec![],
            rays: vec![],
        };

        const ANTS_TO_SPAWN: usize = 200;
        const ANGLE_PER_ANT: f32 = PI * 2. / ANTS_TO_SPAWN as f32;

        for i in 0..ANTS_TO_SPAWN {
            let direction = ANGLE_PER_ANT * i as f32;

            ants.positions.push(Vec2::ZERO);
            ants.dirs.push(direction);
            ants.target_dirs.push(direction);
            ants.caries_foods.push(false);
            ants.pheromone_colors.push((0.0, 0.0, 0.0));
            ants.rays.push(vec![0.; ANT_RAY_COUNT]);
        }

        let mut foods = Grid::new(25, GAME_SIZE);

        for x in 0..50 {
            for y in 0..50 {
                let pos = vec2(300. + x as f32 * 2., 300. + y as f32 * 2.);
                foods.insert(&pos, Food::new(pos));
            }
        }

        Simulation {
            ants,
            pheromones: Pheromones {
                grid: Grid::new(25, GAME_SIZE),
                positions: vec![],
                sizes: vec![],
                colors: vec![],
            },
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
                see_food: Default::default(),
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

    pub fn ants(&self) -> &Ants {
        &self.ants
    }

    pub fn pheromones(&self) -> &Pheromones {
        &self.pheromones
    }
    pub fn neural_network(&self) -> &NeuralNetwork {
        &self.neural_network
    }
    pub fn foods(&self) -> Vec<&Food> {
        self.foods.all()
    }
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    pub fn step(&mut self) {
        self.stats.step_count += 1;

        Simulation::update_network(&mut self.ants, &self.neural_network, &mut self.timings);
        Simulation::update_ants(&mut self.ants, &mut self.timings);
        Simulation::see_food(&mut self.ants, &mut self.foods, &mut self.timings);
        Simulation::keep_ants(&mut self.ants, &mut self.timings);

        if self.ticks_until_pheromone == 0 {
            self.ticks_until_pheromone = TICKS_UNTIL_PHEROMONE;
            Simulation::spawn_pheromones(&mut self.pheromones, &self.ants, &mut self.timings);
        } else {
            self.ticks_until_pheromone -= 1;
        }

        Simulation::update_pheromones(
            &mut self.pheromones,
            self.ants.positions.len(),
            &mut self.timings,
        );
        Simulation::pick_up_food(
            &mut self.ants,
            &mut self.foods,
            &mut self.timings,
            &mut self.stats,
        );
        Simulation::drop_of_food(&mut self.ants, &mut self.timings, &mut self.stats);
    }

    fn update_network(ants: &mut Ants, neural_network: &NeuralNetwork, timings: &mut Timings) {
        let instant = Instant::now();

        for index in 0..ants.positions.len() {
            let pos = &ants.positions[index];
            let dir = &ants.dirs[index];
            let target_dir = &ants.target_dirs[index];
            let carries_food = ants.caries_foods[index];
            let rays = &ants.rays[index];

            let mut values = vec![
                pos.x / GAME_SIZE,
                pos.y / GAME_SIZE,
                dir / (PI * 2.),
                target_dir / (PI * 2.),
                if carries_food { 1. } else { -1. },
            ];

            for ray in rays {
                values.push(*ray);
            }

            let values = neural_network.run(values);

            ants.target_dirs[index] += values[0] / 120.;
            ants.pheromone_colors[index] = (values[1], values[2], values[3]);
        }

        timings.neural_network_updates.add(&instant.elapsed());
    }

    fn update_pheromones(pheromones: &mut Pheromones, ant_count: usize, timings: &mut Timings) {
        let instant = Instant::now();

        pheromones
            .sizes
            .iter_mut()
            .filter(|size| size.is_some())
            .for_each(|size| *size.as_mut().unwrap() *= 1.002);

        timings.pheromone_updates.add(&instant.elapsed());

        let instant = Instant::now();

        let to_be_removed = pheromones
            .sizes
            .iter()
            .enumerate()
            .filter(|(_, size)| size.is_some())
            //todo extract density calc to function
            .filter(|(_, size)| 5. / (size.unwrap() * size.unwrap() * PI) < 0.01)
            .map(|(index, _)| index)
            .next();

        if let Some(to_be_removed) = to_be_removed {
            //todo utility function - use while spawing and rendering
            let index_range = ant_count * to_be_removed..ant_count * (to_be_removed + 1);

            pheromones
                .grid
                .retain(|pheromone| !index_range.contains(pheromone));

            pheromones.sizes[to_be_removed] = None;
        }

        timings.pheromone_remove.add(&instant.elapsed());
    }

    fn update_ants(ants: &mut Ants, timings: &mut Timings) {
        let instant = Instant::now();

        for index in 0..ants.positions.len() {
            let pos = ants.positions[index];
            let mut dir = ants.dirs[index];
            let target_dir = ants.target_dirs[index];

            //rotate to target
            //todo dont do that... dont create vec`s...
            let angle_diff = Vec2::from_angle(target_dir).angle_between(Vec2::from_angle(dir));

            dir += angle_diff * 0.01;
            dir %= PI * 2.;

            ants.dirs[index] = dir;
            ants.target_dirs[index] = target_dir % (PI * 2.);

            //move ant
            //calc how fast to move based on how strong the ant is turning
            let mov_speed = 1. - angle_diff.abs() / (PI * 2.);
            let mov_speed = crate::ants::ANT_SPEED * mov_speed;
            // 60 = frame rate
            let mov_speed = mov_speed / 60.;

            ants.positions[index] = pos + Vec2::from_angle(dir) * mov_speed
        }

        timings.ant_updates.add(&instant.elapsed());
    }

    fn keep_ants(ants: &mut Ants, timings: &mut Timings) {
        let instant = Instant::now();

        //todo maybe not zip but didnt get faster...
        for ((pos, dir), target_dir) in ants
            .positions
            .iter_mut()
            .zip(ants.dirs.iter_mut())
            .zip(ants.target_dirs.iter_mut())
        {
            if pos.x > GAME_SIZE {
                pos.x -= 10.;
                *dir += PI;
                *target_dir += PI;
            }

            if pos.x < -GAME_SIZE {
                pos.x += 10.;
                *dir += PI;
                *target_dir += PI;
            }

            if pos.y > GAME_SIZE {
                pos.y -= 10.;
                *dir += PI;
                *target_dir += PI;
            }

            if pos.y < -GAME_SIZE {
                pos.y += 10.;
                *dir += PI;
                *target_dir += PI;
            }
        }

        timings.keep_ants.add(&instant.elapsed());
    }

    fn spawn_pheromones(pheromones: &mut Pheromones, ants: &Ants, timings: &mut Timings) {
        let instant = Instant::now();

        //todo dont dynamically build up - precompute needed size and use that...
        // no need to handle sizes as optional than simple count on with index and overwrite than

        let to_be_replaced = pheromones
            .sizes
            .iter()
            .find_position(|value| value.is_none());

        if to_be_replaced.is_some() {
            let (to_be_replaced, _) = to_be_replaced.unwrap();

            pheromones.sizes[to_be_replaced] = Some(1.);

            let offset = ants.positions.len() * to_be_replaced;

            for index in 0..ants.positions.len() {
                pheromones.grid.insert(&ants.positions[index], index);
                pheromones.positions[index + offset] = ants.positions[index];
                pheromones.colors[index + offset] = ants.pheromone_colors[index];
            }
        } else {
            pheromones.sizes.push(Some(1.));

            let len = ants.positions.len();

            for index in 0..len {
                pheromones.grid.insert(&ants.positions[index], len + index);
                pheromones.positions.push(ants.positions[index]);
                pheromones.colors.push(ants.pheromone_colors[index]);
            }
        }

        timings.pheromone_spawn.add(&instant.elapsed());
    }

    fn pick_up_food(
        ants: &mut Ants,
        foods: &mut Grid<Food>,
        timings: &mut Timings,
        stats: &mut Stats,
    ) {
        let instant = Instant::now();

        for (index, carries) in ants
            .caries_foods
            .iter_mut()
            .enumerate()
            .filter(|(_, carries)| !**carries)
        {
            let pos = ants.positions[index];

            foods.for_each(pos, ANT_PICK_UP_DISTANCE, |foods| {
                let mut picked_up_food = None;

                for (index, food) in foods.iter().enumerate() {
                    let distance =
                        vec2(food.pos().x - pos.x, food.pos().y - pos.y).length_squared();
                    if distance < ANT_PICK_UP_DISTANCE * ANT_PICK_UP_DISTANCE {
                        stats.picked_up_food += 1;
                        picked_up_food = Some(index);
                        break;
                    }
                }

                if let Some(index) = picked_up_food {
                    *carries = true;
                    foods.remove(index);
                }
            });
        }

        timings.pick_up_food.add(&instant.elapsed());
    }

    fn see_food(ants: &mut Ants, foods: &mut Grid<Food>, timings: &mut Timings) {
        let instant = Instant::now();

        for index in 0..ants.positions.len() {
            let rays = &mut ants.rays[index];
            let pos = ants.positions[index];
            let dir = ants.dirs[index];

            let ray_directions = OnceCell::new();
            let mut nearest_foods = vec![None; rays.len()];

            foods.for_each(pos, ANT_SEE_DISTANCE, |foods| {
                let ray_directions =
                    ray_directions.get_or_init(|| Ants::get_ray_directions(dir).collect_vec());

                for food in &mut *foods {
                    let distance = food.pos().distance_squared(pos);
                    if distance > ANT_SEE_DISTANCE * ANT_SEE_DISTANCE {
                        continue;
                    }

                    for (index, ray_direction) in ray_directions.iter().enumerate() {
                        if let Some(nearest) = nearest_foods[index] {
                            if nearest < distance {
                                continue;
                            }
                        }

                        let intersection =
                            ray_inserect_circle(*food.pos(), FOOD_SIZE, pos, *ray_direction);

                        if let Some(intersection) = intersection {
                            if let Some(nearest) = nearest_foods[index] {
                                if intersection < nearest {
                                    nearest_foods[index] = Some(intersection)
                                }
                            } else {
                                nearest_foods[index] = Some(intersection)
                            }
                        }
                    }
                }
            });

            nearest_foods
                .iter()
                .map(|nearest_food| nearest_food.unwrap_or(-1.0))
                .enumerate()
                .for_each(|(index, value)| rays[index] = value);
        }

        timings.see_food.add(&instant.elapsed());
    }

    fn drop_of_food(ants: &mut Ants, timings: &mut Timings, stats: &mut Stats) {
        let instant = Instant::now();

        for (index, caries) in ants
            .caries_foods
            .iter_mut()
            .enumerate()
            .filter(|(_, carries)| **carries)
        {
            if ants.positions[index].length_squared() > ANT_HILL_RADIUS * ANT_HILL_RADIUS {
                continue;
            }

            stats.dropped_of_food += 1;
            *caries = false
        }

        timings.drop_of_food.add(&instant.elapsed());
    }
}
