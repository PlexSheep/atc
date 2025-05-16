use std::fmt::Display;

use crate::world::World;

pub mod builtin;

#[derive(Debug)]
pub struct Level {
    name: String,
    world: World,
}

impl Level {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    pub fn tick(&mut self) {
        // todo!()
    }
    pub fn render(&self) -> String {
        self.world.to_string()
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.world.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_level_render_default() {
        let level = Level::builtin();
        let rendered = level.render();
        assert!(rendered.contains("+ "));
        assert!(rendered.contains(". "));
        assert!(rendered.contains("e0"));
        assert!(rendered.contains("e1"));
        assert!(rendered.contains("b0"));
    }
}
