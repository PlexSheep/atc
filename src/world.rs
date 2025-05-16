use std::{fmt::Display, ops::RangeBounds, usize};

use tracing::{debug, info, trace};

#[derive(Copy, Clone, Debug)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum PlaneKind {
    Small,
    Jet,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DirectionGrid {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum DirectionCardinal {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct World {
    x: usize,
    y: usize,
    tiles: Vec<Vec<WorldTile>>,
    planes: Vec<Plane>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorldTile {
    Empty,
    Route,
    Airport(DirectionGrid, u8),
    Beacon(u8),
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pos: Pos,
    height: u8,
    direction: DirectionCardinal,
    kind: PlaneKind,
    id: char,
}

impl Pos {
    pub fn sum(self) -> usize {
        self.x + self.y
    }
}

impl World {
    pub fn new(x: usize, y: usize) -> Self {
        World {
            tiles: vec![vec![WorldTile::Empty; x]; y],
            planes: Vec::new(),
            x,
            y,
        }
    }

    pub fn place_tile(&mut self, tile: WorldTile, pos: impl Into<Pos>) -> &mut Self {
        let pos: Pos = pos.into();
        self.tiles[pos.y][pos.x] = tile;
        self
    }

    fn check_pos_bounds(&self, pos: impl Into<Pos>) -> Result<(), String> {
        let pos = pos.into();
        if !pos.x < self.x {
            return Err(format!("x is out of bounds: {} < {}", pos.x, self.x));
        }
        if !pos.y < self.y {
            return Err(format!("y is out of bounds: {} < {}", pos.y, self.y));
        }
        Ok(())
    }

    /// Uses Bresenham's line algorithm to place route tiles between tiles
    pub fn place_route_in_line(
        &mut self,
        a: impl Into<Pos>,
        b: impl Into<Pos>,
    ) -> Result<&mut Self, String> {
        let mut a: Pos = a.into();
        let mut b: Pos = b.into();

        self.check_pos_bounds(a)?;
        self.check_pos_bounds(b)?;

        if a.sum() > b.sum() {
            std::mem::swap(&mut a, &mut b);
        }

        let dx: i32 = b.x as i32 - a.x as i32;
        let dy: i32 = b.y as i32 - a.y as i32;
        let mut d: i32 = 2 * dy - dx;
        let mut y = a.y;

        let range = if a.x < b.x { a.x..b.x } else { b.x..a.x };

        for x in range {
            self.place_tile(WorldTile::Route, [x, y]);
            if d > 0 {
                y += 1;
                d -= 2 * dx;
            }
            d += 2 * dy;
        }

        Ok(self)
    }
}

impl Display for DirectionGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Up => "^",
                Self::Left => "<",
                Self::Down => "v",
                Self::Right => ">",
            }
        )
    }
}

impl Display for WorldTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Empty => ". ".to_string(),
                Self::Route => "+ ".to_string(),
                Self::Beacon(idx) => format!("{idx}"),
                Self::Airport(dir, idx) => format!("{dir}{idx}"),
            }
        )
    }
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        for (y, row) in self.tiles.iter().enumerate() {
            for tile in row {
                buf.push_str(&tile.to_string());
            }
            if y % 2 == 0 {
                buf.push_str(&format!("{:}", y));
            }
            buf.push('\n');
        }
        for x in 0..self.x {
            if x % 2 == 0 {
                buf.push_str(&format!("{:02}", x));
            } else {
                buf.push_str("  ");
            }
        }
        write!(f, "{buf}")
    }
}

impl Display for Plane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self.kind {
            PlaneKind::Small => self.id.to_ascii_lowercase(),
            PlaneKind::Jet => self.id.to_ascii_uppercase(),
        };
        write!(f, "{id}{}", self.height)
    }
}

impl From<[usize; 2]> for Pos {
    fn from(value: [usize; 2]) -> Self {
        Pos {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<DirectionGrid> for DirectionCardinal {
    fn from(value: DirectionGrid) -> Self {
        match value {
            DirectionGrid::Up => DirectionCardinal::Up,
            DirectionGrid::Down => DirectionCardinal::Down,
            DirectionGrid::Left => DirectionCardinal::Left,
            DirectionGrid::Right => DirectionCardinal::Right,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::world::WorldTile;

    use super::World;

    #[test]
    #[should_panic]
    fn test_world_place_route_out_of_bounds() {
        let mut world = World::new(20, 20);
        world.place_route_in_line([0, 0], [21, 20]).unwrap();
        for i in 0..20 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }

        let mut world = World::new(20, 20);
        world.place_route_in_line([0, 0], [20, 21]).unwrap();
        for i in 0..20 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }
    }

    #[test]
    fn test_world_place_route_0() {
        let mut world = World::new(20, 20);
        world.place_route_in_line([0, 0], [20, 20]).unwrap();
        println!("{}", world);
        for i in 0..20 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }
    }

    #[test]
    fn test_world_place_route_1() {
        let mut world = World::new(20, 20);
        world.place_route_in_line([20, 20], [0, 0]).unwrap();
        println!("{}", world);
        for i in 0..20 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }
    }
}
