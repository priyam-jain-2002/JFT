# JFT NSE Short-Horizon Trading System

## Product requirements and gated technical phase plan

This is the single execution source of truth for JFT. It combines the product brief, technical architecture, data contract, research plan, acceptance gates, kill criteria, and phase checklist for an NSE-focused short-horizon quantitative trading system. It replaces the generic lifecycle checklist; it does not authorize trading or establish that the strategy is profitable or legally permitted.

- **Document status:** Governing plan; implementation not started
- **Current phase:** Phase 0 - governance, evidence, licensing, and data qualification
- **Last updated:** 2026-10-15
- **Decision owner:** Project owner
- **Initial market/mode:** NSE Cash Market (CM), offline research only
- **Next gate:** Written data-use clarification and a qualified CM sample-data report
- **Primary red flag:** The supplied NDAL agreement may prohibit commercial use or financial gain, despite later language about trading and investment decisions
- **Related:** [full decisions](decisions/DECISION_LOG.md), [compact decisions](decisions/COMPACT_DECISION_LOG.md), [research index](research/README.md), [change ledger](CHANGE_LEDGER.md)

## 1. Evidence labels

- **CONFIRMED FROM NSE DOCUMENTATION** - stated in supplied NSE/NDAL material. This is not a warranty of correctness, legal advice, or proof of fitness.
- **DESIGN DECISION** - the current engineering/product choice; it governs work until superseded.
- **RESEARCH HYPOTHESIS** - a falsifiable proposition that must survive chronological out-of-sample (OOS) testing after realistic costs.
- **UNRESOLVED / REQUIRES VALIDATION** - missing, ambiguous, conflicting, or sample-dependent information that cannot be silently assumed.

## 2. Governing thesis and evidence chain

**RESEARCH HYPOTHESIS:** A learned representation of high-frequency NSE market state can predict short-horizon return distributions strongly enough to produce positive net expectancy after spread, fees, slippage, impact, latency, fill uncertainty, and risk constraints.

**DESIGN DECISION:** Prediction and execution are separate. A predictive model emits calibrated return, direction, volatility, and liquidity estimates. A deterministic policy and hard risk engine decide whether, when, how, and how much to trade.

```text
DATA -> VALID RECONSTRUCTION -> SIGNAL -> OUT-OF-SAMPLE SIGNAL
     -> NET ALPHA -> LIVE SHADOW ALPHA -> SMALL-CAPITAL ALPHA
     -> SCALABLE ALPHA -> TRADING FARM -> OPTIONAL PRODUCT
```

A later stage may not compensate for a failed earlier stage. More features, compute, models, or capital are not remedies for invalid data, leakage, or negative net expectancy.

## 3. Product definition

### 3.1 User, problem, and scope

The initial user is the project owner acting as an internal quantitative researcher and proprietary-system operator. The project asks:

> At which of 1 second, 5 seconds, 30 seconds, 1 minute, and 5 minutes does NSE CM microstructure retain enough signal for positive OOS expectancy after achievable costs and latency?

- **DESIGN DECISION:** Start with NSE CM, one liquid instrument, then approximately 5-10 liquid names only after the one-instrument pipeline passes.
- **DESIGN DECISION:** Treat 250ms-1s as a later stretch track, not the initial production target.
- **DESIGN DECISION:** Qualify CM Trim Orders and Trades first; buy/use Full only if measured reconstruction, feature, or execution value justifies it.
- **DESIGN DECISION:** Preserve raw exchange values and build deterministic replay before feature engineering.
- **DESIGN DECISION:** Prove interpretable baselines before neural models.
- **DESIGN DECISION:** Keep research, live inference, risk, execution, broker adapters, and any user product as separate boundaries.

### 3.2 Success and non-success

Success requires licensed/auditable data use, byte-correct parsing, deterministic reconstruction, leakage-free chronological validation, predictive value beyond simple baselines, positive net expectancy under conservative costs, stability across instruments/days/regimes, shadow-to-offline parity, bounded small-capital validation, and measured capacity.

These are not success: shuffled or in-sample accuracy; gross P&L before costs; one profitable period; a neural model without economic uplift; impossible fills; irreproducible results; an edge shorter than attainable latency; or a system dependent on unresolved data rights.

### 3.3 Non-goals before validated alpha

Do not build exchange colocation, FPGA/microsecond infrastructure, a full RL trading engine, LLMs in the fast path, mobile/social/copy trading, real-money gamification, multiple brokers, a marketplace, external accounts, global-market support, or a public platform. Do not optimize sub-millisecond latency until measured alpha decay and economics require it.

## 4. Phase status

Legend: `[x]` verified complete, `[ ]` remaining, `[-]` intentionally skipped with a reason.

| Phase | Status | Gate outcome |
|---|---|---|
| 0 - Governance, evidence, licensing, data qualification | **In progress** | Legal/data basis permits bounded reconstruction research |
| 1 - Deterministic parser and book reconstruction | Not started | Canonical replay is identical and passes invariants |
| 2 - Features, labels, baselines | Blocked by Phase 1 | Economically meaningful OOS signal beats naive baselines |
| 3 - Execution-grade simulator | Blocked by Phase 2 | Signal survives conservative cost/fill/latency assumptions |
| 4 - Neural representation learning | Blocked by Phase 3 | Neural model adds stable OOS net value |
| 5 - Regime-aware ensemble | Blocked by Phase 4 | Specialization adds robust value without unstable gating |
| 6 - Walk-forward/final OOS validation | Blocked by Phase 5 | Frozen candidate survives untouched data and stress tests |
| 7 - Live shadow system | Blocked by Phase 6/legal gate | Live paper outcomes match offline tolerances |
| 8 - Broker abstraction/hard risk | Blocked by Phase 7 | Simulated execution fails closed and reconciles |
| 9 - Small-capital validation | Blocked by Phase 8/authorization | Real fills validate net edge within loss limits |
| 10 - Capacity/scaling/trading farm | Blocked by Phase 9 | Measured capacity supports staged scale |
| 11 - Optional product/platform | Blocked by Phase 10 | Separate product, compliance, and safety case passes |

