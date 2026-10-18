# JFT NSE Short-Horizon Trading System

## Product requirements and gated technical phase plan

This is the single execution source of truth for JFT. It combines the product brief, technical architecture, data contract, research plan, acceptance gates, kill criteria, and phase checklist for an NSE-focused short-horizon quantitative trading system. It replaces the generic lifecycle checklist; it does not authorize trading or establish that the strategy is profitable or legally permitted.

- **Document status:** Governing plan; Phase 1 current-Trim parser slice verified, replay semantics not yet frozen
- **Current phase:** Phase 1 - deterministic parser and book reconstruction
- **Last updated:** 2026-10-15
- **Decision owner:** Project owner
- **Initial market/mode:** NSE Cash Market (CM), offline research only
- **Next gate:** Deterministic fixture replay with byte-accounting and invariant evidence
- **External dependency (not repository scope):** Legal/compliance review still needs to be completed before operational live, capital, or external-product activation; it does not block offline repository engineering
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

Repository engineering success requires byte-correct parsing, deterministic reconstruction, leakage-free chronological validation, predictive value beyond simple baselines, positive net expectancy under conservative costs, stability across instruments/days/regimes, shadow-to-offline parity, bounded small-capital validation, and measured capacity. Operational activation separately requires the owner's external compliance and authorization process.

These are not success: shuffled or in-sample accuracy; gross P&L before costs; one profitable period; a neural model without economic uplift; impossible fills; irreproducible results; or an edge shorter than attainable latency.

### 3.3 Non-goals before validated alpha

Do not build exchange colocation, FPGA/microsecond infrastructure, a full RL trading engine, LLMs in the fast path, mobile/social/copy trading, real-money gamification, multiple brokers, a marketplace, external accounts, global-market support, or a public platform. Do not optimize sub-millisecond latency until measured alpha decay and economics require it.

## 4. Phase status

Legend: `[x]` verified complete, `[ ]` remaining, `[-]` intentionally skipped with a reason.

| Phase | Status | Gate outcome |
|---|---|---|
| 0 - Governance, evidence, data qualification; external compliance marker | **In progress as a non-blocking parallel technical track** | Representative current data, delivery controls, and budget are qualified |
| 1 - Deterministic parser and book reconstruction | **Active - current Trim layout/parser/quarantine slice verified** | Canonical replay is identical and passes invariants; representative current data is required for final acceptance |
| 2 - Features, labels, baselines | Blocked by Phase 1 | Economically meaningful OOS signal beats naive baselines |
| 3 - Execution-grade simulator | Blocked by Phase 2 | Signal survives conservative cost/fill/latency assumptions |
| 4 - Neural representation learning | Blocked by Phase 3 | Neural model adds stable OOS net value |
| 5 - Regime-aware ensemble | Blocked by Phase 4 | Specialization adds robust value without unstable gating |
| 6 - Walk-forward/final OOS validation | Blocked by Phase 5 | Frozen candidate survives untouched data and stress tests |
| 7 - Live shadow system | Blocked by Phase 6; operational activation also needs external compliance | Live paper outcomes match offline tolerances |
| 8 - Broker abstraction/hard risk | Blocked by Phase 7 | Simulated execution fails closed and reconciles |
| 9 - Small-capital validation | Blocked by Phase 8; capital activation also needs external authorization | Real fills validate net edge within loss limits |
| 10 - Capacity/scaling/trading farm | Blocked by Phase 9 | Measured capacity supports staged scale |
| 11 - Optional product/platform | Blocked by Phase 10 | Separate product, compliance, and safety case passes |

## 5. Source and evidence register

### 5.1 Supplied governing material

| Source | Classification | Governing use | Limitation/action |
|---|---|---|---|
| `NSE_Hist_Order_Trade_Data_1.19_20260902144045.pdf`, v1.19, 03 Aug 2026 | **CONFIRMED FROM NSE DOCUMENTATION** | CM layouts, jiffies, availability, checksums, delivery, record history | Archive through the read-only source sync; verify against the subscribed product |
| `NDAL-v.2024-Historical Order and Trade Data Agreement.docx` | **CONFIRMED FROM NSE DOCUMENTATION** | Use, confidentiality, redistribution, audit, disclaimer, indemnity | Not legal advice; apparent use conflict needs written NDAL/counsel resolution |
| `NDAL-v.2024-Historical Trade Undertaking.doc` | **CONFIRMED FROM NSE DOCUMENTATION** | Corroborates restrictive historical-data terms | Older/general; Order and Trade agreement is more directly relevant |
| `Data_details_CM_1.pdf` plus the CM EOD sample bundle | **CONFIRMED FROM NSE DOCUMENTATION** | Legacy CM bhavcopy/index/master/snapshot/trade schema and potential reference/reconciliation inputs | A different product from current Historical Orders & Trades; cannot qualify deterministic event replay |
| Other supplied EOD/segment documents and samples | **UNRESOLVED / REQUIRES VALIDATION** | Possible reference and later-segment context | Catalogue individually; defer non-CM products until their approved phase |
| Referenced `Compare Trading Algorithms` conversation | **DESIGN DECISION** | Thesis, architecture, model order, budgets, gates | Rationale, not evidence of law, market behavior, prices, or profitability |

Do not commit the supplied attachments or raw market-data samples. Keep the working source bundle outside Git; `sources/` remains read-only and synchronization-owned.

#### Phase 0 source manifest

Hashes identify the exact 17-file working bundle reviewed on 2026-10-15. They do not grant a right to store or use the content. The bundle remains outside Git and the project source mirror currently contains no archived files, so controlled archival remains open.

