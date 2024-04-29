use glam::Vec2;

pub struct Food {
    pos: Vec2,
}

impl Food {
    pub fn new(pos: Vec2) -> Food {
        Food { pos }
    }
    pub fn pos(&self) -> &Vec2 {
        &self.pos
    }
}