## 5. Source and evidence register

### 5.1 Supplied governing material

| Source | Classification | Governing use | Limitation/action |
|---|---|---|---|
| `NSE_Hist_Order_Trade_Data_1.19_20260902144045.pdf`, v1.19, 03 Aug 2026 | **CONFIRMED FROM NSE DOCUMENTATION** | CM layouts, jiffies, availability, checksums, delivery, record history | Archive through the read-only source sync; verify against the subscribed product |
| `NDAL-v.2024-Historical Order and Trade Data Agreement.docx` | **CONFIRMED FROM NSE DOCUMENTATION** | Use, confidentiality, redistribution, audit, disclaimer, indemnity | Not legal advice; apparent use conflict needs written NDAL/counsel resolution |
| `NDAL-v.2024-Historical Trade Undertaking.doc` | **CONFIRMED FROM NSE DOCUMENTATION** | Corroborates restrictive historical-data terms | Older/general; Order and Trade agreement is more directly relevant |
| Other supplied CM/EOD/segment documents | **UNRESOLVED / REQUIRES VALIDATION** | Possible master/reference and later-segment context | Catalogue individually before reliance |
| Referenced `Compare Trading Algorithms` conversation | **DESIGN DECISION** | Thesis, architecture, model order, budgets, gates | Rationale, not evidence of law, market behavior, prices, or profitability |

No attachment may be committed without confirming repository storage rights. `sources/` remains read-only and synchronization-owned.

### 5.2 Research-paper registry

| ID | Candidate | Question and JFT reproduction | Status/limitation |
|---|---|---|---|
| RP-001 | Cont, Kukanov, Stoikov, *The Price Impact of Order Book Events* | Reproduce OFI sign, monotonicity, and depth interaction on NSE CM | External-market evidence does not establish NSE net alpha |
| RP-002 | Gould and Bonart, *Queue Imbalance as a One-Tick-Ahead Price Predictor in a Limit Order Book* | Compare calibrated imbalance logistic model with naive baselines | Queue state/event order must first be reliable |
| RP-003 | Cartea, Donnelly, Jaimungal, order-book imbalance/trading signals | Test prediction and a separate cost-aware policy | Exact citation/version to verify in Phase 0 |
| RP-004 | Zhang, Zohren, Roberts, *DeepLOB* | Compact CNN/TCN reproduction after Phase 3 | Labels/markets differ; accuracy is insufficient |
| RP-005 | Purging, embargo, walk-forward, multiple-testing literature | Define split/leakage controls | Primary citations to complete in Phase 0 |
| RP-006 | Market-impact and queue/fill literature | Calibrate pessimistic/base/optimistic execution cases | Evidence gap before capital |

Each completed entry must add a citation, stable source, publication status, dataset/period, method, result, limitations, relevance, reproduction plan, and JFT outcome.

### 5.3 Patent and prior-art registry

Patents show prior art and possible architecture, not profitability, patentability, or freedom to operate.

| ID | Candidate | Signal from prior scan | Required action |
|---|---|---|---|
| PA-001 | US7529703B2 | Neural financial-market direction prediction | Verify title, claims, family, and status professionally |
| PA-002 | US11410240B2 | Models specialized by regime/pattern | Verify independent claims; compare mixture-of-experts literature |
| PA-003 | US11593877B2 | Supervised predictor separated from RL execution | Verify claims before RL execution work |
| PA-004 | Identifier TBD | Reconstructed book, imbalance, learned embeddings | Recover exact family/claims; web summaries are insufficient |
| PA-005 | Identifier TBD | Generative/synthetic LOB simulation | Recover family; consider only after real simulator validation |

Any filing or commercial release requires professional patentability/FTO review. Broad neural prediction, regime ensembles, and RL execution already have extensive prior art.

## 6. NSE CM data contract

### 6.1 Product-level facts and unknowns

- **CONFIRMED FROM NSE DOCUMENTATION:** Trim is derived from Full, excludes auxiliary fields, retains core identifiers/timestamps/prices/quantities/indicators, and uses fixed-length records.
- **CONFIRMED FROM NSE DOCUMENTATION:** CM Trim is EOD: 68-byte Orders and 82-byte Trades, LF-delimited.
- **CONFIRMED FROM NSE DOCUMENTATION:** Full is the complete published representation. Full Orders lag 90 calendar days; Full Trades lag 30 days.
- **CONFIRMED FROM NSE DOCUMENTATION:** Current Full CM is 91-byte Orders/103-byte Trades. Historical parsing must handle 87/91-byte Orders and 100/101/103-byte Trades by date; 28 Mar 2025 is a documented 91/103 one-day exception, and 91/103 applies from 16 Jun 2025.
- **CONFIRMED FROM NSE DOCUMENTATION:** CM files were split into streams from 01 Jul 2025; stream count may grow. A symbol may span streams when series differ.
- **CONFIRMED FROM NSE DOCUMENTATION:** Per-file `.trg` files carry MD5 and compressed size. Layout notes show `CM_...`; trigger tables show `CASH_...`.
- **UNRESOLVED / REQUIRES VALIDATION:** Delivered prefixes, stream discovery, missing-stream representation, and trigger delimiters require real samples.
- **UNRESOLVED / REQUIRES VALIDATION:** Trim appears sufficient for aggregate replay, but the specification does not explicitly settle post-trade quantity semantics, modify priority, cross-file ties, or exact queue reconstruction. Sample replay is the gate.

