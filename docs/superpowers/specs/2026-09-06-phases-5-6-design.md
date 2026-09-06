# Phases 5A, 5B and 6: something worth watching

Status: approved by the owner for v1 implementation, 2026-09-06.
The murder-mystery extension remains future design work, not an approved mechanic.
Depends on the [product revision](2026-09-06-product-and-contract-revision.md) and
[runtime contract](2026-09-06-v1-runtime-design.md).

## Experience goal

The owner is the first viewer. Within a minute they can identify the people in the
current exchange, tell what happened, and follow one resident. After ten minutes
they can name a moment they care about and revisit its evidence. More animation or
more model calls alone does not satisfy that goal.

The default is Watch. Economy and operational diagnostics are secondary tabs.
The cast has stable visual identities so rank changes do not shuffle away the person
the viewer was following. A quiet square is allowed; the interface should clearly
distinguish residents waiting from a stalled model, disconnected chain or paused run.

## Illustrative layout at 112 columns

This is fictional content demonstrating presentation, not an observed run. Brackets
and balances below use the proposed thresholds. Avoid displaying an invented
probability, emotion, alliance or promise as fact.

```text
 THE SQUARE   DEMO                                      epoch 31     [1 Watch]  2 Cast  3 Economy
 ─────────────────────────────────────────────────────────────────────────────────────────────
 FOLLOWING ROOK                                      THE CAST                  recent activity
 born poverty · now middle class                    AS Ashford       402.0    spoke
                                                    BL Blythe        488.0    waiting
 ROOK  → MARLOW                          15:47:02    DU Dunn           38.0    gave
 Gave 3.0 SUI     64.5 → 61.5                        IN Ines           50.0    observing
 "Don't thank me. Talk to Dunn."                     OS Osei           52.0    spoke
 [confirmed · gift #88 · post #241]                   VA Varga          41.2    spoke
                                                    ... 10 more
   DUNN replied to ROOK                  15:47:06
   "Marlow, what have you done this week?"           ROOK & MARLOW
                                                    Rook → Marlow     3 gifts · 8.0 SUI
   NITA replied to DUNN                  15:47:08    Marlow → Rook     0 gifts
   "He did. Twice."                                 [inspect source events]

 DUNN  → MARLOW                          15:47:10    ROOK'S NOTE · self-report
 Gave 10.0 SUI     48.0 → 38.0                        "I want Marlow to have room to act."
 [confirmed · gift #89]
                                                    MOMENTS
 VARGA                                  15:47:12    ep18  Rook entered middle class
 "Dunn, you're at 38 now. Slow down."                ep31  Rook gave to Marlow again
 ─────────────────────────────────────────────────────────────────────────────────────────────
 LIVE ●   chain current   2 deciding    11 waiting      4 new events while you were reading
 space pause view   f follow   enter inspect   [ ] moment   r replay   g live   ? help   q quit
```

A title such as "Rook gave to Marlow again" is a deterministic description of events.
It does not assert why. Post-plus-gift cards group by transaction digest; unrelated
adjacent posts are not attached to a gift as its explanation. Replies use on-chain
reply metadata. Ordinary mention text is not proof of a reply.

## Watch, Cast and Economy

Watch gives the active conversation most space, with an optional selected resident
or pair alongside. The full public feed remains accessible. Following a resident
includes their own actions, received gifts and direct replies; excluded events remain
in the feed with a visible count. No automatic camera change while the viewer is
reading or has pinned a resident. Auto-follow, if enabled by the viewer, selects
confirmed moments and has an eight-second minimum dwell time.

Cast shows all residents in stable roster order, origin, current bracket, balance,
last action and activity state. Sort by balance is an explicit view option; selection
tracks AgentId through reorderings. Enter opens biography, self-reported beliefs,
balance history, public reply history and directed gift totals for a selected pair.
Show factual relationship measures, not a computed "friendship" score.

Economy shows totals, class transitions, top-ups, inequality and gas separately.
An admin transfer is visually distinct from a resident gift. Diagnostics show RPC
freshness, model failures, usage, pending transactions and pause reason. The default
Watch status displays only what helps distinguish thinking, waiting and broken.

Activity states come from runtime records: Scheduled, Observing, Deciding,
AwaitingConfirmation, Waiting, Failed, Unknown. Never animate "thinking" after a
stale record as if a provider is still active. Without runtime access the TUI shows
chain actions and Unknown activity; it still works as a read-only chain observer.

## Moments and readable pacing

