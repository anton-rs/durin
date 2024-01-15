# `durin-fault`

> **Note**
> WIP

This crate contains an implementation of a solver for the [OP Stack][op-stack]'s `FaultDisputeGame`. This implementation is currently
generic over the `TraceProvider`, `ClaimSolver`, and local resolution algorithm. This allows for expanding the solver to support multiple
backends, such as [Asterisc][asterisc] or [Cannon][cannon], as well as multiple local resolution algorithms.

## Solvers

- [`AlphaClaimSolver`](./src/solvers/alpha.rs) - [DEPRECATED] The first iteration of the Fault dispute game solver used in the alpha release of the Fault proof system on Optimism.
- [`ChadClaimSolver`](./src/solvers/alpha_chad.rs) - The second iteration of the Fault dispute game solver used in the alpha chad release of the Fault proof system on Optimism.

### Rules

`Rules` (see: [Rules](../../README.md)) in `durin-fault` are defined within the `solvers` module. These rules are used to describe the
expected behavior of all possible state transitions that the solver can suggest to the game's state.

## Trace Providers

- [`SplitTraceProvider`](./src/providers/split.rs) - An abstraction over two implementations of the `TraceProvider` trait that splits which one is used depending on the `Position` passed.
- [`CannonTraceProvider`](./src/providers/cannon.rs) - A trace provider that can issue state witnesses and memory access proofs for instructions within a `cannon` trace.
- [`OutputTraceProvider`](./src/providers/output.rs) - A trace provider that can issue output roots for L2 block numbers.

### Mock Trace Providers

- [`AlphabetTraceProvider`](./src/providers/mocks/alphabet.rs) - A mock trace provider for the `AlphabetVM` used for testing.
- [`MockOutputTraceProvider`](./src/providers/mocks/mock_output.rs) - A mock trace provider for output roots used for testing.

## Resolution Functions

- _todo_
- [`(Planned) Sweep`] - "Sweep" resolution is the first implementation of a global resolution algorithm for the fault dispute game. In reverse
  chronological order, the algorithm looks for the left-most uncountered instruction in the game DAG and compares its
  agreement with the root claim to determine the outcome of the game.
- [`(Planned) @inphi's Sub-Game Resolution`] - @inphi's sub-game resolution algorithm is a new resolution algorithm that allows for
  the resolution of a game to be split into multiple sub-games. This allows for the solver to reduce the amount of
  moves necessary to resolve a game as well as enforce incentive compatibility in bond payouts.

<!-- LINKS -->

[op-stack]: https://github.com/ethereum-optimism/optimism
[cannon]: https://github.com/ethereum-optimism/optimism/tree/develop/cannon
[asterisc]: https://github.com/protolambda/asterisc