### 6.2 CM Trim Orders exact fixed-width layout

Positions are 1-based inclusive byte spans before LF.

| # | Bytes/span | Field | Canonical interpretation |
|---:|---|---|---|
| 1 | 2 / 1-2 | Record indicator | `PO`, `RM`, `CA` |
| 2 | 4 / 3-6 | Segment | `CASH` |
| 3 | 16 / 7-22 | Order number | Preserve string plus validated integer |
| 4 | 14 / 23-36 | Transaction time | Raw jiffy integer; never float-canonicalized |
| 5 | 1 / 37 | Buy/sell | `B` or `S` |
| 6 | 1 / 38 | Activity | `1` entry, `3` cancel, `4` modify |
| 7 | 10 / 39-48 | Symbol | Space-padded; raw and normalized |
| 8 | 2 / 49-50 | Series | Examples `EQ`, `BE`, `BL`, `DR` |
| 9 | 10 / 51-60 | Order quantity | Non-zero, non-negative in current layout |
| 10 | 8 / 61-68 | Order price | Integer paise; decimal is derived |

### 6.3 CM Trim Trades exact fixed-width layout

| # | Bytes/span | Field | Canonical interpretation |
|---:|---|---|---|
| 1 | 2 / 1-2 | Record indicator | `PO`, `RM`, `CA` |
| 2 | 4 / 3-6 | Segment | `CASH` |
| 3 | 14 / 7-20 | Trade time | Raw jiffy integer |
| 4 | 10 / 21-30 | Symbol | Space-padded; raw and normalized |
| 5 | 2 / 31-32 | Series | Instrument series |
| 6 | 8 / 33-40 | Trade price | Positive integer paise |
| 7 | 10 / 41-50 | Trade quantity | Positive integer shares |
| 8 | 16 / 51-66 | Buy order number | Order-lifecycle join candidate |
| 9 | 16 / 67-82 | Sell order number | Order-lifecycle join candidate |

Trim's listed CM trade layout has no trade number or participant/algo flags.

### 6.4 CM Full exact fixed-width layouts

Current Full CM Orders (91 bytes):

| # | Bytes/span | Field |
|---:|---|---|
| 1 | 2 / 1-2 | Record indicator (`PO`, `RM`, `CA`) |
| 2 | 4 / 3-6 | Segment (`CASH`) |
| 3 | 16 / 7-22 | Order number |
| 4 | 14 / 23-36 | Transaction time in jiffies |
| 5 | 1 / 37 | Buy/sell indicator |
| 6 | 1 / 38 | Activity (`1`, `3`, `4`) |
| 7 | 10 / 39-48 | Symbol |
| 8 | 2 / 49-50 | Series |
| 9 | 10 / 51-60 | Volume disclosed; zero for non-disclosed-quantity orders |
| 10 | 10 / 61-70 | Volume original |
| 11 | 8 / 71-78 | Limit price in paise |
| 12 | 8 / 79-86 | Trigger price in paise; zero for non-stop-loss orders |
| 13 | 1 / 87 | Market-order flag (`Y`/`N`) |
| 14 | 1 / 88 | Stop-loss flag (`Y`/`N`) |
| 15 | 1 / 89 | IOC flag (`Y`/`N`) |
| 16 | 1 / 90 | Algo indicator (`0` algo, `1` non-algo, `2` algo via SOR, `3` non-algo via SOR) |
| 17 | 1 / 91 | Client identity (`1` custodian, `2` proprietary, `3` client) |

Current Full CM Trades (103 bytes):

| # | Bytes/span | Field |
|---:|---|---|
| 1 | 2 / 1-2 | Record indicator (`PO`, `RM`, `CA`) |
| 2 | 4 / 3-6 | Segment (`CASH`) |
| 3 | 17 / 7-23 | Trade number |
| 4 | 14 / 24-37 | Trade time in jiffies |
| 5 | 10 / 38-47 | Symbol |
| 6 | 2 / 48-49 | Series |
| 7 | 8 / 50-57 | Trade price in paise |
| 8 | 10 / 58-67 | Trade quantity |
| 9 | 16 / 68-83 | Buy order number |
| 10 | 1 / 84 | Buy algo indicator (`0`-`3`) |
| 11 | 1 / 85 | Buy client identity (`1`-`3`) |
| 12 | 16 / 86-101 | Sell order number |
| 13 | 1 / 102 | Sell algo indicator (`0`-`3`) |
| 14 | 1 / 103 | Sell client identity (`1`-`3`) |

**DESIGN DECISION:** Full-only identity fields are not needed for the initial thesis. Market/IOC/trigger/disclosed/original fields may improve lifecycle and fills; the Trim-vs-Full report must measure their value rather than assume it.

### 6.5 Jiffy policy

- **CONFIRMED FROM NSE DOCUMENTATION:** 65,536 jiffies equal one second, counted from 01 Jan 1980 00:00:00 GMT. NSE's example converts to Unix time then adds 05:30 for IST display; it warns Excel is millisecond-limited.
- **DESIGN DECISION:** `exchange_jiffy` is an exact unsigned integer copied from the 14 bytes.
- **DESIGN DECISION:** Convert with integer/rational arithmetic and retain the fractional remainder; never round-trip through float.
- **DESIGN DECISION:** Store UTC; derive session date/display in `Asia/Kolkata`.
- **DESIGN DECISION:** Same-jiffy events retain file/stream/record provenance. A reproducible tie-breaker is not claimed as exchange causality.
- **UNRESOLVED / REQUIRES VALIDATION:** Cross-stream and Orders-vs-Trades tie ordering, per-stream monotonicity, and session edge cases.

