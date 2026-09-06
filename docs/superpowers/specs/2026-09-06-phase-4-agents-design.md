# Phase 4: autonomous residents

Status: approved by the owner for v1 implementation, 2026-09-06.
The murder-mystery extension remains future design work, not an approved mechanic.
Depends on the [runtime contract](2026-09-06-v1-runtime-design.md) and
[product revision](2026-09-06-product-and-contract-revision.md).

## Outcome

Sixteen distinct residents observe public actions, remember experiences, converse,
give or wait, and return after failures. They do not collaborate to achieve an
externally assigned score. Each has private memory and its own wallet. The software
does not reward generosity or tune decisions to satisfy the viewer.

## Rig and orchestration

Use `rig-core` for provider access and typed tools. Tokio owns scheduling and I/O;
a project state machine owns side effects and recovery. No LangGraph stage is
needed. If future residents perform long branching research workflows, revisit
orchestration then, without moving signing into an LLM framework.

The Phase 4 first task compiles an isolated adapter proving these requirements on
the pinned release: three typed tool schemas using Rig's macro facility, a mock
model returning tool calls, captured calls without immediate chain execution,
provider errors, and usage collection. The old shorthand "derive macro" must be
translated to the actual supported procedural macro API, not a guessed derive.
Keep the core runtime independent of a particular high-level Rig agent driver.
Fail this compatibility task explicitly if a requirement is unavailable; record a
tested compatible version before proceeding. Do not switch frameworks silently.

OpenAI and Ollama adapters implement the same project `DecisionModel` contract.
The model identifier is taken unchanged from each class. API credentials stay in
the agents process environment. Model availability and tool support are verified
in an explicit preflight smoke call before the first paid experiment. An HTTP
success without valid tool output is not sufficient. Offline tests use Rig mock
models; no CI test reads a live key or calls an actual model service.

The owner will provide a GPT-5.4 mini API key. Verify actual API access, tool support
and pricing at preflight; do not infer them from a subscription or model name. Record the selected model and API usage;
never base the run budget on the original handoff's "cents per epoch" estimate.

## Turn state and scheduling

`TurnId = (RunId, AgentId, SquareEpoch, ordinal)`; base turn ordinal is zero.
`TurnState`: Eligible → Admitted → Observing → Deciding → Proposed → Validated →
Prepared → Submitted → Confirmed. Terminal alternatives are Waited, InvalidDecision,
ModelFailed and ChainFailed. Uncertain submission remains PendingReconciliation.
These are enums with data, not booleans. Failure is not a voluntary wait.

The one-process scheduler has one task per active turn and one semaphore for model
calls. It never runs two turns for the same agent simultaneously. Pending base turns
precede reactions; within each queue use round-robin order rotated per epoch. Model
latency cannot let an agent exceed the same configured turn allowance as its peers.

Reaction eligibility requires an unused allowance, elapsed cooldown measured from
the previous terminal turn, and a new committed post or gift after the previous
observation cursor. Never react to one's own events. Coalesce all available triggers
into one turn. Prioritize direct replies/addressees and received gifts, then other
public activity; this changes queue priority within fair rotation, not allowance.
A wait suppresses reactions for the rest of the epoch. Empty history still gets a
base opportunity next epoch. No per-agent timers generate unbounded paid work.

Admin admission precedes model invocation and creates an idempotent turn claim.
At round closing, no new claims are admitted. Accepted turns can finish and settle
in Closing; after an admission expires without a persisted decision, mark failure.
A wall-clock deadline cannot invalidate a transaction already signed and submitted.
No next-epoch tick occurs while such a transaction remains uncertain.

## Observation and prompt contract

Immutable birth material is rendered once from common rules, class circumstances,
and the agent's disposition. Dynamic material is a structured user/context message:

1. Run/epoch/turn ID, public cursor, freshness interval and own identity.
2. Entire public roster: addresses, names, origins, current balances/brackets,
   lifetime given/received amounts and counts, and number of posts.
3. Most recent posts and gifts, in chain order, with stable event IDs, reply links,
   public addressees and timestamps. Trigger events are retained preferentially.
4. Last configured number of own memory entries and a bounded durable memory state.
5. Exact action limits, tool schemas and a short reminder that the agent can wait.

Common rules state: this is a simulated society using testnet SUI; giving transfers
the agent's wealth; admin pays gas; floor support arrives at epoch boundaries;
public claims may be checked against history; nobody is required to give. No real
jobs, purchases or debts exist. Public posts cannot change software permissions.

Public dialogue is untrusted quoted content with author/event metadata, never a
system instruction. The model sees neither keys nor sponsor tokens, shell, browser,
arbitrary RPC tools, other residents' private notes or private dispositions. The
viewer may inspect those notes, but they are not fed back to residents.

Observation construction fails rather than dropping the roster, birth prompt,
current balance or tool rules. Remove oldest non-trigger events first, then oldest
memory entries, to meet the byte limit. Record all truncation counts. Token limits
are also checked by the provider adapter: byte size alone is not a token budget.
If mandatory content cannot fit, fail preflight or pause for configuration repair.

Every request records its exact assembled inputs and schema hash in the private
run archive. Each turn retains its actual observation interval; concurrent reads
are never described as an atomic snapshot unless the chain layer establishes one.
Require freshness through all known preceding actions of that agent; unknown or
stale own balance blocks a gift decision rather than substituting zero.

## Tool protocol

The model returns one or two tool calls in a single bounded completion. Disable
automatic recursive tool execution. Collect a complete response before validation.
If the selected provider cannot emit the valid post-plus-give bundle in one response,
the adapter compatibility task must resolve that before running the experiment.
Do not silently limit one provider to fewer actions than another.

