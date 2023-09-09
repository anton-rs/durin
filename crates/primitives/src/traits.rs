//! The traits module contains traits used throughout the library.

use crate::{dispute_game::Claim, GameStatus};

/// The [DisputeGame] trait is the highest level trait in the library, describing
/// the state of a simple primitive dispute. It has several key properties:
///
/// - It houses a root [Claim], a 32 byte commitment, which is the claim being
///   disputed.
/// - It can exist in one of three states, as indicated by the [GameStatus] enum.
///     1. [GameStatus::InProgress] - The dispute game is still in progress.
///     2. [GameStatus::ChallengerWins] - The challenger of the root claim has won
///        the dispute game.
///     3. [GameStatus::DefenderWins] - The defender of the root claim has won the
///        dispute game.
/// - It has a method to resolve the dispute, which returns the [GameStatus]
///   after resolution. The resolution mechanism can be anything - a fault proof,
///   a validity proof, a multisig, etc. It is up to the implementation of the
///   dispute game to determine the resolution mechanism.
pub trait DisputeGame {
    /// Returns the root claim of the dispute game. The root claim is a 32 byte
    /// commitment to what is being disputed.
    ///
    /// This claim can be about anything - the only requirement is that it is
    /// a 32 byte commitment.
    fn root_claim(&self) -> Claim;

    /// Returns the current status of the dispute game.
    fn status(&self) -> &GameStatus;

    /// Resolves the dispute game, returning the [GameStatus] after resolution.
    fn resolve(&mut self) -> &GameStatus;
}

/// The [DisputeSolver] trait describes the base functionality of a solver for
/// a [DisputeGame].
pub trait DisputeSolver<DG: DisputeGame, C, R> {
    /// Returns the response of the solver provided a [DisputeGame] and a
    /// [Claim] within it. The consumer of the response is responsible for
    /// dispatching the action associated with it.
    fn respond(&self, game: &DG, claim: C) -> R;
}