```text
whole_seconds = exchange_jiffy // 65536
subsecond_jiffies = exchange_jiffy % 65536
utc = 1980-01-01T00:00:00Z + whole_seconds
fraction = subsecond_jiffies / 65536 second
```

### 6.6 Canonical schemas

Every table includes schema/spec version, source hashes/logical path/file part/stream/record index, raw record or immutable pointer, jiffy, session date, indicators, segment, raw/normalized symbol, series, parse status, and deterministic event key.

- **Order event:** order ID, side, activity, quantity, price paise, nullable Full-only attributes.
- **Trade event:** nullable trade ID, price/quantity, buy/sell order IDs, nullable Full-only attributes.
- **Lifecycle state:** instrument/session/order ID, side, price, visible quantity, entry/last-event jiffies, status, last-mutation provenance, anomaly flags.
- **Book delta/snapshot:** levels, order counts, BBO, spread, mid, microprice, last trade, cumulative volume, event sequence, state hash.
- **Feature row:** as-of event/jiffy, sampling policy, lookback boundaries, feature version, past-only inputs.
- **Label row:** horizon, anchor, future observation, raw return, cost threshold, target/class, censor reason, version. Features and labels stay separate.

## 7. Reconstruction semantics and invariants

### 7.1 Candidate lifecycle reducer

1. Entry (`1`) requires no active `(session, segment, symbol, series, order_id)` and creates state.
2. Modify (`4`) requires active state and replaces published fields. For fill simulation, price change or quantity increase conservatively loses queue priority; this is not asserted as an NSE fact.
3. Cancel (`3`) requires active state and closes/removes it; cancel quantity/price validation waits for samples.
4. A trade references buy and sell order IDs. Whether to decrement active quantities directly, await order events, or handle mixed behavior is **UNRESOLVED / REQUIRES VALIDATION**.
5. Unknown-order modify/cancel/trade references are quarantined with context, never silently repaired.
6. `PO`, `RM`, and `CA` are separate validation/analytics sessions; continuous-spread rules are not blindly applied to auctions.

### 7.2 Mandatory invariants

- Trigger size/MD5 pass before decompression; internal SHA-256 establishes lineage.
- Every record has date-appropriate length and valid field structure.
- Parsed plus quarantined equals source count; no silent drop/duplication.
- Enum, numeric, padding, segment, price, and quantity rules match versioned schema.
- Identical bytes/configuration yield byte-identical canonical rows/checkpoint hashes.
- Event keys are unique; duplicate source overlap/conflict is classified.
- Lifecycle transitions are valid or quarantined; quantity/depth never goes negative.
- In regular continuous trading, bid is normally below ask; locked/crossed states are preserved and investigated, not discarded.
- Trade joins report matched, unmatched, late, and ambiguous rates by session/instrument/stream.
- End-state and traded quantity/value reconcile to lawful independent summaries when available.
- Same-jiffy outcomes are sensitivity-tested under defensible tie policies; material variation blocks exactness claims.
- Checkpoints contain source/config/code/schema versions, counts, jiffy bounds, anomalies, and book hash.

## 8. Target architecture

### 8.1 Offline plane

```text
licensed immutable files + triggers -> integrity/manifest -> parser/quarantine
-> canonical events -> deterministic replay -> book deltas/snapshots
-> point-in-time features + separate labels -> chronological datasets
-> experiments/model registry -> locked evaluation/promotion evidence
```

Use encrypted immutable object storage, versioned columnar datasets, lineage/catalog metadata, ephemeral CPU workers, GPU only after baseline evidence, and a registry binding model/data/code/environment/metrics/calibration/approval. Raw NSE data must not enter logs, public artifacts, or unapproved services.

### 8.2 Live plane

```text
licensed live feed -> normalized adapter -> stateful features -> compact inference
-> deterministic policy -> hard risk -> execution -> broker adapter -> broker
all stages -> append-only audit/outcome store -> reconciliation/monitoring
```

Training and live inference are separate failure domains. No LLM or automatic online retraining is allowed in the fast path. Start with a small observable service near the broker, warm standby, monotonic internal sequencing, durable audit, time sync, health checks, and fail-closed behavior.

### 8.3 Promotion and drift

Model states: `experimental -> reproducible -> OOS-qualified -> shadow candidate -> shadow-qualified -> production candidate -> approved -> active -> retired`. Human approval and an immutable evidence bundle are mandatory; a better offline score never auto-promotes.

Monitor schema/data quality, feature/prediction/calibration/outcome/regime drift, net expectancy/slippage/fills/drawdown, simulator divergence, latency/gaps/duplicates/clock offset, and broker reconciliation. Retraining creates a challenger that repeats offline, walk-forward, and shadow gates. Rollback is an approved-model switch.

## 9. Feature, label, validation, and model contract

### 9.1 Features and labels

Feature families: BBO/mid/spread/microprice; depth/order count/slope/concentration; L1-LN queue imbalance and OFI; entry/cancel/modify/trade flow and inter-arrival intensity; short returns/volatility/replenishment/resiliency/level age; time/session/instrument context; and later index/sector/cross-asset/slow context/learned embeddings. Inferred aggressor direction is method/version-labelled, never a fact.

For 1s/5s/30s/1m/5m create future mid return; cost-thresholded down/flat/up; future volatility; future spread/depth; maximum favorable/adverse excursion; and hypothetical-entry adverse selection. Keep wall-clock, event-clock, and decision sampling separate. Specify auction/halts/corporate actions/session-end censoring.

### 9.2 Leakage controls and model sequence

