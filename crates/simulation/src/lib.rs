use crate::ant::{Ant, ANT_PICK_UP_DISTANCE};
use std::f32::consts::PI;

use crate::food::Food;
use crate::grid::Grid;
use crate::timings::Timings;
use ant::{ANT_RAY_COUNT, ANT_SEE_DISTANCE};
use glam::{vec2, Vec2};
use itertools::Itertools;
use math::ray_inserect_circle;
use neural_network::NeuralNetwork;
use rand::{thread_rng, Rng};
use std::time::Instant;

pub mod ant;
mod food;
mod grid;
mod math;
pub mod timings;

const TICKS_UNTIL_PHEROMONE: usize = 10;
pub const ANT_HILL_RADIUS: f32 = 50.;
pub const GAME_SIZE: f32 = 500.;
pub const FOOD_SIZE: f32 = 7.;

pub const NEURAL_NETWORK_INPUT_SIZE: usize = 5 + ANT_RAY_COUNT;
pub const NEURAL_NETWORK_OUTPUT_SIZE: usize = 4;

pub struct Simulation {
    ants: Vec<Ant>,

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

        let mut ants = vec![];

        const ANTS_TO_SPAWN: usize = 200;
        const ANGLE_PER_ANT: f32 = PI * 2. / ANTS_TO_SPAWN as f32;

        for i in 0..ANTS_TO_SPAWN {
            ants.push(Ant::from_direction(ANGLE_PER_ANT * i as f32));
        }

        let mut foods = Grid::new(25, GAME_SIZE);
        let mut rng = thread_rng();

        for _ in 0..500 {
            let pos = vec2(rng.gen_range(300.0..400.0), rng.gen_range(300.0..400.0));

            foods.insert(&pos, Food::new(pos));
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

    pub fn ants(&self) -> &Vec<Ant> {
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

        Simulation::update_pheromones(&mut self.pheromones, self.ants.len(), &mut self.timings);
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
        for ant in ants {
            let values = neural_network.run(ant.get_neural_network_values());
            ant.set_neural_network_values(values);
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

    fn update_ants(ants: &mut Vec<Ant>, timings: &mut Timings) {
        let instant = Instant::now();
        for ant in ants {
            ant.step()
        }
        timings.ant_updates.add(&instant.elapsed());
    }

    fn keep_ants(ants: &mut Vec<Ant>, timings: &mut Timings) {
        let instant = Instant::now();
        for ant in ants {
            if ant.pos().x > GAME_SIZE {
                ant.pos_mut().x -= 10.;
                ant.flip()
            }

            if ant.pos().x < -GAME_SIZE {
                ant.pos_mut().x += 10.;
                ant.flip()
            }

            if ant.pos().y > GAME_SIZE {
                ant.pos_mut().y -= 10.;
                ant.flip()
            }

            if ant.pos().y < -GAME_SIZE {
                ant.pos_mut().y += 10.;
                ant.flip()
            }
        }
        timings.keep_ants.add(&instant.elapsed());
    }

    fn spawn_pheromones(pheromones: &mut Pheromones, ants: &Vec<Ant>, timings: &mut Timings) {
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

            let offset = ants.len() * to_be_replaced;

            for (index, ant) in ants.iter().enumerate() {
                pheromones.grid.insert(ant.pos(), index);
                pheromones.positions[index + offset] = *ant.pos();
                pheromones.colors[index + offset] = ant.pheromone_color();
            }
        } else {
            pheromones.sizes.push(Some(1.));

            for ant in ants {
                let index = pheromones.positions.len();

                pheromones.grid.insert(ant.pos(), index);
                pheromones.positions.push(*ant.pos());
                pheromones.colors.push(ant.pheromone_color());
            }
        }

        timings.pheromone_spawn.add(&instant.elapsed());
    }

    fn pick_up_food(
        ants: &mut [Ant],
        foods: &mut Grid<Food>,
        timings: &mut Timings,
        stats: &mut Stats,
    ) {
        let instant = Instant::now();

        for ant in ants.iter_mut().filter(|ant| !ant.carries_food()) {
            foods.for_each(*ant.pos(), ANT_PICK_UP_DISTANCE, |foods| {
                let mut picked_up_food = None;

                for (index, food) in foods.iter().enumerate() {
                    let distance = vec2(food.pos().x - ant.pos().x, food.pos().y - ant.pos().y)
                        .length_squared();
                    if distance < ANT_PICK_UP_DISTANCE * ANT_PICK_UP_DISTANCE {
                        stats.picked_up_food += 1;
                        picked_up_food = Some(index);
                        break;
                    }
                }

                if let Some(index) = picked_up_food {
                    ant.set_carries_food(true);
                    foods.remove(index);
                }
            });
        }

        timings.pick_up_food.add(&instant.elapsed());
    }

    fn see_food(ants: &mut [Ant], foods: &mut Grid<Food>, timings: &mut Timings) {
        let instant = Instant::now();

        for ant in ants {
            let ray_values = ant
                .get_ray_directions()
                .into_iter()
                .map(|ray_direction| {
                    let mut nearest_food = None;

                    foods.for_each(*ant.pos(), ANT_SEE_DISTANCE, |foods| {
                        for food in foods {
                            let distance = food.pos().distance_squared(*ant.pos());

                            if distance > ANT_SEE_DISTANCE {
                                continue;
                            }

                            if let Some(nearest) = nearest_food {
                                if nearest < distance {
                                    continue;
                                }
                            }

                            let intersection = ray_inserect_circle(
                                *food.pos(),
                                FOOD_SIZE,
                                *ant.pos(),
                                ray_direction,
                            );

                            if let Some(intersection) = intersection {
                                if let Some(nearest) = nearest_food {
                                    if intersection < nearest {
                                        nearest_food = Some(intersection)
                                    }
                                } else {
                                    nearest_food = Some(intersection)
                                }
                            }
                        }
                    });

                    nearest_food.unwrap_or(-1.0)
                })
                .collect_vec();

            ant.set_rays(ray_values);
        }

        timings.see_food.add(&instant.elapsed());
    }

    fn drop_of_food(ants: &mut [Ant], timings: &mut Timings, stats: &mut Stats) {
        let instant = Instant::now();

        ants.iter_mut()
            .filter(|ant| ant.carries_food())
            .filter(|ant| ant.pos().length_squared() < ANT_HILL_RADIUS * ANT_HILL_RADIUS)
            .for_each(|ant| {
                stats.dropped_of_food += 1;
                ant.set_carries_food(false)
            });

        timings.drop_of_food.add(&instant.elapsed());
    }
}
