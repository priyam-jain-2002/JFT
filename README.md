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

The dependency-free Rust workspace and tracked Rust/Python/schema/fixture boundaries are scaffolded. The current evidence-backed slices define CM Trim 68/82-byte, current Full 91/103-byte, and historical Full 87/100/101-byte layouts; validated date dispatch across every documented transition; strict Full/Trim split-or-unsplit data/trigger filename decoding that preserves `CM` versus `CASH`; homogeneous stream-set discovery with gap/duplicate/mixed-mode rejection and no fixed stream count; exact data/trigger member pairing with explicit prefix-conflict reporting; logical-delivery orchestration with nested failure evidence; deterministic multi-delivery inventory discovery that retains rejected filenames and does not hide bad groups; exact LF framing with source indices/offsets and zero-copy corrupt-tail evidence; lazy source-level parsed/quarantined accounting over caller-provided bytes; exact integer jiffies; strict fixture-first Trim parsing; a deterministic order-only entry/modify/cancel reducer with explicit duplicate, unknown-order, and regressing-jiffy anomalies; a checked, pure price-level projection by raw instrument/session that preserves locked/crossed states; single-source caller-ordered Trim Order replay with active/entry provenance, exact failure coordinates/raw bytes, zero-copy framing residue, and reconciled counts; non-mutating Trim trade-to-entry source membership for both sides with distinct match/missing/side-mismatch evidence; raw-byte preservation; and lossless quarantine. One hundred one Rust tests cover all seven documented layout maps, Gregorian-date rejection, every Full/Trim transition boundary, filename/stream-set/pairing/delivery/inventory grammar and failures, input-order-independent inventory/lifecycle/book/replay state, parsed lifecycle transitions and non-mutating anomalies, exact bid/ask aggregation and overflow failure, replay count/provenance/raw-error evidence, both-sided trade-reference source membership, current synthetic boundary fixtures, deterministic source parsing, continuation after field quarantine, stop-on-framing-error behavior, 214 wrong-width cases, framing and corrupt tails, every Trim numeric byte, all 38,400 single-byte Order/Trade mutations, documented session/side/activity codes, zero rules, text encoding, delimiter handling, literal-`b` preservation, and the inconsistency between NSE's stated jiffy epoch and published conversion constant; formatting, compilation, and strict Clippy also pass. Filesystem enumeration, Full semantic parsing, complete successful-transition audit output, temporal/cross-stream replay ordering, checkpoint hashes, disk-backed input, and trade-decrement semantics remain intentionally unimplemented. Raw jiffies remain canonical, but UTC conversion is intentionally unavailable until the 19,800-second epoch discrepancy is resolved; unsigned identifier/jiffy handling and Full Trigger Price rules also remain provisional because v1.19 is internally ambiguous.

The next implementation gate is proposed [DEC-2026-009](docs/decisions/DECISION_LOG.md#dec-2026-009): the deterministic manifest/canonical identity, compatibility policy, and minimal hashing/JSON dependencies. It remains unimplemented until owner review activates or amends it.

Phase 0 remains open only for technical work that cannot be completed from the supplied samples: representative paired current CM Trim Orders/Trades and triggers, actual stream/schema inventory, delivery controls, and budget approval. These items do not block initial Phase 1 fixtures and interfaces, but representative current data is required to accept Phase 1. Legal/compliance remains visibly marked as something the owner needs to complete before operational activation, outside repository scope.

## Current engineering checks

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```
