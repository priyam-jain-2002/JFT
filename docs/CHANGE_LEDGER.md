# Change Ledger

This append-only ledger records meaningful repository changes. Newest entries go first; existing historical entries remain intact.

## 2026-10-19 - Preserve validated raw jiffies on quarantines

- **Changed:** Located Order and Trade quarantines now retain an optional exact `ExchangeJiffy` independently decoded from the record's documented raw jiffy bytes. Valid digit-only bytes are preserved even when another field rejects the record; invalid jiffy bytes yield `None`. Quarantined records remain excluded from parsed-event and jiffy-sequence counts.
- **Why:** Anomaly dossiers need raw timing provenance without accepting an otherwise rejected record or forcing analysts to reparse raw NSE bytes downstream.
- **Validation:** Targeted tests prove exact Order and Trade jiffies survive unrelated activity/price rejection, while an invalid-jiffy quarantine retains `None` and stays excluded from observations. All 161 default Rust tests pass.
- **Related:** Phase 1 raw provenance/quarantine and raw-jiffy evidence gates; Rust/Python ownership boundary.
- **Follow-up:** Representative runs must retain delivery/member/coordinate context around these quarantines; durable file identity remains part of the future canonical manifest decision.

---

## 2026-10-18 - Replace stale Phase 1 immediate actions

- **Changed:** The governing document's immediate-action list now reflects the implemented fixture-first parser, replay, book, jiffy, session, and evidence-reporting work. The ordered next gates are representative current paired data/triggers, full reconciliation runs, ambiguity resolution, owner review of proposed DEC-2026-009, then canonical output/hashes and the Python conformance reader.
- **Why:** The previous list still instructed the project to implement layout, jiffy, quarantine, fixture, and lifecycle foundations that already exist and are verified. Leaving it in place contradicted the phase checklist and could cause duplicated or out-of-order work.
- **Validation:** Every new action cross-references an existing unchecked gate or proposed decision in this document; completed fixture-first work is no longer presented as pending. Markdown whitespace validation passes.
- **Related:** Phase 0 acquisition/budget gates; Phase 1 unchecked acceptance items; proposed DEC-2026-009.
- **Follow-up:** Update this list when representative data is acquired or DEC-2026-009 changes status.

---

## 2026-10-18 - Retain raw-jiffy relations in session routes

- **Changed:** Every cross-session operation/session route now separates entry candidates whose raw exchange jiffy is before, equal to, or after the rejected lifecycle event. A reconciliation invariant requires those three counts to equal the route's entry-candidate total and requires each emitted route to contain at least one event and candidate.
- **Why:** Route frequency alone hides timing ambiguity. Raw-jiffy relations provide evidence needed for later representative review without converting to UTC or selecting an equal-jiffy/cross-stream ordering policy.
- **Validation:** The route integration test uses three independent Regular Market entries around one rejected Pre-Open modify and proves exact before/equal/after counts of 1/1/1, candidate-event count 1, entry-candidate count 3, and reconciliation success. Existing modify/cancel routes both prove before-event relations. All 161 default Rust tests pass.
- **Related:** The cross-session route aggregation entry immediately below; unresolved raw-jiffy ordering and epoch gates.
- **Follow-up:** Measure these relations on representative current paired Trim data; equal or after relations remain observations, not causality or vendor-fault claims.

---

## 2026-10-18 - Aggregate cross-session candidate routes

- **Changed:** Cross-session lifecycle dossiers now aggregate deterministically by rejected operation, rejected-event session, and candidate-entry session. Each route keeps separate candidate-event and entry-candidate counts so duplicate entry evidence across independent streams is not mistaken for multiple rejected events.
- **Why:** Representative analysis needs route-specific measurements such as Pre-Open modify to Regular Market entry, while preserving the distinction between event frequency and source multiplicity.
- **Validation:** Exact integration assertions cover Pre-Open modify and Closing Auction cancel routes to Regular Market. A separate two-stream test proves one rejected event with two independent entry candidates yields event count 1 and entry-candidate count 2. All 161 default Rust tests pass.
- **Related:** The cross-session dossier and observation-taxonomy entries below; Phase 1 session-validation metrics.
- **Follow-up:** Run route aggregation on representative current paired Trim deliveries; use the retained dossiers for timing review before interpreting any route as lifecycle carry.

