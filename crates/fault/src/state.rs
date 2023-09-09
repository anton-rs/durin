//! This module contains the in-memory represtentation of a [crate::FaultDisputeGame]'s state

#![allow(dead_code, unused_variables)]

use crate::{Clock, FaultDisputeGame, Position};
use durin_primitives::{Claim, DisputeGame, GameStatus};

/// The [ClaimData] struct holds the data associated with a claim within a
/// [crate::FaultDisputeGame]'s state on-chain.
#[derive(Debug, Clone, Copy)]
pub struct ClaimData {
    pub parent_index: u32,
    pub visited: bool,
    pub value: Claim,
    pub position: Position,
    pub clock: Clock,
}

/// the [FaultDisputeState] struct holds the in-memory representation of a
/// [crate::FaultDisputeGame]'s state as well as its root claim and
/// local status.
#[derive(Debug, Clone)]
pub struct FaultDisputeState {
    /// The [FaultDisputeState] is modeled as a directed acyclical graph (DAG) of
    /// [ClaimData] structs pointing to their parents, all the way up to the root
    /// claim of the dispute game.
    state: Vec<ClaimData>,
    /// The root claim is the claim that commits to the entirety of the backend
    /// VM's trace. The outcome of the game determines if this claim is true or
    /// false.
    root_claim: Claim,
    /// The status of the dispute game.
    status: GameStatus,
}

impl DisputeGame for FaultDisputeState {
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

impl FaultDisputeGame for FaultDisputeState {
    fn state(&self) -> &Vec<ClaimData> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Vec<ClaimData> {
        &mut self.state
    }
}
