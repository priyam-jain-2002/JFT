# JFT

JFT is an evidence-gated research program for an NSE Cash Market short-horizon quantitative trading system. The repository is in Phase 0: data rights, evidence, and data qualification. No trading or application implementation has started.

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
├── README.md                        # Project entry point
├── docs/
│   ├── CHANGE_LEDGER.md             # Append-only record of meaningful changes
│   ├── PHASE_CHECKLIST.md            # Governing PRD, architecture, data contract, and phases 0-11
│   ├── decisions/
│   │   ├── COMPACT_DECISION_LOG.md  # Brief decision index
│   │   └── DECISION_LOG.md          # Full decision records
│   └── research/                    # Evidence and research notes
└── sources/                          # Read-only synced reference material
```

## Current state

Phase 0 is in progress. The next gate is documented permission for the intended NSE historical-data use plus a representative CM Trim data qualification report. Phase 1 parser/replay work must not begin until that gate passes, and feature/model work must not begin until deterministic reconstruction passes.
