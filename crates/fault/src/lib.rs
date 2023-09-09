//! The fault module contains types and traits related to the FaultDisputeGame.

extern crate durin_primitives;

mod clock;
pub use clock::Clock;

mod position;
pub use position::{compute_gindex, Position};

mod providers;

mod response;
pub use response::FaultSolverResponse;

mod state;
pub use state::{ClaimData, FaultDisputeState};

mod traits;
pub use traits::*;

mod solver;
pub use solver::*;
