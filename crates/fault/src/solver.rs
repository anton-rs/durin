//! This module contains the various implementations of the [crate::FaultDisputeSolver] trait.

use crate::{
    FaultClaimSolver, FaultDisputeGame, FaultDisputeState, FaultSolverResponse, Position,
    TraceProvider,
};
use durin_primitives::{DisputeGame, DisputeSolver};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

/// A [FaultDisputeSolver] is a [DisputeSolver] that is played over a fault proof VM backend. The solver is responsible
/// for honestly responding to any given [ClaimData] in a given [FaultDisputeState]. It uses a [TraceProvider] to fetch
/// the absolute prestate of the VM as well as the state at any given [Position] within the tree.
pub struct FaultDisputeSolver<T, P, S>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
    S: FaultClaimSolver<T, P>,
{
    pub inner: S,
    _phantom_t: PhantomData<T>,
    _phantom_p: PhantomData<P>,
}

impl<T, P, S> FaultDisputeSolver<T, P, S>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
    S: FaultClaimSolver<T, P>,
{
    pub fn provider(&self) -> &P {
        self.inner.provider()
    }
}

#[async_trait::async_trait]
impl<T, P, S> DisputeSolver<FaultDisputeState, FaultSolverResponse<T>>
    for FaultDisputeSolver<T, P, S>
where
    T: AsRef<[u8]> + Sync + Send,
    P: TraceProvider<T> + Sync,
    S: FaultClaimSolver<T, P> + Sync,
{
    async fn available_moves(
        &self,
        game: Arc<Mutex<FaultDisputeState>>,
    ) -> anyhow::Result<Arc<[FaultSolverResponse<T>]>> {
        let game_lock = game.lock().await;

        // Fetch the local opinion on the root claim.
        let attacking_root = self
            .provider()
            .state_hash(Self::ROOT_CLAIM_POSITION)
            .await?
            != game_lock.root_claim();

        // Fetch the indices of all unvisited claims within the world DAG.
        let unvisited_indices = game_lock
            .state()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (!c.visited).then_some(i))
            .collect::<Vec<_>>();

        // Drop the mutex lock prior to creating the tasks.
        drop(game_lock);

        // Solve each unvisited claim, set the visited flag, and return the responses.
        let tasks = unvisited_indices
            .iter()
            .map(|claim_index| {
                self.inner
                    .solve_claim(game.clone(), *claim_index, attacking_root)
            })
            .collect::<Vec<_>>();

        futures::future::join_all(tasks).await.into_iter().collect()
    }
}

impl<T, P, S> FaultDisputeSolver<T, P, S>
where
    T: AsRef<[u8]>,
    P: TraceProvider<T>,
    S: FaultClaimSolver<T, P>,
{
    const ROOT_CLAIM_POSITION: Position = 1;

    pub fn new(claim_solver: S) -> Self {
        Self {
            inner: claim_solver,
            _phantom_t: PhantomData,
            _phantom_p: PhantomData,
        }
    }
}
