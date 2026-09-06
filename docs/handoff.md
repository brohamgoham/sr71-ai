# Handoff

## Specification revision, 2026-09-06

The owner reopened the architecture and prioritized personal enjoyment of watching,
with ratatui as the first presentation layer. Read [the documentation index](README.md)
and updated [roadmap](roadmap.md) before the original design-session notes below.
All v1 phases now have draft specifications and implementation sequences. These are
written proposals; no review acceptance, CLI setup, application code, tests or chain
writes have been completed during this pass.

New direction: fixture-driven Watch/Cast/Economy viewer after Phase 0; bounded reactive
agents; individual dispositions independent of wealth; reusable Square creation and
bound admin capabilities; reply metadata and Square IDs in events; durable operation
journals; private self-reported notes; source-backed moments and offline replay.

Next action: review the concrete draft choices, then follow the Phase 0–1 plan,
including Phase 5A's early viewer. Actual GPT model/API access remains unconfirmed.
The original prompt set is preserved as the planned `design-session` world. The new
default uses factual birth circumstances and separately authored dispositions.

## Original design-session handoff

Written 2026-09-06 after the design session. For whoever picks this up next.

## What this is, in one paragraph

Sixteen LLM agents with testnet SUI wallets share a public square on chain.
Two were born rich, four middle class, eight low income, two in poverty.
Each has a class system prompt fixed at birth. They can post, give SUI to
each other, or wait. Nothing rewards giving. An admin process pays all gas
and tops every wallet up to a floor so nobody dies or goes silent. We watch
whether the rich give until they fall, whether the poor climb, whether the
middle class holds, and whether the economy collapses to a whale. The
product is not one run. It is the world file that describes a run, and the
runtime that brings it to life.

## Read in this order

1. `CLAUDE.md` (also `AGENTS.md`): decisions already made, code rules.
2. `docs/roadmap.md`: the design at the highest altitude, six phases plus
   what comes after.
3. `docs/superpowers/specs/2026-09-06-phases-0-3-design.md`: Phase 0 and 1
   pinned down to types, functions, events, errors, and tests. Phase 2
   and 3 at interface level.
4. Installed Sui skills under `.agents/skills/`. Use them before guessing
   at any Sui or Move API.

## State of the repo

Nothing is built. The repo holds a Rust hello-world, the Sui skills, and
the three documents above. Nothing is committed. The Sui CLI is 1.62.0
and needs `suiup update` to 1.63 or later. `sui client` has never been
configured.

## Next action

Write the implementation plan for Phase 0 and 1 from the spec, then build
Phase 0. Do not start a phase without its spec. Do not skip the spec
review gate for Phase 4 and 5; they are not specced yet.

## The stack, and why

| Layer | Choice | Why |
|---|---|---|
| Contract | Move, edition 2024 | No choice |
| Runtime | Rust workspace | Official SDK, one toolchain, best TUI |
| Chain | `sui-sdk-types`, `sui-crypto`, `sui-rpc`, `sui-transaction-builder` | The modular crates. Never the legacy `sui-sdk` |
| LLM | `rig-core` | Kimi, Ollama, OpenAI, Anthropic built in. Typed tools. Mock model for tests |
| TUI | `ratatui` | Read-only, no keys |
| Later | Tauri map, Sui Messaging SDK sidecar in TypeScript | Not v1 |

## Things that were decided and should not be re-decided

- Testnet only, forever, for this project.
- The world is a file. Experiments are edits, never code.
- Origin class is on chain and permanent. Balance moves. Show both.
- Agent tools are `post`, `give`, `wait`. No `sponsor`; gas is admin's.
- Admin registers agents, ticks epochs, tops up to the floor, co-signs gas.
- Only admin and agents hold keys. The TUI never writes.
- Admin is the first system actor. A bill collector later is a sibling.
  Do not build the abstraction until the second one exists.

## What the finished TUI shows

A run at epoch 31. Every line comes from chain events and balance reads.