- Split by time before scalers, encoders, thresholds, features, calibration, or tuning.
- Point-in-time/as-of joins only; purge overlapping labels and embargo boundaries.
- Keep overlapping instrument-day groups together; use as-known corporate/reference data.
- Lock final OOS before iteration; opening it consumes the holdout.
- Record researcher trials and control selection/multiple testing.

```text
naive -> linear/logistic -> gradient boosting -> small MLP
      -> TCN/CNN/DeepLOB-like -> regime-aware ensemble
```

Naive baselines include no-change, last-sign, unconditional class, simple imbalance, and cost-aware abstain. Report log loss, Brier/calibration, relevant ROC/PR, MAE/rank metrics alongside net expectancy, turnover, risk-adjusted metrics with assumptions, drawdown/tail loss, payoff, capacity, and stress sensitivity. Complexity needs stable OOS net improvement, not a single best score.

## 10. Execution simulator contract

The event-driven simulator consumes the same canonical stream and decision/order schema as shadow/live. It must support market/limit/modify/cancel, partial fills, expiry/rejects; decision/network/broker/exchange/ack/cancel latency distributions; dated fees/taxes; spread/slippage/impact; pessimistic/base/optimistic queue cases; gaps/stale/disconnect/duplicates/out-of-order/reject races; inventory/cash/margin/limits; and complete audit/P&L attribution.

No fill occurs because a bar touched a price. Marketable orders consume observable/estimated liquidity after latency. Passive fills need contra flow and queue depletion. Parameters are calibrated from market data and later real acknowledgements/fills, never tuned for backtest P&L.

## 11. Detailed phase plan

### Phase 0 - Governance, evidence, licensing, and data qualification

**Status:** In progress.

**Objective/rationale:** Establish a legal, factual, budgeted, reproducible basis for a bounded CM reconstruction experiment. Invalid rights or inadequate data stop the project regardless of model quality.

**Prerequisites:** Repository governance and supplied source material.

**Work packages:**

- [x] Preserve the read-only `sources/` boundary and linked governance records.
- [x] Establish this governing plan and evidence labels.
- [ ] Archive/version supplied documents through the source-sync process; record hashes without committing restricted content.
- [ ] Obtain the subscription/order form, tariff, delivery/retention/site/cloud/user/derived-output terms, audit duties, termination/deletion rules, and redistribution limits.
- [ ] Send NDAL the precise intended uses: proprietary training/backtesting, simulator, derived features/models, cloud processing, live proprietary trading, retention, DR, and optional future platform.
- [ ] Obtain written NDAL clarification and counsel review of the conflict among internal use, no commercial use/financial gain, and Annexure-II use facilitating trading/investment decisions.
- [ ] Confirm whether weights, features, aggregate metrics, paper results, and trade decisions are restricted outputs/derivatives.
- [ ] Acquire representative paired CM Trim Orders/Trades and `.trg` files across ordinary, volatile, special/low-volume, pre-open, regular, and closing-auction periods.
- [ ] If allowed/affordable, acquire matched Trim/Full days for field and replay comparison.
- [ ] Inventory dates, symbols/series, streams, sizes, counts, jiffy ranges, missing files, exact names, schema eras, corporate actions, and reference data.
- [ ] Review SFTP SSH-key/static-IP or client-owned Mumbai S3 delivery, least privilege, encryption, retention, deletion, and audit logging.
- [ ] Verify paper citations and patent identifiers; record reproduction/legal needs.
- [ ] Approve the Phase 1 budget and research-spend stop-loss.

**Expected artifacts/files:** This plan and existing governance records; non-secret source manifest; license clarification record; qualification report. Restricted data stays out of Git.

**Tests/metrics:** Hash inventory; clause matrix; sample MD5/size verification; version matrix; representative-day coverage; 100% files catalogued and integrity-paired; 100% legal questions answered or blocking; zero restricted data in Git/logs; cost estimate approved.

**Blockers:** Unresolved use clause; no paired Orders/Trades/triggers; cloud use forbidden; unrepresentative sample; unaffordable data.

**Acceptance/exit:** Offline use is documented as permitted; live/profit use is explicitly permitted or fenced off; samples support Phase 1; owner approves budget.

**Return/kill:** Return to vendor qualification if delivery differs from v1.19. Stop or choose a lawful alternative if rights, quality, or price make the evidence chain infeasible.

**Must not build:** Features, models, simulator, live feed, broker integration, product UI.

### Phase 1 - Deterministic parser and order-book reconstruction

**Objective/rationale:** Produce byte-auditable canonical events and deterministic book replay. Every later claim depends on correct event meaning, ordering, and lineage.

**Prerequisites:** Phase 0 permits offline work; representative paired data/triggers exist.

```text
Orders + Trades -> integrity -> parser -> raw-event validation -> jiffy validation
-> event ordering -> lifecycle replay -> book reconstruction -> trade reconciliation
-> invariant/golden tests -> canonical dataset
```

**Work packages:**

- [ ] Freeze compressed inputs; verify trigger MD5/size, compute SHA-256, write manifest.
- [ ] Discover all parts/streams rather than hard-code a count; reject missing/duplicate parts.
- [ ] Implement date/version-aware schemas for Trim 68/82 and required Full variants.
- [ ] Preserve raw bytes/padding/file/stream/index/jiffy and a reason for every quarantine.
- [ ] Test jiffy epoch, day boundaries, NSE example, fractional remainder, timezone display, and ties.
- [ ] Profile per-stream monotonicity, equal-jiffy groups, overlap, and trade-reference timing before merge policy.
- [ ] Implement explicit entry/modify/cancel reducer and anomaly classes; no silent repair.
- [ ] Reconstruct order- and price-level state by instrument/session; hash checkpoints/final state.
- [ ] Join trades to both order IDs and test competing execution-decrement interpretations.
- [ ] Separate `PO`, `RM`, `CA`; define transitions, halts, special sessions, and daily reset.
- [ ] Reconcile counts, volume/value, residue, and independent summaries where lawful.
- [ ] Compare matched Trim/Full replay, lifecycle matches, state divergence, lost features, storage/runtime.
- [ ] Produce contextual anomaly dossiers and request vendor clarification for systematic ambiguity.
- [ ] Emit canonical columnar data, schema docs, manifests, and deterministic replay interface.

