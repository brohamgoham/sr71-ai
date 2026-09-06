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

The new specifications are **drafts for review**, not an assertion that a review
has occurred. The owner explicitly reopened the earlier architecture during this
specification pass. New choices are enumerated in the product revision and runtime
contract. Where a new draft refines an older
interface or mockup, that refinement is explicit; the roadmap remains the product
source of truth. Review the drafts before building their phases.

## Plans

- [Phase 0–1 implementation plan](superpowers/plans/2026-09-06-phases-0-1-plan.md).
- [Phase 2–6 implementation sequence](superpowers/plans/2026-09-06-phases-2-6-plan.md).

These documents cover all of **v1, Phases 0–6**. Back alleys, jobs, lotteries,
Tauri, human players, and hosted deployment remain future work, without invented
implementation commitments.

Build order now includes **Phase 5A, a fixture-driven ratatui prototype, immediately
after Phase 0**, then Phases 1–4, Phase 5B live integration, and Phase 6. The owner
is the first consumer: test watchability before completing the backend.

## Questions that do not block drafting

- Which GPT model and API access the owner has. Keep `gpt-5.4-mini` as the original
  example identifier until confirmed; never silently replace it or claim access.
- Exact compatible crate versions and RPC capabilities. Implementation spikes have
  explicit exit conditions in the plans; API-shaped sketches are project contracts,
  not promises that similarly named upstream methods exist.

No application code, CLI installation, wallet setup, or chain write was performed
as part of this specification pass.
