use crate::timings::avg_duration::AvgDuration;

pub mod avg_duration;

pub struct Timings {
    pub ant_updates: AvgDuration,
    pub keep_ants: AvgDuration,
    pub neural_network_updates: AvgDuration,
    pub pheromone_updates: AvgDuration,
    pub pheromone_spawn: AvgDuration,
    pub pheromone_remove: AvgDuration,
    pub pick_up_food: AvgDuration,
    pub drop_of_food: AvgDuration,
    pub see_food: AvgDuration,
}
