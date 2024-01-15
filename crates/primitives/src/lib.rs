#![doc = include_str!("../README.md")]

//! Primitives for Durin, a library for building solvers for the OP Stack's
//! dispute protocol.

mod dispute_game;
pub use dispute_game::{Claim, GameStatus, GameType};

mod traits;
pub use traits::{DisputeGame, DisputeSolver};

pub mod rule;
