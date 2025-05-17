use std::{collections::HashMap, fmt::Display};

use tracing::debug;

use crate::{
    error::Error,
    plane::{Destination, Plane},
};

#[derive(Copy, Clone, Debug)]
pub enum State {
    Onging,
    PlaneCollision(Plane, Plane),
    WrongExit(Plane, u8),
    WrongAirport(Plane, u8),
    PlaneTouchesWall(Plane, DirectionGrid, usize),
    PlaneCrash(Plane),
    PlaneNoFuel(Plane),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    planes: HashMap<char, Plane>,
    exits: HashMap<u8, Exit>,
    plane_counter: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct Exit {
    pub wall_direction: DirectionGrid,
    pub plane_out_direction: DirectionCardinal,
    pub wall_pos: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorldTile {
    Empty,
    Route,
    Airport(DirectionGrid, u8),
    Beacon(u8),
}

impl World {
    pub fn new(x: usize, y: usize) -> Self {
        World {
            tiles: vec![vec![WorldTile::Empty; x]; y],
            planes: HashMap::new(),
            exits: HashMap::new(),
            x,
            y,
            plane_counter: 0,
        }
    }

    pub fn place_exit(
        &mut self,
        where_on_wall: DirectionGrid,
        plane_out_direction: DirectionCardinal,
        wall_pos: usize,
        idx: u8,
    ) -> Result<&mut Self, Error> {
        match where_on_wall {
            DirectionGrid::Up | DirectionGrid::Down => {
                if !wall_pos < self.y {
                    return Err(Error::ExitPosOutOfBounds(wall_pos, self.y));
                }
            }
            DirectionGrid::Left | DirectionGrid::Right => {
                if !wall_pos < self.x {
                    return Err(Error::ExitPosOutOfBounds(wall_pos, self.x));
                }
            }
        }

        let exit = Exit {
            wall_direction: where_on_wall,
            plane_out_direction,
            wall_pos,
        };

        self.exits.insert(idx, exit);

        Ok(self)
    }

    pub fn place_tile(&mut self, tile: WorldTile, pos: impl Into<Pos>) -> Result<&mut Self, Error> {
        let pos: Pos = pos.into();
        self.check_pos_bounds(pos)?;
        self.tiles[pos.y][pos.x] = tile;
        Ok(self)
    }

    fn check_pos_bounds(&self, pos: impl Into<Pos>) -> Result<(), Error> {
        let pos = pos.into();
        if pos.x + 1 > self.x {
            return Err(Error::PosOutOfBounds(pos.x, self.x));
        }
        if pos.y + 1 > self.y {
            return Err(Error::PosOutOfBounds(pos.y, self.y));
        }
        Ok(())
    }

    /// Uses Bresenham's line algorithm to place route tiles between tiles
    pub fn place_route_in_line(
        &mut self,
        a: impl Into<Pos>,
        b: impl Into<Pos>,
    ) -> Result<&mut Self, Error> {
        let a: Pos = a.into();
        let b: Pos = b.into();

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
        let mut maybe_exit_idx = None;
        for (idx, exit) in &self.exits {
            if exit.wall_pos == pos && exit.wall_direction == dir {
                maybe_exit_idx = Some(idx)
            }
        }

        match maybe_exit_idx {
            Some(idx) => match dir {
                DirectionGrid::Up | DirectionGrid::Down => format!("{idx}─"),
                DirectionGrid::Left | DirectionGrid::Right => format!("{idx} "),
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

    fn next_plane_idx(&mut self) -> char {
        const ORDER: [char; 25] = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'p', 'q', 'r',
            's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        let out = ORDER[self.plane_counter as usize % 25];
        self.plane_counter += 1;
        out
    }

    pub fn spawn_plane_at_exit(&mut self, exit_id: u8, kind: PlaneKind) -> Result<(), Error> {
        let exit = match self.exits.get(&exit_id) {
            Some(e) => *e,
            None => return Err(Error::NoExitForID(exit_id)),
        };
        let pos = match exit.plane_out_direction {
            DirectionCardinal::North => [exit.wall_pos, 0].into(),
            // DirectionCardinal::NorthEast => [(exit.wall_pos -1).clamp(0, self.x), 0].into(),
            // DirectionCardinal::NorthWest => [(exit.wall_pos +1).clamp(0, self.x), 0].into(),
            DirectionCardinal::South => [exit.wall_pos, self.y - 1].into(),
            // DirectionCardinal::SouthEast => [(exit.wall_pos -1).clamp(0, self.y), 0].into(),
            // DirectionCardinal::SouthWest => [(exit.wall_pos +1).clamp(0, self.y), 0].into(),
            DirectionCardinal::West => [0, exit.wall_pos].into(),
            DirectionCardinal::East => [self.x - 1, exit.wall_pos].into(),
            _ => todo!(),
        };
        let id: char = self.next_plane_idx();
        let plane = Plane::new(
            pos,
            exit.plane_out_direction.opposite(),
            kind,
            id,
            Destination::Exit(1),
        );
        self.planes.insert(id, plane);
        Ok(())
    }

    fn collision_check(&self) -> Option<(Plane, Plane)> {
        None // TODO: add collision
    }

    fn wall_collision_check(&self) -> Option<(Plane, DirectionGrid, usize)> {
        None // TODO: add collision
    }

    fn plane_exit_check_inner(
        &mut self,
        plane: &Plane,
        wall_dir: DirectionGrid,
        plane_pos: usize,
    ) -> Option<(Plane, u8)> {
        for (eid, exit) in self
            .exits
            .iter()
            .filter(|(_id, e)| e.wall_direction == wall_dir)
        {
            if exit.wall_pos == plane_pos {
                // plane takes this exit
                if matches!(plane.destination, Destination::Exit(dest_eid) if dest_eid == *eid) {
                    // right exit
                    self.planes.remove(&plane.id);
                } else {
                    // wrong exit
                    return Some((*plane, *eid));
                }
            }
        }
        None
    }

    /// Removes planes that exit and returns Some if a plane took the wrong exit
    ///
    /// None if everything is ok, some only if a plane took the wrong exit
    fn planes_take_exits(&mut self) -> Option<(Plane, u8)> {
        // TODO: add height check
        for (pid, plane) in self.planes.clone() {
            if plane.just_spawned {
                debug!("Plane {pid} is too new, skipping for exit check");
                continue;
            }
            if plane.pos.y == 0 {
                if let Some(v) = self.plane_exit_check_inner(&plane, DirectionGrid::Up, plane.pos.x)
                {
                    return Some(v);
                }
            }
            if plane.pos.y == self.y {
                if let Some(v) =
                    self.plane_exit_check_inner(&plane, DirectionGrid::Down, plane.pos.x)
                {
                    return Some(v);
                }
            }
            if plane.pos.x == 0 {
                if let Some(v) =
                    self.plane_exit_check_inner(&plane, DirectionGrid::Left, plane.pos.y)
                {
                    return Some(v);
                }
            }
            if plane.pos.x == self.x {
                if let Some(v) =
                    self.plane_exit_check_inner(&plane, DirectionGrid::Right, plane.pos.y)
                {
                    return Some(v);
                }
            }
        }
        None
    }

    /// Removes planes that exit and returns Some if a plane took the wrong exit
    ///
    /// Returns:
    ///
    /// - None: Everything is okay. Maybe a plane landed at the correct airport and was removed
    /// - Some(Plane, None): A plane crashed on the ground (height 0)
    /// - Some(Plane, Some(airport_id)): A plane landed in the wrong airport
    fn planes_land(&mut self) -> Option<(Plane, Option<u8>)> {
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, airport) in row
                .iter()
                .enumerate()
                .filter(|(_, tile)| matches!(tile, WorldTile::Airport(_, _)))
            {
                for (pid, plane) in self
                    .planes
                    .clone()
                    .iter()
                    .filter(|(_, plane)| plane.height == 0)
                {
                    if plane.pos == [x, y].into() {
                        // plane lands at this airport
                        if let Destination::Airport(dest_aid) = plane.destination {
                            match airport {
                                WorldTile::Airport(airdir, actual_aid) => {
                                    if Into::<DirectionCardinal>::into(*airdir) != plane.direction {
                                        panic!("Plane landed in the wrong direction");
                                    }
                                    if dest_aid != *actual_aid {
                                        // right airport, right direction
                                        self.planes.remove(pid);
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            panic!("Landed at airport but should have used an exit")
                        }
                    }
                    // TODO: detect crashing plane
                }
            }
        }
        None
    }

    pub fn tick_planes(&mut self) -> State {
        for plane in self.planes.values_mut() {
            if let Err(()) = plane.tick() {
                return State::PlaneNoFuel(*plane);
            }
        }

        if let Some((plane, exit_id)) = self.planes_take_exits() {
            return State::WrongExit(plane, exit_id);
        }
        if let Some((plane, id_of_wrong_airport)) = self.planes_land() {
            if let Some(airport_id) = id_of_wrong_airport {
                return State::WrongAirport(plane, airport_id);
            } else {
                return State::PlaneCrash(plane);
            }
        }
        if let Some((plane_a, plane_b)) = self.collision_check() {
            return State::PlaneCollision(plane_a, plane_b);
        }
        if let Some((plane, direction, wall_pos)) = self.wall_collision_check() {
            return State::PlaneTouchesWall(plane, direction, wall_pos);
        }

        State::Onging
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
            'tile: for (x, tile) in row.iter().enumerate() {
                for plane in self.planes.values() {
                    if plane.pos == [x, y].into() {
                        buf.push_str(&plane.to_string());
                        continue 'tile;
                    }
                }
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
    type Error = Error;

    fn try_from(value: (i32, i32)) -> Result<Self, Self::Error> {
        if value.0 >= 0 && value.1 >= 0 {
            Ok(Pos {
                x: value.0 as usize,
                y: value.1 as usize,
            })
        } else {
            Err(Error::PosFromSigned(value))
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Onging => unreachable!(),
                Self::WrongExit(plane, eid) =>
                    format!("Plane {} exited at the wrong exit: {eid}", plane.id),
                Self::PlaneCrash(plane) =>
                    format!("Plane {} crashed on the ground (height 0)", plane.id),
                Self::PlaneNoFuel(plane) => format!("Plane {} is out of fuel", plane.id),
                Self::WrongAirport(plane, aid) =>
                    format!("Plane {} landed at the wrong airport: {aid}", plane.id),
                Self::PlaneCollision(pa, pb) =>
                    format!("Plane {} collided with Plane {}", pa.id, pb.id),
                Self::PlaneTouchesWall(plane, _, _) =>
                    format!("Plane {} did not leave through an exit", plane.id),
            }
        )
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
