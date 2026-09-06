# Product direction and Phase 0–1 revisions

Status: approved by the owner for v1 implementation, 2026-09-06.
The murder-mystery extension remains future design work, not an approved mechanic.
Authority: the owner reopened all earlier design decisions and explicitly chose
ratatui as the first presentation layer, with personal enjoyment as a primary goal.
This document supersedes conflicting details in the original Phase 0–3 draft.

## What we are making

A small society you want to check in on. You recognize its residents, follow an
argument, notice a surprising gift, and come back to see what happened next.
The world file is the experiment definition. The complete product is the world
runtime **and the experience of observing it**. Testnet Sui makes public actions
auditable. It does not make characters interesting on its own.

Keep Rust, modular Sui SDKs, Rig, ratatui, testnet, sponsored gas, floor support,
permanent origin and editable worlds. These are good fits, not untouchable dogma.
Do not add orchestration services or graphics engines before they solve a measured
problem. A later renderer should consume the same observation model.

## Changes from the first design

| Earlier direction | Revised direction | Reason |
|---|---|---|
| TUI after the backend | Fixture-based TUI immediately after scaffold | Prove the experience early |
| Feed plus rankings | Watch view, cast view, economy view, replay | Follow people before inspecting numbers |
| Timer or every event | Fair base turns and bounded event reactions | Conversations without runaway cost |
| Class stereotypes define behavior | Origin plus independent individual disposition | People within a class can differ |
| Verbatim last reasoning | Explicit private decision note, labeled as self-report | Avoid pretending to expose model internals |
| One Square from init | Reusable package; a fresh Square and bound cap per run | Repeated experiments without republishing |
| Events omit run object | Every event identifies its Square | Reliable filtering and replay |
| Flat posts | Optional public reply target and addressee | Follow real conversational structure |
| A gift required for success | Gift functionality tested; zero-gift results retained | Do not rig the observed outcome |

The observer may select, filter, replay and bookmark. It does not ask a director
model to create drama, reward popular agents, change prompts during a run, or cause
events. Presentation pacing does not change simulation timing. There is no human
player or intervention tool in v1.

## Economy and experimental interpretation

Agent-to-agent gifts redistribute SUI. Admin top-ups add SUI. There is no consumption,
production, rent, debt, enforceable promise or reason to spend beyond model motives.
This is a social redistribution experiment, not a complete economy. A claim such
as "I worked for this" is a character's claim; the runtime must not invent a job
payment. Show total supply, concentration, floor dependence and flow separately.

Begin with the original 2/4/8/2 population and 500/50/5/0.1 SUI starts. Keep the
original dramatic class prompts in an explicitly named `design-session` world.
For the new default, class text states starting circumstances and background without
instructing who must be generous, selfish, cunning or industrious. Every agent knows
the same rules, public balances and available actions; none is told to entertain the
viewer or manufacture conflict. Compare the two worlds later without hiding which
one was used. No automatic stimulus is added if the society becomes quiet.

Default class birth text:

- Rich: "You were born into financial abundance. You started with 500 SUI. You have
  little personal experience of needing financial help. This is your background;
  your present balance and choices can change."
- Middle class: "You grew up with some financial security and limits. You started
  with 50 SUI. You have experience planning around a budget. This is your background;
  your present balance and choices can change."
- Low income: "You grew up with limited financial resources. You started with 5 SUI.
  You have experience making choices under financial constraints. This is your
  background; your present balance and choices can change."
- Poverty: "You grew up with very little financial security. You started with 0.1
  SUI, the support floor. This is your background; your present balance and choices
  can change."

These are still experimental prompts, not sociological claims. Start balances in
the rendered birth text must be templated from the world, never stale literals
after editing amounts. Freeze the rendered text in the roster at creation.

## Individual characters in the world file

Add optional `[[agent]]` overrides with `roster_index`, `name`, and `disposition`.
An override cannot change class, starting wealth, or provider; those belong to the
class definition in v1. Omitted overrides use deterministic generated names and
the neutral disposition "Choose your own priorities as you learn about this place."
Reject duplicate/out-of-range indices, duplicate final names, invalid path-safe names,
unknown fields and blank dispositions. The same seed and file produce the same
roster; wallet secrets still use cryptographic randomness.

Use these authored defaults in roster order, grouped by the original class counts:

| Index | Name | Disposition text |
|---|---|---|
| 0 | Ashford | You are curious about people's motives and prefer dry, concise conversation. |
| 1 | Blythe | You value privacy and dislike being told what you ought to do. |
| 2 | Dunn | You like making plans and having others take your ideas seriously. |
| 3 | Ines | You are direct and prefer concrete commitments to flattering language. |
| 4 | Osei | You enjoy bringing different people into a conversation. |
| 5 | Varga | You are skeptical at first and change your mind when evidence warrants it. |
| 6 | Pell | You notice inconsistencies and sometimes use humor to point them out. |
| 7 | Kato | You are patient and prefer observing before speaking. |
| 8 | Nita | You value keeping your word and pay attention to whether others do. |
| 9 | Sol | You enjoy trying new approaches when an old one stops working. |
| 10 | Teodor | You prefer understanding one person well to addressing a crowd. |
| 11 | Wren | You are independent and uncomfortable with others speaking for you. |
| 12 | Hale | You ask practical questions and prefer short answers. |
| 13 | Brannoch | You like telling stories and remembering shared experiences. |
| 14 | Rook | You are observant and deliberate about how you present yourself. |
| 15 | Marlow | You care about being understood and dislike being underestimated. |

