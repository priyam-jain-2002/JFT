# Change Ledger

This append-only ledger records meaningful repository changes. Newest entries go first; existing historical entries remain intact.

## 2026-10-17 - Prove the Trim width guard across truncation and oversize cases

- **Changed:** Added exhaustive tested-length sweeps around both current Trim payload widths, covering empty, truncated, delimiter-sized, and up-to-32-byte-oversized inputs.
- **Why:** Fixed-field indexing is safe only if width validation always runs first. A single short-input indexing path could panic and lose ingestion evidence.
- **Validation:** Exercised 100 invalid Order lengths and 114 invalid Trade lengths (214 total). Every case returns the exact expected/actual widths, has no field attribution, and retains every input byte. All 38 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1 parser/quarantine invariants.
- **Follow-up:** Retain these guards when product/date layout dispatch is added; unknown layouts must fail before any field access.

---

## 2026-10-17 - Present the manifest/schema contract for owner decision

- **Changed:** Added proposed DEC-2026-009 with a concrete manifest identity, schema compatibility policy, minimal dependency set, event-key tuple, unified canonical Trim/Full recommendation, alternatives, pre-mortem, and activation tests; cross-linked the governing Phase 1 schema section.
- **Why:** Hashing libraries, serialized fields, compatibility rules, and event identity are difficult to reverse once Python artifacts exist. The owner asked to be consulted on decisions, so implementation stops at an explicit reviewable proposal rather than silently freezing the contract.
- **Validation:** Confirmed both decision logs contain DEC-2026-009 with Proposed status and the governing checklist links to its stable anchor. No Cargo dependencies, schemas, parser behavior, or application code changed.
- **Related:** DEC-2026-008, proposed DEC-2026-009; Phase 1 manifest/canonical boundary.
- **Follow-up:** Owner accepts or amends DEC-2026-009; only then activate it and implement its test-first integrity/manifest slice.

---

## 2026-10-17 - Exhaustively prove Trim single-byte evidence retention

- **Changed:** Added deterministic mutation sweeps that substitute all 256 byte values at every byte position in the current Trim Order and Trade fixtures.
- **Why:** Field-specific tests cover intended rules but can miss panics or evidence loss at unexpected byte values. The parser's minimum safety invariant is total behavior over every fixed-width byte string: exact success bytes or exact quarantine bytes.
- **Validation:** The tests exercise 68 x 256 Order variants and 82 x 256 Trade variants (38,400 total). Every outcome preserves the complete mutated input, and no case panics. All 36 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1 parser/quarantine invariants.
- **Follow-up:** Add coverage-guided fuzzing after the dependency/tooling choice is reviewed; keep semantic assertions separate from the totality/evidence-retention invariant.

---

## 2026-10-17 - Add current Full CM boundary fixtures

- **Changed:** Added synthetic, non-licensed 91-byte Full Order and 103-byte Full Trade fixture records plus tests that frame them and assert every field's exact raw bytes.
- **Why:** Layout constants should be exercised by independently readable golden bytes before adding Full event types or semantic validation. This catches off-by-one and field-shift errors without prematurely deciding conditional market/stop-loss/quantity rules.
- **Validation:** Both fixtures are exactly one documented payload plus one LF byte (92/104 bytes). Two integration tests verify framing and every one of the 17 Order and 14 Trade field slices. All 34 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** NSE Historical Order & Trade Data v1.19; DEC-2026-004, DEC-2026-008; Phase 1 layout contract.
- **Follow-up:** Review Full zero/enum/conditional validation rules and canonical optional fields before implementing Full semantic parsing.

---

## 2026-10-17 - Encode the current Full CM byte maps

- **Changed:** Added the documented current Full CM 91-byte Order and 103-byte Trade layouts as contiguous Rust field maps alongside Trim, without adding Full parsing or lifecycle assumptions.
- **Why:** Date/version dispatch and Trim-versus-Full qualification require exact current Full byte boundaries before fixtures or semantic parsing. Separating byte maps from semantics keeps this step independently auditable.
- **Validation:** Three unit tests assert every Full field name, zero-based offset, width, total record length, contiguity, and the exact eight-field core prefix shared by Trim and Full Orders. All 32 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** NSE Historical Order & Trade Data v1.19; DEC-2026-004, DEC-2026-008; Phase 1 layout contract.
- **Follow-up:** Add synthetic non-licensed Full fixtures and strict Full parsing only after reviewing zero/enum rules; add historical 87/100/101 maps through explicit date/version dispatch rather than width guessing.

