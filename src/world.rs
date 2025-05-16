use std::{collections::HashMap, fmt::Display};

use tracing::{debug, trace};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DirectionGrid {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum DirectionCardinal {
    North,
    East,
    South,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Debug)]
pub struct World {
    x: usize,
    y: usize,
    tiles: Vec<Vec<WorldTile>>,
    planes: Vec<Plane>,
    //              Direction      WallPos Index   DirectionToFlyIn
    exits: HashMap<(DirectionGrid, usize), (usize, DirectionCardinal)>,
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

impl World {
    pub fn new(x: usize, y: usize) -> Self {
        World {
            tiles: vec![vec![WorldTile::Empty; x]; y],
            planes: Vec::new(),
            exits: HashMap::new(),
            x,
            y,
        }
    }

    pub fn place_exit(
        &mut self,
        where_on_wall: DirectionGrid,
        plane_out_direction: DirectionCardinal,
        pos: usize,
        idx: usize,
    ) -> Result<&mut Self, String> {
        match where_on_wall {
            DirectionGrid::Up | DirectionGrid::Down => {
                if !pos < self.y {
                    return Err(format!("exit pos is out of bounds: {} < {}", pos, self.y));
                }
            }
            DirectionGrid::Left | DirectionGrid::Right => {
                if !pos < self.x {
                    return Err(format!("exit pos is out of bounds: {} < {}", pos, self.x));
                }
            }
        }

        self.exits
            .insert((where_on_wall, pos), (idx, plane_out_direction));

        Ok(self)
    }

    pub fn place_tile(
        &mut self,
        tile: WorldTile,
        pos: impl Into<Pos>,
    ) -> Result<&mut Self, String> {
        let pos: Pos = pos.into();
        self.check_pos_bounds(pos)?;
        self.tiles[pos.y][pos.x] = tile;
        Ok(self)
    }

    fn check_pos_bounds(&self, pos: impl Into<Pos>) -> Result<(), String> {
        let pos = pos.into();
        trace!("check if pos in bounds: {pos:?}");
        if pos.x + 1 > self.x {
            return Err(format!("x is out of bounds: {} < {}", pos.x, self.x));
        }
        if pos.y + 1 > self.y {
            return Err(format!("y is out of bounds: {} < {}", pos.y, self.y));
        }
        trace!("pos in is bounds: {pos:?}");
        Ok(())
    }

    /// Uses Bresenham's line algorithm to place route tiles between tiles
    pub fn place_route_in_line(
        &mut self,
        a: impl Into<Pos>,
        b: impl Into<Pos>,
    ) -> Result<&mut Self, String> {
        let a: Pos = a.into();
        let b: Pos = b.into();

        trace!("pos a: {a:?}");
        trace!("pos b: {a:?}");
        self.check_pos_bounds(a)?;
        self.check_pos_bounds(b)?;

        let dx: i32 = b.x as i32 - a.x as i32;
        let dy: i32 = b.y as i32 - a.y as i32;

        let sx: i32 = if dx > 0 { 1 } else { -1 };
        let sy: i32 = if dy > 0 { 1 } else { -1 };

        let mut dx: usize = dx.unsigned_abs() as usize;
        let mut dy: usize = dy.unsigned_abs() as usize;

        let xx;
        let xy;
        let yx;
        let yy;
        if dx > dy {
            (xx, xy, yx, yy) = (sx, 0, 0, sy);
        } else {
            std::mem::swap(&mut dx, &mut dy);
            (xx, xy, yx, yy) = (0, sy, sx, 0);
        }

        let mut d: i32 = 2 * dy as i32 - dx as i32;
        let mut y = 0;
        let mut pos: Pos;

        for x in 0..dx + 1 {
            pos = (
                a.x as i32 + x as i32 * xx + y * yx,
                a.y as i32 + x as i32 * xy + y * yy,
            )
                .try_into()?;
            self.place_tile(WorldTile::Route, pos)?;
            if d >= 0 {
                y += 1;
                d -= 2 * dx as i32;
            }
            d += 2 * dy as i32;
        }

        Ok(self)
    }

