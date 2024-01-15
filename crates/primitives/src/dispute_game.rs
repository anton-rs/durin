//! Types related to the [crate::DisputeGame] trait.

use alloy_primitives::B256;
use anyhow::{bail, Error};
use std::convert::TryFrom;

/// The [Claim] type is an alias to [B256], used to deliniate a claim hash from a regular hash.
pub type Claim = B256;

/// The [GameType] enum is used to indicate which type of dispute game is being played.
#[derive(Debug, Clone)]
pub enum GameType {
    /// The [GameType::FaultCannon] variant is used to indicate that the dispute game is being played over a
    /// FaultDisputeGame with the Cannon VM as its backend source of truth.
    FaultCannon = 0,
    /// The [GameType::Alphabet] variant is used to indicate that the dispute game is being played over a
    /// FaultDisputeGame with the mock Alphabet VM as its backend source of truth. This game is used for
    /// testing purposes.
    Alphabet = 255,
}

impl TryFrom<u8> for GameType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GameType::FaultCannon),
            255 => Ok(GameType::Alphabet),
            _ => bail!("Invalid game type"),
        }
    }
}

/// The [GameStatus] enum is used to indicate the status of a dispute game.
#[derive(Debug, Clone)]
pub enum GameStatus {
    /// The [GameStatus::InProgress] variant is used to indicate that the dispute game is still in progress.
    InProgress = 0,
    /// The [GameStatus::ChallengerWins] variant is used to indicate that the challenger of the root claim has won the
    /// dispute game.
    ChallengerWins = 1,
    /// The [GameStatus::DefenderWins] variant is used to indicate that the defender of the root claim has won the
    /// dispute game.
    DefenderWins = 2,
}

impl TryFrom<u8> for GameStatus {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GameStatus::InProgress),
            1 => Ok(GameStatus::ChallengerWins),
            2 => Ok(GameStatus::DefenderWins),
            _ => bail!("Invalid game status"),
        }
    }
}
