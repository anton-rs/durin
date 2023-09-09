//! This module contains the various implementations of the [crate::FaultDisputeSolver] trait.

#![allow(dead_code, unused_variables)]

use crate::{
    ClaimData, FaultDisputeGame, FaultDisputeState, FaultSolverResponse, Gindex, Position,
    TraceProvider,
};
use durin_primitives::{DisputeGame, DisputeSolver};
use std::marker::PhantomData;

#[cfg(test)]
use proptest::prelude::*;

/// A [FaultDisputeSolver] is a [DisputeSolver] that is played over a fault proof VM backend. The
/// solver is responsible for honestly responding to any given [ClaimData] in a given
/// [FaultDisputeState]. It uses a [TraceProvider] to fetch the absolute prestate of the VM as
/// well as the state at any given [Position] within the tree.
pub struct FaultDisputeSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    pub provider: P,
    _phantom: PhantomData<T>,
}

impl<T, P> DisputeSolver<FaultDisputeState, FaultSolverResponse> for FaultDisputeSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    fn available_moves(
        &self,
        game: &FaultDisputeState,
    ) -> anyhow::Result<Vec<FaultSolverResponse>> {
        // Fetch the local opinion on the root claim.
        let attacking_root =
            self.provider.state_hash(Self::ROOT_CLAIM_POSITION)? != game.root_claim();

        game.state()
            .iter()
            .filter(|c| !c.visited)
            .enumerate()
            .map(|(i, c)| self.solve_claim(game, c, attacking_root))
            .collect()
    }
}

impl<T, P> FaultDisputeSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    const ROOT_CLAIM_POSITION: Position = 1;

    #[inline]
    fn solve_claim(
        &self,
        game: &FaultDisputeState,
        claim: &ClaimData,
        attacking_root: bool,
    ) -> anyhow::Result<FaultSolverResponse> {
        // Fetch the local trace provider's opinion of the state hash at the claim's position
        // as well as whether or not the claim agrees with the local opinion on the root claim.
        let self_state_hash = self.provider.state_hash(claim.position)?;
        let agree_with_level = claim.position.depth() % 2 == attacking_root as u8;

        todo!()
    }
}

// TODO: prop tests for solving claims.
#[cfg(test)]
proptest! {
    #[test]
    fn test_solve(s in any::<u8>()) {
        assert!(s <= u8::MAX);
    }
}