**Expected modules/files (proposed, not created now):**

```text
src/jft/data/nse_cm/{layouts,manifest,parser,jiffy,events,lifecycle,book,reconcile,quarantine}.py
tests/data/nse_cm/test_{layouts,jiffy,lifecycle,replay_determinism,trade_reconciliation}.py
tests/fixtures/nse_cm/synthetic_nonlicensed/
```

The implementation language may be superseded by decision; the interfaces/evidence contract may not.

**Tests:** Golden bytes for every era/type; width/numeric/padding property/fuzz tests; transition matrix; corrupt trigger/partial/missing/duplicate stream; unknown order; overfill; negative depth; repeated ID; same-jiffy permutation sensitivity; auctions/special sessions; daily reset; deterministic outputs across worker counts.

**Metrics/artifacts:** Parsed/quarantined/duplicate counts; trigger rate; lifecycle anomaly and trade-match rates; same-jiffy ambiguity; crossed/locked duration; reconciliation difference; hash stability; runtime/memory/compression; Trim/Full divergence. Deliver schema, package, fixtures, anomaly taxonomy, daily quality report, Trim/Full recommendation, performance profile.

**Blockers:** Undocumented era, missing streams, systematic unknown orders, unresolved trade semantics, material tie sensitivity, or unacceptable Trim/Full divergence.

**Acceptance/exit:** Identical input/configuration yields identical canonical bytes/book hashes; every record is accounted for; invariants pass or have approved exceptions; reconciliation meets a predeclared evidence-backed tolerance; owner selects Trim or approves Full.

**Return/kill:** Return to Phase 0/vendor for unresolved semantics. Kill this alpha path if no viable licensed dataset supports trustworthy reconstruction.

**Must not build:** Alpha features beyond diagnostics, production labels/models/backtests, broker/live code.

### Phase 2 - Features, labels, and simple baselines

**Objective/rationale:** Test auditable microstructure relationships before execution/neural complexity.

**Prerequisites:** Accepted Phase 1 dataset and feature eligibility matrix.

**Work:** Implement Section 9 contracts; choose one instrument and at least one year if available; pre-register splits/metrics; build naive, linear/logistic, gradient-boosted, and small-MLP baselines; calibrate; ablate by feature/horizon/session/regime; audit leakage/trial count.

**Expected modules:** `src/jft/features/`, `labels/`, `datasets/`, `models/baselines/`, `evaluation/`, tests/configs.

**Tests/metrics/artifacts:** As-of/boundary/split/purge/embargo and label tests; log loss, calibration, MAE/rank, gross thresholded expectancy, confidence/stability slices; dataset card, dictionary, pre-registration, leaderboard, leakage/ablation reports.

**Acceptance/exit:** At least one horizon repeatedly improves on naive OOS baselines with calibrated predictions and plausible gross edge large enough for cost testing.

**Blockers/kill/return:** No stable signal, untradeable horizon, or leakage-sensitive result means stop or revise; never jump to deep learning.

**Must not build:** DeepLOB/transformers, ensembles, RL, live or broker code.

### Phase 3 - Execution-grade simulator and cost validation

**Objective/rationale:** Determine whether baseline signal is tradable; predictive accuracy is not alpha.

**Prerequisites:** Phase 2 signal and dated fee/latency/fill evidence.

**Work:** Implement Section 10; define decision/order schemas; version fees; estimate latency; queue/fill/impact cases; risk/accounting/outages/reconciliation; cost/latency/capacity/adversarial sensitivity.

**Expected modules:** `src/jft/sim/{market,orders,latency,fees,fills,impact,account,risk}.py`, scenarios and golden tests.

**Tests/metrics/artifacts:** Cash/position conservation; no impossible fills; fee arithmetic; deterministic seeds; rejects/cancel races/partials/gaps; net expectancy, turnover, fills, shortfall, drawdown, tail loss, break-even cost/latency, capacity. Deliver simulator spec, evidence, scenarios, net-alpha report, limitations.

**Acceptance/exit:** Positive net OOS expectancy persists under conservative base assumptions and remains understandable under adverse cases without impossible queue priority.

**Blockers/kill/return:** Edge smaller than friction, too latency-sensitive, or optimistic-fill-only means stop, change horizon, or return to Phase 2.

**Must not build:** Neural models to rescue negative net baselines, live routing, capital deployment.

### Phase 4 - Neural representation learning

**Objective/rationale:** Test whether learned LOB representations add economic value over qualified baselines.

**Prerequisites:** Phase 3 net-positive baseline and compute approval.

**Work:** Freeze benchmark; compact CNN/TCN/DeepLOB-like encoder; multi-horizon direction/return/volatility/liquidity heads; cost-aware evaluation; calibration; latency/capacity; ablations and multiple seeds.

**Expected modules:** `src/jft/models/{encoders,heads,training}/`, model cards/configs/reproducibility tests.

**Tests/metrics/artifacts:** Seed variance, learning curves, calibration, net uplift under identical policy, inference p50/p95/p99, compute, stability; deliver reproducible candidate, cards, ablations, economic comparison.

