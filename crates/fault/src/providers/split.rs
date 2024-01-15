//! This module contains the implementation of the [crate::TraceProvider] trait for composing two trace providers together
//! based off of the input depth. This implementation can be used to compose several layers of bisection.

use crate::{Gindex, Position, TraceProvider};
use anyhow::Result;
use durin_primitives::Claim;
use std::sync::Arc;

/// The [SplitTraceProvider] is a [TraceProvider] that composes two trace providers together based off of the input depth.
pub struct SplitTraceProvider<Top, Bottom>
where
    Top: TraceProvider,
    Bottom: TraceProvider,
{
    pub top: Top,
    pub bottom: Bottom,
    pub split_depth: u8,
}

impl<Top, Bottom> SplitTraceProvider<Top, Bottom>
where
    Top: TraceProvider,
    Bottom: TraceProvider,
{
    pub fn new(top: Top, bottom: Bottom, split_depth: u8) -> Self {
        Self {
            top,
            bottom,
            split_depth,
        }
    }
}

#[async_trait::async_trait]
impl<Top, Bottom> TraceProvider for SplitTraceProvider<Top, Bottom>
where
    Top: TraceProvider + Sync,
    Bottom: TraceProvider + Sync,
{
    async fn absolute_prestate(&self, position: Position) -> Result<Arc<[u8]>> {
        if position.depth() <= self.split_depth {
            self.top.absolute_prestate(position).await
        } else {
            self.bottom.absolute_prestate(position).await
        }
    }

    async fn absolute_prestate_hash(&self, position: Position) -> Result<Claim> {
        if position.depth() <= self.split_depth {
            self.top.absolute_prestate_hash(position).await
        } else {
            self.bottom.absolute_prestate_hash(position).await
        }
    }

    async fn state_at(&self, position: Position) -> Result<Arc<[u8]>> {
        if position.depth() <= self.split_depth {
            self.top.state_at(position).await
        } else {
            self.bottom.state_at(position).await
        }
    }

    async fn state_hash(&self, position: Position) -> Result<Claim> {
        if position.depth() <= self.split_depth {
            self.top.state_hash(position).await
        } else {
            self.bottom.state_hash(position).await
        }
    }

    async fn proof_at(&self, position: Position) -> Result<Arc<[u8]>> {
        if position.depth() <= self.split_depth {
            self.top.proof_at(position).await
        } else {
            self.bottom.proof_at(position).await
        }
    }
}
