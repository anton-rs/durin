//! This module holds traits related to the [FaultDisputeGame]

use crate::{state::ClaimData, Position};
use durin_primitives::{Claim, DisputeGame};

/// A [FaultDisputeGame] is a [DisputeGame] that is played over a FaultVM backend. This
/// trait extends the [DisputeGame] trait with functionality that is specific to the
/// fault [durin_primitives::GameType] variants.
pub trait FaultDisputeGame: DisputeGame {
    /// Returns a shared reference to the raw state of the game DAG.
    fn state(&self) -> &Vec<ClaimData>;

    /// Returns a mutable reference to the raw state of the game DAG.
    fn state_mut(&mut self) -> &mut Vec<ClaimData>;
}

/// A [TraceProvider] is a type that can provide the raw state (in bytes) at a given
/// [Position] within a [FaultDisputeGame].
pub trait TraceProvider<P: AsRef<[u8]>> {
    /// Returns the raw absolute prestate (in bytes).
    fn absolute_prestate(&self) -> P;

    /// Returns the absolute prestate hash.
    fn absolute_prestate_hash(&self) -> Claim;

    /// Returns the raw state (in bytes) at the given position.
    fn state_at(&self, position: Position) -> anyhow::Result<P>;

    /// Returns the state hash at the given position.
    fn state_hash(&self, position: Position) -> anyhow::Result<Claim>;
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

/// The [ChessClock] trait defines the interface of a single side of a chess clock
/// at a given state in time.
pub trait ChessClock {
    /// Returns the seconds elapsed on the chess clock in seconds when it was
    /// last stopped.
    fn duration(&self) -> u64;

    /// Returns the timestamp of when the chess clock was last stopped.
    fn timestamp(&self) -> u64;
}
