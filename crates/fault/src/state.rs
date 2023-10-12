//! This module contains the in-memory represtentation of a [crate::FaultDisputeGame]'s state

#![allow(dead_code, unused_variables)]

use crate::{ChessClock, Clock, FaultDisputeGame, Gindex, Position};
use durin_primitives::{Claim, DisputeGame, GameStatus};
use std::time::{SystemTime, UNIX_EPOCH};

/// The [ClaimData] struct holds the data associated with a claim within a
/// [crate::FaultDisputeGame]'s state on-chain.
#[derive(Debug, Clone, Copy)]
pub struct ClaimData {
    pub parent_index: u32,
    pub countered: bool,
    pub value: Claim,
    pub position: Position,
    pub clock: Clock,
    pub visited: bool,
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
    /// The max depth of the position tree.
    pub max_depth: u8,
}

impl FaultDisputeState {
    pub fn new(
        state: Vec<ClaimData>,
        root_claim: Claim,
        status: GameStatus,
        max_depth: u8,
    ) -> Self {
        Self {
            state,
            root_claim,
            status,
            max_depth,
        }
    }
}

impl DisputeGame for FaultDisputeState {
    fn root_claim(&self) -> Claim {
        self.root_claim
    }

    fn status(&self) -> &GameStatus {
        &self.status
    }

    // TODO(clabby): Generic resolution mechanisms
    fn resolve(&mut self, sim: bool) -> anyhow::Result<GameStatus> {
        if self.status != GameStatus::InProgress {
            return Ok(self.status);
        }

        let mut left_most_index = self.state.len() - 1;
        let mut left_most_trace_index = u64::MAX;
        for i in (0..=left_most_index).rev() {
            let claim = &self
                .state()
                .get(i)
                .ok_or(anyhow::anyhow!("Could not fetch claim from state"))?;

            if claim.countered {
                continue;
            }

            let trace_index = claim.position.trace_index(self.max_depth);
            if trace_index < left_most_trace_index {
                left_most_trace_index = trace_index;
                left_most_index = i + 1;
            }
        }

        let left_most_uncontested = self
            .state()
            .get(left_most_index)
            .ok_or(anyhow::anyhow!("Could not fetch claim from state"))?;

        let status = if left_most_uncontested.position.depth() % 2 == 0
            && left_most_trace_index != u64::MAX
        {
            GameStatus::DefenderWins
        } else {
            GameStatus::ChallengerWins
        };

        if !sim {
            let parent_index = left_most_uncontested.parent_index;
            let opposing_clock = if parent_index == u32::MAX {
                left_most_uncontested.clock
            } else {
                self.state()
                    .get(parent_index as usize)
                    .ok_or(anyhow::anyhow!("Could not fetch parent claim from state"))?
                    .clock
            };

            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            if opposing_clock.duration() + (now - opposing_clock.timestamp()) <= 604800 >> 1 {
                anyhow::bail!("Clocks have not expired")
            }

            self.status = status;
        }

        Ok(status)
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
