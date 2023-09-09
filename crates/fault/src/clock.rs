//! The position module holds the [Clock] type.

use crate::ChessClock;

pub type Clock = u128;

impl ChessClock for Clock {
    fn duration(&self) -> u64 {
        (self >> 64) as u64
    }

    fn timestamp(&self) -> u64 {
        (self & u64::MAX as u128) as u64
    }
}

#[cfg(test)]
mod test {
    use super::ChessClock;

    #[test]
    fn chess_clock_correctness() {
        let clock = 0xa5000000000000001;
        assert_eq!(clock.duration(), 10);
        assert_eq!(clock.timestamp(), 5764607523034234881);
    }
}
