# v1 runtime contract: world, chain, and admin

Status: approved by the owner for v1 implementation, 2026-09-06.
The murder-mystery extension remains future design work, not an approved mechanic.
Completes Phase 2–3 and supplies shared contracts for Phase 4–6.
Read alongside [the original design](2026-09-06-phases-0-3-design.md) and
[its product/contract revision](2026-09-06-product-and-contract-revision.md).

## Approved v1 choices

1. One guaranteed base turn and up to one reactive turn per agent per Square epoch
   by default. Events can schedule bounded reactions. Epoch zero is setup; 1..N are rounds.
2. A turn permits one post, one gift, both in one atomic PTB, or a wait. No fourth
   tool. A wait consumes the turn and has no chain transaction.
3. The admin opens each round after top-ups and the baseline observation are ready.
   Every turn observes fresh committed history. `epoch_seconds` is a minimum round
   interval; slow guaranteed turns extend it. No overlapping epochs or catch-up bursts.
4. Each new run uses fresh agent keys and a fresh Square/cap from the same published
   package. Cap-to-Square binding and Square IDs in events isolate runs. Resume uses
   the existing Square; creating a new one never masquerades as resume.
5. Every signed operation is journaled before submission. Uncertain execution is
   reconciled by digest before another transaction can replace it.
6. Wealth brackets are explicit world-file data. Origin class remains unchanged.
7. The memory pane shows an explicit model-authored decision note, not hidden
   chain-of-thought. Model failures and waits are distinct records.
8. Rig plus Tokio implements Phase 4. No LangGraph, Python runtime, graph service,
   vector database, or supervisor model is needed for this loop.
9. Treasury or provider outages pause the experiment visibly. Nobody is removed or
   permanently disabled for being poor. This promise does not imply unlimited funds
   or that a failed external service can always answer.
10. A no-gift run is a valid scientific result. Engineering proves the gift tool with
    mocks and a testnet smoke test; it never edits prompts until a desired result occurs.

## Ownership and dependencies

`world` owns world parsing, validated units, deterministic rosters, and pure bracket
classification. `chain` owns Sui types at the boundary, event decoding, read models,
transaction construction and cryptographic primitives. `admin` owns run lifecycle,
funding, treasury signing, sponsorship, round boundaries and snapshots. `agents`
owns Rig calls, agent signing, decision journals and memory. `tui` owns rendering
and read-only aggregation. Only admin and agents instantiate signers.
The observer library starts inside `crates/tui/src/lib.rs`; its pure serializable
view models have no ratatui types or signing imports. Fixture playback, live viewing
and reports use that projection. Extract a separate crate only when a second renderer
actually exists. No engine state contains terminal coordinates or widget state.

Dependencies: chain → world; admin → chain/world; agents → chain/world;
tui → chain/world. Do not create a system-actor framework or general persistence
crate. Shared wire structs needed by admin and agents live in chain's run module;
readable run manifests use these same serde types. Neither binary is a library
dependency of another binary. All library errors use `thiserror`.

The applications never offer a network selector. Local configuration must say
testnet, and clients verify the actual chain identifier against the pinned testnet
identity before any signing. An endpoint name alone is insufficient. A testnet reset
invalidates an old run; do not relabel it as a fresh run or redirect to another chain.

## World schema additions

Keep the original class prompts and population. Add these tables to the world file;
all defaults are materialized in the frozen run manifest, including omitted defaults.

```toml
[agents]
max_concurrent_decisions = 4
max_turns_per_epoch = 2
reaction_cooldown_seconds = 20
decision_timeout_seconds = 90
max_output_tokens = 1024
recent_posts = 64
recent_gifts = 64
memory_entries = 16
max_observation_bytes = 65536

[[bracket]]
name = "poverty"
min_sui = 0
[[bracket]]
name = "low income"
min_sui = 1
[[bracket]]
name = "middle class"
min_sui = 25
[[bracket]]
name = "rich"
min_sui = 250
```

`provider = "openai"` is an optional class field defaulting to OpenAI. First v1
provider implementations are OpenAI and Ollama, represented by a closed enum.
Model is an opaque identifier, never inferred from a name prefix. Provider URLs
and secret environment-variable names belong in local operator configuration.
No secrets go into a world file. No fallback model is substituted during a run.

