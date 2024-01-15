//! The fault module contains types and traits related to the FaultDisputeGame.

#![allow(dead_code, unused_imports)]

#[cfg(test)]
extern crate proptest;

mod types;
pub use types::*;

mod providers;

mod state;
pub use state::{ClaimData, FaultDisputeState};

mod traits;
pub use traits::*;

mod solver;
pub use solver::*;

mod solvers;
pub use solvers::*;
