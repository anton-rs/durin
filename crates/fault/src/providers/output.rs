//! This module contains the implementation of the [crate::TraceProvider] trait for the mock Alphabet VM.

use crate::{Position, TraceProvider};
use alloy_rpc_client::RpcClient;
use alloy_transport::TransportResult;
use alloy_transport_http::Http;
use anyhow::Result;
use durin_primitives::Claim;
use reqwest::{Client, Url};
use std::sync::Arc;

/// The [OutputTraceProvider] is a [TraceProvider] that provides L2 output commitments relative to a [Position] in the
/// output bisection portion of the dispute game.
pub struct OutputTraceProvider {
    pub rpc_client: RpcClient<Http<Client>>,
    pub starting_block_number: u64,
    pub leaf_depth: u64,
}

impl OutputTraceProvider {
    pub fn try_new(
        l2_archive_url: String,
        starting_block_number: u64,
        leaf_depth: u64,
    ) -> Result<Self> {
        let rpc_client = RpcClient::builder().reqwest_http(Url::parse(&l2_archive_url)?);
        Ok(Self {
            rpc_client,
            starting_block_number,
            leaf_depth,
        })
    }
}

#[async_trait::async_trait]
impl TraceProvider<[u8; 32]> for OutputTraceProvider {
    async fn absolute_prestate(&self) -> anyhow::Result<Arc<[u8; 32]>> {
        todo!()
        // let transport_result: TransportResult<> = self.rpc_client.prepare("optimism_outputAtBlock", (self.starting_block_number)).await.map_err(|e| anyhow::anyhow!(e))?
    }

    async fn absolute_prestate_hash(&self) -> anyhow::Result<Claim> {
        todo!()
    }

    async fn state_at(&self, position: Position) -> anyhow::Result<Arc<[u8; 32]>> {
        todo!()
    }

    async fn state_hash(&self, position: Position) -> anyhow::Result<Claim> {
        todo!()
    }

    async fn proof_at(&self, _: Position) -> anyhow::Result<Arc<[u8]>> {
        todo!()
    }
}