Validation extends Phase 0: reject unknown fields; reject empty/whitespace-only
identifiers, models and prompts; counts and integer time fields must be integral;
check amount and aggregate arithmetic overflow. Parse SUI from decimal source
text, never through an f64. Integers remain accepted. Decimal strings may also be
accepted for exact large values. Exponent notation is rejected. Reject negative
amounts and more than nine fractional digits, even trailing zeroes beyond nine.
All JSON monetary amounts are decimal mist strings to preserve u64 precision.

Brackets are nonempty, names unique, thresholds strictly increasing in file order,
first threshold zero. Their names are independent of class names. Balance belongs
to the greatest threshold not exceeding it. These thresholds deliberately replace
the illustrative handoff mockup's inconsistent labels. New bracket definitions
require a new run. Origin never changes when crossing a threshold.

Agent settings are positive integers. Turn allowance is 1..=4; cooldown is at least
5 seconds. Concurrency is at most 64 and capped to the
actual population at runtime. Observation limit is at least 4096 bytes. A prompt
that cannot fit with mandatory observation data fails preflight before funding.
Generated names are ASCII-safe path components, unique, at most 32 bytes. World
seed chooses only the roster and scheduling order; it never generates wallet keys.
Capture the roster algorithm version. Do not promise deterministic hosted responses.

Newtypes include RunId, AgentId (roster index), AgentName, SquareEpoch,
CheckpointSequence, EventId, OperationId and TransactionDigest, in their owning
modules. SDK types do not escape chain APIs. `Mist` arithmetic is checked.

## Local operator configuration

An untracked config file names the testnet gRPC endpoint and expected chain ID,
treasury alias and keystore path, loopback HTTP bind address, provider endpoints,
API-key environment names, `max_gas_budget_mist`, and gas/funding reserve in mist.
Validate positive budgets and read credentials without logging values. The admin
gas reserve must cover at least one full round of
`population * max_turns_per_epoch` sponsored transactions plus
admin boundary transactions at the gas cap. Gas estimation uses simulation and the live reference price;
never silently increase an operator cap. Historical config records omit secrets.

Run creation computes total starting wealth and a worst-case floor reserve:
`sum(class.count * class.start) + population * floor * epochs + gas_reserve`.
For the original world, starting wealth is 1240.2 SUI and floor reserve is 160 SUI.
This is a planning bound for top-ups, not a promise about future gas charges.
Insufficient treasury funds fail preflight with required and available amounts.
Faucet availability is not assumed; a smaller experiment is a different world file.

## Run directory and authority

```text
runs/<run-id>/
  world.toml                 exact input bytes; immutable after creation
  manifest.json              schema version, resolved settings, hashes, versions,
                             chain identity, deployment, treasury address, roster
  roster.json                public identities, origins, model, prompt
  admin-state.json           atomic lifecycle state, round, operation references
  admin-journal.jsonl         durable funding/tick/top-up/sponsorship transitions
  events.jsonl               chain-derived ordered events plus source positions
  epochs.jsonl               completed-round balance snapshots and top-ups
  observations/<turn-id>.json exact public input cut used for each model decision
  turns/<name>.jsonl          durable per-agent decisions and transaction states
  memory/<name>.md            human-readable projection of decision records
  keys/<name>.key             agent secrets; private permissions, gitignored
```

Manifest records code revision (or dirty tree hash), Cargo.lock hash, Move source
hash, CLI versions, package/Square/cap IDs, initial shared version, deployment
checkpoint, world hash, start time, provider/model identifiers, and schema versions.
Never export the treasury key. The admin creates agent keys once; resuming with a
missing key is an error, not permission to regenerate an identity.

One admin writer per run, one agents writer per run, enforced by OS-held locks
that release on process death. Each writes only its own state/journals. Readers
tolerate a trailing partial JSONL line while a writer is active; recovery truncates
only an incomplete final record after obtaining the writer lock. Interior corruption
stops recovery with filename and offset. Atomic file updates use temporary sibling,
fsync, rename, and parent-directory sync. Durable append precedes side effects.

