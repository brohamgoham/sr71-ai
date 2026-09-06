# The Square

A small society to watch. Sixteen residents start with unequal wealth, talk, give,
or wait. The first presentation layer is a read-only ratatui viewer.

**Built now: a fictional viewing prototype.** It does not run LLMs, connect to Sui,
spend money, or load keys. The persistent DEMO label is intentional.

```sh
cargo run -p tui -- demo
```

The ten-minute recording starts immediately. For a quick look at the first change
in Rook's fortunes:

```sh
cargo run -p tui -- demo --at 135
```

Use a 120×40 terminal for the full Watch layout. Smaller terminals down to 80×24
have a compact view. Rust is pinned to 1.97.1 for rustup users; this NixOS workspace
also works with its installed Cargo. No API key or wallet configuration is required.

## Watch it

| Key | Action |
|---|---|
| `1` / `2` / `3` | Watch / Cast / Economy |
| `↑` / `↓` or `k` / `j` | Select a resident; Rook is selected initially |
| `f` | Follow or release the selected resident |
| `Enter` | Inspect biography and source records |
| `Space` | Pause/resume your view; the demo clock continues |
| `r` / `g` | Replay from the start / return to the demo clock |
| `←` / `→` | Step to previous/next event |
| `[` / `]` | Revisit a recorded moment |
| `s` | Cycle replay speed: 0.5×, 1×, 2×, 4× |
| `PageUp` / `PageDown` | Read older/newer cards |
| `/` | Search public text or resident name |
| `Esc` | Close overlay, clear filter, or release follow |
| `?` / `q` | Help / quit |

Gifts and their accompanying posts group only when their transaction IDs match.
Admin floor support is separate from resident gifts. Failed attempts do not change
balances. Decision notes are self-reports, not verified motives or hidden reasoning.
Sealed notes show a reference only; this demonstrates display behavior, not encryption.

## Other scenarios and headless previews

```sh
cargo run -p tui -- demo --scenario quiet
cargo run -p tui -- demo --scenario reconnect --at 180
cargo run -p tui -- demo --scenario burst --at 12
cargo run -p tui -- demo --no-color --ascii --reduced-motion
cargo run -p tui -- demo --at 167 --snapshot /tmp/square.txt
```

`--fixture path.json` loads another recording using the strict version 1 schema in
[the fixture directory](crates/tui/fixtures). Unknown event types, malformed money,
invalid identities, chronology errors and impossible transfers fail before playback.
All supplied fixtures are authored examples, not experimental results.

Logs go to `.local/viewer.log`. No raw escape sequences from resident text are sent
to the terminal. The viewer restores terminal mode when it exits.

## Build status

Implemented: workspace lints, exact money units, pure serializable observer state,
four fixtures, Watch/Cast/Economy, following/filtering, playback/seek, notes and source
inspection, per-agent posting rates, accessible layouts, snapshots and behavior tests.

Pending: the complete world-file parser and remaining Phase 0 tool/wallet setup;
Move contract; chain/admin/agent runtime; live viewer integration and real experiments.
GPT-5.4 mini is the intended initial model once the owner supplies API access.

The [murder-mystery brief](docs/superpowers/specs/2026-09-06-murder-mystery-future.md)
records the future direction and necessary privacy/custody corrections. No murder,
death transition, estate, commit/reveal/accuse action or encryption service is built.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check
```

Visual regressions use reviewed text snapshots at 120×40, 100×30 and 80×24. To
intentionally regenerate them, run `UPDATE_SNAPSHOTS=1 cargo test -p tui --test viewer`,
then inspect the changes. Passing snapshots is not a substitute for the owner's
feedback on whether watching is enjoyable.

See the [documentation index](docs/README.md) and [handoff](docs/handoff.md) for scope,
approved specs, implementation plans and remaining work.
