# JFT

JFT is an evidence-gated research program for an NSE Cash Market short-horizon quantitative trading system. Phase 1 fixture-first engineering is active while the remaining non-legal Phase 0 data qualification continues in parallel. Legal/compliance is an external operational dependency, not a repository engineering gate.

## Start here

1. Read [`AGENTS.md`](AGENTS.md) for the working agreement.
2. Read [`docs/PHASE_CHECKLIST.md`](docs/PHASE_CHECKLIST.md), the single comprehensive PRD, technical architecture, data contract, and phase plan.
3. Review [`docs/decisions/COMPACT_DECISION_LOG.md`](docs/decisions/COMPACT_DECISION_LOG.md) for current decisions.
4. Use [`docs/decisions/DECISION_LOG.md`](docs/decisions/DECISION_LOG.md) for complete rationale and supersession history.
5. Read [`docs/CHANGE_LEDGER.md`](docs/CHANGE_LEDGER.md) for a chronological record of meaningful changes.

## Repository map

```text
.
├── AGENTS.md                         # Agent and contributor operating guide
├── Cargo.toml                        # Rust workspace
├── README.md                        # Project entry point
├── crates/
│   └── jft-nse-cm/                  # Authoritative Rust NSE CM data plane
├── docs/
│   ├── CHANGE_LEDGER.md             # Append-only record of meaningful changes
│   ├── PHASE_CHECKLIST.md            # Governing PRD, architecture, data contract, and phases 0-11
│   ├── decisions/
│   │   ├── COMPACT_DECISION_LOG.md  # Brief decision index
│   │   └── DECISION_LOG.md          # Full decision records
│   └── research/                    # Evidence and research notes
├── python/jft/                      # Downstream Python research plane
├── schemas/nse_cm/                  # Language-neutral canonical contracts
├── tests/fixtures/nse_cm/           # Synthetic, non-licensed golden fixtures
└── sources/                          # Read-only synced reference material
```

## Current state

Phase 1 fixture-first engineering is authorized. Rust owns source integrity, raw NSE parsing, exact raw-jiffy preservation/decomposition, lifecycle reduction, book replay, reconciliation, quarantine, and canonical Arrow/Parquet output. Python starts at that canonical boundary for diagnostics, features, labels, datasets, and modeling; it must not duplicate the raw parser or authoritative book.

The dependency-free Rust workspace and tracked Rust/Python/schema/fixture boundaries are scaffolded. The current evidence-backed slices define the documented CM Trim 68/82-byte and current Full 91/103-byte layouts, exact LF framing with source indices/offsets, lazy source-level parsed/quarantined accounting over caller-provided bytes, exact integer jiffies, strict Trim parsing, raw-byte preservation, and lossless quarantine. Thirty-six Rust tests cover all four current layout maps and synthetic boundary fixtures, deterministic source parsing, continuation after field quarantine, stop-on-framing-error behavior, framing and corrupt tails, every Trim numeric byte, all 38,400 single-byte Order/Trade mutations, documented session/side/activity codes, zero rules, text encoding, delimiter handling, literal-`b` preservation, and the inconsistency between NSE's stated jiffy epoch and published conversion constant; formatting, compilation, and strict Clippy also pass. Full semantic parsing, historical layout variants, lifecycle, book, cross-stream ordering, disk-backed input, and trade-decrement semantics remain intentionally unimplemented. Raw jiffies remain canonical, but UTC conversion is intentionally unavailable until the 19,800-second epoch discrepancy is resolved.

The next implementation gate is proposed [DEC-2026-009](docs/decisions/DECISION_LOG.md#dec-2026-009): the deterministic manifest/canonical identity, compatibility policy, and minimal hashing/JSON dependencies. It remains unimplemented until owner review activates or amends it.

Phase 0 remains open only for technical work that cannot be completed from the supplied samples: representative paired current CM Trim Orders/Trades and triggers, actual stream/schema inventory, delivery controls, and budget approval. These items do not block initial Phase 1 fixtures and interfaces, but representative current data is required to accept Phase 1. Legal/compliance remains visibly marked as something the owner needs to complete before operational activation, outside repository scope.

## Current engineering checks

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```
