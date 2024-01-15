//! Mock implementations of the [crate::TraceProvider] trait for testing.

mod alphabet;
pub use self::alphabet::AlphabetTraceProvider;

mod mock_output;
pub use self::mock_output::MockOutputTraceProvider;
