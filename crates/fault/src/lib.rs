//! The fault module contains types and traits related to the FaultDisputeGame.

extern crate alloy_primitives;
extern crate alloy_sol_types;
extern crate durin_primitives;

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
