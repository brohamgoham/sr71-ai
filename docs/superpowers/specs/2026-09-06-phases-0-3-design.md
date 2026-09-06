# Phases 0 to 3: scaffold, Square contract, chain crate, admin

Status: original design-session draft; revised before implementation.
See [product/contract revision](2026-09-06-product-and-contract-revision.md) for the
new world schema and Phase 1 ABI, and [runtime contract](2026-09-06-v1-runtime-design.md)
for completed Phase 2–3. Conflicting original details below are historical; do not
implement both versions. All v1 phase specs are indexed in [docs/README](../../README.md).
Covers: roadmap Phase 0 and 1 in full, Phase 2 and 3 at interface level.
Not covered: agents (Phase 4), TUI (Phase 5).

## Decisions made in this spec

1. **Agent tools in v1 are `post`, `give`, `wait`.** The roadmap listed
   `sponsor` as an agent action. The admin pays all gas, so an agent paying
   gas for another has no effect. Sponsorship is admin-only in v1. If gas is
   ever made an agent cost, `sponsor` returns as a tool. Roadmap updated.
2. **Registration is admin-only.** Only the holder of `AdminCap` can add an
   agent to the Square. Random testnet addresses cannot join a run.
3. **Origin class is stored on chain** at registration and never changes.
   The contract enforces "origin is permanent."
4. **Posts are events, not storage.** The Square keeps counters per agent.
   Post text lives in the emitted event. History is read back through
   the gRPC event list, not from object state. This keeps the shared object
   small and avoids storage growth.
5. **Epoch is on chain.** The admin calls `tick`, the Square increments its
   epoch and emits an event. The TUI learns the epoch from the chain, which
   keeps the "TUI reads chain only" rule intact.
6. **Sponsored flow: agent builds, admin fills gas and co-signs, agent
   submits.** Matches the Sui docs recommendation that the sender submits
   the final transaction to a full node directly.
7. **Admin gas: address-balance gas if the Rust SDK supports it on testnet,
   otherwise a pool of 32 pre-split gas coins.** Decided in Phase 2 by a
   spike, recorded in the plan.

## Phase 0: scaffold and world file

### Workspace

```
Cargo.toml                 [workspace], resolver 3, shared lints
crates/world/              world file parsing and validation
crates/chain/              Phase 2
crates/admin/              Phase 3
crates/agents/             Phase 4
crates/tui/                Phase 5
contracts/square/          Move package
worlds/default.toml
deny.toml
.pre-commit-config.yaml    prek: cargo fmt, clippy, sui move test
```

The existing `src/main.rs` hello-world is deleted. The root `Cargo.toml`
becomes a pure workspace manifest with `[workspace.lints]` carrying the
clippy set from the global standards, and `[workspace.dependencies]`
pinning exact versions. Current versions as of this spec:

| Crate | Version |
|---|---|
| sui-sdk-types | 0.3.2 |
| sui-crypto | 0.3.1 |
| sui-rpc | 0.3.2 |
| sui-transaction-builder | 0.3.2 |
| rig-core | 0.42.0 |
| ratatui | 0.30.2 |
| tokio | 1.53.1 |
| toml | 1.1.5 |
| serde | 1.0.229 |
| thiserror | 2.0.20 |
| anyhow | 1.0.104 |
| tracing | 0.1.44 |

Re-check on `crates.io` when Phase 0 is executed; pin whatever is current.

### Toolchain

- Rust stable via rustup. Installed cargo is a nightly; the workspace pins
  stable in `rust-toolchain.toml`.
- Sui CLI via `suiup`. Installed version is 1.62.0. The current Move.toml
  format (framework resolved automatically) needs 1.63 or later, so Phase 0
  runs `suiup update` first.
- `prek install` with hooks: `cargo fmt --check`, `cargo clippy` with
  `-D warnings`, `sui move test` inside `contracts/square`.
- `cargo deny check` with a `deny.toml` allowing MIT, Apache-2.0, BSD,
  and the licenses the Sui crates carry. Advisories fail the build.

### Sui client

`sui client` configured against testnet with one Ed25519 address, the
**treasury**. Funded from the faucet. Its key stays in the default Sui
keystore and is never copied into the repo. The admin process loads it by
alias name from the keystore path in its config.

### `crates/world`

Parses a world file into typed values. No I/O beyond reading the file.

```toml
[world]
name = "default"
floor_sui = 0.1
epoch_seconds = 120
epochs = 100
seed = 7

[[class]]
name = "rich"
count = 2
start_sui = 500
model = "gpt-5.4-mini"
prompt = """..."""
```

