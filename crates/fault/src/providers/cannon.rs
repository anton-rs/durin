//! This module contains the implementation of the [crate::TraceProvider] trait for calling out to `cannon` to fetch
//! state witnesses and proof values.

use crate::{Gindex, Position, TraceProvider};
use alloy_primitives::keccak256;
use anyhow::Result;
use durin_primitives::Claim;
use std::{marker::PhantomData, sync::Arc};

/// The [CannonTraceProvider] is a [TraceProvider] that runs `cannon` to retrieve state witnesses and proof values.
pub struct CannonTraceProvider {
    pub split_depth: u8,
}

#[async_trait::async_trait]
impl TraceProvider<[u8; 32]> for CannonTraceProvider {
    async fn absolute_prestate(&self) -> Result<Arc<[u8; 32]>> {
        todo!()
    }

    async fn absolute_prestate_hash(&self) -> Result<Claim> {
        todo!()
    }

    async fn state_at(&self, position: Position) -> Result<Arc<[u8; 32]>> {
        todo!()
    }

    async fn state_hash(&self, position: Position) -> Result<Claim> {
        todo!()
    }

    async fn proof_at(&self, _: Position) -> Result<Arc<[u8]>> {
        todo!()
    }
}
