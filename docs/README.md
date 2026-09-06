# Build documentation

The product and current direction live in [the roadmap](roadmap.md) and
[`CLAUDE.md`](../CLAUDE.md). [The handoff](handoff.md) records the starting state.

## Specifications

| Document | Responsibility |
|---|---|
| [Phase 0–3 original design](superpowers/specs/2026-09-06-phases-0-3-design.md) | Phase 0–1 baseline: parser, contract, tests |
| [Product and contract revision](superpowers/specs/2026-09-06-product-and-contract-revision.md) | Viewer-first direction, revised world and Phase 1 contract |
| [v1 runtime contract](superpowers/specs/2026-09-06-v1-runtime-design.md) | Cross-phase rules, files, chain, admin; completes Phase 2–3 |
| [Agents](superpowers/specs/2026-09-06-phase-4-agents-design.md) | Phase 4: Rig, turns, tools, memory, recovery |
| [Observer and experiment](superpowers/specs/2026-09-06-phases-5-6-design.md) | Phase 5–6: TUI, accounting, replay, experiment report |

The owner approved the v1 specifications and authorized building the ratatui
prototype. The original design-session spec is superseded where the revisions say
so. Start with the Rust scaffold needed by Phase 5A; wallet configuration and the
Move contract are not prerequisites for fictional fixture playback.

[Murder-mystery future brief](superpowers/specs/2026-09-06-murder-mystery-future.md)
records the intended evolution and its privacy/custody constraints. Only the listed
v1 observation hooks are in scope now; no murder mechanic ships in v1.

## Plans

- [Phase 0–1 implementation plan](superpowers/plans/2026-09-06-phases-0-1-plan.md).
- [Phase 2–6 implementation sequence](superpowers/plans/2026-09-06-phases-2-6-plan.md).

These documents cover all of **v1, Phases 0–6**. Back alleys, jobs, lotteries,
Tauri, human players, and hosted deployment remain future work, without invented
implementation commitments.

The owner authorized **Phase 5A, a fixture-driven ratatui prototype, with the minimum
Rust scaffold first**. Complete the rest of Phase 0 before Phases 1–4, Phase 5B live
integration, and Phase 6. The owner
is the first consumer: test watchability before completing the backend.

## Implementation notes

- The owner will provide a GPT-5.4 mini API key. Actual access is verified during
  Phase 4 preflight; no key is needed for the TUI prototype.
- Exact compatible crate versions and RPC capabilities. Implementation spikes have
  explicit exit conditions in the plans; API-shaped sketches are project contracts,
  not promises that similarly named upstream methods exist.

Implementation status and verification are recorded in [the handoff](handoff.md).
