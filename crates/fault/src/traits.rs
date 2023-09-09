//! This module holds traits related to the [FaultDisputeGame]

use crate::{
    prelude::FaultSolverResponse,
    state::{ClaimData, FaultDisputeState},
};
use durin_primitives::{Claim, DisputeGame, DisputeSolver};

/// A [FaultDisputeGame] is a [DisputeGame] that is played over a [FaultVM] backend. This
/// trait extends the [DisputeGame] trait with functionality that is specific to the
/// fault [crate::dispute_game::GameType] variants.
pub trait FaultDisputeGame<P: Position>: DisputeGame {
    /// Performs an move against the given [ClaimData].
    fn do_move(&mut self, claim: ClaimData<P>, is_attack: bool) -> anyhow::Result<()>;

    /// Step against the given [ClaimData].
    fn step(&mut self, claim: ClaimData<P>, is_attack: bool) -> anyhow::Result<()>;
}

pub trait FaultDisputeSolver: DisputeSolver<FaultSolverResponse, FaultDisputeState<u128>> {
    /// Returns the raw state (in bytes) at the given position.
    fn trace_at(&self, position: u128) -> anyhow::Result<Vec<u8>>;

    /// Returns the state hash (in bytes) at the given position.
    fn state_hash(&self, position: u128) -> anyhow::Result<Claim>;
}

/// The [Position] trait defines the interface of a generalized index within a binary tree.
/// A "Generalized Index" is calculated as `2^{depth} + index_at_depth`.
pub trait Position {
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