**Acceptance/exit:** Stable OOS net uplift over best simpler model across folds/seeds; inference is comfortably below signal half-life; cost is acceptable.

**Blockers/kill/return:** Accuracy-only, unstable, expensive, leakage-sensitive, or period-concentrated uplift means retain the simpler model.

**Must not build:** Giant model, online learning, RL execution, automatic promotion.

### Phase 5 - Regime-aware ensemble

**Objective/rationale:** Test specialization across states without post-hoc regime overfit.

**Prerequisites:** At least two qualified behaviorally distinct candidates.

**Work:** Define observable ex-ante volatility/liquidity/trend/session regimes; compare hard routing, soft mixture, simple averaging; enforce samples; regularize gate; transition/abstention/fallback tests.

**Expected modules:** `src/jft/regimes/`, `src/jft/models/ensemble/`, gating/fallback tests/configs.

**Tests/metrics/artifacts:** Stability/turnover/occupancy, conditional calibration/net result, transition loss, entropy, worst-regime drawdown, single-model uplift; deliver regime definition, cards, fallback runbook, robustness report.

**Acceptance/exit:** Ensemble improves aggregate and worst-regime OOS net outcomes without future-known regime labels or unstable gating.

**Blockers/kill/return:** Small samples, unstable regimes/gate, or no uplift means keep one model.

**Must not build:** Post-hoc P&L regimes or autonomous model selection.

### Phase 6 - Rigorous walk-forward and final OOS validation

**Objective/rationale:** Challenge a frozen strategy/model/policy as a release candidate; iteration consumes information.

**Prerequisites:** Frozen candidate/simulator/cost assumptions/predeclared rules.

**Work:** Rolling/expanding walk-forward; purging/embargo/nested tuning; locked final OOS; subgroup slices; cost/latency/impact and missing-data shocks; confidence/bootstrap; trial audit; independent manifest reproduction.

**Expected modules:** `src/jft/validation/`, locked config, manifest, evidence bundle.

**Tests/metrics/artifacts:** Net expectancy/return, drawdown/tails, risk-adjusted metrics with assumptions, turnover/payoff/calibration, concentration/probability of loss, parameter/capacity/break-even surfaces; signed OOS report and go/hold/stop record.

**Acceptance/exit:** Predeclared thresholds pass untouched data; no narrow domination; adverse cases fit risk; independent rerun matches.

**Blockers/kill/return:** OOS failure, concentration, fragility, or trivial capacity returns to a named phase; the opened holdout is never called untouched again.

**Must not build:** Live trading or retroactive thresholds.

### Phase 7 - Live shadow system

**Objective/rationale:** Validate live data/state/predictions/hypothetical orders without capital.

**Prerequisites:** Phase 6 candidate, live-data rights, security/operations approval.

**Work:** Live adapter; online/offline parity; prediction/hypothetical-order audit; time sync; gap/backfill; shadow simulator; monitoring; outcome joins; captured-session replay; incidents/rollback.

**Expected modules:** `src/jft/live/{feed,state,inference,shadow,observability}/`, deployments/runbooks.

**Tests/metrics/artifacts:** Parity vectors; restart/recovery; gap/duplicate/out-of-order/stale injection; latency/uptime/state divergence/drift/shadow expectancy/simulator divergence; deliver shadow dataset, parity/latency/live-vs-offline reports, incidents.

**Acceptance/exit:** Predeclared duration/sample across regimes; replay parity; no unexplained material decay; latency within signal life; safe recovery.

**Blockers/kill/return:** Rights, parity, latency, reliability, or shadow-alpha failure returns to Phase 1-6.

**Must not build:** Order placement or capital deployment.

### Phase 8 - Broker abstraction and hard risk engine

**Objective/rationale:** Create broker-independent execution and fail-closed risk, initially against simulators only.

**Prerequisites:** Phase 7, current broker/API/regulatory review, threat model.

**Work:** Broker contract for quotes/status/order/modify/cancel/positions/funds; Zerodha only as a possible first adapter; idempotent IDs; state reconciliation; rates/retries; approved auth/static-IP/secrets; manual/global/strategy/instrument kills; conformance harness.

Hard checks: stale/missing feed, clock skew, model/config version, quantity/notional/price collar, position/gross/net/concentration/participation, order rate/duplicate, daily loss/drawdown, rejects, unexpected position, disconnect, reconciliation mismatch, and EOD state. Default is reject/halt.

**Expected modules:** `src/jft/brokers/{base,simulated,zerodha}.py` if approved, `execution/`, `risk/`, conformance/runbooks.

**Tests/metrics/artifacts:** Contract/idempotency, timeout, partial/cancel race, restart, broker drift, kill latency, no unbounded path, reconciliation; deliver contract, adapter, policy, threat model, evidence, go-live/rollback checklist.

**Acceptance/exit:** Every order path passes hard risk; failures are survived; positions/cash/orders reconcile; manual kill meets budget; owner/compliance approves only a bounded live test.

**Blockers/kill/return:** Regulatory/API/auth/reconciliation issue or risk bypass blocks live use.

**Must not build:** Multiple production brokers, user connections, RL execution, unattended trading.

### Phase 9 - Small-capital production validation

**Objective/rationale:** Measure real fills/costs/latency/reliability with capital small enough to lose without material harm.

**Prerequisites:** Written live/profit-use permission, Phase 8, broker/compliance approval, ring-fenced capital, explicit loss authority.

**Work:** One broker/account/strategy; narrow instruments/hours; tiny supervised size; gradual order-count ramp; real-time risk; daily reconciliation; fill attribution; shadow twin; incidents; tax/accounting records.