| Supplied filename | Size / pages | SHA-256 | Phase 0 disposition |
|---|---:|---|---|
| `NSE_Hist_Order_Trade_Data_1.19_20260902144045.pdf` | 1,813,532 bytes / 85 pages | `8521521dbe048338ac1a2f9ba3593972c7c85d09aa8346b4165c431d3f8ea16b` | Governing technical specification |
| `NDAL-v.2024-Historical Order and Trade Data Agreement.docx` | 31,195 bytes | `b8605507bdef3a753ba6644669024612a5d5f080e62c8829a19e25a7bfb0dd28` | Governing legal template; executed Annexure II absent |
| `NDAL-v.2024-Historical Trade Undertaking.doc` | 60,928 bytes | `0d406b1f198f897785df773ce3ec5114ef9dcd791cd2b13f1302480bd2b1fbf4` | Secondary legal comparison |
| `EOD_CM_20251120105001.pdf` | 684,829 bytes / 23 pages | `ce8ac4ce858884392ccb68f4d74ac1549b20695da53824ed82cb7e03cac251b8` | Relevant for CM corporate/reference/EOD reconciliation candidates |
| `Data_details_CM_1.pdf` | 136,529 bytes / 6 pages | `e8ac6bb76e428c1345827bc616faa4c0ff3043fb90e603338cf2599dcb9c2ff9` | Legacy CM historical-package specification; not the Order & Trade layout |
| `Download Sample Files for CM (.zip)_0.zip` | 22,783,237 bytes | `306e5602a02795c3e8b00502f1c966a155aed962985e61360668de5cca6f43c1` | Legacy CM EOD/historical-product sample; qualified separately below |
| `Sample_CMsegment.zip` | 9,201,101 bytes | `0469792bafa2320814edc526cfb2af50e7144d80c294a535666376b0d647bdb4` | Official legacy Full Orders/Trades public sample; qualified below |
| `Download_End_of_the_Day_Data_Tariff_20260424120810.pdf` | 65,590 bytes / 1 page | `c73448b94520e3c1e228b1199eebb5b559a0734189db00f348efc357e7a6ec4e` | EOD display tariff only; not an Order & Trade historical-data quote |
| `EOD_FAO_20251120105405.pdf` | 642,333 bytes / 20 pages | `7d2026c309c0ff721da207635e4807502eacb111d11f12ff1df369a2476bfc2f` | Deferred; derivatives are out of initial scope |
| `EOD_CD_20251120105503.pdf` | 619,309 bytes / 18 pages | `5c09190d08c76e81569cc45f9a75d85758df46fc5289d90e09b2dc420c37b44c` | Deferred; currency derivatives are out of scope |
| `EOD_WDM_20251120105551.pdf` | 596,824 bytes / 16 pages | `833ed1b00a70aee0700e00a19fd7649dbaf1d3f2f852f89064c0b488ddce6dfa` | Deferred; wholesale debt is out of scope |
| `Data_Details_F_n_O.pdf` | 158,829 bytes / 7 pages | `69a2c3c1d0104bb177cd61c8d0b564ab9f35aea26194ed4a8a0ccc7e20b3e013` | Deferred until an approved derivatives phase |
| `Data_Details_WDM.pdf` | 199,630 bytes / 3 pages | `6fbe1751e46243eb1ec31053cd72effb79639e2e5c308bebe23a254e544c7497` | Deferred; wholesale debt is out of scope |
| `Sample_FOsegment.zip` | 9,267,715 bytes | `9d3eb704a03935b8a3c831c62a161a6070228dd17cce4f7320f400ab63062f85` | Deferred parser evidence; derivatives are out of initial scope |
| `Sample_CDsegment.zip` | 3,046,002 bytes | `88bf9e3c68c6b792b44e403ccfb12d0fd80cc5920fdeb97e1c59a3bf41c8d8f7` | Deferred parser evidence; currency derivatives are out of scope |
| `Sample_WDM.zip` | 219,228 bytes | `a56447a3ae268eca0c932bbf574c55302031a8b9f5c073af981831820b154919` | Deferred parser evidence; wholesale debt is out of scope |
| `Download Sample Files for F&O.zip` | 50,244,150 bytes | `c883387f2b4d148a278d5e8b949d3006647ee0c2b278ebfa0cdd7d8a0b6f59bb` | Deferred EOD/reference sample; derivatives are out of initial scope |

**UNRESOLVED / REQUIRES VALIDATION:** Actual subscribed current CM Trim Orders, Trades, trigger files, an executed agreement/Annexure II, and a Historical Order & Trade tariff/quote are absent. The supplied CM historical package and public Order/Trade sample do not close those gaps.

#### Current official acquisition channel

