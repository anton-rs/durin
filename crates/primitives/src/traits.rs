//! The traits module contains traits used throughout the library.

use crate::{
    dispute_game::{Claim, GameType},
    GameStatus,
};
use alloy_primitives::Bytes;

/// The [DisputeGame] trait is the highest level trait in the library, describing
/// a simple primitive dispute. It has several key properties:
///
/// - It houses
pub trait DisputeGame {
    /// Returns the root claim of the dispute game. The root claim is a 32 byte
    /// commitment to what is being disputed.
    ///
    /// This claim can be about anything - the only requirement is that it is
    /// a 32 byte commitment.
    fn root_claim(&self) -> Claim;

    /// Returns the type of the dispute game being played.
    fn game_type(&self) -> GameType;

    /// Returns the current status of the dispute game.
    fn status(&self) -> GameStatus;

    /// Returns the UNIX timestamp of the creation of the dispute game on-chain.
    fn created_at(&self) -> u64;

    /// Returns the extra data passed to the [DisputeGame] by its creator. This
    /// data is generic and it is up to the implementation of the game to
    /// determine its decoding.
    fn extra_data(&self) -> Bytes;
}

/// The [DisputeAgent] trait describes the base functionality of a dispute agent
/// for any given [DisputeGame]. It serves as the highest level agent trait, and
/// only enforces functionality that is common to all dispute agents.
///
/// All other agent traits should be subtraits of the [DisputeAgent].
pub trait DisputeAgent<DG: DisputeGame> {}
