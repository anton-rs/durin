//! The position module holds the types specific to the [crate::FaultDisputeGame] solver.

use crate::ChessClock;
use crate::Gindex;
use durin_primitives::Claim;
use std::sync::Arc;

pub type Position = u128;
pub type Clock = u128;

/// The [FaultSolverResponse] enum describes the response that a solver should return when asked to make a move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaultSolverResponse<T: AsRef<[u8]>> {
    /// A response indicating that the proper move is to attack or defend the given claim.
    Move(bool, usize, Claim),
    /// A response indicating that the proper move is to skip the given claim.
    Skip(usize),
    /// A response indicating that the proper move is to perform a VM step against
    /// the given claim.
    Step(bool, usize, Arc<T>, Arc<[u8]>),
}

/// The [VMStatus] enum describes the status of a VM at a given position.
/// - [VMStatus::Valid]: The VM is exited with a valid status.
/// - [VMStatus::Invalid]: The VM is exited with an invalid status.
/// - [VMStatus::Panic]: The VM is exited with a panic status.
/// - [VMStatus::Unfinished]: The VM is not yet exited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMStatus {
    Valid = 0,
    Invalid = 1,
    Panic = 2,
    Unfinished = 3,
}

/// Computes a generalized index from a depth and index at depth.
///
/// ### Takes
/// - `depth`: The depth of the generalized index.
/// - `index_at_depth`: The index at depth of the generalized index.
///
/// ### Returns
/// - `u128`: The generalized index: `2^{depth} + index_at_depth`.
pub fn compute_gindex(depth: u8, index_at_depth: u64) -> u128 {
    2u128.pow(depth as u32) + index_at_depth as u128
}

/// Implementation of the [Gindex] trait for the [Position] type alias.
impl Gindex for Position {
    fn depth(&self) -> u8 {
        127 - self.leading_zeros() as u8
    }

    fn index_at_depth(&self) -> u64 {
        (self - (1 << self.depth())) as u64
    }

    fn left(&self) -> Self {
        self << 1
    }

    fn right(&self) -> Self {
        self.left() | 1
    }

    fn parent(&self) -> Self {
        self >> 1
    }

    fn right_index(&self, max_depth: u8) -> Self {
        let remaining = max_depth - self.depth();
        (self << remaining) | ((1 << remaining) - 1)
    }

    fn trace_index(&self, max_depth: u8) -> u64 {
        self.right_index(max_depth).index_at_depth()
    }

    fn make_move(&self, is_attack: bool) -> Self {
        ((!is_attack as u128) | self) << 1
    }
}

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
    use super::{Gindex, Position};

    #[test]
    fn chess_clock_correctness() {
        let clock = 0xa5000000000000001;
        assert_eq!(clock.duration(), 10);
        assert_eq!(clock.timestamp(), 5764607523034234881);
    }

    /// A helper struct for testing the [Position] trait implementation for [std::u128].
    /// 0. `u64` - `depth`
    /// 1. `u64` - `index_at_depth`
    /// 2. `u128` - `right_index`
    /// 3. `u64` - `trace_index`
    struct PositionMetaData(u8, u64, u128, u64);

    const MAX_DEPTH: u8 = 4;
    const EXPECTED_VALUES: &[PositionMetaData] = &[
        PositionMetaData(0, 0, 31, 15),
        PositionMetaData(1, 0, 23, 7),
        PositionMetaData(1, 1, 31, 15),
        PositionMetaData(2, 0, 19, 3),
        PositionMetaData(2, 1, 23, 7),
        PositionMetaData(2, 2, 27, 11),
        PositionMetaData(2, 3, 31, 15),
        PositionMetaData(3, 0, 17, 1),
        PositionMetaData(3, 1, 19, 3),
        PositionMetaData(3, 2, 21, 5),
        PositionMetaData(3, 3, 23, 7),
        PositionMetaData(3, 4, 25, 9),
        PositionMetaData(3, 5, 27, 11),
        PositionMetaData(3, 6, 29, 13),
        PositionMetaData(3, 7, 31, 15),
        PositionMetaData(4, 0, 16, 0),
        PositionMetaData(4, 1, 17, 1),
        PositionMetaData(4, 2, 18, 2),
        PositionMetaData(4, 3, 19, 3),
        PositionMetaData(4, 4, 20, 4),
        PositionMetaData(4, 5, 21, 5),
        PositionMetaData(4, 6, 22, 6),
        PositionMetaData(4, 7, 23, 7),
        PositionMetaData(4, 8, 24, 8),
        PositionMetaData(4, 9, 25, 9),
        PositionMetaData(4, 10, 26, 10),
        PositionMetaData(4, 11, 27, 11),
        PositionMetaData(4, 12, 28, 12),
        PositionMetaData(4, 13, 29, 13),
        PositionMetaData(4, 14, 30, 14),
        PositionMetaData(4, 15, 31, 15),
    ];

    #[test]
    fn position_correctness_static() {
        for (p, v) in EXPECTED_VALUES.iter().enumerate() {
            let pos = (p + 1) as Position;
            assert_eq!(pos.depth(), v.0);
            assert_eq!(pos.index_at_depth(), v.1);
            let r = pos.right_index(MAX_DEPTH);
            assert_eq!(r, v.2);
            assert_eq!(r.index_at_depth(), v.3);
        }
    }
}
