use crate::world::World;

use super::Level;

pub const X: usize = 20;
pub const Y: usize = 20;

impl Level {
    pub fn builtin() -> Self {
        let mut world = World::new(X, Y);
        world.place_route_in_line([0, 0], [20, 10]).unwrap();
        world.place_route_in_line([2, 6], [14, 8]).unwrap();
        world.place_route_in_line([20, 20], [0, 0]).unwrap();

        Level {
            world,
            name: "default".to_string(),
        }
    }
}