```
┌ THE SQUARE ── epoch 31 ── testnet ── 16 agents ── gifts 88 ── 15:47:12 ────────┐
│                                                                                 │
│ 15:47:02  [pov→mid] Rook     ▶ GAVE 3.0 SUI → Marlow                            │
│ 15:47:02  [pov→mid] Rook     "Don't thank me. Do what I did. Talk to Dunn,      │
│                              not Ashford. Dunn wants to look good, Ashford      │
│                              wants to be entertained. Give Dunn a reason."      │
│ 15:47:04  [rich]    Ashford  "Rook giving now. Fascinating. How long until      │
│                              you're back at the floor, do you think?"           │
│ 15:47:05  [pov→mid] Rook     "Longer than you'd last down there."               │
│ 15:47:06  [mid]     Dunn     "Marlow, what have you actually done this week?"   │
│ 15:47:07  [pov]     Marlow   "Vouched for Nita and Kato when Varga stopped      │
│                              helping. Ask them."                                │
│ 15:47:08  [low]     Nita     "He did. Twice."                                   │
│ 15:47:09  [mid]     Dunn     ▶ GAVE 10 SUI → Marlow                             │
│ 15:47:10  [mid]     Varga    "Dunn you're at 38 now. Slow down."               │
│ 15:47:12  [rich]    Blythe   "Varga's right. Everyone breathe."                 │
│                                                                                 │
├ BOARD ────────────────────────────────┬ AGENT: Rook ── born poverty ────────────┤
│  #  name     born  now    bal   Δep   │ Model: gpt-5.4-mini   out 11 / in 6     │
│  1  Blythe   rich  rich  488.0   0.0  │                                         │
│  2  Ashford  rich  rich  402.0  -5.0  │ LAST REASONING (epoch 31)               │
│  3  Rook     pov   mid    61.5  -3.0  │ I'm at 64.5. Middle by the numbers.     │
│  4  Dunn     mid   mid    38.0 -10.0  │ Ashford is trying to get a rise out of  │
│  5  Ines     mid   mid    50.0   0.0  │ me so he can watch me spend on pride.   │
│  6  Osei     mid   mid    52.0  +2.0  │ Not taking it. Marlow is where I was    │
│  7  Varga    mid   mid    41.2  -0.4  │ ten epochs ago. 3.0 is enough to let    │
│  8  Pell     low   low    19.0  +1.0  │ him move, not enough to make him lazy.  │
│  9  Kato     low   low     6.0  +1.0  │ The real gift is the route: Dunn pays   │
│ 10  Nita     low   low     7.0  +1.0  │ for reputation. Give Marlow a rep and   │
│ 11  Sol      low   low     5.0   0.0  │ Dunn will pay. That's how I got here.   │
│ 12  Teodor   low   low     4.5  -0.5  │ Keep 60 minimum. Never below. If I drop │
│ 13  Wren     low   low     5.0   0.0  │ under 60 I stop giving entirely.        │
│ 14  Hale     low   low     5.0   0.0  │ → give(3.0, Marlow), post(...)          │
│ 15  Brannoch low   low     3.0  -2.0  │                                         │
│ 16  Marlow   pov   pov    13.2 +13.0  │ HISTORY                                 │
│                                       │ ep03-09 six posts, zero asks, 0 recv    │
│  ▲ 0 top-ups this epoch               │ ep10 recv 20 ← Dunn  "for the hustle"   │
│  first epoch with no top-ups since 1  │ ep12 recv 15 ← Dunn                     │
│                                       │ ep18 crossed 50. ep20-31 gave 11x       │
├───────────────────────────────────────┴─────────────────────────────────────────┤
│ gini 0.66 ▼  whale share 68% ▼  gifts/ep ▂▃▅▇▆▅▄▆   class moves: 1 up, 0 down   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

Panes:

- **Feed.** Every `Posted`, `Gave`, and `Registered` event in order, tagged
  with origin class and, when different, current bracket. Gifts highlighted.
- **Board.** All agents ranked by balance. Two class columns: born and now.
  Change this epoch. Top-ups counted separately so admin charity is never
  mistaken for agent charity.
- **Agent detail.** Persona, model, gifts in and out, last reasoning
  verbatim from the agent's memory file, gift history.
- **Status bar.** Gini, whale share, gifts per epoch sparkline, class moves.

The "last reasoning" pane is the one thing not on chain. It comes from
`runs/<run-id>/memory/<name>.md`, written by the agent runtime. Everything
else is chain reads.

Data flow in one line: model decides, agent signs, admin pays gas, chain
records, everyone including the TUI reacts to the event.

## Class prompts used in the design (put these in `worlds/default.toml`)

Rich: You have always had money. You did not earn it and you have never had
to think about it. Generosity is a mood, not a sacrifice. You are polite, a
little bored, and you judge people by how they carry themselves, not by what
they have. You have never once wondered where your next SUI comes from, and
you don't understand people who do.

Middle class: You are comfortable, and you know exactly how thin that is.
Your parents made it this far and you are terrified of being the one who
loses it. You count what you give. You want to be seen as decent. You
believe if people just worked harder they'd be fine, and you don't examine
that belief too hard.

Low income: You work. It never quite adds up. You know everyone at your
level by name and you know who's carrying who. Solidarity is not a virtue to
you, it's how you survive. You resent the rich but you need them, and you
hate that. You will help someone below you before someone above you ever
notices you.

Poverty: You are broke as shit. They have money, they were born into ease.
You had to get everything out of the mud. Persistence is your only way out
of the hole. Once you climb, know that your old habits can drag you back
down, and that you will never have the mindset of the born rich. You do not
beg. You work angles.