---

## 2026-10-18 - Surface cross-session candidates in the observation taxonomy

- **Changed:** The composed Trim evidence taxonomy now emits a nonzero `OrderCrossSessionLifecycleCandidate` count per Order stream, derived from the retained cross-session lifecycle candidate dossiers.
- **Why:** Representative profiling should surface these ambiguity candidates through the same deterministic report used for parser, lifecycle, framing, jiffy, and trade-reference observations rather than requiring a separate manual scan.
- **Validation:** The existing modify/cancel integration dossier now proves the taxonomy reports exactly two candidates for member 1, reports the underlying unknown-modify/unknown-cancel counts, and does not classify member 2's same-session unknown modify as cross-session. All 160 default Rust tests pass.
- **Related:** The immediately following cross-session lifecycle-correlation entry; Phase 1 observation-taxonomy gate.
- **Follow-up:** Representative-data runs must retain the dossiers behind this count and must not assign causality or severity from the count alone.

---

## 2026-10-18 - Correlate rejected lifecycle events across sessions

- **Changed:** Independent Order stream evidence can now correlate a rejected unknown modify/cancel with retained successful-entry evidence having the same exact segment/raw-symbol/raw-series/order-number fields in another session. The report retains the complete rejected event/raw record, anomaly member and coordinate, plus deterministically ordered other-session entry candidates; replay remains unchanged and non-mutating.
- **Why:** Entry reuse alone cannot reveal whether a rejected lifecycle mutation has a plausible entry in another session. This correlation makes that ambiguity directly measurable without weakening session isolation.
- **Validation:** A synthetic integration test proves a Pre-Open modify and Closing Auction cancel are still rejected, then reports their exact Regular Market entry candidate; a Regular Market unknown modify with only Regular Market entry evidence is excluded. The result is invariant to caller source order. All 160 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 session-validation gate.
- **Follow-up:** Measure candidate rates and timing on representative current paired Trim deliveries. A candidate is not proof of lifecycle continuation because identity reuse, cross-stream ordering, and session rules remain unresolved.

---

## 2026-10-18 - Report session-excluded successful-entry occurrences

- **Changed:** Independent Order stream replay can now group successful entry evidence by exact segment/raw-symbol/raw-series/order-number fields while excluding session, and report only identities observed under more than one session code. Every occurrence retains session, stream member, side, raw jiffy, and source coordinate in deterministic order.
- **Why:** Representative delivery analysis needs a direct way to locate possible cross-session identity reuse before deciding whether sessions reset, isolate, or carry any lifecycle state.
- **Validation:** A synthetic integration test proves one exact identity is reported across `PO`, `RM`, and `CA`, proves a different identity repeated only within `RM` is excluded, preserves leading raw symbol padding and exact provenance, and returns the same report when caller source order is reversed. All 159 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 session-validation gate.
- **Follow-up:** Run this report on representative current paired Trim deliveries. An occurrence across sessions is not evidence of lifecycle continuation; session transition/reset/carry semantics remain unresolved and consultation-gated.

---

## 2026-10-18 - Enforce per-source Order session reconciliation

- **Changed:** The Order source replay invariant now requires its `PO`/`RM`/`CA` total to equal its parsed-record count, matching the already-enforced combined Order and per-source Trade invariants.
- **Why:** The session counters were populated correctly, but the public per-source reconciliation method did not verify them. That made the verifier weaker than the documented guarantee.
- **Validation:** A regression test first accepts an intact replay, then mutates its Regular Market count and proves reconciliation rejects the mismatch. All 158 default Rust tests pass.
- **Related:** The immediately following session-count evidence entry; Phase 1 delivery-quality gates.
- **Follow-up:** Keep session occurrence evidence separate from any future transition/reset/carry policy.

---

## 2026-10-18 - Count every documented session in delivery evidence

