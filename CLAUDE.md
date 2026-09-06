# sr71-ai

LLM agents with testnet SUI wallets living in a shared world. Some are born
rich, some born in poverty. Nothing forces anyone to give. We watch.

Read `docs/handoff.md` first, then `docs/roadmap.md`. The roadmap is the
source of truth for what this project is. Specs for each phase live in
`docs/superpowers/specs/`, plans in `docs/superpowers/plans/`. Do not start
a phase without its spec.

## Current architectural direction

The owner reopened the original design on 2026-09-06. Improvements are welcome;
document tradeoffs and update the specifications before changing implementation.
Read `docs/README.md` for the current spec set, revisions and build order. The TUI
prototype now comes immediately after Phase 0; the owner is the first viewer.
Testnet and read-only observer boundaries remain the current intended constraints.

- **Testnet only.** Never mainnet. Never real money.
- **Rust workspace** for admin, agents, TUI. Move for the contract. TypeScript
  only where a Sui Stack SDK forces it (not in v1).
- **Sui SDK:** the modular `sui-rust-sdk` crates (`sui-sdk-types`,
  `sui-crypto`, `sui-rpc`, `sui-transaction-builder`). Never the legacy
  monorepo `sui-sdk` crate. gRPC for reads and event streams, not JSON-RPC.
- **LLM layer:** `rig-core`. Tools are typed Rust structs via Rig's derive
  macro. Tests use Rig's mock model, never a live API key.
- **TUI:** `ratatui`. It holds no keys and never writes to chain.
- **The world is a file.** `worlds/*.toml` defines classes, counts, start
  balances, prompts, models, floor, epoch length. Experiments are edits to
  that file, never code changes.
- **Origin is permanent, wealth is not.** A class prompt is fixed at birth.
  Balance moves. Both are shown.
- **Admin pays gas.** Every agent transaction is a Sui sponsored transaction
  with the admin as gas owner. The admin tops wallets up to a floor and no
  higher. No agent ever dies or goes silent.
- **Only admin and agents hold keys.** Nothing else signs.
- **The admin is the first system actor,** not the only one. Build it so a
  bill collector or lottery later is a sibling, not a special case. Do not
  build the abstraction yet.

## Layout

```
Cargo.toml              workspace root, lints live here
contracts/square/       Move package: Square shared object, register, post, give
crates/chain/           build, sign, stream events, read balances
crates/admin/           treasury, gas sponsorship, floor top-ups, epoch tick
crates/agents/          rig tools, decide loop, memory files
crates/tui/             ratatui, read-only
worlds/                 world files
docs/                   roadmap, specs, plans, run write-ups
```

## Code rules

- Rust: `cargo clippy --all-targets --all-features -- -D warnings` must be
  clean. `cargo fmt`. `cargo deny check`. Lints are in the workspace
  `Cargo.toml`; do not weaken them. `unwrap` is denied, `expect` warns.
- Functions under 100 lines, at most 5 parameters, 100-char lines.
- `thiserror` in library crates, `anyhow` in binaries. `tracing`, never
  `println`. Errors say what operation, what input, and what to do.
- Newtypes over primitives (`AgentId`, `Sui` amount). Enums for state, not
  bools. No wildcard `match` arms.
- Move: 2024 edition syntax. Unit tests for every public function including
  failure paths. Follow the installed `naming-conventions`,
  `composable-move-functions`, and `move-security` skills.
- Tests verify behavior, not implementation. Every error path has a test
  that triggers it.
- No speculative features, flags, or abstractions. Three real uses before a
  helper exists.
- Commits: imperative mood, one logical change, never to `main` directly.

## Sui Development Skills

Install community-maintained skills for Sui development:

```sh
npx skills https://github.com/MystenLabs/skills
```

They are already installed under `.agents/skills/` and `.claude/skills/`.
Use them before guessing at any Sui or Move API.

## Official Resources

When unsure about Move patterns or Sui APIs, consult these sources. Do not guess or
extrapolate from other blockchains.

- Move Book: https://move-book.com (use https://move-book.com/llms.txt)
- Sui Docs: https://docs.sui.io (use https://docs.sui.io/llms.txt)
- Sui Move examples: https://github.com/MystenLabs/sui/tree/main/examples/move
- Sui docs MCP server, when available: `https://sui.mcp.kapa.ai`
- Rust SDK: https://mystenlabs.github.io/sui-rust-sdk/ and docs.rs per crate
- Rig: https://docs.rig.rs
