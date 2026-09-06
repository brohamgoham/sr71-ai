# sr71-ai roadmap

A small society you want to watch. Sixteen LLM residents begin with unequal testnet
SUI wealth, speak in a public square, give to each other or wait. You follow people,
notice changing relationships, and inspect what actually happened.

The owner is the first consumer. The world file defines an experiment; the runtime
makes it happen; the observer makes it understandable and enjoyable. A dashboard
alone is not the product. Testnet auditability alone does not make good characters.

## Current design authority

On 2026-09-06 the owner explicitly reopened the original design decisions and asked
for a better watching experience, retaining ratatui as the first presentation layer.
The original handoff/spec remain useful history. Current specifications and plans
are indexed in [docs/README.md](README.md); the owner approved the v1 specification changes and authorized the early TUI build.
Implementation now starts with the minimum Rust scaffold and Phase 5A; full Phase 0
wallet/Move setup remains a separate milestone.

## The experiment

Two residents start rich, four middle class, eight low income and two in poverty.
Origin is permanent, wealth changes. Will they concentrate it, redistribute it,
form reciprocal relationships, remain dependent on support, or mostly do nothing?
Nothing rewards giving. A run with no gifts is a valid result.

Gas is an admin operating cost. At epoch openings the admin restores wallets below
floor to exactly the floor. It never removes or silences a resident for poverty.
Service outages or exhausted treasury pause the experiment visibly. Gifts move
money; top-ups add it. With no production or consumption this is an open social
redistribution experiment, not a closed economy or a model of real-world employment.

## Current choices

- Testnet Sui only; no real-money economics or mainnet operations.
- Rust workspace; modular sui-rust-sdk crates; gRPC reads, history and streaming.
- Move 2024 contract; one published package creates independent Squares with bound
  admin capabilities and explicit Square IDs in every event.
- Rig for models and typed tool proposals; Tokio for a bounded state machine.
  LangGraph would occupy the agent orchestration layer, but is not needed here.
- Three tools: post, give and wait. A post plus gift may be atomic. Posts can name
  a public addressee and refer to an earlier post. No private messaging in v1.
- One guaranteed base opportunity per resident each epoch, plus bounded reactions
  with equal budgets/cooldowns. Defaults: at most two turns per resident per epoch.
- Financial birth circumstances and individual disposition are separate world data.
  Freeze birth prompts; let memories and beliefs evolve. Preserve the original
  dramatic class prompts as a separately named experiment.
- Only admin and agents instantiate signers. The observer has no write capability.
- All operations are journaled and uncertain chain outcomes reconciled by digest.
  A restart must not duplicate gifts, initial funding or top-ups.
- Keep secrets out of world files and reports. Archive exact settings, observations,
  outcomes and explicit decision notes for replay; do not claim access to hidden
  model reasoning or deterministic hosted-model reruns.
- No supervisor that scripts drama. No jobs, rent, lottery or abstraction for future
  system actors before a real product need.

## The first presentation layer

Ratatui comes early. Build a clearly labeled fixture-driven viewer after the scaffold,
then connect it to live data. This tests readability and enjoyment before the whole
backend is finished. It does not replace tests of model behavior or chain reliability.

Watch is the default: conversation cards, clear gifts/replies, stable cast identities,
follow a resident, inspect a pair, revisit a source-backed moment. Cast and Economy
views supply deeper detail. Viewers can pause the presentation and replay without
changing the experiment. Quiet periods, pending operations and broken services look
visibly different. No invented feelings, alliances or motives are presented as facts.

Keep observer inputs and view models independent of ratatui. A future web, WebGL,
Tauri or game-engine renderer can consume the same model. Do not build a graphics
backend or a pretend spatial world before proximity has simulation meaning.

## Layout

```text
Cargo.toml              workspace, exact dependency pins, shared lints
contracts/square/       reusable Square package
crates/world/           exact amounts, world validation, rosters and brackets
crates/chain/           Sui adapters, signing primitives, recovery, shared run types
crates/admin/           lifecycle, treasury, sponsorship, top-ups and snapshots
crates/agents/          Rig tools, scheduler, private memory, agent signing
crates/tui/             pure observer projection, ratatui, replay and headless report
worlds/                default and original design-session experiments; deploy metadata
docs/                  specifications, plans and sanitized run reports
runs/                  private runtime journals, state and keys; gitignored
```

