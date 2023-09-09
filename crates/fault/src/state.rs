//! This module contains the in-memory represtentation of a
//! [crate::prelude::FaultDisputeGame]'s state

#![allow(dead_code, unused_variables)]

use crate::prelude::{FaultDisputeGame, Position};
use durin_primitives::{Claim, DisputeGame, GameStatus};

/// The [ClaimData] struct holds the data associated with a claim within a
/// [crate::prelude::FaultDisputeGame]'s state on-chain.
pub struct ClaimData<P: Position> {
    parent_index: u32,
    countered: bool,
    value: Claim,
    position: P,
    clock: u64,
}

/// the [FaultDisputeState] struct holds the in-memory representation of a
/// [crate::prelude::FaultDisputeGame]'s state as well as its root claim and
/// local status.
pub struct FaultDisputeState<P: Position> {
    /// The [FaultDisputeState] is modeled as a directed acyclical graph (DAG) of
    /// [ClaimData] structs pointing to their parents, all the way up to the root
    /// claim of the dispute game.
    pub state: Vec<ClaimData<P>>,
    /// The root claim is the claim that commits to the entirety of the backend
    /// VM's trace. The outcome of the game determines if this claim is true or
    /// false.
    root_claim: Claim,
    /// The status of the dispute game.
    status: GameStatus,
}

impl DisputeGame for FaultDisputeState<u128> {
    fn root_claim(&self) -> Claim {
        self.root_claim
    }

    fn status(&self) -> &GameStatus {
        &self.status
    }

    fn resolve(&mut self) -> &GameStatus {
        &self.status
    }
}

impl FaultDisputeGame<u128> for FaultDisputeState<u128> {
    fn step(&mut self, claim: ClaimData<u128>, is_attack: bool) -> anyhow::Result<()> {
        todo!()
    }

    fn do_move(&mut self, claim: ClaimData<u128>, is_attack: bool) -> anyhow::Result<()> {
        todo!()
    }
}
