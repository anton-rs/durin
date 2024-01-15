//! This module contains the implementation of the [crate::TraceProvider] trait for composing two trace providers together
//! based off of the input depth. This implementation can be used to compose several layers of bisection.

use crate::{Gindex, Position, TraceProvider};
use alloy_primitives::keccak256;
use anyhow::Result;
use durin_primitives::Claim;
use std::{marker::PhantomData, sync::Arc};

/// The [SplitTraceProvider] is a [TraceProvider] that composes two trace providers together based off of the input depth.
pub struct SplitTraceProvider<T, TOP, BOTTOM>
where
    T: AsRef<[u8]> + Send + Sync,
    TOP: TraceProvider<T>,
    BOTTOM: TraceProvider<T>,
{
    pub top: TOP,
    pub bottom: BOTTOM,
    pub split_depth: u8,
    pub _phantom: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T, TOP, BOTTOM> TraceProvider<T> for SplitTraceProvider<T, TOP, BOTTOM>
where
    T: AsRef<[u8]> + Send + Sync,
    TOP: TraceProvider<T> + Sync,
    BOTTOM: TraceProvider<T> + Sync,
{
    async fn absolute_prestate(&self) -> Result<Arc<T>> {
        todo!()
    }

    async fn absolute_prestate_hash(&self) -> Result<Claim> {
        todo!()
    }

    async fn state_at(&self, position: Position) -> Result<Arc<T>> {
        if position.depth() <= self.split_depth {
            self.top.state_at(position).await
        } else {
            // TODO: Pass relative position based on split depth?
            self.bottom.state_at(position).await
        }
    }

    async fn state_hash(&self, position: Position) -> Result<Claim> {
        Ok(keccak256(self.state_at(position).await?.as_ref()))
    }

    async fn proof_at(&self, _: Position) -> Result<Arc<[u8]>> {
        todo!()
    }
}
