# Change Ledger

This append-only ledger records meaningful repository changes. Newest entries go first; existing historical entries remain intact.

## 2026-10-18 - Classify exact top-of-book state

- **Changed:** Added a typed, integer-only top-of-book view for each projected price book: empty, bid-only, ask-only, normal with exact spread, locked, or crossed with exact cross amount. Every state retains the underlying best-level order counts and quantities.
- **Why:** Locked and crossed observations must be measurable anomalies, not silently removed or normalized. A typed result also prevents downstream code from recomputing spread/cross arithmetic inconsistently.
- **Validation:** Four new tests prove empty/one-sided classification, exact normal spread, locked preservation, and exact crossed amount without repair. All 116 Rust tests pass; formatting, compilation, strict Clippy, and diff checks pass.
- **Related:** DEC-2026-008; Phase 1 book invariant contract.
- **Follow-up:** Count and contextualize these states on representative current data only after stream ordering is evidenced; synthetic states validate classification, not real-market incidence.

---

## 2026-10-17 - Stream exact successful lifecycle transitions

- **Changed:** Added an allocation-free replay observer that receives every successful Trim Order lifecycle transition with exact source coordinate, parsed event, complete 68-byte raw record, and entered/modified/cancelled state delta. Default replay delegates to the same path with a no-op observer.
- **Why:** Successful transitions need an auditable streaming boundary without forcing replay to retain the entire accepted source. Durable/fallible serialization remains a later canonical-contract concern.
- **Validation:** Four new tests prove exact transition order and coordinates, exact event/raw preservation, exclusion of parser/lifecycle failures, applied-count agreement, and identical replay reports with or without observation. All 112 Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 single-source replay contract.
- **Follow-up:** Connect this observer to versioned, fallible canonical serialization only after DEC-2026-009 is approved; do not treat the observer as a committed wire schema.

---

## 2026-10-17 - Add lossless Order and Trade jiffy profilers

- **Changed:** Added shared streaming profiling for Trim Order and Trade sources: parsed observation count, raw-jiffy min/max, consecutive parsed ties, maximum equal-jiffy run, every regression with both exact source coordinates/jiffies, interval overlap, quarantines, and borrowed framing residue.
- **Why:** A cross-stream merge policy must be selected from measured monotonicity, ties, overlap, and reference timing—not from file naming or synthetic intuition. The profiler creates that evidence boundary without choosing an ordering.
- **Validation:** Seven new tests prove empty/increasing profiles, equal-jiffy runs, exact regression coordinates, quarantine exclusion with count reconciliation, framing-stop evidence, Order/Trade range overlap, and repeat determinism. All 108 Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 ordering-evidence gate.
- **Follow-up:** Run these profiles over every stream in representative paired current deliveries; synthetic results validate the profiler only and make no claim about real NSE ordering.

---

## 2026-10-17 - Reconcile both Trim trade references to Order entries

- **Changed:** Retained the first successful entry side/coordinate for every replayed order key and added pure buy/sell trade-reference reconciliation against that Order-source evidence. Each side independently returns exact entry match, entry-side mismatch, or missing identity.
- **Why:** Reference coverage can be measured without selecting a cross-stream ordering or execution-decrement model. Calling this source membership—not a temporal match—prevents closed orders or late file ordering from being misrepresented as active at trade time.
- **Validation:** Six new tests prove exact two-sided coordinate matches, partial match preservation, side-mismatch versus missing classification, strict session/raw-instrument identity, cancelled-order source membership without temporal claims, and repeat determinism. All 101 Rust tests pass; formatting, compilation, strict Clippy, and diff checks pass.
- **Related:** DEC-2026-008; Phase 1 trade-reference contract.
- **Follow-up:** Measure per-stream and cross-file timing on representative paired data before selecting a merge policy or evaluating competing quantity-decrement interpretations.

---

## 2026-10-17 - Replay one Trim Order source with exact failure provenance

- **Changed:** Added streaming replay for one caller-ordered Trim Order byte source. It reduces parsed events into lifecycle state, tracks entry/last-mutation coordinates for active orders, retains exact event/raw/coordinate evidence for parser and lifecycle failures, preserves a zero-copy unconsumed framing tail, exposes final price projection, and checks count reconciliation.
- **Why:** Parser, lifecycle, and book components need an evidence-preserving integration boundary before any cross-stream merge or hashing policy is selected. Retaining only failures and active provenance avoids a whole-source success vector while keeping every rejected byte auditable.
- **Validation:** Seven new tests prove complete entry→modify→cancel replay, quarantine continuation, duplicate/unknown/regressing lifecycle evidence, zero-copy framing stop, repeat determinism, active provenance, final book projection, and `framed = parsed + quarantined` / `parsed = applied + anomaly` reconciliation. All 95 Rust tests pass; formatting, compilation, strict Clippy, and diff checks pass.
- **Related:** DEC-2026-008; Phase 1 single-source replay contract.
- **Follow-up:** Define a streaming successful-transition sink and cross-stream merge only after ordering evidence; add stable checkpoint serialization/hashes only after DEC-2026-009 is approved.

