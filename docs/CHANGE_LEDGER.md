# Change Ledger

This append-only ledger records meaningful repository changes. Newest entries go first; existing historical entries remain intact.

## 2026-10-16 - Verify the first CM Trim parser slice

- **Changed:** Added contiguous 68-byte Order and 82-byte Trade layouts, exact `u64` jiffy representation, canonical Rust event types, strict fixed-width parsing, raw-byte preservation, lossless field-addressed quarantine, and synthetic Order/Trade golden fixtures.
- **Why:** Prove the source-byte-to-event boundary with documented widths and failure evidence before implementing lifecycle or order-book semantics.
- **Validation:** Verified fixture sizes are exactly 69/83 bytes including LF; three unit tests prove both layouts and the NSE jiffy example; seven integration tests prove valid Order/Trade parsing, raw preservation, order references, all documented sessions, wrong-width quarantine, invalid numeric quarantine, zero-quantity quarantine, and unknown-activity field location. Formatting, compilation, all ten tests, and strict Clippy pass.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1.
- **Follow-up:** Expand parser fixtures/error coverage and freeze the versioned canonical schema before lifecycle reduction.

---

## 2026-10-16 - Scaffold the Phase 1 Rust data plane

- **Changed:** Added a dependency-free Cargo workspace, the `jft-nse-cm` Rust crate with the approved Phase 1 module boundaries, and tracked placeholders for downstream Python canonical I/O/diagnostics, language-neutral schemas, and synthetic fixtures.
- **Why:** Establish a compile-checked vertical ownership boundary before choosing record semantics or third-party libraries, so raw NSE parsing and authoritative replay cannot drift into Python.
- **Validation:** `cargo fmt --all --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets`, and `cargo clippy --workspace --all-targets -- -D warnings` all pass on Rust 1.99.1. No supplied source files, schema fields, parser semantics, or application behavior were changed.
- **Related:** DEC-2026-008; Phase 1.
- **Follow-up:** Consult on and freeze the canonical schema/versioning/error contract, then implement the first synthetic Trim Order/Trade parsing slice.

---

## 2026-10-15 - Authorize Phase 1 and freeze the Rust/Python boundary

- **Changed:** Moved legal/compliance to a visible external operational dependency; kept Phase 0 open for representative-data, stream/schema, delivery, and budget qualification; authorized fixture-first Phase 1; recorded Rust as the authoritative parser/replay data plane and Python as the downstream research/modeling plane.
- **Why:** Controlled offline engineering can proceed without treating legal review as repository scope, while the remaining non-legal Phase 0 work and the established language boundary must not be lost.
- **Validation:** Reviewed all phase status, prerequisites, blockers, immediate actions, decision-log parity, expected module paths, and canonical-interface language for contradictions. No trading/application code or supplied source files were changed.
- **Related:** DEC-2026-004, DEC-2026-007, DEC-2026-008; Phase 0 and Phase 1.
- **Follow-up:** Freeze canonical schemas and Rust crate interfaces, build synthetic fixtures, and continue representative current-data/delivery qualification in parallel.

---

## 2026-10-15 - Complete local source-bundle inventory and separate CM products

- **Changed:** Expanded the Phase 0 manifest from ten to all 17 local files; qualified the legacy CM historical/EOD sample separately from the Order/Trade sample; recorded delivery timing, retention, AWS controls, and remaining vendor-security questions.
- **Why:** The newly located CM detail document and sample bundle are useful reference inputs but do not contain current jiffy-timestamped order lifecycle events. Keeping those products separate prevents false deterministic-replay claims.
- **Validation:** Recomputed all source SHA-256 hashes and sizes; extracted all nine PDFs and both legal documents for review; inspected the outer and nested CM archives, record counts, delimiter shapes, compression signatures, and representative records; confirmed the supplied `Sample_CMsegment.zip` matches the previously reviewed official sample hash.
- **Related:** DEC-2026-002, DEC-2026-004, DEC-2026-006; Phase 0.
- **Follow-up:** Obtain a current quote, executed Annexure II and written use clarification, production delivery/security details, and representative current paired Trim Orders/Trades with `.trg` files.

---

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
