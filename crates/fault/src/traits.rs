//! This module holds traits related to the [FaultDisputeGame]

use crate::{state::ClaimData, FaultDisputeState, FaultSolverResponse, Position};
use anyhow::Result;
use durin_primitives::{Claim, DisputeGame};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A [FaultDisputeGame] is a [DisputeGame] that is played over a FaultVM backend. This
/// trait extends the [DisputeGame] trait with functionality that is specific to the
/// fault [durin_primitives::GameType] variants.
pub trait FaultDisputeGame: DisputeGame {
    /// Returns a shared reference to the raw state of the game DAG.
    fn state(&self) -> &Vec<ClaimData>;

    /// Returns a mutable reference to the raw state of the game DAG.
    fn state_mut(&mut self) -> &mut Vec<ClaimData>;
}

/// A [FaultClaimSolver] is a solver that finds the correct response to a given [durin_primitives::Claim]
/// within a [FaultDisputeGame].
#[async_trait::async_trait]
pub trait FaultClaimSolver<T: AsRef<[u8]>, P: TraceProvider<T>> {
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
    ) -> Result<FaultSolverResponse<T>>;

    /// Returns a shared reference to the [TraceProvider] that the solver uses to fetch the state of the VM and
    /// commitments to it.
    fn provider(&self) -> &P;
}

/// A [TraceProvider] is a type that can provide the raw state (in bytes) at a given [Position] within
/// a [FaultDisputeGame].
#[async_trait::async_trait]
pub trait TraceProvider<P: AsRef<[u8]>> {
    /// Returns the raw absolute prestate (in bytes).
    async fn absolute_prestate(&self) -> Result<Arc<P>>;

    /// Returns the absolute prestate hash.
    async fn absolute_prestate_hash(&self) -> Result<Claim>;

    /// Returns the raw state (in bytes) at the given position.
    async fn state_at(&self, position: Position) -> Result<Arc<P>>;

    /// Returns the state hash at the given position.
    async fn state_hash(&self, position: Position) -> Result<Claim>;

    /// Returns the raw proof for the commitment at the given position.
    async fn proof_at(&self, position: Position) -> Result<Arc<[u8]>>;
}

/// The [Gindex] trait defines the interface of a generalized index within a binary tree.
/// A "Generalized Index" is calculated as `2^{depth} + index_at_depth`.
pub trait Gindex {
    /// Returns the depth of the [Position] within the tree.
    fn depth(&self) -> u8;

    /// Returns the index at depth of the [Position] within the tree.
    fn index_at_depth(&self) -> u64;

    /// Returns the left child [Position] relative to the current [Position].
    fn left(&self) -> Self;

    /// Returns the right child [Position] relative to the current [Position].
    fn right(&self) -> Self;

    /// Returns the parent [Position] relative to the current [Position].
    fn parent(&self) -> Self;

    /// Returns the rightmost [Position] that commits to the same trace index as the current [Position].
    fn right_index(&self, max_depth: u8) -> Self;

    /// Returns the trace index that the current [Position] commits to.
    fn trace_index(&self, max_depth: u8) -> u64;

    /// Returns the relative [Position] for an attack or defense move against the current [Position].
    fn make_move(&self, is_attack: bool) -> Self;
}

/// The [ChessClock] trait defines the interface of a single side of a chess clock at a given state in time.
pub trait ChessClock {
    /// Returns the seconds elapsed on the chess clock in seconds when it was last stopped.
    fn duration(&self) -> u64;

    /// Returns the timestamp of when the chess clock was last stopped.
    fn timestamp(&self) -> u64;
}