Create deterministic moment records for first gift, first gift between a pair,
largest gift so far (strict increase), bracket crossing, richest-agent change, and
completed epochs with zero top-ups. Each carries rule version, involved AgentIds,
event/snapshot references and source position. A missing history segment disables
"first" and "largest so far" claims until completeness is restored.

Do not assert a broken promise, betrayal, deception or motive from dialogue parsing
in v1. Quotes and actual transfers let the viewer make that judgment. No narrator
model chooses events or manufactures dialogue. Add semantic interpretation only
later, explicitly labeled and evaluated against evidence.

Live ingestion and presentation run independently. Cards have a small insertion
transition (at most 200 ms) and gifts briefly accent the participants' balances.
No per-character typewriter effect and no forced delay before confirmation appears.
The viewer can pause scrolling; events continue buffering and a backlog count grows.
Jump to live is explicit. Never drop an event to keep an animation smooth.

No ASCII world map until location has actual simulation meaning. Stable avatars
(initials), sparklines, reply indentation and transfer cards provide terminal-scale
visual identity without pretending people occupy simulated rooms.

## Controls, responsiveness and accessibility

1/2/3 switch tabs; arrows or j/k move selection; Enter inspects; Esc returns; f pins
follow; Space pauses presentation; g goes live; r enters replay; [ and ] select
moments; / filters names/text; ? shows keys; q/Ctrl-C exits the observer only.
Replay supports 0.5x, 1x, 2x and 4x plus event stepping. It never changes live engine
speed, sends a sponsor request or signals the admin to pause.

At 120×40 use the full split view; at 100×30 narrow the side panel; at 80×24 use
one main panel with tab navigation and compact status. Below 60×15 show a resize
message while preserving state. UTF-8 grapheme/display width drives wrapping;
truncate names with a visible ellipsis only in views, never in identity matching.
Terminal-control sequences, ESC and dangerous control characters in posts/notes are
rendered as harmless visible text, never emitted as terminal instructions.

Color is optional. Origin and action type have textual labels; support no-color,
reduced-motion and ASCII-border modes in local viewer settings. No emoji-dependent
alignment. Redraw only when dirty, with a 20 fps animation ceiling; no busy-spin loop.
Resize preserves selection and scroll anchor. Error/panic/exit paths restore raw
mode, cursor and alternate screen. Application tracing writes to a file while TUI
owns the terminal.

## Presentation boundary

Use `ObserverInput` as a versioned enum over ChainEvent, BalanceObservation,
RunStatus and AgentTurnStatus. Every item carries run identity, source position or
record sequence, provenance and timestamps. Sources are ConfirmedChain, Runtime,
Derived or Fixture. Unknown fields/version mismatches fail with an actionable message.

A pure reducer consumes these items into `ObservedWorld`. It produces serializable
view models: AgentCard, Conversation, TransferCard, RelationshipSummary,
Moment and EconomySnapshot. Ratatui converts them into widgets. The live adapter,
fixture adapter and replay adapter share the same reducer. No ratatui types, signing,
provider calls or terminal dimensions appear in the observed data model.

Chain-derived facts are authoritative for balances, origin and economic actions.
Runtime files supply activity and private self-reports; derived records retain
source references. TUI may read and verify these files without acquiring signing
capability. A future web or game client can serialize this model, but no HTTP server,
WebGL, Unity integration or general plugin protocol is built yet.

## Replay and metric definitions

Replay uses event source order, confirmed snapshot positions and recorded runtime
timestamps with a stable tiebreaker `(source, writer sequence)`. A seek rebuilds the
reducer from baseline and records through the chosen cut; sixteen agents do not
require a new database. Private notes appear only after their original availability,
never as hindsight leaking into an earlier view. Persist display preferences and
bookmarks locally, outside authoritative run records.

For agent balances b_i and n agents, compute with integer mist and checked u128
intermediates, converting only for display:

- Total wealth W = sum(b_i); no treasury balance in this total.
- Whale share = max(b_i)/W; show N/A if W is zero.
- Gini = sum_i sum_j |b_i-b_j| / (2*n*W); define zero when W=0, with zero-total label.
- Gifts per epoch = confirmed Gave count tagged with that Square epoch.
- Top-up amount/count = verified admin floor transfers, excluding initial funding.
- Balance delta = closing balance minus previous closing balance (epoch 1 vs baseline).
- Net social flow = gifts received minus gifts sent; distinct from balance delta.
- Floor dependence = agents topped up / population for that epoch, plus SUI added.
- Class moves = adjacent completed-snapshot bracket changes, with threshold labels;
  intra-epoch crossings remain separate moments.