Types:

```rust
pub struct World { name, floor: Mist, epoch: Duration, epochs: u32, seed: u64, classes: Vec<Class> }
pub struct Class { name: ClassName, count: u16, start: Mist, model: ModelName, prompt: String }
pub struct Mist(u64);            // 1 SUI = 1_000_000_000 mist
pub struct ClassName(String);
pub struct ModelName(String);
pub struct AgentSpec { name: AgentName, class: ClassName, start: Mist, model: ModelName, prompt: String }
```

`World::load(path) -> Result<World, WorldError>` and
`World::agents(&self) -> Vec<AgentSpec>` which expands classes into agents
with names drawn deterministically from `seed` and a built-in name list,
so the same world file always produces the same roster.

Validation, each with its own `WorldError` variant naming the field:

- at least one class
- class names unique and non-empty
- every `count` at least 1, total at most 64
- every `start_sui` at least `floor_sui`
- `floor_sui` greater than 0
- `epoch_seconds` at least 10
- `epochs` at least 1
- `prompt` and `model` non-empty
- SUI amounts have at most 9 decimal places

Tests: one per validation rule using inline TOML strings, one round-trip of
the default world, one asserting `agents()` is deterministic for a seed and
produces `sum(count)` agents with unique names.

Done when: `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo deny check`, and `sui move build` in the (still empty) contract all
pass, and `cargo run -p world -- worlds/default.toml` prints the roster.

## Phase 1: the Square contract

Package `square`, module `square::square`, edition 2024, no explicit
dependencies.

### Objects

```move
public struct Square has key {
    id: UID,
    epoch: u64,
    agents: Table<address, Agent>,
    roster: vector<address>,      // Table cannot enumerate keys
    posts: u64,
    gifts: u64,
}

public struct Agent has store {
    name: String,
    class: String,                // origin, never changes
    joined_epoch: u64,
    posts: u64,
    given: u64,                   // total mist given
    received: u64,                // total mist received
}

public struct AdminCap has key, store { id: UID }
```

`init` creates the `Square`, shares it, and transfers `AdminCap` to the
publisher.

### Functions

| Function | Who | Effect |
|---|---|---|
| `register(&AdminCap, &mut Square, addr, name, class)` | admin | adds `Agent`, pushes to roster, emits `Registered` |
| `post(&mut Square, text, ctx)` | registered agent | increments counters, emits `Posted` |
| `give(&mut Square, Coin<SUI>, to, ctx)` | registered agent | transfers coin to `to`, updates both records, emits `Gave` |
| `tick(&AdminCap, &mut Square)` | admin | increments `epoch`, emits `Ticked` |

Read-only getters: `epoch`, `is_registered`, `agent(&Square, addr): &Agent`
and field getters on `Agent`, `roster(&Square): &vector<address>`.

`post` and `give` are `public fun`, not `entry`, per the composable
functions skill. They take `ctx: &mut TxContext` and use `ctx.sender()`.

### Events

```move
public struct Registered has copy, drop { agent: address, name: String, class: String, epoch: u64 }
public struct Posted has copy, drop { agent: address, seq: u64, epoch: u64, text: String }
public struct Gave has copy, drop { from: address, to: address, amount: u64, seq: u64, epoch: u64 }
public struct Ticked has copy, drop { epoch: u64 }
```

`seq` is the Square-wide counter after increment, so consumers can detect
gaps.

### Errors

| Constant | When |
|---|---|
| `ENotRegistered` | sender or `to` not in `agents` |
| `EAlreadyRegistered` | `register` on an existing address |
| `ESelfGift` | `to == sender` |
| `EZeroGift` | coin value is 0 |
| `EPostTooLong` | text longer than `MAX_POST_BYTES = 1024` |
| `EPostEmpty` | text is empty |
| `ENameEmpty`, `EClassEmpty` | `register` with empty strings |

### Tests (`tests/square_tests.move`, `test_scenario`)

Happy path: init creates a shared Square and an owned AdminCap; register
two agents; post from one; give from one to the other and check both
records and the recipient's coin balance; tick twice.

Failure path, one test each with `#[expected_failure(abort_code = ...)]`:
post before register, post empty, post too long, give to unregistered,
give from unregistered, give to self, give zero, register twice, register
with empty name, register with empty class.

Event tests: after each action, `event::events_by_type<T>()` has exactly
one event with the expected fields.

