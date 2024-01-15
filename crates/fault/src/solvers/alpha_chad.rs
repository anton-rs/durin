//! Implementation of the [FaultClaimSolver] trait on the [FaultDisputeSolver].

use crate::{
    providers::SplitTraceProvider, ClaimData, FaultClaimSolver, FaultDisputeGame,
    FaultDisputeState, FaultSolverResponse, Gindex, Position, TraceProvider,
};
use anyhow::{anyhow, Result};
use durin_primitives::Claim;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The alpha chad claim solver is the second iteration of the fault claim solver. It contains logic for handling
/// multiple bisection layers and acting on preimage hints.
struct ChadClaimSolver<Top: TraceProvider, Bottom: TraceProvider> {
    provider: SplitTraceProvider<Top, Bottom>,
}

#[async_trait::async_trait]
impl<Top, Bottom> FaultClaimSolver<SplitTraceProvider<Top, Bottom>> for ChadClaimSolver<Top, Bottom>
where
    Top: TraceProvider + Sync,
    Bottom: TraceProvider + Sync,
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
    ) -> Result<FaultSolverResponse> {
        let mut world_lock = world.lock().await;

        // Fetch the split & maximum depth of the game's position tree.
        let (split_depth, max_depth) = (world_lock.split_depth, world_lock.max_depth);

        // Fetch the ClaimData and its position's depth from the world state DAG.
        let claim = world_lock
            .state_mut()
            .get_mut(claim_index)
            .ok_or(anyhow!("Failed to fetch claim from passed state"))?;
        let claim_depth = claim.position.depth();

        // Mark the claim as visited. This mutates the passed state and must be reverted if an error is thrown below.
        claim.visited = true;

        let local_claim = Self::fetch_state_hash(self.provider(), claim.position, claim).await?;
        let local_agree = local_claim == claim.value;
        let right_level = attacking_root != (claim_depth % 2 == 0);

        // Check if the observed claim is the root claim.
        if claim.parent_index == u32::MAX {
            // If we agree with the root claim and it is on a level that we are defending, we ignore it.
            if local_agree && right_level {
                return Ok(FaultSolverResponse::Skip(claim_index));
            }

            // The parent claim is the root claim, so if we disagree with it, by definition we must begin the game with
            // an attack move.
            let claimed_hash =
                Self::fetch_state_hash(self.provider(), claim.position.make_move(true), claim)
                    .await?;
            return Ok(FaultSolverResponse::Move(true, claim_index, claimed_hash));
        } else {
            // Never attempt to defend an execution trace subgame root. We only attack if we disagree with it, otherwise
            // we want to do nothing.
            // TODO: This isn't entirely right. See `op-challenger` semantics.
            if claim_depth == split_depth + 1 && local_agree {
                return Ok(FaultSolverResponse::Skip(claim_index));
            }

            // Never counter a claim that is on a level we agree with, even if it is wrong. If it is uncountered, it
            // furthers the goal of the honest challenger, and even if it is countered, the step will prove that it is
            // also wrong.
            // TODO: This isn't entirely right. See `op-challenger` semantics.
            if right_level {
                return Ok(FaultSolverResponse::Skip(claim_index));
            }

            // Compute the position of the next move. If we agree with the claim, we bisect right, otherwise we bisect
            // left.
            let move_pos = claim.position.make_move(!local_agree);

            // If the move position's depth is less than the max depth, it is a bisection move. If it is 1 greater than
            // the max depth, it is a step move.
            if move_pos.depth() <= max_depth {
                let move_claim = Self::fetch_state_hash(self.provider(), move_pos, claim).await?;
                Ok(FaultSolverResponse::Move(
                    !local_agree,
                    claim_index,
                    move_claim,
                ))
            } else {
                // If the move is an attack against the first leaf, the prestate is the absolute prestate. Otherwise,
                // the prestate is present in the branch taken during bisection.
                let prestate = if move_pos.index_at_depth()
                    % 2u64.pow((max_depth - split_depth) as u32)
                    != 0
                {
                    // If the move is an attack, the prestate commits to `claim.position - 1`.
                    // If the move is a defense, the prestate commits to `claim.position`.
                    if local_agree {
                        Self::fetch_state_at(self.provider(), claim.position, claim).await?
                    } else {
                        Self::fetch_state_at(self.provider(), claim.position - 1, claim).await?
                    }
                } else {
                    Self::fetch_absolute_prestate(self.provider(), move_pos, claim).await?
                };

                let proof = Self::fetch_proof_at(self.provider(), move_pos, claim).await?;
                Ok(FaultSolverResponse::Step(
                    !local_agree,
                    claim_index,
                    prestate,
                    proof,
                ))
            }
        }
    }

    fn provider(&self) -> &SplitTraceProvider<Top, Bottom> {
        &self.provider
    }
}