    fn get_wall(&self, pos: usize, dir: DirectionGrid) -> String {
        match self.exits.get(&(dir, pos)) {
            Some((exit_idx, _plane_out_dir)) => match dir {
                DirectionGrid::Up | DirectionGrid::Down => format!("{exit_idx}─"),
                DirectionGrid::Left | DirectionGrid::Right => format!("{exit_idx} "),
            },
            None => match dir {
                DirectionGrid::Up => "──",
                DirectionGrid::Down => "──",
                DirectionGrid::Left => "│ ",
                DirectionGrid::Right => "│ ",
            }
            .to_string(),
        }
    }
}

impl DirectionCardinal {
    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::NorthEast => Self::SouthWest,
            Self::NorthWest => Self::SouthEast,
            Self::SouthEast => Self::NorthWest,
            Self::SouthWest => Self::NorthEast,
        }
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
                Self::Beacon(idx) => format!("b{idx}"),
                Self::Airport(dir, idx) => format!("{dir}{idx}"),
            }
        )
    }
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();

        let mut lines = Vec::new();

        // top border
        buf.push_str("┌─");
        for x in 0..self.x {
            buf.push_str(&self.get_wall(x, DirectionGrid::Up));
        }
        buf.push('┐');
        lines.push(buf.clone());
        buf.clear();

        // inner map
        for (y, row) in self.tiles.iter().enumerate() {
            buf.push_str(&self.get_wall(y, DirectionGrid::Left));
            for tile in row {
                buf.push_str(&tile.to_string());
            }
            buf.push_str(&self.get_wall(y, DirectionGrid::Right));
            lines.push(buf.clone());
            buf.clear();
        }

        // top border
        buf.push_str("└─");
        for x in 0..self.x {
            buf.push_str(&self.get_wall(x, DirectionGrid::Down));
        }
        buf.push('┘');
        lines.push(buf.clone());
        buf.clear();

        // actual output
        buf = lines.join("\n");

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
            DirectionGrid::Up => DirectionCardinal::North,
            DirectionGrid::Down => DirectionCardinal::South,
            DirectionGrid::Left => DirectionCardinal::West,
            DirectionGrid::Right => DirectionCardinal::East,
        }
    }
}

impl TryFrom<(i32, i32)> for Pos {
    type Error = String;

    fn try_from(value: (i32, i32)) -> Result<Self, Self::Error> {
        if value.0 >= 0 && value.1 >= 0 {
            Ok(Pos {
                x: value.0 as usize,
                y: value.1 as usize,
            })
        } else {
            Err("Negative Positions are not allowed".to_string())
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
        world.place_route_in_line([0, 0], [20, 19]).unwrap();
        for i in 0..19 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }

        let mut world = World::new(20, 20);
        world.place_route_in_line([0, 0], [19, 20]).unwrap();
        for i in 0..19 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }
    }

    #[test]
    fn test_world_place_route_0() {
        let mut world = World::new(20, 20);
        world.place_route_in_line([0, 0], [19, 19]).unwrap();
        println!("{}", world);
        for i in 0..19 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }
    }

    #[test]
    fn test_world_place_route_1() {
        let mut world = World::new(20, 20);
        world.place_route_in_line([19, 19], [0, 0]).unwrap();
        println!("{}", world);
        for i in 0..19 {
            assert_eq!(world.tiles[i][i], WorldTile::Route);
        }
    }

    #[test]
    fn test_world_place_route_2() {
        let mut world = World::new(20, 20);
        world.place_route_in_line([19, 0], [0, 19]).unwrap();
        println!("{}", world);
        for i in 0..19 {
            assert_eq!(world.tiles[19 - i][i], WorldTile::Route);
        }
    }
}