---

## 2026-10-17 - Project active orders into deterministic price levels

- **Changed:** Added a pure Rust projection from active lifecycle state into deterministic per-session/raw-instrument bid and ask maps. Levels aggregate checked `u64` order counts and quantities, expose exact best prices, retain locked/crossed states, and return typed overflow evidence instead of wrapping.
- **Why:** Price-level correctness can be proven independently of incremental replay, trades, queue priority, and hashing. Recomputing from authoritative active orders avoids introducing a second mutable state before provenance orchestration exists.
- **Validation:** Eight new tests prove empty state, same-price aggregation, exact side/BBO separation, locked/crossed preservation, session/instrument isolation, modify/cancel reflection, multi-book input-order independence, and explicit quantity overflow. All 88 Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 book-state contract.
- **Follow-up:** Add provenance-carrying replay orchestration and deterministic checkpoint serialization only after canonical identity v1 is approved; do not add trade-driven decrements before representative evidence.

---

## 2026-10-17 - Add the deterministic order lifecycle reducer

- **Changed:** Added a Rust order-only lifecycle reducer keyed by raw instrument/session/order identity. Entry creates active state, modify replaces published side/quantity/price while retaining entry time, and cancel removes state. Duplicate entry, unknown modify/cancel, and per-order regressing jiffies return typed anomalies without mutation. Added hashed synthetic modify/cancel fixtures.
- **Why:** Book reconstruction needs an explicit, auditable transition boundary before price-level aggregation. Keeping trades and queue-priority assumptions out prevents unresolved execution semantics from contaminating order lifecycle state.
- **Validation:** Seven new tests exercise parsed entry→modify→cancel transitions, duplicate and unknown-order anomalies, non-mutating modify/cancel time regressions, same-jiffy caller order, and input-order-independent active state. All 80 Rust tests pass; both new fixtures are exactly 69 bytes and have frozen SHA-256 identities.
- **Related:** DEC-2026-008; Phase 1 lifecycle contract.
- **Follow-up:** Add source-coordinate/provenance orchestration after the canonical identity decision; then build order-level book projection without trade decrements or queue-priority claims.

---

## 2026-10-17 - Partition complete CM delivery inventories deterministically

- **Changed:** Added dependency-free inventory discovery that parses arbitrary CM filenames, retains exact rejected names and reasons, sorts valid inputs canonically, partitions all date/product/record-kind groups, and returns an independent delivery outcome for each group in `O(n log n)` time.
- **Why:** Filesystem order is not stable, and one malformed file or incomplete delivery must not hide evidence for other groups. Deterministic pre-sorting also prevents input order from reversing expected/actual values in nested errors.
- **Validation:** Five new tests prove canonical ordering across three groups, good-group survival beside a bad group, exact sorted filename rejections, empty-inventory behavior, and identical results under reversed input including a mixed-prefix failure. All 73 Rust tests pass; formatting, compilation, strict Clippy, and diff checks pass.
- **Related:** DEC-2026-008; Phase 1 delivery-discovery contract.
- **Follow-up:** Add a filesystem boundary only after defining how paths, non-files, symlinks, and inaccessible entries are represented; keep it outside deterministic logical discovery.

---

## 2026-10-17 - Orchestrate one complete CM logical delivery

- **Changed:** Added a delivery-level function that verifies one logical product/date/record-kind group, partitions data and triggers, validates each dynamic stream set, and performs exact pairing.
- **Why:** Callers should not have to reimplement validation order or lose the causal error. A missing role, stream gap, logical mismatch, or prefix conflict must remain distinguishable before integrity checks begin.
- **Validation:** Five new tests prove arbitrary member order succeeds and exact nested errors survive for empty/missing roles, mixed logical groups, a data-stream gap, and the `CM`/`CASH` prefix conflict. All 68 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** DEC-2026-008; Phase 1 delivery-discovery contract.
- **Follow-up:** Deterministically partition a directory-sized decoded inventory into logical deliveries and report all groups without losing per-file parse failures.