### Publish

`sui client publish` from `contracts/square` on testnet from the treasury
address. Record package ID, Square object ID with its initial shared
version, and AdminCap ID in `worlds/testnet.toml` (a small deploy file,
separate from the world file, committed).

Done when: `sui move test` passes, package is on testnet, and one
`sui client ptb` call to `post` from the treasury address (after
registering it) shows a `Posted` event on SuiVision.

## Phase 2: `crates/chain` (interface level)

One crate wrapping the four Sui crates. Nothing else in the workspace
imports `sui-*` directly.

```rust
pub struct ChainClient           // gRPC connection + deploy ids
pub struct Signer                // wraps SuiKeyPair; loaded from keystore or generated
pub struct Deploy { package, square: SharedRef, admin_cap: ObjectId }

// Builders return transaction *kinds* without gas, for the sponsored flow
pub fn post_kind(deploy, text) -> TransactionKind
pub fn give_kind(deploy, coin: ObjectRef, to) -> TransactionKind
pub fn register_kind(deploy, cap, addr, name, class) -> TransactionKind
pub fn tick_kind(deploy) -> TransactionKind

// Sponsored assembly
pub fn attach_gas(kind, sender, gas: GasData) -> Transaction
pub fn submit(client, tx, sigs: [Signature; 2]) -> Result<Effects>

// Reads
pub async fn balance(client, addr) -> Mist
pub async fn square(client, deploy) -> SquareView   // epoch, roster, per-agent records
pub async fn events_since(client, deploy, cursor) -> (Vec<SquareEvent>, Cursor)
pub fn subscribe(client, deploy) -> impl Stream<Item = SquareEvent>
```

`SquareEvent` is an enum over the four Move events with BCS decoding.

Reconnect rule: `subscribe` is a thin live stream. Consumers keep a cursor
from the last event they processed and call `events_since` on reconnect to
fill the gap, then resume the stream. The crate provides
`follow(client, deploy, cursor)` that does both.

Tests: BCS round-trips for every event type against bytes captured from a
real testnet event; PTB builders checked against expected command lists
offline with `try_build`; one ignored integration test that posts against
testnet.

Spike at the start of Phase 2: does `sui-transaction-builder` 0.3 support
address-balance gas (empty payment) on testnet? Yes: admin uses it, no
gas pool. No: admin splits 32 gas coins at start and leases them.

Done when: a test binary posts as a fresh keypair with gas paid by the
treasury, and sees its own `Posted` event arrive through `follow`.

## Phase 3: `crates/admin` (interface level)

A single long-running process. Holds the treasury key and the `AdminCap`.
Owns one run.

Startup, in order:
1. Load world file and deploy file.
2. Generate one Ed25519 keypair per `AgentSpec`. Write them to
   `runs/<run-id>/keys/<name>.key` (gitignored). Keys are testnet-only.
3. Fund each agent to its class `start` from the treasury in one PTB per
   batch of 16.
4. `register` each agent with name and class, batched.
5. Write `runs/<run-id>/roster.json` with name, class, address, model,
   prompt, so the agent runtime and TUI can find everyone.

Loop, every `epoch_seconds`:
1. `tick`.
2. Read every balance. Any agent under `floor` is topped up to exactly
   `floor`. Emit a `tracing` info line per top-up.
3. Append `{epoch, balances, topups}` to `runs/<run-id>/epochs.jsonl`.
4. Stop after `epochs` ticks.

Sponsorship API, local HTTP on `127.0.0.1`:

```
POST /sponsor   body: { sender, kind_bcs }   -> { tx_bcs, admin_signature }
```

The admin verifies the sender is in the roster, that the kind contains
only calls into the `square` package or `TransferObjects`/`SplitCoins` on
the sender's own coins, and that the gas coin is not referenced by any
command. Then it attaches gas and signs. The agent signs and submits.
Rejections return a reason string.

Tests: roster generation from a world file; top-up arithmetic against
mocked balances; sponsorship validation rejects a kind that touches the
gas coin, a call outside the package, and an unknown sender. One ignored
integration test that runs three epochs on testnet with a script draining
one agent.

Done when: sixteen addresses are funded at their class balances, all
registered, and a drain script is held at the floor across three epochs.

## Out of scope for these phases

- Any LLM call. Phase 4.
- Any terminal rendering. Phase 5.
- Move package upgrades. If the contract changes, republish and update the
  deploy file. Upgrade policy is decided before the first mainnet-like run,
  which is not planned.
