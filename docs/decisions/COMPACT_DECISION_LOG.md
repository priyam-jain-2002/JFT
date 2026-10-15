# Compact Decision Log

Use this file for a quick view. The complete rationale and history live in [`DECISION_LOG.md`](DECISION_LOG.md). Update both files together.

| ID | Status | Brief | Practical effect | Full record |
|---|---|---|---|---|
| DEC-2026-001 | Active | Keep durable governance records in the repository. | Update the decision logs, change ledger, and phase checklist when applicable. | [Details](DECISION_LOG.md#dec-2026-001) |
| DEC-2026-002 | Active | Synced source material is immutable. | Read from `sources/`; write derived work elsewhere. | [Details](DECISION_LOG.md#dec-2026-002) |
| DEC-2026-003 | Active | Use an evidence-gated NSE CM research path. | Advance only through reconstruction, OOS, costs, shadow, small-live, and capacity gates. | [Details](DECISION_LOG.md#dec-2026-003) |
| DEC-2026-004 | Active | Qualify CM Trim before Full. | Buy/use Full only when matched evidence shows material value. | [Details](DECISION_LOG.md#dec-2026-004) |
| DEC-2026-005 | Active | Separate prediction, execution, and hard risk. | Baselines precede neural/ensemble models; execution and risk remain deterministic boundaries. | [Details](DECISION_LOG.md#dec-2026-005) |
| DEC-2026-006 | Superseded | Hold live/profit use behind written data-rights clearance. | Replaced by the external-dependency boundary in DEC-2026-007. | [Details](DECISION_LOG.md#dec-2026-006) |
| DEC-2026-007 | Active | Legal compliance is external to repository engineering. | Phase 1 offline work may proceed while Phase 0 technical qualification continues; operational activation remains an owner-controlled external dependency. | [Details](DECISION_LOG.md#dec-2026-007) |
| DEC-2026-008 | Active | Rust owns deterministic market-data replay; Python owns research/modeling. | Cross at versioned Arrow/Parquet plus manifest/schema/hash artifacts; no duplicate Python raw parser/book. | [Details](DECISION_LOG.md#dec-2026-008) |

## Maintenance rule

- Keep superseded and rejected entries visible for history.
- When a decision changes, add a new row and mark the old row `Superseded`; do not replace the old summary.
- If this file conflicts with the full record, the full record wins and this file must be corrected.
