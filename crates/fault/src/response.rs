//! Holds the response type for a [crate::FaultDisputeGame]

/// The [FaultSolverResponse] enum describes the response that a solver should
/// return when asked to make a move.
pub enum FaultSolverResponse {
    /// A response indicating that the proper move is to attack the given claim.
    Attack,
    /// A response indicating that the proper move is to defend the given claim.
    Defend,
    /// A response indicating that the proper move is to skip the given claim.
    Skip,
}