- **Changed:** Order replay, Order summaries, raw Trade profiles, and the combined Trade report now retain exact `PO`, `RM`, and `CA` parsed-event counts. Per-source counts reconcile to parsed records; cross-stream merging uses checked arithmetic with session-specific overflow errors.
- **Why:** Representative data must reveal actual session coverage before defining resets, transitions, halts, or cross-session carry. Counting session observations is evidence collection, not a session-state policy.
- **Validation:** An integration fixture proves exactly one Pre-Open, Regular Market, and Closing Auction event on both the Order and Trade sides. Existing overflow coverage now also forces and rejects each session-counter overflow. All 157 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 session-validation and delivery-quality gates.
- **Follow-up:** Measure session distributions and cross-session order identities on representative current data, then consult before defining transition/reset semantics.

---

## 2026-10-18 - Expose cross-kind jiffy range overlap

- **Changed:** The composed Trim evidence report now derives deterministic pair lists for overlapping Order stream ranges, overlapping Trade stream ranges, and every overlapping Order-versus-Trade stream range from the one-pass raw-jiffy sequences.
- **Why:** Whether stream ranges overlap is necessary evidence for a merge policy, but overlap alone does not establish event order. Keeping it as explicit member pairs prevents a boolean aggregate from hiding which streams may interleave.
- **Validation:** The clean two-stream fixture proves no Order/Order overlap, exactly one Trade/Trade pair `(1,2)`, and exactly two Order/Trade pairs `(2,1)` and `(2,2)`. The delivery-evidence suite remains 5/5 green; all 156 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 jiffy-overlap and merge-evidence gates.
- **Follow-up:** Measure these pairs and actual equal-jiffy groups on representative paired current data before consulting on cross-stream ordering.

---

## 2026-10-18 - Integrate source jiffy evidence into replay

- **Changed:** Order replay and raw Trade candidate profiling now accumulate the existing min/max, equal-run, and coordinate-exact regression evidence during their primary parse pass. The composed report classifies nonzero Order and Trade source regressions without reparsing or selecting cross-stream order.
- **Why:** Delivery evidence should contain source monotonicity and tie evidence, but a second parsing pass would add avoidable cost and could drift from replay accounting. Reusing one sequence accumulator keeps the standalone profiler and replay semantics aligned.
- **Validation:** A new integration test injects decreasing raw jiffies in both Order and Trade sources and proves exact one-pass sequence counts, regression counts, entry-after-Trade evidence, and deterministic observation order. The existing 12 jiffy-profiler and 16 replay tests remain green. All 156 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 jiffy, replay, and delivery-quality gates.
- **Follow-up:** Representative paired current data remains required to measure real monotonicity, tie runs, range overlap, and regression rates before any merge policy.

---

## 2026-10-18 - Classify delivery evidence observations

- **Changed:** Added a deterministic per-stream observation taxonomy over the composed Trim evidence report. It emits only nonzero counts for Order quarantine, four lifecycle-anomaly classes, Order framing loss, Trade quarantine/framing loss, zero/multiple entry-candidate references, side mismatches, and equal/after raw-jiffy candidates.
- **Why:** A useful anomaly dossier needs stable, contextual categories without prematurely assigning severity, causality, or vendor fault. The complete underlying coordinate/raw evidence remains available in the report.
- **Validation:** One integration test injects one Order quarantine, duplicate entry, Order framing tail, Trade quarantine, Trade framing tail, zero-candidate sell reference, and multiple-candidate buy reference. It proves the exact seven observations are emitted in deterministic member/kind order. All 155 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 anomaly-dossier and delivery-quality gates.
- **Follow-up:** Add representative-data rates and exact example selection only after current paired Trim data is available; thresholds and severity remain owner-reviewed decisions.

---

## 2026-10-18 - Compose deterministic Trim delivery evidence

