//! This module contains the implementation of the [crate::TraceProvider] trait for fetching output roots from the
//! rollup node.

use crate::{Gindex, Position, TraceProvider};
use alloy_primitives::B256;
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
    pub leaf_depth: u8,
}

/// A minified response of the `optimism_outputAtBlock` RPC method from the rollup node, containing only the output root
/// requested.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputAtBlockResponse {
    pub output_root: B256,
}

impl OutputTraceProvider {
    pub fn try_new(
        l2_archive_url: impl AsRef<str>,
        starting_block_number: u64,
        leaf_depth: u8,
    ) -> Result<Self> {
        let rpc_client = RpcClient::builder().reqwest_http(Url::parse(l2_archive_url.as_ref())?);
        Ok(Self {
            rpc_client,
            starting_block_number,
            leaf_depth,
        })
    }
}

#[async_trait::async_trait]
impl TraceProvider for OutputTraceProvider {
    async fn absolute_prestate(&self, _: Position) -> Result<Arc<[u8]>> {
        let result: TransportResult<OutputAtBlockResponse> = self
            .rpc_client
            .prepare("optimism_outputAtBlock", self.starting_block_number)
            .await;
        Ok(Arc::new(*result?.output_root))
    }

    async fn absolute_prestate_hash(&self, position: Position) -> Result<Claim> {
        // The raw state is equivalent to the state hash in the output trace provider. It must be 32 bytes in size.
        Ok((*self.absolute_prestate(position).await?).try_into()?)
    }

    async fn state_at(&self, position: Position) -> Result<Arc<[u8]>> {
        let result: TransportResult<OutputAtBlockResponse> = self
            .rpc_client
            .prepare(
                "optimism_outputAtBlock",
                self.starting_block_number + position.trace_index(self.leaf_depth) + 1,
            )
            .await;
        Ok(Arc::new(*result?.output_root))
    }

    async fn state_hash(&self, position: Position) -> Result<Claim> {
        // The raw state is equivalent to the state hash in the output trace provider. It must be 32 bytes in size.
        Ok((*self.state_at(position).await?).try_into()?)
    }

    async fn proof_at(&self, _: Position) -> Result<Arc<[u8]>> {
        unimplemented!("Proofs are not supported for the OutputTraceProvider")
    }
}
