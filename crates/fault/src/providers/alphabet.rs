//! This module contains the implementation of the [crate::TraceProvider] trait for the mock Alphabet VM.

use crate::{Gindex, Position, TraceProvider, VMStatus};
use alloy_primitives::{keccak256, U256};
use alloy_sol_types::{sol, SolType};
use durin_primitives::Claim;
use std::{convert::TryInto, sync::Arc};

type AlphabetClaimConstruction = sol! { tuple(uint256, uint256) };

/// The [AlphabetTraceProvider] is a [TraceProvider] that provides the correct trace for the mock Alphabet VM.
pub struct AlphabetTraceProvider {
    /// The absolute prestate of the alphabet VM is the setup state.
    /// This will be the ascii representation of letter prior to the first
    /// in the honest alphabet trace.
    pub absolute_prestate: u8,
    /// The maximum depth of the dispute game position tree.
    pub max_depth: u8,
}

impl AlphabetTraceProvider {
    pub fn new(absolute_prestate: u8, max_depth: u8) -> Self {
        Self {
            absolute_prestate,
            max_depth,
        }
    }
}

#[async_trait::async_trait]
impl TraceProvider<[u8; 1]> for AlphabetTraceProvider {
    async fn absolute_prestate(&self) -> anyhow::Result<Arc<[u8; 1]>> {
        Ok(Arc::new([self.absolute_prestate]))
    }

    async fn absolute_prestate_hash(&self) -> anyhow::Result<Claim> {
        let prestate = U256::from(self.absolute_prestate);
        let mut prestate_hash = keccak256(<sol!(uint256)>::abi_encode(&prestate));
        prestate_hash[0] = VMStatus::Unfinished as u8;
        Ok(prestate_hash)
    }

    async fn state_at(&self, position: Position) -> anyhow::Result<Arc<[u8; 1]>> {
        let absolute_prestate = self.absolute_prestate as u64;
        let trace_index = position.trace_index(self.max_depth);

        let state = (absolute_prestate + trace_index + 1)
            .try_into()
            .unwrap_or(self.absolute_prestate + 2u8.pow(self.max_depth as u32));
        Ok(Arc::new([state]))
    }

    async fn state_hash(&self, position: Position) -> anyhow::Result<Claim> {
        let state_sol = (
            U256::from(position.trace_index(self.max_depth)),
            U256::from(self.state_at(position).await?[0]),
        );
        let mut state_hash = keccak256(AlphabetClaimConstruction::abi_encode(&state_sol));
        state_hash[0] = VMStatus::Invalid as u8;
        Ok(state_hash)
    }

    async fn proof_at(&self, _: Position) -> anyhow::Result<Arc<[u8]>> {
        Ok(Arc::new([]))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compute_gindex;
    use alloy_primitives::hex;

    #[tokio::test]
    async fn alphabet_encoding() {
        let provider = AlphabetTraceProvider {
            absolute_prestate: b'a',
            max_depth: 4,
        };

        let prestate_sol = U256::from(provider.absolute_prestate().await.unwrap()[0]);
        let prestate = <sol!(uint256)>::abi_encode(&prestate_sol);
        assert_eq!(
            hex!("0000000000000000000000000000000000000000000000000000000000000061"),
            prestate.as_slice()
        );

        let mut prestate_hash = provider.absolute_prestate_hash().await.unwrap();
        prestate_hash[0] = VMStatus::Unfinished as u8;
        assert_eq!(
            hex!("03ecb75dd1820844c57b6762233d4e26853b3a7b8157bbd9f41f280a0f1cee9b"),
            prestate_hash.as_slice()
        );
    }

    #[tokio::test]
    async fn alphabet_trace_at() {
        let provider = AlphabetTraceProvider {
            absolute_prestate: b'a',
            max_depth: 4,
        };

        for i in 0..16 {
            let expected = b'a' + i + 1;
            let position = compute_gindex(provider.max_depth, i as u64);

            let expected_encoded = (U256::from(i), U256::from(expected));
            let mut expected_hash =
                keccak256(AlphabetClaimConstruction::abi_encode(&expected_encoded));
            expected_hash[0] = VMStatus::Invalid as u8;

            assert_eq!(provider.state_at(position).await.unwrap()[0], expected);
            assert_eq!(provider.state_hash(position).await.unwrap(), expected_hash);
        }
    }
}
