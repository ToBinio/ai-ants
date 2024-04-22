use glam::{vec2, Vec2};

pub struct Ant {
    pos: Vec2,
    dir: Vec2,
}

impl Ant {
    pub fn random() -> Ant {
        //todo random
        Ant {
            pos: vec2(0.0, 0.0),
            dir: vec2(1.0, 0.0),
        }
    }

    pub fn pos(&self) -> &Vec2 {
        &self.pos
    }

    pub fn dir(&self) -> &Vec2 {
        &self.dir
    }
}
