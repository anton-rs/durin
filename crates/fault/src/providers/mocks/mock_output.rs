//! This module contains the implementation of the [crate::TraceProvider] trait for serving mock output commitments.

use crate::{Gindex, Position, TraceProvider};
use alloy_primitives::U256;
use anyhow::Result;
use durin_primitives::Claim;
use std::sync::Arc;

/// The [MockOutputTraceProvider] is a [TraceProvider] that provides mock L2 output commitments for a [Position].
pub struct MockOutputTraceProvider {
    pub starting_block_number: u64,
    pub leaf_depth: u8,
}

impl MockOutputTraceProvider {
    pub fn new(starting_block_number: u64, leaf_depth: u8) -> Self {
        Self {
            starting_block_number,
            leaf_depth,
        }
    }
}

#[async_trait::async_trait]
impl TraceProvider for MockOutputTraceProvider {
    async fn absolute_prestate(&self, _: Position) -> Result<Arc<[u8]>> {
        Ok(Arc::<[u8; 32]>::new(
            U256::from(self.starting_block_number).to_be_bytes(),
        ))
    }

    async fn absolute_prestate_hash(&self, position: Position) -> Result<Claim> {
        // The raw state is equivalent to the state hash in the output trace provider. It must be 32 bytes in size.
        Ok((*self.absolute_prestate(position).await?).try_into()?)
    }

    async fn state_at(&self, position: Position) -> Result<Arc<[u8]>> {
        let state =
            U256::from(position.trace_index(self.leaf_depth) + self.starting_block_number + 1);
        Ok(Arc::<[u8; 32]>::new(state.to_be_bytes()))
    }

    async fn state_hash(&self, position: Position) -> Result<Claim> {
        // The raw state is equivalent to the state hash in the output trace provider. It must be 32 bytes in size.
        Ok((*self.state_at(position).await?).try_into()?)
    }

    async fn proof_at(&self, _: Position) -> Result<Arc<[u8]>> {
        unimplemented!("Proofs are not supported for the OutputTraceProvider")
    }
}
