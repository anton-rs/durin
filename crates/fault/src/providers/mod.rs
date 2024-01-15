//! This modules contains trace providers for the variants of the [crate::FaultDisputeGame].

mod alphabet;
pub use self::alphabet::AlphabetTraceProvider;

mod output;
pub use self::output::OutputTraceProvider;

mod split;
pub use self::split::SplitTraceProvider;
