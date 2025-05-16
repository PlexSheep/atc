use std::{fmt::Display, usize};

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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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
            x,
            y,
        }
    }

    pub fn place_tile(&mut self, tile: WorldTile, pos: impl Into<Pos>) -> &mut Self {
        let pos: Pos = pos.into();
        self.tiles[pos.y][pos.x] = tile;
        self
    }

    /// Uses Bresenham's line algorithm to place route tiles between tiles
    pub fn place_route_in_line(&mut self, a: impl Into<Pos>, b: impl Into<Pos>) -> &mut Self {
        let a: Pos = a.into();
        let b: Pos = b.into();

        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let mut d = 2 * dy - dx;
        let mut y = a.y;

        for x in a.x..b.x {
            self.place_tile(WorldTile::Route, [x, y]);
            if d > 0 {
                y += 1;
                d -= 2 * dx;
            }
            d += 2 * dy;
        }
        self
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
                Self::Empty => format!(". "),
                Self::Route => format!("+ "),
                Self::Beacon(idx) => format!("{idx}"),
                Self::Airport(dir, idx) => format!("{dir}{idx}"),
            }
        )
    }
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        for row in &self.tiles {
            for tile in row {
                buf.push_str(&tile.to_string());
            }
            buf.push('\n');
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