**Expected artifacts:** Secret-free production config references, signed limits, deployment/rollback runbook, reconciliation/live reports.

**Tests/metrics:** Realized net expectancy, shortfall, fill/cancel/reject, live-vs-shadow divergence, latency, errors, drawdown/tails, capital use, zero risk-limit breach.

**Acceptance/exit:** Predeclared sample/duration over multiple regimes; positive net result consistent with uncertainty; understood simulator errors; no material control failure.

**Blockers/kill/return:** Loss/risk/legal/position/negative-edge/divergence/operations trigger kills first and diagnosis later.

**Must not build:** Large capital, many strategies, unattended scale, customer features.

### Phase 10 - Capacity, capital scaling, and trading farm

**Objective/rationale:** Scale only where measured capacity/diversification support it; return is not linear in capital.

**Prerequisites:** Phase 9 across sufficient duration/regimes and capital approval.

**Work:** Participation/impact experiments; reversible capital steps; portfolio allocation among independently validated strategies; crowding/correlation limits; resilience where justified; DR/on-call/finance/compliance; cloud-vs-colocation from measured alpha half-life.

**Expected modules/artifacts:** `src/jft/portfolio/`, capacity reports, allocation policy, SLO/DR/runbooks, strategy registry, trading-farm architecture, capital memo per step.

**Tests/metrics:** Marginal return per capital increment, impact/capacity, correlations, portfolio drawdown/tails, RTO/RPO/failover, uptime, cost-to-alpha, operating load.

**Acceptance/exit:** Each increment meets live gates without impact degradation; aggregate risk and operations remain bounded.

**Blockers/kill/return:** Falling marginal return, crowding/correlation, insufficient capacity, fragility, or infra cost scales down to the last validated level.

**Must not build:** Colocation for prestige, unvalidated strategies, external platform.

### Phase 11 - Optional product/platform

**Objective/rationale:** Evaluate a separate multi-user research/deployment product only after proprietary validation. Consumer safety, data redistribution, broker integration, and regulation are a different program.

**Prerequisites:** Phase 10; separate PRD/business case; explicit data/broker/regulatory permission; privacy/security/accessibility review.

**Potential scope:** Quant lab, versioned strategy registry, backtesting, walk-forward verification, paper accounts, evidence-based competitions, performance profiles, optional broker link. Gamify robustness, duration, calibration, and drawdown discipline - never frequency, leverage, streaks, or raw P&L.

**Expected artifacts:** Separate architecture/repository decision, PRD, approvals, threat model, design, operating model. The alpha/execution core never depends on UI/gamification.

**Tests/metrics:** Consumer harm, truthful performance, suitability/disclosure, privacy/security/accessibility, tenancy, entitlements, broker/regulatory conformance, fraud/abuse.

**Acceptance/exit:** Separately approved safe/legal case that cannot compromise the core.

**Blockers/kill:** Prohibited simulation/platform/derived outputs, unmet retail-algo duties, harmful incentives, or failed economics.

**Must not build:** Any product feature under this roadmap before the separate gate.

## 12. Budget and compute gates

These **DESIGN DECISION** envelopes come from prior discussion, not current quotes or commitments. Validate data tariffs, tax, legal, cloud, broker, and regulatory costs before approval.

| Gate | Planning envelope | Rule |
|---|---:|---|
| Phase 0-2 POC | Rs 1.5-5 lakh external; no trading capital | Buy only enough evidence to test signal |
| Through Phase 3/6 | Rs 3-8 lakh cumulative | Raise after reconstruction and baseline proof |
| Phase 7 shadow | Rs 4-10 lakh cumulative; paper only | Shadow is not live proof |
| Phase 9 initial live | Rs 6-15 lakh cumulative external plus Rs 1-5 lakh first experiment; Rs 5-20 lakh later needs a new gate | Capital is an experimental loss budget |
| Phase 10 farm | Indicative Rs 15-50 lakh/year infra/data and Rs 50 lakh-2 crore capital | Capacity justifies each increment |
| Colocation/institutional | Separate case, potentially crores | Only measured alpha half-life can require it |

Compute order: CPU baseline -> small GPU -> compact optimized neural model -> larger experiments only with demonstrated scaling. Spend first on lawful data and simulator fidelity, not permanent GPUs.

## 13. Dependencies, blockers, and kill criteria

Dependencies: written data rights; paired Orders/Trades and reference/corporate-action data; secure compute/storage; current fee/broker/regulatory evidence; secure credentials/network; time diversity; owner approvals; later legal/compliance/accounting support.

Global blockers: legal ambiguity, unaffordable/inadequate data, non-deterministic reconstruction, signal decay faster than latency, edge below costs, insufficient capacity, unstable models, risk-control failure, or prohibitive regulation/operations.

Stop the project or alpha path if, after a predeclared remediation attempt: intended use is not permitted; no viable dataset reconstructs; ambiguity changes features/labels materially; honest baselines lack OOS signal; signal fails costs; complexity adds no net value; locked OOS/shadow/live fails; loss/divergence exceeds limits; capacity cannot cover cost/risk; or safe operation exceeds the approved business case. Stopping is a successful result when it prevents unsafe capital deployment.

## 14. Immediate next actions

1. Record the evidence behind any request to clear the licensing red flag; absent evidence, keep it open.
2. Submit the exact Annexure-II intended use/questions to NDAL and counsel.
3. Acquire representative paired CM Trim files/triggers; add matched Full only if permitted.
4. Produce the Phase 0 source/rights/data qualification report without committing restricted data.
5. Resolve filenames/streams/ties/modify-cancel quantities/trade reductions with samples/vendor.
6. Approve or stop Phase 1. Do not begin features before its gate.
