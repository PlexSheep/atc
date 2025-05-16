use crate::world::World;

use super::Level;

pub const X: usize = 20;
pub const Y: usize = 20;

impl Level {
    pub fn builtin() -> Self {
        let mut world = World::new(X, Y);
        // world.place_route_in_line([0, 0], [19, 19]);

        Level {
            world,
            name: "default".to_string(),
        }
    }
}
