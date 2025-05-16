use crate::world::{DirectionGrid, World};

use super::Level;

pub const X: usize = 20;
pub const Y: usize = 20;

impl Level {
    pub fn builtin() -> Self {
        let mut world = World::new(X, Y);

        fn place_stuff(world: &mut World) -> Result<(), String> {
            world.place_route_in_line([19, 10], [0, 10])?;
            world.place_route_in_line([5, 0], [5, 19])?;
            world.place_route_in_line([10, 0], [10, 19])?;

            world.place_tile(crate::world::WorldTile::Beacon(0), [10, 10])?;
            world.place_tile(
                crate::world::WorldTile::Airport(DirectionGrid::Left, 0),
                [5, 10],
            )?;

            world.place_exit(DirectionGrid::Up, 10, 0)?;
            world.place_exit(DirectionGrid::Down, 10, 1)?;

            Ok(())
        };
        place_stuff(&mut world).expect("could not place tiles in world");

        Level {
            world,
            name: "default".to_string(),
        }
    }
}
