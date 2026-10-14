# Change Ledger

This append-only ledger records meaningful repository changes. Newest entries go first; existing historical entries remain intact.

## 2026-10-15 - Begin Phase 0 source and rights qualification

- **Changed:** Added the hashed ten-file source manifest, current official acquisition channel, legacy public CM sample diagnostics, data-rights clause matrix, clearance requirements, and a ready-to-send NDAL question set to the governing phase document.
- **Why:** Convert Phase 0 from a plan into an evidence-backed working record and identify exactly what remains before deterministic reconstruction can start.
- **Validation:** Verified supplied-file SHA-256 hashes and PDF page counts; inspected the official 9.2 MB CM sample's three files, widths, counts, event mix, timestamp monotonicity, and order-reference coverage; confirmed it is incomplete legacy Full data rather than current Trim/Full qualification data.
- **Related:** DEC-2026-004, DEC-2026-006; Phase 0.
- **Follow-up:** Send the clarification packet, record the executed license/Annexure, obtain a current quote, and acquire representative current paired Orders/Trades with trigger files.

---

## 2026-10-15 - Establish the NSE trading-system governing plan

- **Changed:** Replaced the generic phase checklist with the comprehensive NSE CM PRD, data contract, architecture, evidence registries, phase 0-11 gates, budgets, blockers, and kill criteria; aligned the README, research index, and decision logs.
- **Why:** Make the repository sufficient for another engineer to execute the work without relying on chat history, while separating NSE-documented facts from decisions, hypotheses, and unresolved items.
- **Validation:** Reviewed the supplied NSE v1.19 layouts and NDAL terms, checked internal links/decision parity, and ran repository documentation checks recorded in the resulting commit.
- **Related:** DEC-2026-003 through DEC-2026-006; Phase 0.
- **Follow-up:** Record written data-use clearance and qualify representative CM Trim files before parser implementation.

---

## 2026-10-14 — Initialize repository governance

- **Changed:** Added the agent operating guide, project README, Git ignore rules, full and compact decision logs, phase checklist, research location, and ledger.
- **Why:** Establish durable project practices for research, product work, development, validation, risk escalation, decisions, and progress tracking before implementation begins.
- **Validation:** Confirmed all governance files exist, link to one another, use consistent decision IDs/statuses, and preserve the read-only `sources/` boundary.
- **Related:** DEC-2026-001, DEC-2026-002; Phase 0.
- **Follow-up:** Complete Phase 1 discovery and research.

---

## Entry template

```markdown
## YYYY-MM-DD — Outcome-oriented title

- **Changed:** What changed.
- **Why:** Why it changed.
- **Validation:** Checks performed and results.
- **Related:** Decision IDs, issue/PR, and phase.
- **Follow-up:** Remaining work, risk, or `None`.
```