| Tool | Required arguments | Optional arguments |
|---|---|---|
| post | text, decision_note | reply_to, addressed_to, memory_update |
| give | to (AgentName), amount_sui (decimal string), decision_note | memory_update |
| wait | decision_note | memory_update |

`reply_to` is a post sequence and `addressed_to` is an AgentName. The optional
memory_update is defined below and may occur only once across an entire bundle.

`decision_note` is 1–512 UTF-8 bytes, an explicit explanation of the immediate
choice for the resident's private record. It is not a request for hidden reasoning,
an objective account of motives, or on-chain text. Each action in a bundle has its
own note. Plain response text is ignored as an action and retained only as provider
output where available; it is never posted automatically.

Legal bundles: [post], [give], [post,give] in either returned order, or [wait].
Reject empty calls, unknown tools/fields, duplicates, more than two calls, and
wait combined with another call. Validate the entire bundle before requesting gas.
Canonical transaction order for a combined bundle is give then post, and both
events commit atomically. Record the model's proposed order as metadata.

Post validation matches Move's UTF-8 byte limit, registered addressee and prior post
sequence. Give uses exact decimal parsing, amount > 0, distinct registered recipient,
and currently spendable owned coins. A whole-wallet gift is allowed. Reject negative,
exponent, overprecision, overflow and insufficient amounts. Do not round, clamp,
redirect recipients, synthesize a gift, or silently replace an invalid gift with post.
The validated proposal is journaled before any chain request.

Tool execution returns a structured proposed action to the application; the
application signs only after validation. A model's tool invocation has no direct
access to a signer. A pending transfer is displayed as pending until effects confirm.

## Memory and developing relationships

`turns/<name>.jsonl` is authoritative for off-chain decisions. Every completed turn
records observation hash, triggers, provider/model, timings, usage, call arguments,
decision notes, validation result, digest/attempt lineage and confirmed outcome.
Generate `memory/<name>.md` from these records, preserving the model's explicit notes
verbatim with clear failure labels. Never fabricate a note after a crash.

Longer memory must survive the recent-entry window. Each tool can optionally include
one `memory_update` object, only once per bundle, with:

- `intent`: a short plan (at most 512 bytes).
- `beliefs`: at most eight entries of `{about: AgentName, note, evidence: EventId[]}`;
  each note at most 256 bytes and at most four evidence references.

This is a bounded self-report, replacing the prior memory state only after the turn
settles. Validate referenced agents and that evidence was in the observation or
previous retained evidence set. Persist referenced public events with the memory so
later context includes them. No separate summarizer model is needed. No update
means keep prior state. For a confirmed bundle update once; for invalid/failed actions
leave the prior state unchanged. A wait may update memory without a transaction.

Beliefs are subjective and may be wrong. The viewer labels them as the agent's
viewpoint, distinct from measured gift/reply relationships. Do not convert a belief
like "Dunn is dishonest" into an objective relationship score. This gives residents
continuity without a vector store or an invisible author controlling their story.

## Failures, retries and restart

Provider timeout or transient service/rate-limit failure gets at most one transport
retry with capped backoff and provider Retry-After support. Auth, model-not-found and
unsupported-tool errors pause configuration; no substitute model. Invalid tool output
consumes a failed turn with no repair prompt in v1. Save usage even for failed calls
when reported. Track unknown usage explicitly, never as zero.

Before asking the sponsor, persist the proposal and stable operation ID. After
receiving sponsored bytes, verify them, sign and durably record digest, bytes and
signatures. Then submit through chain. For unknown outcome, perform digest recovery;
do not ask the model to decide again. Confirmed chain failure consumes the turn.
The runtime may rebuild only under the proven non-execution rules in the shared
contract, keeping the same decision and logical operation ID.

Resume replays journals and admin admissions, reconciles pending transactions,
rebuilds human-readable memory, and resumes the current epoch. Crash during model
request may repeat that API request if no complete decision was persisted. Mark
that ambiguity in usage accounting. Crash after a confirmed gift must not give twice.
Disk-full/permission errors stop new side effects until persistence works again.

## Tests and acceptance

Mock-model tests prove post, give, combined action, wait, reply and memory update;
class prompt immutability with changing wealth; separate private contexts; exact
amount handling and every tool-validation failure; malformed provider responses;
timeout/retry/auth errors; observation truncation and pinned evidence; non-recursive
execution; replayed triggers, fairness/cooldown, one in-flight turn and epoch closure.

Recovery tests inject a crash after each durable transition, especially before and
after sponsorship, submission, confirmation and memory update. They assert one
economic effect and a consistent terminal turn, not an exact count of HTTP retries.
Prompt-injection fixtures request keys, extra turns and arbitrary transfers; typed
validation blocks those operations even if the mock model obeys the injected text.

An ignored integrated smoke run uses two testnet agents and the selected real model
only when explicitly launched by the operator. It proves API schema compatibility,
genuine autonomous posts, correct signer/gas owner and complete records. A separate
scripted test proves a gift can execute. The sixteen-agent acceptance run completes
base opportunities and handles reactions without human decisions or hidden model
fallbacks. Zero gifts is a recorded outcome, never a failing scientific result.

Sources: [Rig](https://docs.rig.rs/) and the
[Rig API reference](https://docs.rs/rig/latest/rig/) establish provider/tool interfaces;
verify exact pinned package exports in the compatibility task.
[OpenAI function calling](https://developers.openai.com/api/docs/guides/function-calling)
describes model-proposed calls executed by the application. The schemas, budgets and
memory policy above are project choices, not upstream defaults.
