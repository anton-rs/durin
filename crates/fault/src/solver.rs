//! This module contains the various implementations of the [crate::FaultDisputeSolver] trait.

#![allow(dead_code, unused_variables)]

use crate::{
    ClaimData, FaultDisputeGame, FaultDisputeState, FaultSolverResponse, Gindex, Position,
    TraceProvider,
};
use durin_primitives::{Claim, DisputeGame, DisputeSolver};
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
        game: &mut FaultDisputeState,
    ) -> anyhow::Result<Vec<FaultSolverResponse>> {
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

    fn new(provider: P) -> Self {
        Self {
            provider,
            _phantom: PhantomData,
        }
    }

    /// Finds the best move against a [crate::ClaimData] in a given [FaultDisputeState].
    ///
    /// ### Takes
    /// - `world`: The [FaultDisputeState] to solve against.
    /// - `claim_index`: The index of the claim within the state DAG.
    /// - `attacking_root`: A boolean indicating whether or not the solver is attacking the root.
    ///
    /// ### Returns
    /// - [FaultSolverResponse] or [Err]: The best move against the claim.
    fn solve_claim(
        &self,
        world: &mut FaultDisputeState,
        claim_index: usize,
        attacking_root: bool,
    ) -> anyhow::Result<FaultSolverResponse> {
        // Fetch the maximum depth of the game's position tree.
        let max_depth = world.max_depth;

        // Fetch the ClaimData and its position's depth from the world state DAG.
        let claim = world
            .state_mut()
            .get_mut(claim_index)
            .ok_or(anyhow::anyhow!("Failed to fetch claim from passed state"))?;
        let claim_depth = claim.position.depth();

        // Mark the claim as visited. This mutates the passed state and must be reverted if an
        // error is thrown.
        claim.visited = true;

        // In the case that the claim's opinion about the root claim is the same as the local
        // opinion, we can skip the claim. It does not matter if this claim is valid or not
        // because it supports the local opinion of the root claim. Countering it would put the
        // solver in an opposing position to its final objective.
        if claim_depth % 2 == attacking_root as u8 {
            return Ok(FaultSolverResponse::Skip(claim_index));
        }

        // If the claim's parent index is `u32::MAX`, it is the root claim. In this case, the only
        // opportunity is to attack if we disagree with the root - there is no other valid move.
        if claim.parent_index == u32::MAX && attacking_root {
            let claim_hash =
                Self::fetch_state_hash(&self.provider, claim.position.make_move(true), claim)?;
            return Ok(FaultSolverResponse::Move(true, claim_index, claim_hash));
        }

        // Fetch the local trace provider's opinion of the state hash at the claim's position
        let self_state_hash = Self::fetch_state_hash(&self.provider, claim.position, claim)?;

        // TODO(clabby): Consider that because we'll have to search for the pre/post state for the
        // step instruction, we may also need to know if all claims at agreed levels are correct in
        // the path up to the root claim.

        // Determine if the response will be an attack or a defense.
        let is_attack = self_state_hash != claim.value;

        // If the next move will be at the max depth of the game, then the proper move is to
        // perform a VM step against the claim. Otherwise, move in the appropriate direction.
        //
        // TODO(clabby): Return the data necessary for the inputs to the contract calls in the
        // `FaultSolverResponse` variants.
        if claim_depth == max_depth - 1 {
            Ok(FaultSolverResponse::Step(is_attack))
        } else {
            // Fetch the local trace provider's opinion of the state hash at the move's position.
            let claim_hash =
                Self::fetch_state_hash(&self.provider, claim.position.make_move(is_attack), claim)?;

            // If the local opinion of the state hash at the claim's position is different than
            // the claim's opinion about the state, then the proper move is to attack the claim.
            // If the local opinion of the state hash at the claim's position is the same as the
            // claim's opinion about the state, then the proper move is to defend the claim.
            Ok(FaultSolverResponse::Move(
                is_attack,
                claim_index,
                claim_hash,
            ))
        }
    }

    /// Fetches the state hash at a given position from a [TraceProvider].
    /// If the fetch fails, the claim is marked as unvisited and the error is returned.
    #[inline]
    fn fetch_state_hash(
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
}

// TODO: prop tests for solving claims.
#[cfg(test)]
mod test {
    use super::*;
    use crate::{providers::AlphabetTraceProvider, ClaimData};
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
                FaultSolverResponse::Skip(0),
            ),
            (
                root_claim,
                FaultSolverResponse::Move(true, 0, solver.provider.state_hash(2).unwrap()),
            ),
        ];

        for (claim, expected_move) in moves {
            let mut state = FaultDisputeState::new(
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

            let moves = solver.available_moves(&mut state).unwrap();
            assert_eq!(&[expected_move], moves.as_slice());
        }
    }

    #[test]
    fn available_moves_static() {
        let (solver, root_claim) = mocks();
        let moves = [
            (
                solver.provider.state_hash(4).unwrap(),
                FaultSolverResponse::Move(false, 2, solver.provider.state_hash(10).unwrap()),
            ),
            (
                root_claim,
                FaultSolverResponse::Move(true, 2, solver.provider.state_hash(8).unwrap()),
            ),
        ];

        for (claim, expected_move) in moves {
            let mut state = FaultDisputeState::new(
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

            let moves = solver.available_moves(&mut state).unwrap();
            assert_eq!(&[expected_move], moves.as_slice());
        }
    }

    #[test]
    fn available_moves_static_many() {
        let (solver, root_claim) = mocks();
        let mut state = FaultDisputeState::new(
            vec![
                // Invalid root claim - ATTACK
                ClaimData {
                    parent_index: u32::MAX,
                    visited: false,
                    value: root_claim,
                    position: 1,
                    clock: 0,
                },
                // Right level; Wrong claim - SKIP
                ClaimData {
                    parent_index: 0,
                    visited: false,
                    value: root_claim,
                    position: 2,
                    clock: 0,
                },
                // Wrong level; Right claim - DEFEND
                ClaimData {
                    parent_index: 1,
                    visited: false,
                    value: solver.provider.state_hash(4).unwrap(),
                    position: 4,
                    clock: 0,
                },
                // Right level; Wrong claim - SKIP
                ClaimData {
                    parent_index: 3,
                    visited: false,
                    value: root_claim,
                    position: 8,
                    clock: 0,
                },
            ],
            root_claim,
            GameStatus::InProgress,
            4,
        );

        let moves = solver.available_moves(&mut state).unwrap();
        assert_eq!(
            &[
                FaultSolverResponse::Move(true, 0, solver.provider.state_hash(2).unwrap()),
                FaultSolverResponse::Skip(1),
                FaultSolverResponse::Move(false, 2, solver.provider.state_hash(10).unwrap()),
                FaultSolverResponse::Skip(3)
            ],
            moves.as_slice()
        );
    }
}