- **Changed:** Added one in-memory evidence object that validates and independently replays Trim Order sources, computes the checked Order summary, validates the matching Trade delivery, and attaches per-stream Trade candidate/value evidence. Order and Trade stream-set reconciliation now also verifies stored member order against the validated delivery descriptors.
- **Why:** The separate evidence slices need one fail-closed orchestration boundary before they can support a delivery quality report. Composition must not silently cross dates/prefixes, skip members, or imply a temporal merge.
- **Validation:** Three tests prove reversed caller source arrays produce identical complete evidence, incomplete Order membership stops before invalid Trade bytes are read, and a cross-date Trade delivery is rejected before invalid Trade bytes are read. All 154 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 deterministic interface and quality-report gates.
- **Follow-up:** Add contextual anomaly classification to this in-memory report. Durable canonical serialization and hashes remain gated on DEC-2026-009 and evidenced ordering.

---

## 2026-10-18 - Reconcile independent Order replay counts

- **Changed:** Added exact per-source and checked combined Order replay summaries for framed records, parsed records, applied events, quarantines, lifecycle anomalies, and active-order observations. Combined summaries add observations across independent streams without merging equal order identities.
- **Why:** A delivery quality report needs compact Order-side accounting that reconciles parser and lifecycle outcomes while cross-stream event ordering remains unresolved. Naming active rows as observations avoids implying a globally deduplicated book.
- **Validation:** A two-stream test proves framed 3 = parsed 3 + quarantines 0 and parsed 3 = applied 3 + anomalies 0, with two same-key active observations retained independently. A second test forces and rejects overflow for all six counters. All 151 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 independent replay, accounting, and quality-report gates.
- **Follow-up:** Compose Order and Trade summaries into one delivery evidence report, still without temporal merging; validate it on representative paired current data when acquired.

---

## 2026-10-18 - Reconcile independent Trade quantity and notional

- **Changed:** Added an exact integer Trade value summary with parsed Trade count, total quantity, and `price_paise * quantity` notional. Raw Trade profiles now retain per-stream summaries and a checked combined summary while remaining independently ordered; every accumulator and merge is fallible.
- **Why:** Phase 1 needs count/volume/value evidence that is independent of any unresolved event merge. Exact `u128` totals avoid floating-point loss, and explicit overflow errors prevent silent wraparound.
- **Validation:** The synthetic profile proves each Trade contributes count 1, quantity 25, and 6,250,000 paise-notional; two streams reconcile to count 2, quantity 50, and 12,500,000 paise. A separate test forces and rejects count, quantity, notional, and merge overflow. All 149 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 independent-summary and reconciliation gates.
- **Follow-up:** Compare these totals with vendor/reference summaries on representative paired current data; do not interpret the combined sum as causal replay order.

---

## 2026-10-18 - Measure candidate entry jiffies against Trades

- **Changed:** Preserved the exact raw entry jiffy in Order-entry evidence and extended candidate diagnostics with entry-before, equal-jiffy, and entry-after-Trade counts. The count invariant now requires those three raw relations to reconcile to every observed candidate.
- **Why:** Representative-data merge decisions need direct evidence about whether referenced entries numerically precede, tie, or follow their Trades. Raw comparison is useful without converting to UTC or pretending equal jiffies establish event order.
- **Validation:** A dedicated three-stream test proves one candidate in each before/equal/after category from deliberately unsorted sources. Existing missing, multiplicity, side, and raw-stream tests now also prove their exact timing counts. All 148 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 raw-jiffy and trade-reference timing gates.
- **Follow-up:** Measure these relations on representative paired current Trim data; consult before defining equal-jiffy tie-breaking, global merge order, or trade-driven quantity effects.

---

## 2026-10-18 - Bind raw Trade diagnostics to delivery identity

- **Changed:** Retained the validated paired-delivery descriptor on independent Order stream replays and added per-member raw Trim Trade profiling against those replays. The profiler validates Trade source coverage, rejects Order/Trade prefix or date mismatch, preserves parsing quarantine and framing residue, and emits reconciled candidate diagnostics per independent Trade stream.
- **Why:** Aggregate candidate counts are unsafe if Orders and Trades can silently come from different deliveries. Binding the evidence to exact delivery identity creates a fail-closed path for future representative-data measurements without inventing a cross-stream event order.
- **Validation:** Three tests prove deterministic per-member raw Trade profiles, rejection of mismatched dates before deliberately invalid Trade bytes are parsed, and exact preservation/reconciliation of a quarantined record plus partial framing tail. All 147 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 delivery identity, raw-event validation, and trade-reference gates.
- **Follow-up:** Run the raw profile on representative paired current Trim deliveries when acquired; stream-wide aggregation, temporal ordering, and quantity effects remain separate decisions.

