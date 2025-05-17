use crate::world::{DirectionCardinal, PlaneKind, Pos};

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
}

impl Plane {
    /// Err if no fiel left on plane
    pub fn tick(&mut self) -> Result<(), ()> {
        self.ticks += 1;

        if self.out_of_fuel() {
            return Err(());
        }

        if self.moves_this_tick() {
            self.next_pos();
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
    fn next_pos(&mut self) {
        match self.direction.clone() {
            DirectionCardinal::North => self.pos.y += 1,
            DirectionCardinal::NorthEast => {
                self.pos.y += 1;
                self.pos.x += 1
            }
            DirectionCardinal::NorthWest => {
                self.pos.y += 1;
                self.pos.x -= 1
            }
            DirectionCardinal::South => self.pos.y -= 1,
            DirectionCardinal::SouthEast => {
                self.pos.y -= 1;
                self.pos.x += 1
            }
            DirectionCardinal::SouthWest => {
                self.pos.y -= 1;
                self.pos.x -= 1
            }
            DirectionCardinal::West => self.pos.x += 1,
            DirectionCardinal::East => self.pos.x -= 1,
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
