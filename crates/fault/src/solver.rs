//! This module contains the various implementations of the [crate::FaultDisputeSolver] trait.

#![allow(dead_code, unused_variables)]

use crate::{
    ClaimData, FaultDisputeGame, FaultDisputeState, FaultSolverResponse, Gindex, Position,
    TraceProvider,
};
use durin_primitives::{DisputeGame, DisputeSolver};
use std::marker::PhantomData;

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
            .map(|c| self.solve_claim(game, c, attacking_root))
            .collect()
    }
}

impl<T, P> FaultDisputeSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    const ROOT_CLAIM_POSITION: Position = 1;

    fn new(provider: P) -> Self {
        Self {
            provider,
            _phantom: PhantomData,
        }
    }

    #[inline]
    fn solve_claim(
        &self,
        game: &FaultDisputeState,
        claim: &ClaimData,
        attacking_root: bool,
    ) -> anyhow::Result<FaultSolverResponse> {
        let claim_depth = claim.position.depth();

        // In the case that the claim's opinion about the root claim is the same as the local
        // opinion, we can skip the claim. It does not matter if this claim is valid or not
        // because it supports the local opinion of the root claim. Countering it would put the
        // solver in an opposing position to its final objective.
        if claim_depth % 2 == attacking_root as u8 {
            return Ok(FaultSolverResponse::Skip);
        }

        // Fetch the local trace provider's opinion of the state hash at the claim's position
        let self_state_hash = self.provider.state_hash(claim.position)?;

        let move_direction = if self_state_hash == claim.value {
            // If the local opinion of the state hash at the claim's position is the same as the
            // claim's opinion about the state, then the proper move is to defend the claim.
            FaultSolverResponse::Defend
        } else {
            // If the local opinion of the state hash at the claim's position is different than
            // the claim's opinion about the state, then the proper move is to attack the claim.
            FaultSolverResponse::Attack
        };

        // If the next move will be at the max depth of the game, then the proper move is to
        // perform a VM step against the claim. Otherwise, move in the appropriate direction.
        //
        // TODO(clabby): Return the data necessary for the inputs to the contract calls in the
        // `FaultSolverResponse` variants.
        if claim_depth == game.max_depth - 1 {
            Ok(FaultSolverResponse::Step(Box::new(move_direction)))
        } else {
            Ok(move_direction)
        }
    }
}

// TODO: prop tests for solving claims.
#[cfg(test)]
mod test {
    use super::*;
    use crate::providers::AlphabetTraceProvider;
    use alloy_primitives::hex;
    use durin_primitives::{Claim, GameStatus};

    fn mocks() -> (FaultDisputeSolver<[u8; 1], AlphabetTraceProvider>, Claim) {
        let provider = AlphabetTraceProvider::new(b'a', 4);
        let solver = FaultDisputeSolver::new(provider);
        let root_claim = Claim::from_slice(&hex!(
            "c0ffee00c0de0000000000000000000000000000000000000000000000000000"
        ));
        (solver, root_claim)
    }

    #[test]
    fn available_moves_root_only() {
        let (solver, root_claim) = mocks();
        let moves = [
            (
                solver.provider.state_hash(1).unwrap(),
                FaultSolverResponse::Skip,
            ),
            (root_claim, FaultSolverResponse::Attack),
        ];

        for (claim, expected_move) in moves {
            let state = FaultDisputeState::new(
                vec![ClaimData {
                    parent_index: u32::MAX,
                    visited: false,
                    value: claim,
                    position: 1,
                    clock: 0,
                }],
                claim,
                GameStatus::InProgress,
                4,
            );

            let moves = solver.available_moves(&state).unwrap();
            assert_eq!(&[expected_move], moves.as_slice());
        }
    }

    #[test]
    fn available_moves_static() {
        let (solver, root_claim) = mocks();
        let moves = [
            (
                solver.provider.state_hash(4).unwrap(),
                FaultSolverResponse::Defend,
            ),
            (root_claim, FaultSolverResponse::Attack),
        ];

        for (claim, expected_move) in moves {
            let state = FaultDisputeState::new(
                vec![
                    ClaimData {
                        parent_index: u32::MAX,
                        visited: true,
                        value: root_claim,
                        position: 1,
                        clock: 0,
                    },
                    ClaimData {
                        parent_index: 0,
                        visited: true,
                        value: solver.provider.state_hash(2).unwrap(),
                        position: 2,
                        clock: 0,
                    },
                    ClaimData {
                        parent_index: 1,
                        visited: false,
                        value: claim,
                        position: 4,
                        clock: 0,
                    },
                ],
                root_claim,
                GameStatus::InProgress,
                4,
            );

            let moves = solver.available_moves(&state).unwrap();
            assert_eq!(&[expected_move], moves.as_slice());
        }
    }
}