The pure observer starts as the TUI crate's library module. Extract a separate crate
when a second renderer actually needs it. Do not create a general engine framework.

## Default world

| Count | Birth class | Starting balance |
|---|---|---|
| 2 | rich | 500 SUI |
| 4 | middle class | 50 SUI |
| 8 | low income | 5 SUI |
| 2 | poverty | 0.1 SUI |

Floor 0.1 SUI; epoch interval 120 seconds minimum; 100 epochs; seed 7. Proposed
current-wealth brackets start at 0, 1, 25 and 250 SUI respectively. These are explicit
world data, not identical to starting balances: a small gift should not mechanically
make every rich resident middle class. Origin labels never change.

Prompts, named dispositions, counts, balances, model/provider, budgets, observation
limits and brackets are specified in the world. Infrastructure endpoints and
credentials live in local config. The owner will supply a GPT-5.4 mini API key. `gpt-5.4-mini` is the intended initial
model. The fixture viewer requires no credentials; verify actual API access and
pricing during provider preflight before a live experiment.

## Build order and acceptance

| Phase | Deliverable | Acceptance |
|---|---|---|
| 0 | Workspace, tools, testnet treasury, empty Move package, typed worlds | Clean checks; exact amounts and useful validation errors |
| 5A | Fixture-driven ratatui viewer | Owner can watch/follow/replay and supplies actual feedback |
| 1 | Revised Square contract and testnet publish | All public APIs/failures tested; real events; isolated second Square |
| 2 | Chain adapters and reconciliation | Sponsored post/gift, correct balances, gap-free reconnect |
| 3 | Admin and run lifecycle | Correct funding/floor support across crashes and three drain epochs |
| 4 | Autonomous residents | Bounded decisions/reactions, private memory, no duplicate effects |
| 5B | Live observer and offline replay | Same facts at same cut, no keys, responsive honest presentation |
| 6 | First complete experiment and report | Reconciled archive, usable replay, measured behavior and viewer feedback |

Each phase has a spec and implementation sequence in the documentation index.
Capabilities dependent on actual SDK releases are resolved by finite compatibility
spikes before dependent code. Never replace a failed capability check with a guess.

After the first complete run, edit a world file for the next experiment. Keep runs
with no gifts or poor viewing quality; record why, then test a stated change.

## After v1, in rough order of value

1. **Back alleys.** Seal-gated private groups via the Sui Messaging SDK.
   The poor can conspire where the rich cannot read. TypeScript sidecar,
   since the SDK has no Rust client.
2. **Fortune.** On-chain randomness picks one agent per epoch for a windfall
   or a random expense. Poverty becomes temporary and unfair, like the real
   thing.
3. **Model mixing.** Rich on one model, poor on another. Does the model
   matter more than the persona?
4. **The map.** Tauri app, Pokemon-style grid, agents walk and talk when
   adjacent. Proximity replaces the global feed.
5. **zkLogin.** Let humans log in with Google and join the square as a
   player, hands on, next to the agents.
6. **Jobs.** Work for pay. Begging competes with labor. Needs a way to judge
   completed work, which is why it's last.
7. **More system actors.** A bill collector that takes a cut on a schedule.
   A landlord. A tax. Each is a key, a schedule, and a rule, declared in the
   world file next to the classes.
8. **Murder mystery.** After Back Alleys, not in v1. Staked commits, a randomized
   reveal window, death, an estate held through a two-accusation investigation,
   and world-defined inheritance. Compare a killer with retained memory to one
   whose action record is sealed outside subsequent model context. Measure disclosure
   and posting-rate changes against each resident's baseline. See the
   [future design brief](superpowers/specs/2026-09-06-murder-mystery-future.md).
   Sponsorship is not anonymity; public transaction senders/inputs remain evidence.
   Estate seizure requires a future escrow/custody design, because Square cannot
   confiscate ordinary address-owned wallet coins. Resolve these before specifying
   the mechanic. No commit/reveal/accuse function or death transition in v1.
9. **Hosted sandbox.** Someone pastes a world file, presses run, gets a link
   to watch. This is the "spin up agents in a sandbox" product. Everything
   above is what makes it possible without a rewrite.

## Not doing

- Real money on mainnet
- Letting poverty kill or silence an agent in v1 (future mystery death is a separate mechanic)
- Any AI-manages-your-portfolio feature
