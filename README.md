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

The dependency-free Rust workspace and tracked Rust/Python/schema/fixture boundaries are scaffolded. The current evidence-backed slices define CM Trim 68/82-byte, current Full 91/103-byte, and historical Full 87/100/101-byte layouts; validated date dispatch across every documented transition; date-dispatched structural Full-file and per-field byte-domain profiling with exact byte/residue accounting but no semantic claims; content-based empty/gzip/ZIP/ASCII/unknown-binary signature detection that never trusts suffixes; strict Full/Trim split-or-unsplit data/trigger filename decoding that preserves `CM` versus `CASH`; homogeneous stream-set discovery with gap/duplicate/mixed-mode rejection and no fixed stream count; exact data/trigger member pairing with explicit prefix-conflict reporting; logical-delivery orchestration with nested failure evidence; deterministic multi-delivery inventory discovery that retains rejected filenames and does not hide bad groups; exact LF framing with source indices/offsets and zero-copy corrupt-tail evidence; lazy source-level parsed/quarantined accounting over caller-provided bytes; exact integer jiffies; strict fixture-first Trim parsing; a deterministic order-only entry/modify/cancel reducer with explicit duplicate, unknown-order, regressing-jiffy, and fail-closed cross-session anomalies while `PO`/`RM`/`CA` identities remain isolated; a checked, pure price-level projection by raw instrument/session with typed empty/one-sided/normal/locked/crossed top-of-book evidence; single-source caller-ordered Trim Order replay with active/entry provenance, exact failure coordinates/raw bytes, zero-copy framing residue, reconciled counts, and an allocation-free observer for every successful transition; exact paired-delivery member validation plus deterministic independent per-stream Order replay and checked combined count/anomaly/active-observation summaries that retain stream context but perform no merge; non-mutating single-source trade-to-entry membership plus deterministic all-stream candidate collection, reconciled multiplicity/side/raw-jiffy-relation diagnostics, and a deterministic one-pass in-memory Order/Trade evidence report with per-source jiffy sequences, within-/cross-kind range-overlap pairs, and a per-stream nonzero observation taxonomy that preserves validated delivery/member identity, Order/Trade summaries, checked `PO`/`RM`/`CA` counts, deterministic session-excluded successful-entry occurrence groups and rejected cross-session lifecycle candidates with deterministic operation/session-route counts, candidate diagnostics, quarantine, and framing residue without choosing a causal match; Order/Trade jiffy profilers for ranges, consecutive ties, maximum tie runs, coordinate-exact regressions, exact paired-delivery stream coverage, deterministic member order, and pairwise range overlap without selecting replay order; raw-byte preservation; and lossless quarantine. One hundred sixty-one default Rust tests cover all seven documented layout maps, Gregorian-date rejection, every Full/Trim transition boundary, content signatures, filename/stream-set/pairing/delivery/inventory grammar and failures, input-order-independent inventory/lifecycle/book/replay/profile state, parsed lifecycle transitions, exact session counts, session isolation, and non-mutating anomalies, exact successful-transition coordinates/events/raw bytes, exact bid/ask aggregation, overflow failure, and integer top-of-book classification, single- and multi-stream replay count/provenance/raw-error evidence plus checked independent Order summaries and overflow rejection, both-sided single-source references, multi-stream trade candidate preservation, reconciled reference multiplicity/side/raw-jiffy-relation counts, fail-closed and input-order-independent Order/Trade evidence composition and delivery identity, deterministically classified Order/Trade quarantine, lifecycle, framing, candidate-ambiguity, and raw-jiffy observations, exact quantity/notional totals, and accumulator-overflow rejection, standalone and replay-integrated single-/multi-stream jiffy range/tie/regression/coverage accounting, date-dispatched Full structural and field-byte profiles, current synthetic boundary fixtures, deterministic source parsing, continuation after field quarantine, stop-on-framing-error behavior, 214 wrong-width cases, framing and corrupt tails, every Trim numeric byte, all 38,400 single-byte Order/Trade mutations, documented session/side/activity codes, zero rules, text encoding, delimiter handling, literal-`b` preservation, and the inconsistency between NSE's stated jiffy epoch and published conversion constant. Opt-in external tests also frame all 600,000 public legacy Order/Trade records with zero residue, prove digit-only bytes in twelve documented numeric fields, confirm literal-`b` symbol padding, and reproduce all five supplied legacy `.gz` suffix/content mismatches; formatting, compilation, and strict Clippy pass. Filesystem enumeration, representative current-data profiling runs, Full semantic parsing, durable canonical transition serialization, temporal/cross-stream replay ordering, session-transition/daily-reset semantics, checkpoint hashes, disk-backed input, decompression, and trade-decrement semantics remain intentionally unimplemented. Raw jiffies remain canonical, but UTC conversion is intentionally unavailable until the 19,800-second epoch discrepancy is resolved; equal-jiffy cross-stream order, unsigned identifier/jiffy handling, and Full Trigger Price rules also remain provisional because v1.19 is internally ambiguous.

The next implementation gate is proposed [DEC-2026-009](docs/decisions/DECISION_LOG.md#dec-2026-009): the deterministic manifest/canonical identity, compatibility policy, and minimal hashing/JSON dependencies. It remains unimplemented until owner review activates or amends it.

Phase 0 remains open only for technical work that cannot be completed from the supplied samples: representative paired current CM Trim Orders/Trades and triggers, actual stream/schema inventory, delivery controls, and budget approval. These items do not block initial Phase 1 fixtures and interfaces, but representative current data is required to accept Phase 1. Legal/compliance remains visibly marked as something the owner needs to complete before operational activation, outside repository scope.

## Current engineering checks

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```