---

## 2026-10-17 - Account for every framed source record

- **Changed:** Added a lazy source-level Trim Order/Trade iterator over caller-provided bytes that yields one ordered result per framed record, including zero-based record index and byte offset; field failures remain lossless quarantine outcomes and do not conceal later well-framed records, while framing loss stops without guessed resynchronization.
- **Why:** Per-record parsing alone cannot prove whole-source completeness. Replay inputs need deterministic accepted/quarantined counts and exact source coordinates before manifest identity, lifecycle reduction, or book state.
- **Validation:** Four source tests prove three-record accounting across a middle field failure, exact offsets, lossless unknown-tail capture, complete Trade-source parsing, correct accepted/quarantined counts, iterator termination after framing loss, and identical results across repeat runs. All 29 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1 byte-accounting contract.
- **Follow-up:** Add reviewed source identity/hash and schema-version fields to these coordinates, then freeze the manifest schema before lifecycle logic. Select buffered versus memory-mapped disk input only after representative file-size and throughput evidence; this milestone does not claim disk streaming.

---

## 2026-10-17 - Prove exact source-record framing and byte accounting

- **Changed:** Added a zero-copy Rust iterator for fixed-width LF-delimited source records, with zero-based record indices, byte offsets, exact payload slices, explicit framing errors, and lossless preservation of every unconsumed byte after a failure.
- **Why:** The field parser must never receive delimiters or silently repair framing. Deterministic replay requires source positions and complete byte accounting before manifest hashes or lifecycle state are introduced.
- **Validation:** Six framing tests prove exact multi-record offsets, full byte accounting, Trade payload separation, empty input behavior, rejection of a missing final LF and CRLF, lossless corrupt-tail capture, and stop-on-first-error behavior. Together with the existing suite, all 25 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1 byte-accounting contract.
- **Follow-up:** Connect framed records to explicit source provenance and the versioned manifest contract; select audited hashing/serialization dependencies before implementing trigger MD5, internal SHA-256, and JSON output.

---

## 2026-10-16 - Expose and contain the NSE jiffy-epoch inconsistency

- **Changed:** Removed the unsupported UTC implication from the Rust jiffy API and governing plan, retained only lossless raw/integer decomposition, added a regression test for the published offset discrepancy, and added a precise vendor clarification question.
- **Why:** NSE v1.19 says the epoch is 1980-01-01 00:00:00 GMT but publishes a `315513000`-second Unix offset. The actual UTC offset is `315532800` seconds; the 19,800-second difference is exactly IST (05:30). Silently choosing either interpretation would contaminate timestamps, session dates, ordering diagnostics, labels, and later backtests.
- **Validation:** The independent calendar calculation and Rust regression test prove the 19,800-second discrepancy. All 19 Rust tests pass; formatting, compilation, and strict Clippy pass. No UTC conversion is exposed while the source contradiction remains unresolved.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1 jiffy policy.
- **Follow-up:** Resolve the epoch from representative dated files and a written NSE/vendor answer before exposing UTC/session-date fields; continue Phase 1 with lossless raw-jiffy provenance.

---

## 2026-10-16 - Exhaust the current Trim field-validation matrix

- **Changed:** Extended the Rust integration suite across every numeric byte, all documented sessions/sides/activities, zero-value rules, unknown segment/enum handling, non-ASCII text, included-LF rejection, and literal-`b` symbol padding.
- **Why:** Demonstrate strict byte accounting and evidence-preserving failure behavior before freezing schemas or introducing lifecycle state.
- **Validation:** All 18 tests pass. The numeric mutation loops exercise all 48 Order numeric positions and all 64 Trade numeric positions; each mutation is attributed to the correct field. Formatting, compilation, and strict Clippy also pass. Literal `b` bytes are preserved verbatim rather than treated as blanks.
- **Related:** DEC-2026-004, DEC-2026-008; Phase 1.
- **Follow-up:** Freeze the versioned canonical schema and manifest/provenance contract before adding lifecycle reduction.

---

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
