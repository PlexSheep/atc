use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    usize,
};

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
    exits: HashMap<usize, HashSet<DirectionGrid>>,
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
            exits: HashMap::new(),
            x,
            y,
        }
    }

    pub fn place_exit(&mut self, dir: DirectionGrid, pos: usize) -> Result<&mut Self, String> {
        match dir {
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

        self.exits.entry(pos).or_default().insert(dir);

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

    fn get_wall(&self, pos: usize, dir: DirectionGrid) -> String {
        match self.exits.get(&pos) {
            Some(set) if set.contains(&dir) => {
                format!("e{pos}")
            }
            _ => match dir {
                DirectionGrid::Up => "──",
                DirectionGrid::Down => "──",
                DirectionGrid::Left => "│",
                DirectionGrid::Right => "│",
            }
            .to_string(),
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
        buf.push('┌');
        for x in 1..self.x + 1 {
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
        buf.push('└');
        for x in 1..self.x + 1 {
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