---

## 2026-10-18 - Quantify cross-stream trade-reference ambiguity

- **Changed:** Added a pure aggregate profile over Trim Trade events and independently replayed Order streams. It reports trades, two-sided references, total candidates, zero/single/multiple-candidate references, expected-side and side-mismatched candidates, and maximum candidates per reference, with a count-reconciliation invariant.
- **Why:** Candidate evidence needs a compact measurement layer before representative current data can inform a merge policy. The profile quantifies missing and ambiguous source membership without selecting an event order or causal match.
- **Validation:** Three tests prove exact missing/unique counts, multiple-versus-side-mismatch separation across unsorted source members, and a reconciled all-zero profile. All 144 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 representative-data and trade-reference gates.
- **Follow-up:** Run this profile on representative paired current Trim data when acquired; consult before implementing temporal matching, merge ordering, or trade-driven quantity effects.

---

## 2026-10-18 - Preserve all cross-stream trade-reference candidates

- **Changed:** Added deterministic collection of every exact Order-entry candidate for each Trim Trade buy/sell reference across independently replayed streams. Each candidate retains stream member, entry side, and source coordinate; missing, duplicate, and side-mismatched evidence is preserved without ranking.
- **Why:** A single selected match would hide cross-stream ambiguity before a merge policy exists. Candidate preservation lets representative data measure ambiguity without inventing temporal causality or quantity effects.
- **Validation:** Three tests prove unique buy/sell candidates across separate streams, deterministic preservation of two same-key candidates in member order, and retention of a side-mismatched candidate. All 141 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 trade-reference and cross-stream ordering gates.
- **Follow-up:** Measure candidate multiplicity and timing on representative paired data, then consult before defining temporal matches or trade-driven decrements.

---

## 2026-10-18 - Replay paired Order streams independently

- **Changed:** Generalized exact Trim stream-source validation for both profiling and replay, then added deterministic paired-delivery Order orchestration that sorts by member and retains each member around its complete independent replay report.
- **Why:** We need stream provenance and complete-member enforcement before a merge policy exists. Combining streams now would invent causality; dropping stream identity would make quarantines and anomalies unauditable.
- **Validation:** Three tests prove reversed source enumeration yields an identical member-ordered result, identical order keys in separate streams remain independent rather than becoming a false duplicate, quarantine bytes retain stream context, and missing membership fails before parsing. All 138 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 multi-stream provenance and ordering gates.
- **Follow-up:** Run independent replay/profiling on representative paired deliveries; choose and sensitivity-test a cross-stream merge only after measured timing evidence and owner consultation.

---

## 2026-10-18 - Prove fail-closed session isolation

- **Changed:** Made the existing session-key boundary explicit in tests and documentation: the same instrument/order number can coexist independently in `PO`, `RM`, and `CA`; a modify or cancel under another session is an unknown-order anomaly and cannot mutate the active order.
- **Why:** Silent cross-session mutation would corrupt replay. Failing closed is deterministic and reversible while representative data is absent, but it must not be mistaken for evidence that NSE orders never transition between sessions.
- **Validation:** Two tests prove three-way same-ID coexistence and exact non-mutating cross-session modify/cancel anomalies. All 135 default Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 lifecycle/session gates.
- **Follow-up:** Measure real session transitions, halts, file/day boundaries, and closing-auction behavior before defining reset or cross-session carry rules.

---

## 2026-10-18 - Detect payload signatures instead of trusting suffixes

