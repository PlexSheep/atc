use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Plane {0} tried to go to a bad position")]
    PlaneNextPosBad(char),
    #[error("Exit position is out of bounds: not {0} < {1}")]
    ExitPosOutOfBounds(usize, usize),
    #[error("Position is out of bounds: not {0} < {1}")]
    PosOutOfBounds(usize, usize),
    #[error("No Exit exists for ID {0}")]
    NoExitForID(u8),
    #[error("Negative Positions are not allowed: {0:?}")]
    PosFromSigned((i32, i32)),
}