impl<Top, Bottom> ChadClaimSolver<Top, Bottom>
where
    Top: TraceProvider + Sync,
    Bottom: TraceProvider + Sync,
{
    fn new(provider: SplitTraceProvider<Top, Bottom>) -> Self {
        Self { provider }
    }

    /// Fetches the state hash at a given position from a [TraceProvider].
    /// If the fetch fails, the claim is marked as unvisited and the error is returned.
    #[inline]
    pub(crate) async fn fetch_absolute_prestate(
        provider: &SplitTraceProvider<Top, Bottom>,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> Result<Arc<[u8]>> {
        let absolute_prestate = provider.absolute_prestate(position).await.map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(absolute_prestate)
    }

    /// Fetches the state hash at a given position from a [TraceProvider].
    /// If the fetch fails, the claim is marked as unvisited and the error is returned.
    #[inline]
    pub(crate) async fn fetch_state_hash(
        provider: &SplitTraceProvider<Top, Bottom>,
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
        provider: &SplitTraceProvider<Top, Bottom>,
        position: Position,
        observed_claim: &mut ClaimData,
    ) -> Result<Arc<[u8]>> {
        let state_at = provider.state_at(position).await.map_err(|e| {
            observed_claim.visited = false;
            e
        })?;
        Ok(state_at)
    }

    #[inline]
    pub(crate) async fn fetch_proof_at(
        provider: &SplitTraceProvider<Top, Bottom>,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        providers::{AlphabetTraceProvider, MockOutputTraceProvider},
        ClaimData, FaultDisputeSolver,
    };
    use alloy_primitives::{hex, Address, U128};
    use durin_primitives::{Claim, DisputeSolver, GameStatus};
    use tokio::sync::Mutex;

    // Test tree configurations.
    const MAX_DEPTH: u8 = 8;
    const SPLIT_DEPTH: u8 = 4;

    fn mocks() -> (
        FaultDisputeSolver<
            ChadClaimSolver<MockOutputTraceProvider, AlphabetTraceProvider>,
            SplitTraceProvider<MockOutputTraceProvider, AlphabetTraceProvider>,
        >,
        Claim,
    ) {
        let output_provider = MockOutputTraceProvider::new(0, SPLIT_DEPTH);
        let trace_provder = AlphabetTraceProvider::new(0, MAX_DEPTH);
        let claim_solver = ChadClaimSolver::new(SplitTraceProvider::new(
            output_provider,
            trace_provder,
            SPLIT_DEPTH,
        ));

        let state_solver = FaultDisputeSolver::new(claim_solver);
        let root_claim = Claim::from_slice(&hex!(
            "c0ffee00c0de0000000000000000000000000000000000000000000000000000"
        ));
        (state_solver, root_claim)
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
            let state = FaultDisputeState::new(
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
                SPLIT_DEPTH,
                MAX_DEPTH,
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
            let state = FaultDisputeState::new(
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
                SPLIT_DEPTH,
                MAX_DEPTH,
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
        let state = FaultDisputeState::new(
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
                // Right level; right claim - SKIP
                ClaimData {
                    parent_index: 0,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: solver.provider().state_hash(2).await.unwrap(),
                    position: 2,
                    clock: 0,
                },
                // Wrong level; wrong claim - ATTACK
                ClaimData {
                    parent_index: 1,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: root_claim,
                    position: 4,
                    clock: 0,
                },
                // Right level; right claim - SKIP
                ClaimData {
                    parent_index: 2,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: solver.provider().state_hash(8).await.unwrap(),
                    position: 8,
                    clock: 0,
                },
                // Wrong level; wrong claim - ATTACK
                ClaimData {
                    parent_index: 3,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: root_claim,
                    position: 16,
                    clock: 0,
                },
                // Right level; right claim - SKIP
                ClaimData {
                    parent_index: 4,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: solver.provider().state_hash(32).await.unwrap(),
                    position: 32,
                    clock: 0,
                },
                // Wrong level; wrong claim - ATTACK
                ClaimData {
                    parent_index: 5,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: root_claim,
                    position: 64,
                    clock: 0,
                },
                // Right level; right claim - SKIP
                ClaimData {
                    parent_index: 6,
                    countered_by: Address::ZERO,
                    claimant: Address::ZERO,
                    bond: U128::ZERO,
                    visited: false,
                    value: solver.provider().state_hash(128).await.unwrap(),
                    position: 128,
                    clock: 0,
                },
            ],
            root_claim,
            GameStatus::InProgress,
            SPLIT_DEPTH,
            MAX_DEPTH,
        );

        let moves = solver
            .available_moves(Arc::new(Mutex::new(state)))
            .await
            .unwrap();
        assert_eq!(
            &[
                FaultSolverResponse::Move(true, 0, solver.provider().state_hash(2).await.unwrap()),
                FaultSolverResponse::Skip(1),
                FaultSolverResponse::Move(true, 2, solver.provider().state_hash(8).await.unwrap()),
                FaultSolverResponse::Skip(3),
                FaultSolverResponse::Move(true, 4, solver.provider().state_hash(32).await.unwrap()),
                FaultSolverResponse::Skip(5),
                FaultSolverResponse::Move(
                    true,
                    6,
                    solver.provider().state_hash(128).await.unwrap()
                ),
                FaultSolverResponse::Skip(7),
            ],
            moves.as_ref()
        );
    }
}
