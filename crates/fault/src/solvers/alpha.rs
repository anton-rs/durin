//! Implementation of the [FaultClaimSolver] trait on the [FaultDisputeSolver].

#![allow(dead_code, unused_variables)]

use crate::{
    ClaimData, FaultClaimSolver, FaultDisputeGame, FaultDisputeState, FaultSolverResponse, Gindex,
    Position, TraceProvider,
};
use anyhow::{anyhow, Result};
use durin_primitives::Claim;
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

/// The alpha claim solver is the first iteration of the Fault dispute game solver used
/// in the alpha release of the Fault proof system on Optimism.
struct AlphaClaimSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    provider: P,
    _phantom: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T, P> FaultClaimSolver<T, P> for AlphaClaimSolver<T, P>
where
    T: AsRef<[u8]> + Send + Sync,
    P: TraceProvider<T> + Sync,
{
    /// Finds the best move against a [crate::ClaimData] in a given [FaultDisputeState].
    ///
    /// ### Takes
    /// - `world`: The [FaultDisputeState] to solve against.
    /// - `claim_index`: The index of the claim within the state DAG.
    /// - `attacking_root`: A boolean indicating whether or not the solver is attacking the root.
    ///
    /// ### Returns
    /// - [FaultSolverResponse] or [Err]: The best move against the claim.
    async fn solve_claim(
        &self,
        world: Arc<Mutex<FaultDisputeState>>,
        claim_index: usize,
        attacking_root: bool,
    ) -> Result<FaultSolverResponse<T>> {
        let mut world_lock = world.lock().await;

        // Fetch the maximum depth of the game's position tree.
        let max_depth = world_lock.max_depth;

        // Fetch the ClaimData and its position's depth from the world state DAG.
        let claim = world_lock
            .state_mut()
            .get_mut(claim_index)
            .ok_or(anyhow!("Failed to fetch claim from passed state"))?;
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
                Self::fetch_state_hash(&self.provider, claim.position.make_move(true), claim)
                    .await?;
            return Ok(FaultSolverResponse::Move(true, claim_index, claim_hash));
        }

        // Fetch the local trace provider's opinion of the state hash at the claim's position
        let self_state_hash = Self::fetch_state_hash(&self.provider, claim.position, claim).await?;

        // TODO(clabby): Consider that because we'll have to search for the pre/post state for the
        // step instruction, we may also need to know if all claims at agreed levels are correct in
        // the path up to the root claim.

        // Determine if the response will be an attack or a defense.
        let is_attack = self_state_hash != claim.value;

        // If the next move will be at the max depth of the game, then the proper move is to
        // perform a VM step against the claim. Otherwise, move in the appropriate direction.
        if claim_depth == max_depth {
            // There is a special case when we are attacking the first leaf claim at the max
            // level where we have to provide the absolute prestate. Otherwise, we can derive
            // the prestate position based off of `is_attack` and the incorrect claim's
            // position.
            let (pre_state, proof) = if claim.position.index_at_depth() == 0 && is_attack {
                let pre_state = self.provider.absolute_prestate().await?;
                // TODO(clabby): There may be a proof for the absolute prestate in Cannon.
                let proof: Arc<[u8]> = Arc::new([]);

                (pre_state, proof)
            } else {
                // If the move is an attack, the pre-state is left of the attacked claim's
                // position. If the move is a defense, the pre-state for the step is at the
                // claim's position.
                //
                // SAFETY: We can subtract 1 here due to the above check - we will never
                // underflow the level.
                let pre_state_pos = claim.position - is_attack as u128;

                let pre_state = Self::fetch_state_at(&self.provider, pre_state_pos, claim).await?;
                let proof = Self::fetch_proof_at(&self.provider, pre_state_pos, claim).await?;
                (pre_state, proof)
            };

            Ok(FaultSolverResponse::Step(
                is_attack,
                claim_index,
                pre_state,
                proof,
            ))
        } else {
            // Fetch the local trace provider's opinion of the state hash at the move's position.
            let claim_hash =
                Self::fetch_state_hash(&self.provider, claim.position.make_move(is_attack), claim)
                    .await?;

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

    fn provider(&self) -> &P {
        &self.provider
    }
}

impl<T, P> AlphaClaimSolver<T, P>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
{
    fn new(provider: P) -> Self {
        Self {
            provider,
            _phantom: PhantomData,
        }
    }

    /// Fetches the state hash at a given position from a [TraceProvider].
    /// If the fetch fails, the claim is marked as unvisited and the error is returned.
    #[inline]
    pub(crate) async fn fetch_state_hash(
        provider: &P,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> Result<Claim> {
        let state_hash = provider.state_hash(position).await.map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(state_hash)
    }

    #[inline]
    pub(crate) async fn fetch_state_at(
        provider: &P,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> Result<Arc<T>> {
        let state_at = provider.state_at(position).await.map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(state_at)
    }

    #[inline]
    pub(crate) async fn fetch_proof_at(
        provider: &P,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> Result<Arc<[u8]>> {
        let proof_at = provider.proof_at(position).await.map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(proof_at)
    }
}

/// The rules module contains implementations of the [Rule] type for the
/// alpha solver.
///
/// These rules define the conditions of the game state that must be met before
/// and after state transitions and are used to test the validity of the solving
/// algorithm with various resolution methods.
pub mod rules {
    use crate::FaultDisputeState;
    use durin_primitives::rule::Rule;
    use std::sync::Arc;

    fn pre_move_rules() -> &'static [Rule<Arc<FaultDisputeState>>] {
        &[]
    }

    fn post_move_rules() -> &'static [Rule<Arc<FaultDisputeState>>] {
        &[]
    }
}

// TODO: prop tests for solving claims.
#[cfg(test)]
mod test {
    use super::*;
    use crate::{providers::AlphabetTraceProvider, ClaimData, FaultDisputeSolver};
    use alloy_primitives::{hex, Address, U128};
    use durin_primitives::{Claim, DisputeSolver, GameStatus};
    use tokio::sync::Mutex;

    fn mocks() -> (
        FaultDisputeSolver<
            [u8; 1],
            AlphabetTraceProvider,
            AlphaClaimSolver<[u8; 1], AlphabetTraceProvider>,
        >,
        Claim,
    ) {
        let provider = AlphabetTraceProvider::new(b'a', 4);
        let claim_solver = AlphaClaimSolver::new(provider);
        let solver = FaultDisputeSolver::new(claim_solver);
        let root_claim = Claim::from_slice(&hex!(
            "c0ffee00c0de0000000000000000000000000000000000000000000000000000"
        ));
        (solver, root_claim)
    }

    #[tokio::test]
    async fn available_moves_root_only() {
        let (solver, root_claim) = mocks();
        let moves = [
            (
                solver.provider().state_hash(1).await.unwrap(),
                FaultSolverResponse::Skip(0),
            ),
            (
                root_claim,
                FaultSolverResponse::Move(true, 0, solver.provider().state_hash(2).await.unwrap()),
            ),
        ];

        for (claim, expected_move) in moves {
            let mut state = FaultDisputeState::new(
                vec![ClaimData {
                    parent_index: u32::MAX,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: claim,
                    position: 1,
                    clock: 0,
                }],
                claim,
                GameStatus::InProgress,
                4,
            );

            let moves = solver
                .available_moves(Arc::new(Mutex::new(state)))
                .await
                .unwrap();
            assert_eq!(&[expected_move], moves.as_ref());
        }
    }

    #[tokio::test]
    async fn available_moves_static() {
        let (solver, root_claim) = mocks();
        let moves = [
            (
                solver.provider().state_hash(4).await.unwrap(),
                FaultSolverResponse::Move(
                    false,
                    2,
                    solver.provider().state_hash(10).await.unwrap(),
                ),
            ),
            (
                root_claim,
                FaultSolverResponse::Move(true, 2, solver.provider().state_hash(8).await.unwrap()),
            ),
        ];

        for (claim, expected_move) in moves {
            let mut state = FaultDisputeState::new(
                vec![
                    ClaimData {
                        parent_index: u32::MAX,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: true,
                        value: root_claim,
                        position: 1,
                        clock: 0,
                    },
                    ClaimData {
                        parent_index: 0,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: true,
                        value: solver.provider().state_hash(2).await.unwrap(),
                        position: 2,
                        clock: 0,
                    },
                    ClaimData {
                        parent_index: 1,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
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

            let moves = solver
                .available_moves(Arc::new(Mutex::new(state)))
                .await
                .unwrap();
            assert_eq!(&[expected_move], moves.as_ref());
        }
    }

    #[tokio::test]
    async fn available_moves_static_many() {
        let (solver, root_claim) = mocks();
        let mut state = FaultDisputeState::new(
            vec![
                // Invalid root claim - ATTACK
                ClaimData {
                    parent_index: u32::MAX,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: root_claim,
                    position: 1,
                    clock: 0,
                },
                // Right level; Wrong claim - SKIP
                ClaimData {
                    parent_index: 0,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: root_claim,
                    position: 2,
                    clock: 0,
                },
                // Wrong level; Right claim - DEFEND
                ClaimData {
                    parent_index: 1,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: solver.provider().state_hash(4).await.unwrap(),
                    position: 4,
                    clock: 0,
                },
                // Right level; Wrong claim - SKIP
                ClaimData {
                    parent_index: 3,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
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

        let moves = solver
            .available_moves(Arc::new(Mutex::new(state)))
            .await
            .unwrap();
        assert_eq!(
            &[
                FaultSolverResponse::Move(true, 0, solver.provider().state_hash(2).await.unwrap()),
                FaultSolverResponse::Skip(1),
                FaultSolverResponse::Move(
                    false,
                    2,
                    solver.provider().state_hash(10).await.unwrap()
                ),
                FaultSolverResponse::Skip(3)
            ],
            moves.as_ref()
        );
    }

    #[tokio::test]
    async fn available_moves_static_step() {
        let (solver, root_claim) = mocks();
        let cases = [
            (
                FaultSolverResponse::Step(true, 4, Arc::new([b'a']), Arc::new([])),
                true,
            ),
            (
                FaultSolverResponse::Step(false, 4, Arc::new([b'b']), Arc::new([])),
                false,
            ),
        ];

        for (expected_response, wrong_leaf) in cases {
            let mut state = FaultDisputeState::new(
                vec![
                    // Invalid root claim - ATTACK
                    ClaimData {
                        parent_index: u32::MAX,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: true,
                        value: root_claim,
                        position: 1,
                        clock: 0,
                    },
                    // Honest Attack
                    ClaimData {
                        parent_index: 0,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: true,
                        value: solver.provider().state_hash(2).await.unwrap(),
                        position: 2,
                        clock: 0,
                    },
                    // Wrong level; Wrong claim - ATTACK
                    ClaimData {
                        parent_index: 1,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: true,
                        value: root_claim,
                        position: 4,
                        clock: 0,
                    },
                    // Honest Attack
                    ClaimData {
                        parent_index: 2,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: true,
                        value: solver.provider().state_hash(8).await.unwrap(),
                        position: 8,
                        clock: 0,
                    },
                    // Wrong level; Wrong claim - ATTACK STEP
                    ClaimData {
                        parent_index: 3,
                        countered_by: Address::ZERO,
                        claimant: Address::ZERO,
                        bond: U128::ZERO,
                        visited: false,
                        value: if wrong_leaf {
                            root_claim
                        } else {
                            solver.provider().state_hash(16).await.unwrap()
                        },
                        position: 16,
                        clock: 0,
                    },
                ],
                root_claim,
                GameStatus::InProgress,
                4,
            );

            let moves = solver
                .available_moves(Arc::new(Mutex::new(state)))
                .await
                .unwrap();
            assert_eq!(&[expected_response], moves.as_ref());
        }
    }
}