---

## 2026-10-17 - Require exact data/trigger stream-set pairing

- **Changed:** Added typed pairing for validated data and trigger sets, requiring correct roles, the same documented prefix, logical product/date/kind identity, split form, and exact observed stream members.
- **Why:** A complete data set with a missing or orphan trigger cannot pass integrity verification. Treating `CM` and `CASH` as implicit aliases would conceal the v1.19 naming conflict and could pair unrelated objects.
- **Validation:** Six new tests prove matched split and unsplit pairing; missing trigger members, prefix conflict, logical group mismatch, split-form mismatch, and reversed roles all fail with specific evidence. All 63 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** NSE Historical Order & Trade Data v1.19 sections 2.1.9 and 3.1.10; DEC-2026-008; Phase 1 delivery-integrity contract.
- **Follow-up:** Add directory-level grouping of decoded names and require exactly one validated data/trigger pair per logical Order/Trade group before hashing or decompression.

---

## 2026-10-17 - Discover homogeneous CM stream sets without a fixed count

- **Changed:** Added typed CM file-group keys and set-level discovery for unsplit or contiguous split streams, retaining prefix/product/date/kind/data-trigger identity.
- **Why:** NSE states that stream count may grow. Hard-coding a count would silently drop future parts; merging unlike metadata or split modes would hide delivery defects.
- **Validation:** Six new tests prove arbitrary input order becomes `[1..N]`, the first gap is rejected, split and unsplit duplicates are rejected, mixed split mode and group metadata are rejected, a singleton unsplit file is accepted, and empty discovery fails. All 57 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** NSE Historical Order & Trade Data v1.19 sections 2.1.1, 3.1.1, and 5.3; DEC-2026-008; Phase 1 input-discovery contract.
- **Follow-up:** Pair homogeneous data and trigger sets by explicit group identity, report prefix conflicts rather than normalizing them, and require matching stream members before integrity checks.

---

## 2026-10-17 - Decode CM source filenames without resolving the prefix conflict

- **Changed:** Added strict decoding for CM Full/Trim Order/Trade data and `.trg` names, split and unsplit forms, validated DDMMYYYY dates, and positive two-digit stream identifiers. `CM` and `CASH` remain distinct enum values.
- **Why:** Stream discovery needs typed filename evidence before it can detect gaps or duplicates. NSE v1.19 uses `CM_...` in layout examples and `CASH_...` in trigger tables, so silently normalizing either prefix would hide a source contradiction.
- **Validation:** Six integration tests cover both prefixes, Full/Trim, data/trigger, split/unsplit (including an explicit unsplit-Trim regression), exact dates, and rejection of invalid calendar dates, zero/wrong-width/non-numeric streams, unknown prefixes/types/shapes, and case-wrong suffixes. Strict Clippy initially caught a shadowed unsplit-Trim pattern; the corrected decoder and regression assertion pass all 51 Rust tests, formatting, compilation, and strict Clippy.
- **Related:** NSE Historical Order & Trade Data v1.19 sections 2.1.9, 3.1.10, and 5.3; DEC-2026-008; Phase 1 input-discovery contract.
- **Follow-up:** Build set-level discovery that detects duplicate/gapped streams and data/trigger mismatches without assuming a fixed maximum stream count.

---

## 2026-10-17 - Expose Full numeric and Trigger Price specification conflicts

- **Changed:** Added the exact documented Full CM value domains, recorded the Trigger Price and signed-numeric ambiguities, added two precise vendor questions, and made Full semantic parsing contingent on representative evidence/clarification.
- **Why:** Implementing the valid-range column literally would reject the documented zero Trigger Price for non-stop-loss orders. Treating every numeric field as unsigned also conflicts with section 5.4's default-signed instruction for fields without explicit ranges. Either silent assumption could corrupt or discard valid history.
- **Evidence:** v1.19 Full CM Order pages label Trigger Price non-zero/non-negative and simultaneously state it is `0` for non-stop-loss orders. Section 5.4 defines unspecified integral fields as signed; CM ID/jiffy rows show no range. Other explicit Full value domains are unambiguous and now recorded.
- **Validation:** Reviewed the complete relevant Full CM Order/Trade tables and sections 5.4/5.5 in the supplied PDF; checked the governing plan, README, and clarification list for consistent status. No parser behavior, dependency, schema, or application code changed.
- **Related:** NSE Historical Order & Trade Data v1.19 sections 3.1.1, 3.1.2, and 5.4; DEC-2026-004, DEC-2026-008; Phase 1 semantic-validation gate.
- **Follow-up:** Validate delivered encodings and conditional price rules before implementing Full wire-to-event parsing; keep exact raw bytes authoritative meanwhile.