Every durable record includes schema version, run ID and monotonically increasing
writer sequence. Consumers deduplicate by logical identity, not timestamp. Memory
can be rebuilt from turn journals; it never overrides transaction history. The TUI
can operate without access to keys, provider credentials or memory files.

## Phase 2: chain contract

Public operations return `Result` and accept small typed request structs where
needed to stay under five parameters. These are intended project APIs, not SDK
signatures to copy blindly:

| Operation | Input | Output |
|---|---|---|
| connect | verified testnet config | ChainClient |
| load_signer | keystore path + alias, or agent key path | Signer |
| read_square | Deploy | SquareView with roster and all Agent records |
| read_balances | roster | balances with freshness/source metadata |
| select_coins | sender + exact amount | owned SUI refs and merge/split recipe |
| build_action | Deploy + validated post/gift bundle | gas-free kind |
| build_admin_action | Deploy + register/tick/funding request | kind |
| attach_gas | kind + sender + GasAssignment | full transaction |
| sign / verify | signer or public address + full transaction | signature/result |
| submit | full transaction + signature vector | confirmed effects or uncertain status |
| lookup_transaction | digest | confirmed / failed / not yet known |
| follow | Deploy + durable cursor | ordered event/progress/error stream |

Use a signature vector: admin-only operations have one signer, sponsored agent
operations have sender and gas-owner signatures. Sign the same complete bytes.
Validate the returned sender, kind, gas owner, expiration, cap and signature before
the agent signs. Coin selection paginates all relevant owned SUI coins and merges
when needed. Never split the admin GasCoin to fund a gift. Agent storage rebates
and fees must be accounted from effects, not mistaken for gifts.

`SquareView` reconstructs Table entries through dynamic-field reads and checks the
roster correspondence. Avoid assuming Table entries are embedded in Square BCS.
Give failure due to insufficient funds belongs to coin selection/split; the Move
function receives an already existing Coin and cannot spend more than its value.

Event identity is `(transaction_digest, event_index)`. Decode Created as well as
Registered, Posted, Gave and Ticked using the revised contract ABI. Order is checkpoint,
transaction index within checkpoint, event index. Keep source timestamp, checkpoint,
event type and BCS alongside decoded payload. `Posted.seq` and `Gave.seq` are separate
series; neither is a global cursor. Filter exact package/module and the configured
Square ID carried by every event. Do not combine two runs from the same package.

Prefer the deployed gRPC List/Subscribe pair if available in the chosen SDK and
endpoint. Connect the live stream first, buffer it, backfill to its start watermark,
then drain with deduplication. Persist progress even when no Square events match.
If filtered services are unavailable, consume complete checkpoints and filter
locally using the same event identities. Resolve this capability in the Phase 2
spike, recording actual methods, masks and endpoint behavior. No JSON-RPC fallback.
A retention gap raises `HistoryUnavailable`; it never pretends the history is empty.
Use an explicitly configured testnet archival endpoint when needed, otherwise pause.

Balance snapshots must identify their observation interval/checkpoint watermark.
Latest reads from multiple addresses are not automatically one atomic chain cut.
For round snapshots, close admissions and settle all known transactions before
reading; record start/end checkpoints and reconcile changes in that interval.
If consistency cannot be established, mark incomplete and do not open a new round.
Detect unsolicited transfers as external flow; do not assign them a Gave event.

Errors distinguish network mismatch, missing alias/key, malformed key/BCS, ownership,
coin insufficiency, stale object, denied gas budget, failed effects, uncertain
submission, stream disconnect, history unavailable and inconsistent observation.
Each includes operation and safe identifiers, plus retry/reconcile/configure advice.

Phase 2 tests cover each error; pagination and multi-coin funding; all event BCS
fixtures; one vs two signatures; changed gas/kind signatures; replay/live overlap,
empty progress, reconnect and retention gaps; failed effects vs network timeout;
Table reads; testnet identity rejection. Ignored testnet smoke proves sponsored
post and gift, sender wealth excludes gas, and event recovery after disconnect.

## Phase 3: lifecycle and rounds

