use glam::Vec2;

pub struct Grid<T> {
    data: Vec<Vec<T>>,
    size: usize,
}

impl<T> Grid<T> {
    pub fn new(size: usize) -> Grid<T> {
        let mut data = vec![];

        for _ in 0..(size * size) {
            data.push(vec![]);
        }

        Grid { data, size }
    }

    pub fn insert(&mut self, pos: Vec2, val: T) {}
}
