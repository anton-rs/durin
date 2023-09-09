//! The fault module contains types and traits related to the FaultDisputeGame.

extern crate durin_primitives;

mod clock;
mod position;
mod response;
mod state;
mod traits;

pub mod prelude {
    pub use super::{clock::*, position::*, response::*, traits::*};
}
