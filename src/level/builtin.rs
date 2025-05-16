use crate::world::{DirectionCardinal, DirectionGrid, World};

use super::Level;

pub const X: usize = 20;
pub const Y: usize = 20;

impl Level {
    pub fn builtin() -> Self {
        let mut world = World::new(X, Y);

        fn place_stuff(world: &mut World) -> Result<(), String> {
            world.place_route_in_line([19, 10], [0, 10])?;
            world.place_route_in_line([5, 0], [5, 19])?;
            world.place_route_in_line([12, 0], [12, 19])?;
            world.place_route_in_line([12, 10], [19, 3])?;

            world.place_tile(crate::world::WorldTile::Beacon(0), [12, 10])?;
            world.place_tile(
                crate::world::WorldTile::Airport(DirectionGrid::Right, 0),
                [5, 10],
            )?;

            world.place_exit(DirectionGrid::Up, DirectionCardinal::South, 12, 0)?;
            world.place_exit(DirectionGrid::Right, DirectionCardinal::SouthWest, 2, 1)?;
            world.place_exit(DirectionGrid::Right, DirectionCardinal::West, 10, 2)?;
            world.place_exit(DirectionGrid::Left, DirectionCardinal::East, 10, 3)?;
            world.place_exit(DirectionGrid::Down, DirectionCardinal::North, 12, 4)?;

            Ok(())
        };
        place_stuff(&mut world).expect("could not place tiles in world");

        Level {
            world,
            name: "default".to_string(),
        }
    }
}
