use crate::math::circle_intersects_rect;
use glam::{vec2, Vec2};
use itertools::Itertools;

pub struct Grid<T> {
    data: Vec<Vec<T>>,
    size: usize,
    width: f32,
}

impl<T> Grid<T> {
    pub fn new(size: usize, half_width: f32) -> Grid<T> {
        let mut data = vec![];

        for _ in 0..(size * size) {
            data.push(vec![]);
        }

        Grid {
            data,
            size,
            width: half_width,
        }
    }

    pub fn for_each_all<F>(&mut self, f: F)
    where
        Self: Sized,
        F: Fn(&mut T),
    {
        self.data
            .iter_mut()
            .for_each(|data| data.iter_mut().for_each(|item| f(item)));
    }

    pub fn for_each<F>(&mut self, pos: Vec2, radius: f32, mut f: F)
    where
        Self: Sized,
        F: FnMut(&mut Vec<T>),
    {
        let width_per_tile = (self.width * 2.) / self.size as f32;

        let min_x =
            (((pos.x - radius / 2.0 + self.width) / width_per_tile).floor() as usize).max(0);
        let max_x = (((pos.x + radius / 2.0 + self.width) / width_per_tile).ceil() as usize)
            .min(self.size - 1);

        let min_y =
            (((pos.y - radius / 2.0 + self.width) / width_per_tile).floor() as usize).max(0);
        let max_y = (((pos.y + radius / 2.0 + self.width) / width_per_tile).ceil() as usize)
            .min(self.size - 1);

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let tile_x = x as f32 * width_per_tile + width_per_tile / 2.0 - self.width;
                let tile_y = y as f32 * width_per_tile + width_per_tile / 2.0 - self.width;

                let data = self.get_from_index_mut((x, y));

                if data.len() == 0 {
                    continue;
                }

                if circle_intersects_rect(pos, radius, vec2(tile_x, tile_y), width_per_tile) {
                    f(data);
                }
            }
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        Self: Sized,
        F: Fn(&T) -> bool,
    {
        self.data
            .iter_mut()
            .for_each(|data| data.retain(|item| f(item)));
    }

    pub fn all(&self) -> Vec<&T> {
        self.data.iter().flatten().collect_vec()
    }

    pub fn insert(&mut self, pos: &Vec2, val: T) {
        let (x, y) = self.indexes_from_pos(pos);

        self.data.get_mut(y * self.size + x).unwrap().push(val);
    }

    #[inline]
    pub fn get_from_index_mut(&mut self, indexes: (usize, usize)) -> &mut Vec<T> {
        &mut self.data[indexes.1 * self.size + indexes.0]
    }

    pub fn indexes_from_pos(&self, pos: &Vec2) -> (usize, usize) {
        let width_per_tile = (self.width * 2.) / self.size as f32;

        let x = ((pos.x + self.width) / width_per_tile).floor() as usize;
        let y = ((pos.y + self.width) / width_per_tile).floor() as usize;

        (x.max(0).min(self.size - 1), y.max(0).min(self.size - 1))
    }
}
