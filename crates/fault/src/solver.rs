//! This module contains the various implementations of the [crate::FaultDisputeSolver] trait.

#![allow(dead_code, unused_variables)]

use crate::{
    ClaimData, FaultClaimSolver, FaultDisputeGame, FaultDisputeState, FaultSolverResponse,
    Position, TraceProvider,
};
use durin_primitives::{Claim, DisputeGame, DisputeSolver};
use std::{marker::PhantomData, sync::Arc};

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

impl<T, P> DisputeSolver<FaultDisputeState, FaultSolverResponse<T>> for FaultDisputeSolver<T, P>
where
    FaultDisputeSolver<T, P>: FaultClaimSolver<T>,
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    fn available_moves(
        &self,
        game: &mut FaultDisputeState,
    ) -> anyhow::Result<Arc<[FaultSolverResponse<T>]>> {
        // Fetch the local opinion on the root claim.
        let attacking_root =
            self.provider.state_hash(Self::ROOT_CLAIM_POSITION)? != game.root_claim();

        // Fetch the indices of all unvisited claims within the world DAG.
        let unvisited_indices = game
            .state()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (!c.visited).then_some(i))
            .collect::<Vec<_>>();

        // Solve each unvisited claim, set the visited flag, and return the responses.
        unvisited_indices
            .iter()
            .map(|claim_index| self.solve_claim(game, *claim_index, attacking_root))
            .collect()
    }
}

impl<T, P> FaultDisputeSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    const ROOT_CLAIM_POSITION: Position = 1;

    pub fn new(provider: P) -> Self {
        Self {
            provider,
            _phantom: PhantomData,
        }
    }

    /// Fetches the state hash at a given position from a [TraceProvider].
    /// If the fetch fails, the claim is marked as unvisited and the error is returned.
    #[inline]
    pub(crate) fn fetch_state_hash(
        provider: &P,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> anyhow::Result<Claim> {
        let state_hash = provider.state_hash(position).map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(state_hash)
    }

    #[inline]
    pub(crate) fn fetch_state_at(
        provider: &P,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> anyhow::Result<Arc<T>> {
        let state_at = provider.state_at(position).map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(state_at)
    }

    #[inline]
    pub(crate) fn fetch_proof_at(
        provider: &P,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> anyhow::Result<Arc<[u8]>> {
        let proof_at = provider.proof_at(position).map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(proof_at)
    }
}