---

## 2026-10-17 - Add validated CM source-date layout dispatch

- **Changed:** Added a dependency-free validated Gregorian `SourceDate`, typed Full Order/Trade and Trim layout versions, field/length accessors, and source-date dispatch for all documented CM transitions.
- **Why:** Width guessing can misclassify corruption and cannot safely handle the one-day 28 March 2025 expansion followed by rollback. The file's authoritative trading date must select the wire schema before framing/parsing.
- **Validation:** Tests cover invalid months/days and Gregorian leap-year rules; 30 June/01 July 2024, 27/28/29 March 2025, 15/16 June 2025, and 17/18 May 2026 boundaries; the exception rollback; and field-list totals for every selected version. All 45 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** NSE Historical Order & Trade Data v1.19 section 5; DEC-2026-004, DEC-2026-008; Phase 1 version-dispatch contract.
- **Follow-up:** Bind selected layouts to explicit product/date metadata in the manifest after DEC-2026-009 approval; never auto-select a schema solely because bytes happen to fit a width.

---

## 2026-10-17 - Encode the historical Full CM byte maps from v1.19

- **Changed:** Added contiguous historical Full CM layouts for 87-byte Orders, 100-byte Trades, and 101-byte Trades; updated the governing facts to record CM Trim's documented 18 May 2026 introduction date.
- **Why:** Historical data spans three pre-current widths. Treating every file as 91/103 would shift fields or reject valid source bytes, while width-only guessing without documented field changes would be unsafe.
- **Evidence:** Visually inspected v1.19 section 5.1/5.2. It documents CM Trade 100->101 from 01 July 2024, the one-day 91/103 exception on 28 March 2025, permanent 87->91 Order and 101->103 Trade expansion from 16 June 2025, and Trim 68/82 introduction on 18 May 2026. Version history identifies the changed fields as Trade Number, Order Volume Disclosed/Original, and Trade Quantity.
- **Validation:** Three new unit tests assert every historical field name, zero-based offset, width, contiguity, and exact 87/100/101-byte total. All 41 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** NSE Historical Order & Trade Data v1.19 section 5; DEC-2026-004, DEC-2026-008; Phase 1 layout contract.
- **Follow-up:** Add validated date/version dispatch with boundary tests; keep 28 March 2025 as an explicit exception and do not infer layout from width alone.

---

## 2026-10-17 - Remove unbounded framing-error tail copies

- **Changed:** Replaced the owned `Vec` in `FramingError` with a lifetime-bound slice into the immutable source and propagated that lifetime through the lazy parser iterator.
- **Why:** Copying the entire unconsumed tail after one malformed delimiter could allocate gigabytes and turn a data-quality error into an out-of-memory failure. Borrowing preserves identical evidence with constant additional memory.
- **Validation:** Existing framing/source tests still prove lossless stop-on-first-error behavior; added pointer-identity assertions prove missing-LF, CRLF, and post-record partial tails reference the original source allocation. All 38 Rust tests pass; formatting, compilation, and strict Clippy pass.
- **Related:** DEC-2026-008; Phase 1 framing, reliability, and memory-safety contract.
- **Follow-up:** Disk-backed ingestion must preserve the same bounded-memory behavior, using source-range provenance or a bounded copied error window rather than an unbounded tail allocation.

---

## 2026-10-17 - Freeze the synthetic fixture identities

- **Changed:** Expanded the existing synthetic-fixture README with provenance, exact payload/LF sizes, SHA-256 digests for all four current Trim/Full golden files, reproduction commands, and an update rule.
- **Why:** Later integrity and manifest code needs independent immutable expected values. The registry also prevents synthetic test bytes from being mistaken for licensed or representative NSE samples.
- **Validation:** Recomputed each digest and byte count from the tracked files: Trim Order 69, Trim Trade 83, Full Order 92, and Full Trade 104 bytes including LF. The inventory values match both command outputs exactly. No manifest schema, dependency, parser behavior, or application code changed.
- **Related:** DEC-2026-004, DEC-2026-008, proposed DEC-2026-009; Phase 1 fixture and future integrity contract.
- **Follow-up:** After DEC-2026-009 is approved, make the Rust integrity implementation reproduce these hashes as golden tests.

---

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
