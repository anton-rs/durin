//! The fault module contains types and traits related to the FaultDisputeGame.

extern crate durin_primitives;

mod position;
mod traits;

pub mod prelude {
    pub use super::{position::*, traits::*};
}
