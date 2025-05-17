use crate::{
    error::Error,
    world::{DirectionCardinal, PlaneKind, Pos},
};

pub const START_HEIGHT: u8 = 7;
pub const EXIT_HEIGHT: u8 = 9;

#[derive(Copy, Clone, Debug)]
pub enum Destination {
    Exit(u8),
    Airport(u8),
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pub pos: Pos,
    pub height: u8,
    pub direction: DirectionCardinal,
    pub kind: PlaneKind,
    pub id: char,
    pub ticks: usize,
    pub destination: Destination,
    pub just_spawned: bool,
}

impl Plane {
    pub fn new(
        pos: Pos,
        direction: DirectionCardinal,
        kind: PlaneKind,
        id: char,
        destination: Destination,
    ) -> Self {
        Self {
            pos,
            height: START_HEIGHT,
            direction,
            kind,
            id: match kind {
                PlaneKind::Small => id.to_ascii_uppercase(),
                PlaneKind::Jet => id.to_ascii_lowercase(),
            },
            ticks: 0,
            destination,
            just_spawned: true,
        }
    }

    /// Err if no fiel left on plane
    pub fn tick(&mut self) -> Result<(), ()> {
        self.ticks += 1;

        if self.out_of_fuel() {
            return Err(());
        }

        if self.moves_this_tick() {
            self.next_pos();
        }

        if self.ticks == 2 {
            self.just_spawned = false;
        }

        Ok(())
    }
    fn out_of_fuel(&self) -> bool {
        self.ticks
            >= match self.kind {
                PlaneKind::Jet => 120,
                PlaneKind::Small => 50,
            }
    }
    fn next_pos(&mut self) -> Result<(), Error> {
        fn do_stuff(p: &mut Plane) -> Option<()> {
            match p.direction {
                DirectionCardinal::North => p.pos.y = p.pos.y.checked_sub(1)?,
                DirectionCardinal::NorthEast => {
                    p.pos.y = p.pos.y.checked_sub(1)?;
                    p.pos.x = p.pos.x.checked_add(1)?;
                }
                DirectionCardinal::NorthWest => {
                    p.pos.y = p.pos.y.checked_sub(1)?;
                    p.pos.x = p.pos.x.checked_sub(1)?;
                }
                DirectionCardinal::South => p.pos.y = p.pos.y.checked_add(1)?,
                DirectionCardinal::SouthEast => {
                    p.pos.y = p.pos.y.checked_add(1)?;
                    p.pos.x = p.pos.x.checked_add(1)?;
                }
                DirectionCardinal::SouthWest => {
                    p.pos.y = p.pos.y.checked_add(1)?;
                    p.pos.x = p.pos.x.checked_sub(1)?;
                }
                DirectionCardinal::West => p.pos.x = p.pos.x.checked_sub(1)?,
                DirectionCardinal::East => p.pos.x = p.pos.x.checked_add(1)?,
            };

            Some(())
        }
        if do_stuff(self).is_none() {
            Err(Error::PlaneNextPosBad(self.id))
        } else {
            Ok(())
        }
    }

    fn moves_this_tick(&self) -> bool {
        self.ticks
            % match self.kind {
                PlaneKind::Jet => 1,
                PlaneKind::Small => 2,
            }
            == 0
    }
}
