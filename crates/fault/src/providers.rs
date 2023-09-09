//! This modules contains trace providers for the variants of the [crate::FaultDisputeGame].

use crate::{Gindex, Position, TraceProvider};
use alloy_primitives::{keccak256, U256};
use alloy_sol_types::{sol, SolType};
use durin_primitives::Claim;
use std::convert::TryInto;

sol! {
    struct AlphabetPrestate {
        uint8 letter;
    }

    struct AlphabetEncoding {
        uint256 instruction;
        uint256 claim;
    }
}

/// The [AlphabetTraceProvider] is a [TraceProvider] that provides the
struct AlphabetTraceProvider {
    /// The absolute prestate of the alphabet VM is the setup state.
    /// This will be the ascii representation of letter prior to the first
    /// in the honest alphabet trace.
    pub absolute_prestate: u8,
    /// The maximum depth of the dispute game position tree.
    pub max_depth: u8,
}

impl TraceProvider<u8> for AlphabetTraceProvider {
    fn absolute_prestate(&self) -> &u8 {
        &self.absolute_prestate
    }

    fn absolute_prestate_hash(&self) -> Claim {
        let prestate_sol = AlphabetPrestate {
            letter: self.absolute_prestate,
        };
        keccak256(AlphabetPrestate::encode(&prestate_sol))
    }

    fn trace_at(&self, position: Position) -> anyhow::Result<u8> {
        let absolute_prestate = *self.absolute_prestate() as u64;
        let trace_index = position.trace_index(self.max_depth);

        Ok((absolute_prestate + trace_index + 1)
            .try_into()
            .unwrap_or(self.absolute_prestate + 2u8.pow(self.max_depth as u32)))
    }

    fn state_hash(&self, position: Position) -> anyhow::Result<Claim> {
        let state_sol = AlphabetEncoding {
            instruction: U256::from(position.trace_index(self.max_depth)),
            claim: U256::from(self.trace_at(position)?),
        };
        Ok(keccak256(AlphabetEncoding::encode(&state_sol)))
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

        let prestate_sol = &AlphabetPrestate {
            letter: *provider.absolute_prestate(),
        };
        let prestate = AlphabetPrestate::encode(prestate_sol);
        assert_eq!(
            hex!("0000000000000000000000000000000000000000000000000000000000000061"),
            prestate.as_slice()
        );

        let prestate_hash = provider.absolute_prestate_hash();
        assert_eq!(
            hex!("f0ecb75dd1820844c57b6762233d4e26853b3a7b8157bbd9f41f280a0f1cee9b"),
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

            let expected_encoded = &AlphabetEncoding {
                instruction: U256::from(i),
                claim: U256::from(expected),
            };
            let expected_hash = keccak256(AlphabetEncoding::encode(expected_encoded));

            assert_eq!(provider.trace_at(position).unwrap(), expected);
            assert_eq!(provider.state_hash(position).unwrap(), expected_hash);
        }
    }
}
