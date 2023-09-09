//! The fault module contains types and traits related to the FaultDisputeGame.

extern crate durin_primitives;

mod clock;
mod position;
mod providers;
mod response;
mod solver;
mod state;
mod traits;

pub use solver::*;

pub mod prelude {
    pub use super::{clock::*, position::*, providers::*, response::*, traits::*};
}
