//! This modules contains trace providers for the variants of the [crate::FaultDisputeGame].

mod split;
pub use self::split::SplitTraceProvider;

mod output;
pub use self::output::OutputTraceProvider;

mod cannon;
pub use self::cannon::CannonTraceProvider;

mod mocks;
pub use self::mocks::{AlphabetTraceProvider, MockOutputTraceProvider};