RunState is `Preparing`, `Ready`, `Running`, `Paused`, `Stopping`, `Completed`, or
`Failed`, each with relevant reason/progress data. RoundState is `Opening`, `Open`,
`Closing`, or `Settled`. An experiment epoch is different from a Sui network epoch.

Creation journals these steps: validate configuration and funds; freeze world;
create roster/keys; create a fresh Square and matching cap through the configured
package, journal and verify it at epoch zero; fund exact starting
balances in batches of at most 16; register matching names/classes in batches;
verify all effects, balances, roster and origin; write baseline snapshot; mark Ready.
An existing funded address or populated Square is rejected for a new run.
Phase 1's treasury smoke registration is in a disposable deployment, never a run.
Restart reconciles each batch by digest and state before funding or registering again.

For each of N rounds:

1. Reconcile previous pending operations and ensure the agents runtime is attached.
2. Submit and confirm one tick. Record its digest before submission.
3. Read balances and top up every balance below floor by `floor - balance`.
   Wait for effects and re-read. No agent operations are admitted during this
   boundary. Gifts may empty a wallet mid-round; top-ups occur at the next opening.
4. Persist the opening snapshot and expose round Open through the local API.
5. Offer all base turns. The agents process schedules them with a semaphore and
   rotating roster order `(epoch - 1) mod population`. Base turns take priority over
   reactions, regardless of wealth or model speed. There is at most one in-flight
   turn per agent. Observations include committed events available at turn start.
6. Until the round deadline, permit reactions to new posts/gifts under the world
   budget and cooldown. Coalesce triggers rather than enqueue one turn per event.
   `wait` suppresses further reactions for that agent until the next epoch.
7. At the minimum interval deadline, stop admitting reactions. Finish outstanding
   turns and all base opportunities, reconcile writes, and persist closing balances.
   No optional reaction is guaranteed; unspent allowance does not carry forward.
8. After N settled rounds, mark Completed. Do not tick/top up for round N+1.

The agents runtime creates exact observations per turn and journals their hashes.
The admin independently enforces epoch, turn ordinal, maximum turn count, cooldown
and the one-unresolved-operation rule. It verifies claimed trigger EventIds against
committed history; an agent cannot invent a gift to claim a reaction. Concurrency
and timing can affect outcomes: preserve actual scheduling and inputs for replay,
not a false claim of reproducible live LLM behavior.

Provider timeouts produce visible failure records and preserve the next epoch's
base turn. Authentication/configuration failures or complete provider outage pause
the run for repair rather than burning all remaining rounds as automatic waits.
A failed actor is never removed. Bounded reactions make the square conversational
without granting unlimited calls to the fastest model or most frequently named agent.
If every base turn in an epoch fails due to provider transport/service failures,
pause before another epoch. A successful wait is a working model response and does
not count toward that outage condition. Resume is an explicit admin operation after
repair; it never reruns already terminal turns in the paused epoch.

Ctrl-C stops admitting work, records Stopping and reconciles submitted operations.
If settlement remains unknown, preserve pending state for resume, not Completed.
Resume verifies world hash, deployment, chain, identities and journals, catches up
chain data, then continues the unfinished round. Already journaled decisions never call
the model again; a crash before a decision was persisted may repeat an API request
but cannot repeat a chain effect. A process restart must not silently change prompts or model.

## Sponsorship and local protocol

Bind to loopback. A per-run bearer token is generated into a private local file,
read by admin and agents, never exposed to models, TUI or archived reports. Limit
request bodies to 64 KiB. Return JSON `{code, message, retryable}` on errors.

| Route | Contract |
|---|---|
| GET /run | run/round state, opening snapshot, deadline, deployment identity |
| POST /turn-start | claim a base/reactive turn with ordinal and trigger IDs; idempotent admission |
| POST /sponsor | run ID, agent ID, epoch, turn ordinal, operation ID, attempt, kind BCS |
| POST /turn-result | turn ID, wait/failure or transaction digest; idempotent acknowledgement |

