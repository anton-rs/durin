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
    fn test_chess_clock() {
        let clock = 0xa0000000000000001;
        assert_eq!(clock.duration(), 10);
        assert_eq!(clock.timestamp(), 1);
    }
}