- **Changed:** Added dependency-free content classification for empty, gzip, standard ZIP starts, plain ASCII text, and unknown binary payloads. The classifier accepts bytes only and therefore cannot silently infer compression from a filename.
- **Why:** The supplied legacy CM package contains `.gz` files that are actually plain text or ZIP archives. Extension-driven decompression would fail unpredictably and could route the wrong product into the wrong parser.
- **Validation:** Five default tests prove empty, gzip, three ZIP signatures, permitted ASCII controls, unknown binary, and truncated-magic behavior. The opt-in package test classifies Index/Snapshot/Trades as ASCII and Masters/Circulars as ZIP despite all five names ending in `.gz`. All 133 default Rust tests pass; the external-data test passes independently.
- **Related:** Phase 0 legacy-product qualification; Phase 1 source-integrity gate.
- **Follow-up:** Add explicit suffix/signature mismatch policy and decompression only when real delivery packaging is available; never route the legacy EOD package through the v1.19 parser.

---

## 2026-10-18 - Profile Full field byte domains without decoding

- **Changed:** Added checked per-field byte histograms over exact Full layout slices. Profiles expose observed bytes/counts and digit-only evidence, reject an out-of-bounds field map before reading records, count only complete frames, and retain byte-reconciled framing residue.
- **Why:** Raw byte domains are factual evidence that can narrow parser questions without prematurely choosing signedness, conditional zero rules, or semantic decoding. The public legacy sample is useful for this bounded observation but cannot establish current delivery behavior.
- **Validation:** Three default tests prove exact field-position totals on the current synthetic Full fixture, pre-framing rejection of an invalid map, and exclusion of a partial record with retained residue. The opt-in test scanned all 600,000 public legacy records: every observed byte in six Order and six Trade numeric fields was an ASCII digit, and literal `b` occurred in Order symbols. All 128 default Rust tests pass; the external-data test passes independently.
- **Related:** DEC-2026-008; Phase 0 public-sample diagnostics; Phase 1 Full ambiguity gate.
- **Follow-up:** Keep signedness and Full semantic validation unresolved until representative current files or written vendor clarification are available.

---

## 2026-10-18 - Structurally profile every Full layout era

- **Changed:** Added streaming structural profiles that count exact-width LF-framed records, source bytes, and borrowed failure residue; expose checked byte reconciliation/completeness; and dispatch Full Orders and Trades to 87/91 and 100/101/103-byte widths by validated source date. No Full field semantics are decoded.
- **Why:** The supplied public legacy CM sample can provide real framing evidence even though it cannot validate current Trim, triggers, ordering, or lifecycle semantics. Separating structural evidence from semantic claims lets us use it without overstating what it proves.
- **Validation:** Four default tests prove current fixtures, every historical date/width transition including the one-day exception and rollback, exact partial-tail evidence, repeatability, and full byte accounting. One explicitly invoked local-data test consumed all 300,000 public legacy Order records and all 300,000 Trade records with zero residue. All 125 default Rust tests pass; the external-data test also passes independently.
- **Related:** DEC-2026-008; Phase 0 sample qualification; Phase 1 layout/framing gates.
- **Follow-up:** Keep Full semantic parsing blocked on representative files and clarification of signed fields and Trigger Price rules; acquire current paired data and triggers before any Trim/ordering acceptance claim.

---

## 2026-10-18 - Bind jiffy profiles to paired stream sets

- **Changed:** Added Trim Order and Trade stream-set profiling bound to an already discovered paired delivery. The profiler requires exact member coverage before reading records, normalizes source input to member order, preserves every per-source profile, reconciles counts per member, and reports every pair whose observed raw-jiffy ranges overlap.
- **Why:** Representative-data qualification must not accidentally omit, duplicate, or add a stream, and caller enumeration order must not alter the diagnostic result. Range overlap is evidence for later ordering analysis, not a merge rule.
- **Validation:** Five new tests prove deterministic split-member ordering, identical reports for reversed inputs, exact rejection of missing/duplicate/extra membership before parsing, fail-fast product/kind mismatch, and unsplit Trade handling without inventing a stream number. All 121 Rust tests pass.
- **Related:** DEC-2026-008; Phase 1 jiffy and multi-stream qualification gates.
- **Follow-up:** Run these profiles on representative paired current deliveries, retain the reports with source integrity evidence, and consult before selecting any cross-stream or Orders-versus-Trades replay policy.

---

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
