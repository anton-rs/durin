//! This module contains the implementation of the [crate::TraceProvider] trait for the
//! mock Alphabet VM.

use crate::{Gindex, Position, TraceProvider, VMStatus};
use alloy_primitives::{keccak256, U256};
use alloy_sol_types::{sol, SolType};
use durin_primitives::Claim;
use std::convert::TryInto;

type AlphabetClaimConstruction = sol! { tuple(uint256, uint256) };

/// The [AlphabetTraceProvider] is a [TraceProvider] that provides the correct
/// trace for the mock Alphabet VM.
pub struct AlphabetTraceProvider {
    /// The absolute prestate of the alphabet VM is the setup state.
    /// This will be the ascii representation of letter prior to the first
    /// in the honest alphabet trace.
    pub absolute_prestate: u8,
    /// The maximum depth of the dispute game position tree.
    pub max_depth: u8,
}

impl TraceProvider<[u8; 1]> for AlphabetTraceProvider {
    fn absolute_prestate(&self) -> [u8; 1] {
        [self.absolute_prestate]
    }

    fn absolute_prestate_hash(&self) -> Claim {
        let prestate = U256::from(self.absolute_prestate);
        let mut prestate_hash = keccak256(<sol!(uint256)>::encode_single(&prestate));
        prestate_hash[0] = VMStatus::Unfinished as u8;
        prestate_hash
    }

    fn state_at(&self, position: Position) -> anyhow::Result<[u8; 1]> {
        let absolute_prestate = self.absolute_prestate as u64;
        let trace_index = position.trace_index(self.max_depth);

        let state = (absolute_prestate + trace_index + 1)
            .try_into()
            .unwrap_or(self.absolute_prestate + 2u8.pow(self.max_depth as u32));
        Ok([state])
    }

    fn state_hash(&self, position: Position) -> anyhow::Result<Claim> {
        let state_sol = (
            U256::from(position.trace_index(self.max_depth)),
            U256::from(self.state_at(position)?[0]),
        );
        let mut state_hash = keccak256(AlphabetClaimConstruction::encode(&state_sol));
        state_hash[0] = VMStatus::Invalid as u8;
        Ok(state_hash)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compute_gindex;
    use alloy_primitives::hex;

    #[test]
    fn alphabet_encoding() {
        let provider = AlphabetTraceProvider {
            absolute_prestate: b'a',
            max_depth: 4,
        };

        let prestate_sol = U256::from(provider.absolute_prestate()[0]);
        let prestate = <sol!(uint256)>::encode_single(&prestate_sol);
        assert_eq!(
            hex!("0000000000000000000000000000000000000000000000000000000000000061"),
            prestate.as_slice()
        );

        let mut prestate_hash = provider.absolute_prestate_hash();
        prestate_hash[0] = VMStatus::Unfinished as u8;
        assert_eq!(
            hex!("03ecb75dd1820844c57b6762233d4e26853b3a7b8157bbd9f41f280a0f1cee9b"),
            prestate_hash.as_slice()
        );
    }

    #[test]
    fn alphabet_trace_at() {
        let provider = AlphabetTraceProvider {
            absolute_prestate: b'a',
            max_depth: 4,
        };

        for i in 0..16 {
            let expected = b'a' + i + 1;
            let position = compute_gindex(provider.max_depth, i as u64);

            let expected_encoded = (U256::from(i), U256::from(expected));
            let mut expected_hash = keccak256(AlphabetClaimConstruction::encode(&expected_encoded));
            expected_hash[0] = VMStatus::Invalid as u8;

            assert_eq!(provider.state_at(position).unwrap()[0], expected);
            assert_eq!(provider.state_hash(position).unwrap(), expected_hash);
        }
    }
}