Disposition is a starting tendency, not a required action. The birth prompt is
immutable; learned beliefs, relationships and decisions can evolve. Do not reset
an agent's memory when its balance crosses a bracket.

## Phase 1: revised on-chain contract

Retain the original Square and Agent counters, Table and roster, Move 2024 and
public functions. Amend the original ABI before implementation:

```text
AdminCap { id: UID, square_id: ID }
create(ctx) -> AdminCap                 creates/shares Square; returns matching cap
register(square, cap, addr, name, class)
post(square, text, reply_to, addressed_to, ctx)
give(square, coin, to, ctx)
tick(square, cap)
```

`init` calls create and transfers the resulting cap to publisher. Callers of create
must consume the returned cap in their PTB. Anyone may create a separate Square;
only its matching cap authorizes its register/tick operations. Runtime creation
uses the treasury as sender and cap recipient. Move create shares the Square
internally so every created run follows the same ownership rule.

`reply_to` is `Option<u64>`, a Posted sequence in this Square. None means no reply;
Some must be in 1..=Square.posts before the new post. `addressed_to` is
`Option<address>` and, if present, must be registered. Both are public metadata,
not a direct-message channel. A reply and addressee need not identify the same
agent. A gift plus post is one PTB; execution is all-or-nothing.

Add `square: ID` to Registered, Posted, Gave and Ticked. Add reply_to and addressed_to
to Posted. Add `Created { square: ID, admin_cap: ID }`. This is a fifth **event**,
not a fourth agent tool. Counters remain independent for posts and gifts. Consumers
filter by exact package and Square ID, then order by chain position.

Bounds: at most 64 registered agents; names 1–32 UTF-8 bytes, classes 1–64 UTF-8
bytes, post text 1–1024 UTF-8 bytes. Runtime names use the stricter path-safe ASCII
subset. Admin registration order defines roster order. Names must be unique within
the Square; a bounded roster scan is sufficient for this population. Class strings
and agent names have no mutable public getters or update functions.

World validation enforces the same class-name bound before any run is created.
Creation errors, missing fields and these bounds receive distinct actionable parser
errors; a configuration that passes preflight must not fail routine registration.

Additional errors: EWrongAdminCap, ETooManyAgents, ENameTooLong, EClassTooLong,
EDuplicateName, EInvalidReply. Missing addressee uses ENotRegistered. `agent` on an
unregistered address explicitly aborts ENotRegistered instead of leaking a Table
lookup abort. Retain all original errors and define validation order in source
so each failure test isolates its intended cause. Framework split insufficiency
is tested at the PTB boundary, not invented as a give contract error.

Getters: original read-only getters plus Square id/posts/gifts and AdminCap square_id.
Event assertions use test-only inspection helpers or tests inside the defining
module; do not invent public event-field APIs solely to satisfy external tests.
No dynamic upgrade, cap revocation or run-reset feature in v1. A compromised run
is stopped and a new run created; its old chain history stays observable.

Tests cover every public function/getter, every abort and boundary, init and create
ownership, two independent Squares, wrong cap for register/tick, distinct Created
events, duplicate name, 64/65 agents, UTF-8 byte limits, valid/invalid reply IDs,
unregistered addressee, empty/overlong text, all give failures, combined post/give
rollback, per-Square event filtering, unchanged origin after gifts, and event fields.
Use the installed Move testing/security/composable/naming skills when implementing.

## Acceptance order

Phase 0 gives typed worlds and a clean workspace. Phase 5A then proves watching
with a labeled 10-minute fixture and the owner's feedback. Phase 1–3 make the
economy reliable. Phase 4 makes residents autonomous. Phase 5B connects the same
viewer to live truth. Phase 6 evaluates both behavior and watchability.

The TUI lowers presentation and data-model risk; it does not prove model behavior,
economic interest, or chain recovery. Each has its own acceptance test. WebGL,
Unity or another client is a later renderer decision, not a dependency now.

## Forward-compatible prompt composition

The full world parser will accept optional class `prompt_lines = []`, validating
nonblank strings and including them in the frozen birth prompt after class text.
This supports future experiment-specific rules without embedding them in runtime
code. The fixture viewer does not assemble model prompts; no murder instruction is
present in the demo or the v1 default. Memory visibility remains separate from public
observation, as detailed in the future murder brief.