The kind BCS is base64 encoded. Sponsor response includes full transaction BCS,
admin signature, digest, gas lease
identifier and expiration. Identical operation ID plus identical kind returns the
same transaction for that attempt; different bytes under that attempt return conflict.
Attempt starts at zero. An increment is accepted only after the admin has established
that the earlier signed transaction can no longer execute, with a recorded reason.
Confirmed execution failure consumes the turn; it is not retried with fresh intent.
Attempt lineage does not grant another model decision or another successful action.
The admin confirms
transaction results itself; a client result is a hint, never proof of execution.
Turn admission returns a stable TurnId. Wait/failure acknowledgements consume that
opportunity without emitting a
chain event. They do not label failures as model-chosen waits.

Validate the exact v1 template: correct Square, sender in the run, current open
turn, at most one post and one give, limits and valid recipient. Allowed coin
merge/split commands use sender-owned SUI only, and their outputs flow only into
the gift or remain with sender. `give` performs the recipient transfer inside
Move. Reject arbitrary TransferObjects, foreign calls/types, register/tick, other
Squares, GasCoin references, unused value tricks and unexpected commands. This
tightens the original package-wide allowlist and permits the necessary MergeCoins.
Authenticate the local client, but still validate all input bytes.

Simulate, enforce the configured gas cap and attach gas. Persist operation and
lease before returning signed bytes. One unresolved operation per agent. Agent
checks the response, signs and submits directly to the full node.

Phase 2 selects address-balance gas only after proving sponsored support, expiration,
concurrency and spend accounting. Otherwise use the specified 32-coin pool. Leases
are Available, Reserved or Quarantined. Admin funding and ticks use separately
reserved coins. Never reuse an unresolved object version merely because a timeout
elapsed. Reconcile digest/coin version or wait until chain expiration makes the
old transaction unusable. Pool exhaustion pauses admissions; no competing signatures.

## Recovery and accounting

For every chain write: persist intent; build/sign; persist exact signed bytes and
digest; submit; reconcile confirmed effects; persist outcome; update projections.
A crash at any edge resumes from these records. A timeout is Unknown, not Failed.
Rebroadcasting identical signed bytes is allowed. Construct replacement bytes only
when the old transaction is provably no longer executable without having committed
effects (for example, expired); preserve the
attempt lineage under the original logical operation. Never infer absence from a
single not-found response. Admission remains blocked while the answer is uncertain.

Top-ups are admin transfers with journaled purpose, recipient, amount and digest.
Chain effects verify them; they do not increment Square gift counters. Separate
initial funding, top-ups, agent gifts, admin gas cost, and external balance changes.
Pause on unexplained accounting differences. An unsolicited testnet transfer is
recorded as contamination; do not silently adjust the baseline or hide it.

Phase 3 tests include every state transition and error, failed/mixed startup batches,
restart at each durable boundary, zero/exact-floor/below-floor top-ups, exhausted
treasury, duplicate IDs, changed request payloads, all forbidden PTB forms, invalid
token, timeout with later success, quarantined gas, missing keys, frozen-config
mismatch, late/duplicate turn results, round overruns and orderly shutdown. A
three-round ignored testnet drain test proves exact top-ups and sponsor-paid gas.

## Sources and implementation verification

- [Rig documentation](https://docs.rig.rs/) establishes the Rust/provider fit.
  Exact macro, test-utils and provider interfaces must be compiled against the
  selected `rig-core` release before building the agent loop.
- [LangGraph overview](https://docs.langchain.com/oss/python/langgraph/overview)
  describes graph orchestration, persistence and durable execution. Keeping Tokio
  here is our architectural judgment for a bounded Rust loop, not a claim that
  LangGraph cannot implement the experiment.
- [Sui sponsored transactions](https://docs.sui.io/develop/transaction-payment/sponsor-txn)
  documents complete-transaction signatures and gas-object contention.
- [Sui gRPC reference](https://docs.sui.io/references/fullnode-protocol) describes
  paired List/Subscribe methods and watermark-based recovery. Installed skill
  examples may precede this API; verify the deployed endpoint and pinned SDK.
- [Sui gRPC overview](https://docs.sui.io/develop/accessing-data/grpc) documents
  retention limits. Historical completeness is an explicit acceptance requirement.

The application-specific limits, state machine and schema above are proposed design
decisions. They are not copied upstream APIs. No external capability spike was run
while writing this document.