**CONFIRMED FROM NSE DOCUMENTATION:** On 2026-10-15, the [official NSE EOD/Historical subscription page](https://www.nseindia.com/static/market-data/eod-historical-data-subscription) was updated 02 Sep 2026 and stated that EOD Order & Trade data is delivered through SFTP, Historical Order & Trade data through the online platform, and CM coverage includes order/trade information plus index value. It links a public CM sample and directs subscription questions to `marketdata@nse.co.in` / `+91-22-2659 8385`. The [NSE Data & Analytics vending page](https://www.nseindia.com/static/nse-data-and-analytics/data-information-vending) lists pricing effective 01 Apr 2026, but a current product-specific written quote and permitted-use terms are still required.

The supplied EOD tariff lists domestic CM EOD data at Rs 1,00,000 per display medium and requires consent for client usage. It must not be used as the budget for Historical Order & Trade data.

#### Official CM Order/Trade public-sample qualification

The supplied `Sample_CMsegment.zip` is byte-identical to the official public sample previously downloaded to temporary storage. It was inspected without committing its contents. Archive SHA-256: `0469792bafa2320814edc526cfb2af50e7144d80c294a535666376b0d647bdb4` (9,201,101 bytes).

| File in official sample | Records | Width before LF | Finding |
|---|---:|---:|---|
| `CASH_Index_DDMMYYYY.txt` | 22,500 | 38 | Index stream with placeholder date filename |
| `CASH_Orders_DDMMYYYY.txt` | 300,000 | 87 | Legacy Full CM order width, not current Full 91 or Trim 68 |
| `CASH_Trades_DDMMYYYY.txt` | 300,000 | 100 | Legacy Full CM trade width, not current Full 103 or Trim 82 |

Observed sample diagnostics:

- order events: 213,707 entries, 68,121 modifications, 18,172 cancellations; 138,838 buys and 161,162 sells;
- order sessions: 157,484 `PO` and 142,516 `RM`; trade sessions: 15,122 `PO` and 284,878 `RM`;
- jiffies were non-decreasing within each Orders and Trades file and no adjacent equal-jiffy records appeared;
- all 300,000 trade numbers were unique, while 213,527 distinct order numbers appeared;
- only 44,520 buy-order references (14.84%) and 45,129 sell-order references (15.04%) matched an order number present in the sample Orders file;
- every observed byte in the documented legacy Order/Trade numeric positions was an ASCII digit, with no sign byte observed; this is evidence about this public placeholder sample only and does not resolve v1.19's signed-integral ambiguity for current deliveries;
- there are no `.trg` files, split streams, Trim records, or closing-auction (`CA`) examples; and
- the data uses literal `b` characters in left-padded symbols, so production padding behavior must be checked against real subscribed bytes rather than inferred from this sample.

**Phase 0 conclusion:** The public archive is suitable for legacy fixed-width parser fixtures and negative/incomplete-replay tests. It is not representative paired data and cannot satisfy the deterministic-reconstruction, Trim qualification, trigger-integrity, current-layout, or Trim-versus-Full gates.

#### Legacy CM historical/EOD sample-product qualification

**CONFIRMED FROM NSE DOCUMENTATION:** `Data_details_CM_1.pdf` defines a separate six-directory historical package: Bhav copy, Index, Masters, Snapshots, Trades, and Circulars. Its snapshots are point-in-time book states and its Trades data carries second-resolution timestamps; neither is the jiffy-timestamped lifecycle event stream defined by v1.19.

The supplied `Download Sample Files for CM (.zip)_0.zip` contains seven nested sample archives matching that product family. Direct inspection found:

| Sample component | Observed sample | Phase 0 use |
|---|---:|---|
| Index | 500,000 pipe-delimited rows; 3 fields | Index/context fixture only; placeholder data is not a representative trading day |
| Snapshot | 500,000 rows; 14 pipe-delimited positions including trailing empty GTD date | Snapshot-schema and coarse book sanity fixture; not event replay |
| Trades | 500,000 rows; 7 fields; second-resolution clock | Trade-schema fixture and possible aggregate reconciliation; no order references/jiffies |
| Bhavcopy | 168,875 rows | Daily aggregate/reference reconciliation fixture |
| PDBhavcopy | 22 dated January 2024 CSV files | Daily reference fixture |
| Masters | Nested ZIP mislabeled `.gz` | Symbol/series/ISIN reference candidate; ingestion must detect content by magic bytes, not extension |
| Circulars | Nested ZIP mislabeled `.gz` | Market-rule research source; not a parser input |

The Index, Snapshot, and Trades payloads also use `.gz` names while containing plain ASCII rather than gzip streams. This reinforces the requirement to identify payloads from magic bytes/content and to quarantine extension/content mismatches instead of assuming decompression rules from names.

**DESIGN DECISION:** Rust now classifies payload evidence as empty, gzip, ZIP, plain ASCII text, or unknown binary from content alone. An opt-in test reproduces all five observed `.gz` suffix/content mismatches in the supplied package. This is signature detection only; decompression and routing remain separate gates.

**DESIGN DECISION:** Keep this package in a separate `legacy_cm_eod` data contract if it is later ingested. Never route it through the v1.19 Order/Trade parser or use its snapshots to claim exact queue history.

**UNRESOLVED / REQUIRES VALIDATION:** The index, snapshot, and trade samples use placeholder filenames and appear synthetic or excerpted. Their dates, completeness, provenance, and suitability for quantitative validation are not established. They cannot satisfy the representative-day acquisition gate.

#### Data-rights clause and clearance matrix

**Repository treatment: OUT OF SCOPE / EXTERNAL FOLLOW-UP REQUIRED.** Legal compliance is not a repository workstream or engineering phase gate. It still needs to be completed by the project owner before operational live-data use, capital deployment, redistribution, or external-product release. No legal clearance is claimed here, and the absence of clearance does not block local, offline repository engineering in the owner-controlled environment.

| Agreement topic | Supplied wording/effect | Project treatment | Evidence needed to close |
|---|---|---|---|
| Internal use | Use is limited to internal purposes described in Annexure II | Offline work must remain within the executed use description | Executed agreement and Annexure II |
| Raw-data redistribution | Publication/reproduction/redistribution requires prior written consent | No raw or reconstructive data in Git, logs, reports, or third-party services | NDAL confirmation of allowed users, systems, sites, and processors |
| Generated output | Output redistribution is restricted except as Annexure II permits | Treat features, datasets, weights, metrics, and reports as potentially restricted | Written classification of each derived artifact |
| Commercial/financial gain | Data must not be exploited commercially or for financial gain/profit | Hard hold on profit-seeking/live use | Written NDAL interpretation plus legal review |
| Trading/investment decisions | A later clause contemplates lawful activities facilitating trading/investment decisions | This conflicts with the financial-gain restriction; do not resolve by inference | Written reconciliation from NDAL |
| Platform/simulation/gaming | Platform/app/website, virtual trading, simulation, and gaming language is restrictive/ambiguous | Internal simulator and any future product require express treatment | Written permission/scope for backtest, simulator, paper trading, and product |
| Confidentiality/access | Access is need-to-know and employees require confidentiality commitments | Least privilege, access register, encryption, no public artifacts | Approved user/site/cloud list and confidentiality process |
| Audit/penalty | NDAL may inspect records/logs/systems and impose penalties/terminate for lapses | Retain auditable access/processing/deletion logs without leaking data | Audit-scope, retention, and evidence requirements |
| Accuracy/sequence | NDAL disclaims sequence, accuracy, completeness, and error-free delivery | Parser/replay must detect and evidence defects; vendor data is not presumed correct | Sample qualification and defect-escalation procedure |
| Cloud/retention/termination | Not resolved by the supplied template | Do not assume AWS, backups, DR copies, or post-termination retention | Written cloud, region, subprocessors, backup, retention, and deletion terms |

#### External NDAL clarification checklist (reference only)

This checklist is retained so the external dependency is not forgotten; completing it is outside repository scope and is not a Phase 0/1 engineering gate. If the owner pursues operational use, send it to `marketdata@nse.co.in` with subject `Clarification of permitted use - CM Historical Order and Trade Data`, attach the exact proposed Annexure-II wording, and request a written answer to each question:

1. May the subscriber use CM Historical Orders/Trades internally to backtest and train statistical/ML models?
2. May derived features, labels, embeddings, model weights, calibration parameters, and non-reconstructive aggregate metrics be retained and used after training?
3. May those models inform or execute proprietary trades for the subscriber's own account and generate financial gain? How should this be reconciled with the no-commercial-use/no-financial-gain clause?
4. Are internal deterministic replay, execution simulation, paper trading, and live shadow evaluation permitted? Does the platform/app/simulation clause restrict any of them?
5. May processing occur in a subscriber-controlled AWS Mumbai account? Which employees, contractors, managed services, regions, sites, backups, and disaster-recovery copies are allowed?
6. What data/output may be shown to advisers, auditors, counsel, investors, regulators, or in aggregated research without separate consent?
7. What retention, deletion, backup purge, audit-log, confidentiality-undertaking, and post-termination obligations apply?
8. Are CM Trim Orders/Trades licensed on the same use terms as Full? What are current tariffs, minimum period/site/user terms, and cloud/SFTP charges?
9. Can NDAL provide representative current Trim 68/82 and Full 91/103 paired sample days with trigger files and written answers on same-jiffy ordering, modify/cancel quantities, and trade-driven quantity reduction?
10. Would an external multi-user backtest/paper/live platform require a separate redistribution/non-display/product license, and is it prohibited under the supplied agreement?
11. Is the exchange-jiffy epoch actually `1980-01-01 00:00:00 UTC`, or local IST midnight? Please reconcile the v1.19 prose with its published `315513000`-second Unix-offset constant, which is 19,800 seconds smaller than the actual UTC offset of `315532800` seconds.
12. Section 5.4 says integral fields are signed unless explicitly stated otherwise, but CM order/trade identifiers and jiffies have no displayed range. Can any CM identifier or jiffy contain a leading minus sign, or are these fields always unsigned decimal digits in delivered files?
13. For Full CM Orders, Trigger Price's valid-range column says non-zero while its comment says the value is zero for non-stop-loss orders. Please confirm the exact conditional rules for Trigger Price, Limit Price, Market Order Flag, and Stop Loss Flag, including how market and stop-loss market orders are encoded.

Before operational live-data use, capital deployment, redistribution, or external-product release, the owner should retain the dated NDAL response, executed agreement/Annexure II, permitted/prohibited-use matrix, conditions, legal reviewer, approval, and next review trigger outside or alongside the appropriate operational governance process. The repository does not determine or certify legal compliance.

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
- **CONFIRMED FROM NSE DOCUMENTATION:** CM Trim was introduced on 18 May 2026 and is EOD: 68-byte Orders and 82-byte Trades, LF-delimited.
- **CONFIRMED FROM NSE DOCUMENTATION:** Full is the complete published representation. Full Orders lag 90 calendar days; Full Trades lag 30 days.
- **CONFIRMED FROM NSE DOCUMENTATION:** Current Full CM is 91-byte Orders/103-byte Trades. Historical parsing must handle 87/91-byte Orders and 100/101/103-byte Trades by date; 28 Mar 2025 is a documented 91/103 one-day exception, and 91/103 applies from 16 Jun 2025.
- **CONFIRMED FROM NSE DOCUMENTATION:** CM files were split into streams from 01 Jul 2025; stream count may grow. A symbol may span streams when series differ.
- **CONFIRMED FROM NSE DOCUMENTATION:** Per-file `.trg` files carry MD5 and compressed size. Layout notes show `CM_...`; trigger tables show `CASH_...`.
- **CONFIRMED FROM NSE DOCUMENTATION:** Trim Orders and Trades are made available daily for the current date; Full Trades follow after 30 calendar days and Full Orders after 90 calendar days. The specification describes EOD SFTP, the NDAL downloader, Historical Data through AWS, and EOD Data through AWS as four delivery modes.
- **CONFIRMED FROM NSE DOCUMENTATION:** SFTP retains the previous three days and gives provisional CM availability at 21:00 IST. Historical delivery is tentatively 01:00-03:00 IST. EOD cloud delivery begins after 23:00 IST and transfers only completed files.
- **CONFIRMED FROM NSE DOCUMENTATION:** The documented cloud path copies compressed, unencrypted files from NDAL into a client-owned S3 bucket in `ap-south-1`; the example bucket policy grants NDAL's role broad object read/write/tag/ACL/list/multipart permissions. Same-region transfer is described as free, but that statement does not cover the project's storage, request, backup, security, or later-region-transfer costs.
- **DESIGN DECISION:** If AWS delivery is approved, use a dedicated Mumbai-region ingestion bucket with no unrelated/proprietary objects, block public access, enable encryption/versioning/access logging, constrain retention, validate the final vendor principal/policy, and copy validated immutable objects into a separately controlled research zone. Do not paste the example policy onto a shared bucket.
- **UNRESOLVED / REQUIRES VALIDATION:** Delivered prefixes, stream discovery, missing-stream representation, and trigger delimiters require real samples.
- **UNRESOLVED / REQUIRES VALIDATION:** NDAL must confirm whether its production transfer can use a narrower write-only prefix policy, SSE-KMS, object ownership without ACLs, and an NDAL-controlled manifest/checksum stronger than MD5.
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

**CONFIRMED FROM NSE DOCUMENTATION:** Full Order Volume Disclosed is non-negative and explicitly zero for non-disclosed-quantity orders; Volume Original and Limit Price are labelled non-zero/non-negative. Full Trade Price and Quantity are labelled non-zero/non-negative. Market, Stop Loss, and IOC flags are `Y`/`N`; algo indicators are `0`-`3`; client identity flags are `1`-`3`.

**CONFIRMED SOURCE CONTRADICTION:** Full Order Trigger Price is labelled non-zero/non-negative in the valid-range column, but the same field's comment says it is `0` for non-stop-loss orders. Section 5.4 also says integral fields are signed unless a field says otherwise, while order/trade identifiers and jiffies show no explicit range even though their delivered encoding is described as numeric.

**DESIGN DECISION:** Do not implement Full semantic parsing or conditional zero-value rejection from the contradictory table alone. Preserve the exact wire bytes, obtain representative files/vendor clarification, and separate structural numeric decoding from conditional semantic validation. The existing fixture-first Trim parser's unsigned ID/jiffy handling remains provisional evidence, not an accepted all-era claim.

**UNRESOLVED / REQUIRES VALIDATION:** Whether a minus sign can ever appear in CM IDs/jiffies; Trigger/Limit Price behavior for regular, market, stop-loss limit, and stop-loss market orders; and whether violations should be quarantined or classified as contextual anomalies.

### 6.5 Jiffy policy

- **CONFIRMED FROM NSE DOCUMENTATION:** 65,536 jiffies equal one second. The v1.19 prose says the counter starts at 01 Jan 1980 00:00:00 GMT; its worked conversion instead uses a `315513000`-second Unix offset and then adds 05:30 for IST display. It warns Excel is millisecond-limited.
- **CONFIRMED FROM INDEPENDENT ARITHMETIC:** The actual Unix-to-1980-01-01-UTC offset is `315532800` seconds. NSE's published constant is 19,800 seconds smaller, exactly 05:30. Therefore the prose, constant, and timezone step do not jointly define one unambiguous UTC conversion.
- **DESIGN DECISION:** The raw 14-byte jiffy token and source record are authoritative. The current Trim fixture parser additionally exposes an exact `u64` only after all 14 bytes pass ASCII-digit validation; that unsigned interpretation is provisional until the section 5.4 ambiguity is resolved against delivered data.
- **DESIGN DECISION:** Decompose with integer/rational arithmetic and retain the fractional remainder; never round-trip through float.
- **DESIGN DECISION:** Do not emit or label a UTC timestamp, session date, or timezone-derived display until the epoch ambiguity is resolved. The raw jiffy, integer whole-second quotient, and remainder remain canonical and lossless.
- **DESIGN DECISION:** Same-jiffy events retain file/stream/record provenance. A reproducible tie-breaker is not claimed as exchange causality.
- **UNRESOLVED / REQUIRES VALIDATION:** Resolve the epoch using a written NSE/vendor answer and representative files whose trading date/time can be independently checked. Also validate cross-stream and Orders-vs-Trades tie ordering, per-stream monotonicity, and session edge cases.

```text
whole_exchange_seconds = exchange_jiffy // 65536
subsecond_jiffies = exchange_jiffy % 65536
fraction = subsecond_jiffies / 65536 second
UTC conversion = intentionally unavailable until epoch validation
```

### 6.6 Canonical schemas

Every table includes schema/spec version, source hashes/logical path/file part/stream/record index, raw record or immutable pointer, jiffy, session date, indicators, segment, raw/normalized symbol, series, parse status, and deterministic event key.

**UNRESOLVED / REQUIRES OWNER DECISION:** [DEC-2026-009](decisions/DECISION_LOG.md#dec-2026-009) proposes the manifest/canonical identity v1, compatibility policy, unified canonical Trim/Full shape, and minimal hashing/JSON dependencies. It is intentionally not implemented while its status is Proposed.

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
immutable source files + triggers -> integrity/manifest -> parser/quarantine
-> canonical events -> deterministic replay -> book deltas/snapshots
-> point-in-time features + separate labels -> chronological datasets
-> experiments/model registry -> locked evaluation/promotion evidence
```

Use encrypted immutable object storage, versioned columnar datasets, lineage/catalog metadata, ephemeral CPU workers, GPU only after baseline evidence, and a registry binding model/data/code/environment/metrics/calibration/approval. Raw NSE data must not enter logs, public artifacts, or unapproved services.

### 8.2 Live plane

```text
authorized live feed -> normalized adapter -> stateful features -> compact inference
-> deterministic policy -> hard risk -> execution -> broker adapter -> broker
all stages -> append-only audit/outcome store -> reconciliation/monitoring
```

Training and live inference are separate failure domains. No LLM or automatic online retraining is allowed in the fast path. Start with a small observable service near the broker, warm standby, monotonic internal sequencing, durable audit, time sync, health checks, and fail-closed behavior.

### 8.3 Promotion and drift

Model states: `experimental -> reproducible -> OOS-qualified -> shadow candidate -> shadow-qualified -> production candidate -> approved -> active -> retired`. Human approval and an immutable evidence bundle are mandatory; a better offline score never auto-promotes.

Monitor schema/data quality, feature/prediction/calibration/outcome/regime drift, net expectancy/slippage/fills/drawdown, simulator divergence, latency/gaps/duplicates/clock offset, and broker reconciliation. Retraining creates a challenger that repeats offline, walk-forward, and shadow gates. Rollback is an approved-model switch.

### 8.4 Rust/Python boundary

- **DESIGN DECISION:** Rust owns the deterministic data plane from compressed/raw NSE bytes through integrity checks, version-aware fixed-width parsing, exact raw-jiffy preservation/decomposition, validated timestamp conversion, canonical event construction, lifecycle reduction, order-book replay, reconciliation, quarantine, checkpoint hashes, and canonical columnar output.
- **DESIGN DECISION:** Python begins at the versioned canonical-data boundary. It owns research orchestration, diagnostics, feature and label generation, datasets, baselines, model training, evaluation, and visualization. Python must not independently parse raw NSE Order/Trade bytes or become the authoritative order-book implementation.
- **DESIGN DECISION:** The initial cross-language interface is file/process based: versioned Parquet or Arrow IPC datasets plus JSON manifests, schemas, anomaly reports, and hashes emitted by a Rust CLI. Do not introduce PyO3/FFI or an in-process Python extension until profiling demonstrates a concrete need.
- **DESIGN DECISION:** Rust and Python share language-neutral schema definitions, integer units, nullability, enum codes, provenance fields, and deterministic fixture expectations. Python reader tests must reject unknown schema versions and verify row counts, manifest hashes, jiffy bounds, and checkpoint identities.
- **UNRESOLVED / REQUIRES VALIDATION:** The Phase 3 simulator's performance-critical core may later warrant Rust, but that decision must not move research/model logic into Rust or duplicate the authoritative parser/replay logic in Python.

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

### Phase 0 - Governance, evidence, and data qualification

**Status:** In progress as a non-blocking parallel technical track. Legal/compliance is an external follow-up, while data qualification, delivery controls, and budget remain repository work.

**Objective/rationale:** Establish the factual, reproducible data and governance basis needed to begin bounded CM reconstruction engineering. The repository records legal/compliance risk but does not own, resolve, or certify it.

**Prerequisites:** Repository governance and supplied source material.

**Work packages:**

- [x] Preserve the read-only `sources/` boundary and linked governance records.
- [x] Establish this governing plan and evidence labels.
- [x] Hash and catalogue all 17 files in the supplied working bundle without copying restricted content into Git.
- [x] Verify the current official subscription/contact channel and inspect the official public CM Order/Trade sample.
- [x] Classify and qualify the separate legacy CM historical/EOD sample product; prevent it from being treated as current lifecycle-event data.
- [x] Produce the initial data-rights clause matrix and ready-to-send NDAL clarification request.
- [ ] Archive/version supplied documents through the approved source-sync process without committing restricted content; hashes are already recorded.
- [-] Complete agreement, Annexure II, permitted-use, audit, retention, deletion, and redistribution review; retained as an external owner dependency, not a repository work package.
- [-] Obtain legal/compliance interpretation for training, derived artifacts, simulation, proprietary trading, cloud processing, and optional product use; required before relevant operational activation, not offline engineering.
- [ ] Obtain a current Historical Order & Trade commercial quote covering Trim/Full scope, periods, delivery charges, minimum term, and expected storage/compute cost; the supplied EOD display tariff is not sufficient.
- [ ] Acquire representative paired current CM Trim Orders/Trades and `.trg` files across ordinary, volatile, special/low-volume, pre-open, regular, and closing-auction periods; this is required to finish Phase 0/accept Phase 1, but not to start fixture engineering.
- [ ] Acquire matched Trim/Full days when available and justified; quantify replay/field value before choosing Full.
- [ ] Inventory real subscribed dates, symbols/series, streams, sizes, counts, jiffy ranges, missing files, schema eras, corporate actions, and reference data.
- [ ] Qualify one production delivery design: SFTP SSH-key/static-IP or dedicated Mumbai S3 ingestion bucket, including least privilege, encryption, retention, deletion, integrity, and audit logging.
- [ ] Verify the research-paper citations needed for Phase 2; defer patent/FTO work until a product or filing decision makes it relevant.
- [ ] Approve the Phase 1 data/compute budget and research-spend stop-loss.
- [x] Authorize Phase 1 fixture-first engineering in the owner-controlled local environment; no live trading, capital deployment, redistribution, or external product activation is implied.

**Expected artifacts/files:** This plan and existing governance records; non-secret source manifest; sample qualification report. Supplied raw data stays out of Git.

**Tests/metrics:** Hash inventory; sample MD5/size verification; version matrix; representative-day/session coverage; file/stream/trigger completeness; all supplied files catalogued; sample limitations documented; zero supplied raw data committed to Git; delivery-control review and budget approval.

**Blockers:** None for starting local fixture-first Phase 1 engineering. Representative current paired data, delivery qualification, and budget remain blockers for completing Phase 0 and accepting Phase 1. Operational live/capital/external release has separate external dependencies.

**Acceptance/exit:** Source inventory/product separation are complete; representative current paired data and triggers are qualified; actual files/streams/schema eras are inventoried; a delivery design and budget are approved. Legal compliance is neither an exit criterion nor claimed complete.

**Return/kill:** Return to data qualification if real delivery differs from v1.19 or fixture assumptions fail. Operational activation remains an owner decision outside repository scope.

**Must not build:** Features, models, simulator, live feed, broker integration, product UI.

### Phase 1 - Deterministic parser and order-book reconstruction

**Objective/rationale:** Produce byte-auditable canonical events and deterministic book replay. Every later claim depends on correct event meaning, ordering, and lineage.

**Prerequisites:** The completed Phase 0 inventory is sufficient to start fixture-first engineering while the remaining technical Phase 0 work continues in parallel. Initial engineering may use synthetic non-licensed fixtures and the qualified public legacy sample. Representative paired current data/triggers are required for Phase 1 acceptance, not for implementation to begin.

```text
Orders + Trades -> integrity -> parser -> raw-event validation -> jiffy validation
-> event ordering -> lifecycle replay -> book reconstruction -> trade reconciliation
-> invariant/golden tests -> canonical dataset
```

**Work packages:**

- [x] Create the dependency-free Cargo workspace and tracked Rust/Python/schema/synthetic-fixture boundaries; verify format, compile, tests, and Clippy without freezing record semantics.
- [x] Define contiguous current CM Trim 68-byte Order and 82-byte Trade layouts; parse one synthetic golden record of each, preserve exact input bytes, and quarantine wrong width, invalid numerics, zero quantity, and unknown activity with field evidence.
- [x] Exhaustively mutate every numeric byte and test all documented session/side/activity codes, zero price/quantity rules, non-ASCII text, LF rejection, and literal-`b` preservation; all failures retain raw input and field evidence.
- [x] Sweep every possible byte value through every current Trim Order/Trade position (38,400 single-byte variants) and prove every success or quarantine retains the exact mutated input without panic.
- [x] Exercise 214 wrong-width Order/Trade payloads from empty through 32 bytes oversized and prove width rejection occurs before field attribution while retaining every rejected byte.
- [x] Frame exact-width LF-delimited records without copying valid payloads; attach zero-based record index and source byte offset; stop without resynchronization guesses and preserve the entire unconsumed tail on partial records or unexpected delimiters.
- [x] Preserve framing-error tails as borrowed slices into immutable source bytes rather than copying the remaining file; prove pointer identity for first-record and post-record failures.
- [x] Lazily yield one ordered, byte-addressable outcome per exactly framed record from caller-provided bytes; continue after field-level quarantine, stop after framing loss, and prove repeat runs return identical outcomes without retaining a whole-file result vector in the parser. Disk-backed streaming/mapping remains unselected pending real file-size and throughput evidence.
- [x] Encode and exactly test the current v1.19 Full 91-byte Order and 103-byte Trade field maps, including contiguity and the shared Trim/Full Order core prefix; Full parsing and historical 87/100/101 variants remain open.
- [x] Encode and exactly test the historical Full 87-byte Order plus 100-byte and 101-byte Trade maps from the documented trade-number and volume/quantity width changes; date dispatch and semantic parsing remain open.
- [x] Dispatch Full wire layouts from validated source dates across 01 July 2024, the explicit 28 March 2025 exception/rollback, and 16 June 2025; expose Trim only from its documented 18 May 2026 introduction. Never infer the era from record width alone.
- [x] Structurally profile date-dispatched Full 87/91-byte Order and 100/101/103-byte Trade sources with exact record/source-byte/residue accounting and no semantic field claims. An opt-in local test frames all 600,000 records in the supplied public legacy CM Order/Trade sample with zero residue; the placeholder filenames and absent triggers do not qualify it as a current delivery.
- [x] Profile exact observed byte counts for every documented Full field without decoding values; reject out-of-bounds layouts before framing and exclude incomplete records while preserving their residue. The opt-in public-sample test proves digit-only bytes in twelve legacy numeric fields and literal-`b` symbol padding, without generalizing either behavior to current data.
- [x] Strictly decode CM Full/Trim Order/Trade data and trigger filenames, optional two-digit positive streams, and DDMMYYYY dates while retaining `CM` versus `CASH` prefixes as distinct evidence; do not guess which documented prefix production uses.
- [x] Discover a homogeneous split stream set from observed members without a fixed maximum; sort it and reject empty input, gaps, duplicates, mixed split/unsplit forms, or mixed prefix/product/date/kind/data-trigger metadata.
- [x] Pair explicit data and trigger stream sets only when role, prefix, product, date, record kind, split mode, and observed members match exactly; report `CM`/`CASH` prefix conflicts rather than aliasing them.
- [x] Orchestrate one decoded logical delivery by partitioning data/triggers, validating both stream sets, and pairing them while preserving nested missing-role, gap, group, form, member, and prefix-conflict evidence.
- [x] Partition an arbitrary filename inventory into deterministic date/product/kind deliveries; retain every rejected filename and independent per-delivery outcome so one invalid group cannot hide valid groups. Prove reversed input order produces identical evidence.
- [x] Add non-licensed synthetic current Full Order/Trade fixtures and assert the exact bytes at every documented field boundary without claiming Full semantic validation.
- [x] Inventory all current synthetic fixture sizes and complete-file SHA-256 digests (including LF), with an explicit non-NSE/non-representative provenance warning.
- [x] Detect empty, gzip, ZIP, plain ASCII, and unknown binary payload signatures from content rather than suffix; prove against synthetic magic/truncation cases and all five supplied legacy `.gz` suffix/content mismatches. Do not decompress or route from extension alone.
- [ ] Freeze compressed inputs; verify trigger MD5/size, compute SHA-256, write manifest.
- [ ] Discover all parts/streams rather than hard-code a count; reject missing/duplicate parts.
- [ ] Implement date/version-aware schemas for Trim 68/82 and required Full variants.
- [ ] Resolve signed numeric encoding and Full Trigger/Limit/flag conditional rules before Full semantic parsing; do not promote the current unsigned fixture behavior into a cross-era canonical contract without evidence.
- [ ] Preserve raw bytes/padding/file/stream/index/jiffy and a reason for every quarantine.
- [x] Preserve and decompose raw jiffies exactly; test the NSE example and fractional remainder; prove that NSE's published epoch constant differs from the actual UTC offset by exactly 19,800 seconds.
- [ ] Resolve the jiffy epoch, then test independently known timestamps, day boundaries, timezone display, and ties before exposing UTC/session-date fields.
- [x] Implement lossless Trim Order/Trade jiffy profiling for min/max range, consecutive parsed ties, maximum equal-jiffy run, every coordinate-exact regression, quarantine accounting, and framing residue. Bind multi-stream runs to the exact paired-delivery member set, normalize caller input to deterministic member order, reject coverage mismatch before parsing, and report pairwise range overlap; the profiler selects no merge policy.
- [ ] Run the profiler across every stream of representative paired current deliveries and measure stream monotonicity, equal-jiffy groups, overlap, and trade-reference timing before choosing a merge policy.
- [x] Implement a deterministic order-only entry/modify/cancel reducer: active-state replacement/removal, duplicate entry, unknown modify/cancel, and regressing-jiffy anomalies are explicit and never mutate state. Same-jiffy events follow caller-supplied order; cross-stream ordering, provenance orchestration, and trade-driven quantity semantics remain separate gates.
- [x] Reconstruct deterministic active-order state and a pure price-level bid/ask projection by raw instrument/session; aggregate exact order count/quantity with checked arithmetic and classify top-of-book as empty, bid-only, ask-only, normal with exact spread, locked, or crossed with exact cross amount. Locked/crossed levels are evidence, never filtered or repaired.
- [x] Stream one caller-ordered Trim Order source through framing, parsing, lifecycle reduction, and final price projection; retain active entry/last-mutation coordinates plus exact raw bytes/coordinates for parser and lifecycle failures, stop with a borrowed framing tail, and reconcile every framed/parsed/applied count.
- [x] Expose every successful single-source lifecycle transition through an allocation-free observer carrying exact source coordinate, canonical event, raw 68-byte record, and before/after transition evidence; failures remain excluded and separately retained.
- [ ] Add fallible durable canonical transition serialization, cross-stream merge/replay, and checkpoint/final-state hashes after ordering is evidenced and the canonical identity decision is active.
- [x] Join each Trim trade's buy/sell order IDs to successfully entered orders by exact session/segment/raw symbol/series/order identity; retain entry coordinates and classify each side independently as matched, missing, or entry-side mismatch without claiming temporal validity.
- [ ] Establish temporal trade-reference timing through an evidenced cross-stream merge, then test no-decrement, direct-decrement, and mixed execution interpretations; source membership alone must not choose quantity semantics.
- [x] Preserve `PO`, `RM`, and `CA` as distinct typed components of provisional order, book, and trade-reference identity. Prove that equal numeric order IDs coexist across all three and that cross-session modify/cancel attempts fail closed without mutating another session.
- [ ] Validate from representative data whether any order lifecycle crosses session codes; then define session transitions, halts, special-session behavior, file/day boundaries, and daily reset. Current isolation is a conservative implementation invariant, not an exchange-causality claim.
- [ ] Reconcile counts, volume/value, residue, and independent summaries where lawful.
- [ ] Compare matched Trim/Full replay, lifecycle matches, state divergence, lost features, storage/runtime.
- [ ] Produce contextual anomaly dossiers and request vendor clarification for systematic ambiguity.
- [ ] Emit canonical columnar data, schema docs, manifests, and deterministic replay interface.
- [ ] Add a Python canonical-data reader/conformance suite that never reads raw NSE Order/Trade files.

**Expected modules/files (proposed, not created now):**

```text
crates/jft-nse-cm/src/{framing,layouts,manifest,parser,jiffy,events,lifecycle,book,reconcile,quarantine}.rs
crates/jft-nse-cm/tests/{file_names,record_framing,source_parse,layouts,jiffy,lifecycle,replay_determinism,trade_reconciliation}.rs
python/jft/{io,diagnostics}/
schemas/nse_cm/{canonical.schema.json,manifest.schema.json}
tests/fixtures/nse_cm/synthetic_nonlicensed/
```

Rust is authoritative for Phase 1 parsing and replay; Python is a downstream canonical-data consumer. Changing that boundary requires a superseding architecture decision and cross-language conformance evidence.

**Tests:** Golden bytes for every era/type; width/numeric/padding property/fuzz tests; transition matrix; corrupt trigger/partial/missing/duplicate stream; unknown order; overfill; negative depth; repeated ID; same-jiffy permutation sensitivity; auctions/special sessions; daily reset; deterministic outputs across worker counts.

**Metrics/artifacts:** Parsed/quarantined/duplicate counts; trigger rate; lifecycle anomaly and trade-match rates; same-jiffy ambiguity; crossed/locked duration; reconciliation difference; hash stability; runtime/memory/compression; Trim/Full divergence. Deliver schema, package, fixtures, anomaly taxonomy, daily quality report, Trim/Full recommendation, performance profile.

**Blockers:** Undocumented era, missing streams, systematic unknown orders, unresolved trade semantics, material tie sensitivity, or unacceptable Trim/Full divergence.

**Acceptance/exit:** Identical input/configuration yields identical canonical bytes/book hashes; every record is accounted for; invariants pass or have approved exceptions; reconciliation meets a predeclared evidence-backed tolerance; owner selects Trim or approves Full.

**Return/kill:** Return to Phase 0/vendor for unresolved semantics. Kill this alpha path if no viable qualified dataset supports trustworthy reconstruction.

**Must not build:** Alpha features beyond diagnostics, production labels/models/backtests, broker/live code.

### Phase 2 - Features, labels, and simple baselines

**Objective/rationale:** Test auditable microstructure relationships before execution/neural complexity.

**Prerequisites:** Accepted Phase 1 dataset and feature eligibility matrix.

**Work:** Implement Section 9 contracts; choose one instrument and at least one year if available; pre-register splits/metrics; build naive, linear/logistic, gradient-boosted, and small-MLP baselines; calibrate; ablate by feature/horizon/session/regime; audit leakage/trial count.

**Expected modules:** `python/jft/{features,labels,datasets,models/baselines,evaluation}/`, tests/configs.

**Tests/metrics/artifacts:** As-of/boundary/split/purge/embargo and label tests; log loss, calibration, MAE/rank, gross thresholded expectancy, confidence/stability slices; dataset card, dictionary, pre-registration, leaderboard, leakage/ablation reports.

**Acceptance/exit:** At least one horizon repeatedly improves on naive OOS baselines with calibrated predictions and plausible gross edge large enough for cost testing.

**Blockers/kill/return:** No stable signal, untradeable horizon, or leakage-sensitive result means stop or revise; never jump to deep learning.

**Must not build:** DeepLOB/transformers, ensembles, RL, live or broker code.

### Phase 3 - Execution-grade simulator and cost validation

**Objective/rationale:** Determine whether baseline signal is tradable; predictive accuracy is not alpha.

**Prerequisites:** Phase 2 signal and dated fee/latency/fill evidence.

**Work:** Implement Section 10; define decision/order schemas; version fees; estimate latency; queue/fill/impact cases; risk/accounting/outages/reconciliation; cost/latency/capacity/adversarial sensitivity.

**Expected modules:** `python/jft/sim/{market,orders,latency,fees,fills,impact,account,risk}.py`, scenarios and golden tests. A later Rust simulator core requires its own measured architecture decision and preserves the same canonical contracts.

**Tests/metrics/artifacts:** Cash/position conservation; no impossible fills; fee arithmetic; deterministic seeds; rejects/cancel races/partials/gaps; net expectancy, turnover, fills, shortfall, drawdown, tail loss, break-even cost/latency, capacity. Deliver simulator spec, evidence, scenarios, net-alpha report, limitations.

**Acceptance/exit:** Positive net OOS expectancy persists under conservative base assumptions and remains understandable under adverse cases without impossible queue priority.

**Blockers/kill/return:** Edge smaller than friction, too latency-sensitive, or optimistic-fill-only means stop, change horizon, or return to Phase 2.

**Must not build:** Neural models to rescue negative net baselines, live routing, capital deployment.

### Phase 4 - Neural representation learning

**Objective/rationale:** Test whether learned LOB representations add economic value over qualified baselines.

**Prerequisites:** Phase 3 net-positive baseline and compute approval.

**Work:** Freeze benchmark; compact CNN/TCN/DeepLOB-like encoder; multi-horizon direction/return/volatility/liquidity heads; cost-aware evaluation; calibration; latency/capacity; ablations and multiple seeds.

**Expected modules:** `python/jft/models/{encoders,heads,training}/`, model cards/configs/reproducibility tests.

**Tests/metrics/artifacts:** Seed variance, learning curves, calibration, net uplift under identical policy, inference p50/p95/p99, compute, stability; deliver reproducible candidate, cards, ablations, economic comparison.

**Acceptance/exit:** Stable OOS net uplift over best simpler model across folds/seeds; inference is comfortably below signal half-life; cost is acceptable.

**Blockers/kill/return:** Accuracy-only, unstable, expensive, leakage-sensitive, or period-concentrated uplift means retain the simpler model.

**Must not build:** Giant model, online learning, RL execution, automatic promotion.

### Phase 5 - Regime-aware ensemble

**Objective/rationale:** Test specialization across states without post-hoc regime overfit.

**Prerequisites:** At least two qualified behaviorally distinct candidates.

**Work:** Define observable ex-ante volatility/liquidity/trend/session regimes; compare hard routing, soft mixture, simple averaging; enforce samples; regularize gate; transition/abstention/fallback tests.

**Expected modules:** `python/jft/regimes/`, `python/jft/models/ensemble/`, gating/fallback tests/configs.

**Tests/metrics/artifacts:** Stability/turnover/occupancy, conditional calibration/net result, transition loss, entropy, worst-regime drawdown, single-model uplift; deliver regime definition, cards, fallback runbook, robustness report.

**Acceptance/exit:** Ensemble improves aggregate and worst-regime OOS net outcomes without future-known regime labels or unstable gating.

**Blockers/kill/return:** Small samples, unstable regimes/gate, or no uplift means keep one model.

**Must not build:** Post-hoc P&L regimes or autonomous model selection.

### Phase 6 - Rigorous walk-forward and final OOS validation

**Objective/rationale:** Challenge a frozen strategy/model/policy as a release candidate; iteration consumes information.

**Prerequisites:** Frozen candidate/simulator/cost assumptions/predeclared rules.

**Work:** Rolling/expanding walk-forward; purging/embargo/nested tuning; locked final OOS; subgroup slices; cost/latency/impact and missing-data shocks; confidence/bootstrap; trial audit; independent manifest reproduction.

**Expected modules:** `python/jft/validation/`, locked config, manifest, evidence bundle.

**Tests/metrics/artifacts:** Net expectancy/return, drawdown/tails, risk-adjusted metrics with assumptions, turnover/payoff/calibration, concentration/probability of loss, parameter/capacity/break-even surfaces; signed OOS report and go/hold/stop record.

**Acceptance/exit:** Predeclared thresholds pass untouched data; no narrow domination; adverse cases fit risk; independent rerun matches.

**Blockers/kill/return:** OOS failure, concentration, fragility, or trivial capacity returns to a named phase; the opened holdout is never called untouched again.

**Must not build:** Live trading or retroactive thresholds.

### Phase 7 - Live shadow system

**Objective/rationale:** Validate live data/state/predictions/hypothetical orders without capital.

**Prerequisites:** Phase 6 candidate and security/operations approval. Activating a real live feed also requires the owner's external authorization process, which is outside repository scope.

**Work:** Live adapter; online/offline parity; prediction/hypothetical-order audit; time sync; gap/backfill; shadow simulator; monitoring; outcome joins; captured-session replay; incidents/rollback.

**Expected modules:** `src/jft/live/{feed,state,inference,shadow,observability}/`, deployments/runbooks.

**Tests/metrics/artifacts:** Parity vectors; restart/recovery; gap/duplicate/out-of-order/stale injection; latency/uptime/state divergence/drift/shadow expectancy/simulator divergence; deliver shadow dataset, parity/latency/live-vs-offline reports, incidents.

**Acceptance/exit:** Predeclared duration/sample across regimes; replay parity; no unexplained material decay; latency within signal life; safe recovery.

**Blockers/kill/return:** Parity, latency, reliability, or shadow-alpha failure returns to Phase 1-6. Missing external authorization blocks activation, not repository implementation.

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

**Prerequisites:** Phase 8, ring-fenced capital, and explicit loss authority. Written live/profit-use and broker/compliance authorization are owner-controlled external activation requirements, not repository engineering deliverables.

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

These **DESIGN DECISION** envelopes come from prior discussion, not current quotes or commitments. Repository budgeting must validate data tariffs plus cloud, compute, storage, and broker costs; the owner separately handles tax, legal, and regulatory cost review.

| Gate | Planning envelope | Rule |
|---|---:|---|
| Phase 0-2 POC | Rs 1.5-5 lakh external; no trading capital | Buy only enough evidence to test signal |
| Through Phase 3/6 | Rs 3-8 lakh cumulative | Raise after reconstruction and baseline proof |
| Phase 7 shadow | Rs 4-10 lakh cumulative; paper only | Shadow is not live proof |
| Phase 9 initial live | Rs 6-15 lakh cumulative external plus Rs 1-5 lakh first experiment; Rs 5-20 lakh later needs a new gate | Capital is an experimental loss budget |
| Phase 10 farm | Indicative Rs 15-50 lakh/year infra/data and Rs 50 lakh-2 crore capital | Capacity justifies each increment |
| Colocation/institutional | Separate case, potentially crores | Only measured alpha half-life can require it |

Compute order: CPU baseline -> small GPU -> compact optimized neural model -> larger experiments only with demonstrated scaling. Spend first on qualified data and simulator fidelity, not permanent GPUs.

## 13. Dependencies, blockers, and kill criteria

Engineering dependencies: paired Orders/Trades and reference/corporate-action data; secure compute/storage; current fee/broker evidence; secure credentials/network; time diversity; owner approvals. Operational activation separately depends on the owner's external legal/compliance/accounting process.

Repository engineering blockers: unaffordable/inadequate data, non-deterministic reconstruction, signal decay faster than latency, edge below costs, insufficient capacity, unstable models, or risk-control failure. External compliance or regulatory constraints may block operational activation, but do not block offline repository engineering.

Stop the engineering project or alpha path if, after a predeclared remediation attempt: no viable dataset reconstructs; ambiguity changes features/labels materially; honest baselines lack OOS signal; signal fails costs; complexity adds no net value; locked OOS/shadow/live fails; loss/divergence exceeds limits; capacity cannot cover cost/risk; or safe operation exceeds the approved business case. Separately, do not operationally activate a use that the owner's external process has not authorized. Stopping is a successful result when it prevents unsafe capital deployment.

## 14. Immediate next actions

1. Consult and freeze the Phase 1 canonical schema, compatibility policy, Rust parser interfaces, error taxonomy, deterministic event keys, and byte-accounting rules; the Rust/Python ownership boundary itself is already frozen.
2. Expand the synthetic, non-licensed golden fixtures from the verified entry/trade slice to remaining side/activity/session combinations, timestamp edges, and malformed-input classes before lifecycle work.
3. Implement date/version-aware layout validation, exact raw-jiffy handling, validation-gated timestamp conversion, and quarantine behavior before book logic.
4. Use the qualified public legacy sample only for legacy-width parser and incomplete-replay tests; do not treat it as representative market evidence.
5. Add current paired CM Trim files/triggers when available and resolve streams, ties, modify/cancel quantities, and trade reductions before Phase 1 acceptance.
6. Keep legal/compliance as an external “needs to be done” marker; it must not block offline repository engineering and the repository must not claim clearance.
7. Do not begin Phase 2 features or models before the deterministic replay gate passes on representative current data.