Whole-run conservation: ending wealth = starting wealth + confirmed top-ups +
external net flow + any other verified non-gift agent balance adjustments. Internal
gifts cancel. Admin gas charges are excluded from agent wealth and measured from
effects. Show storage rebates/other adjustments explicitly if the protocol credits
an agent; do not assume every balance change is a gift or top-up.

Metrics become Incomplete when source continuity or snapshot consistency is missing.
Never recompute Gini with the missing agent treated as zero. For proposed thresholds,
488 SUI is rich and 13.2 SUI is low income; the older mockup's labels were
illustrative, not usable bracket rules.

## Phase 5A: early presentation prototype

Build after Phase 0 with no wallet, chain service or model key. Fixture files contain
the versioned ObserverInput schema and a persistent DEMO badge. Include a ten-minute
story with replies, several characters, a gift, a crossing, quiet periods and a
pending→failed transfer. Include a burst fixture, an all-wait fixture and a
disconnect/reconnect fixture. Never archive these as real experiments.

Done when the owner can watch, follow a person, inspect a gift's origin, pause, catch
up and replay a moment without reading docs. Gather actual feedback on readability,
pacing, recognition and whether they wanted to keep watching. Do not invent a passed
user test; implementation can pass automation while this feedback remains pending.
This is a reviewable prototype, not permission to build an unspecced backend.

## Phase 5B: live observer

Wire chain and runtime adapters to the same reducer. Pending turns never increment
confirmed gift counters; reconnect deduplicates events; missing local memory displays
"No decision note available." Notes and activity can lag independently of chain
freshness. Starting without keystore permissions must succeed. Quitting leaves the
experiment running. Replay of an archived real run works completely offline.

Tests cover reducer idempotence, ordering, reply grouping, confirmed vs pending,
moment evidence, all metric boundaries, missing history, file truncation/corruption,
selection stability, replay seek, terminal control injection and no-color layouts.
Use ratatui TestBackend for representative screen snapshots at 120×40, 100×30 and
80×24, plus reduced size; test keyboard behavior rather than private widget structure.
Test with a burst of 1,000 fixture events: no event loss and responsive input; record
machine-specific timing, targeting input response below 100 ms. Manually verify
resize and terminal restoration. Do not equate snapshot approval with enjoyment.

## Phase 6: first experiment and run report

Commands to implement: `world check <file>`; `admin prepare --world ... --deploy ...`;
`admin run --run ...`; `agents run --run ...`; `tui watch --run ...`;
`tui replay --run ...`; `tui report --run ... --out docs/runs/<id>`.
Keep `cargo run -p world -- worlds/default.toml` as the Phase 0 roster command.
Report is a read-only subcommand, requires no terminal and holds no keys.

Preflight validates tests, model/tool capability, testnet identity, funds, world,
storage and the chosen package. Record model access and estimated/requested usage
budgets before an explicit live run. Run a short two-agent smoke, then the configured
sixteen-agent world for its declared epochs. Retain failures and no-gift runs.
Never covertly swap models or edit personas to obtain a more exciting result.

Archive world, sanitized manifest, roster, public event/effect evidence, snapshots,
turn records/explicit notes, and report. Exclude keys, tokens, signed pending bytes,
local credential paths and environment values. Include hashes and schema versions.
The report command must write to a new output directory and refuse to overwrite a
different archive. A truncated run is Incomplete with reason, not a successful run.

Report fields: run conditions and duration; initial/final wealth; per-agent origin
and bracket movement; gifts vs top-ups; concentration/Gini series; floor dependence;
model usage and unknown usage; invalid turns/provider errors; external transfers;
source-backed moments; limitations and unresolved transactions. Include whether the
owner watched, which moments they found interesting, and any readability/pacing
feedback they actually supplied. Do not generate favorable viewer feedback.

Engineering acceptance: coherent archive, reconciled balances, no duplicate effects,
complete source references, offline replay and all planned failure tests pass.
Scientific interpretation: prompts are interventions, dispositions are confounders,
hosted outputs are stochastic, and sixteen residents in one run do not establish a
general result about people or model classes. Reproduce the configuration and replay
recorded events; do not promise exact reruns. Compare seeds, prompt variants and models
in separately labeled runs only after the first end-to-end run works.

Sources: [Ratatui application patterns](https://ratatui.rs/concepts/application-patterns/)
and [snapshot testing](https://ratatui.rs/recipes/testing/snapshots/) support separating
state/update/render and terminal-buffer tests. The screen, controls, metrics and
watchability criteria above are product design proposals.
