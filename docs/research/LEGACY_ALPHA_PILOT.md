# Legacy CM limited-data alpha pilot

Date: 2026-10-27

## Evidence status

- **CONFIRMED FROM SUPPLIED DATA:** The supplied public legacy CM files contain 300,000 fixed-width Order records and 300,000 fixed-width Trade records. Their SHA-256 digests are recorded below.
- **DESIGN DECISION:** Raw fixed-width decoding and book reconstruction remain in Rust. Python consumes only the CSV research boundary emitted by Rust.
- **RESEARCH HYPOTHESIS:** Top-of-book quantity imbalance or recent trade-price momentum may predict a future price direction.
- **UNRESOLVED / REQUIRES VALIDATION:** The files are one anonymized legacy sample day with placeholder filenames, no triggers, no current Trim streams, no executable fill evidence, and no independent day. They cannot establish current NSE alpha or profitability.

## Inputs and provenance

| Input | Records | SHA-256 |
|---|---:|---|
| `CASH_Orders_DDMMYYYY.txt` | 300,000 | `d91b3c7949db616a0fae05267006295613a1cff4be022b636f690ec0b0e2b108` |
| `CASH_Trades_DDMMYYYY.txt` | 300,000 | `d0567d230235847435b7ff71738d1a08f31e6814aba74855c6a406565de0512e` |

The instrument is selected by event count using only the first 60% of each source. Both bounded pilots selected raw symbol `bbbbbbbBEL`, series `EQ`. Raw `b` padding is deliberately preserved rather than normalized by guesswork.

## Pilot A: reconstructed top-of-book imbalance

The Rust exporter projects the documented legacy 87-byte Full Order fields `Original Quantity` and `Limit Price` into the existing deterministic lifecycle and book implementation. It then emits only normal or locked top-of-book states for downstream analysis.

Observed evidence:

| Metric | Result |
|---|---:|
| Selected Order events | 12,892 |
| Applied lifecycle events | 12,829 |
| Lifecycle anomalies | 63 |
| Normal book observations | 21 |
| Locked observations | 0 |
| Crossed observations | 12,786 |
| One-sided/empty observations | 22 |

**Outcome: REJECTED AS AN ALPHA DATASET.** Only 21 usable snapshots remain, while 99.7% of applied observations are crossed (`12,786 / 12,829`). The present order-only reconstruction is therefore not a credible basis for a book-imbalance alpha claim on this file. Likely explanations include missing trade-driven quantity depletion, legacy Full quantity semantics, or other absent ordering/session evidence; the pilot does not choose among them.

## Pilot B: trade-price momentum

Because it does not require book semantics, a second fixed hypothesis tests whether the sign of the previous trade-price return predicts the sign of the next return at 10-, 50-, and 200-trade horizons.

Protocol:

1. Rust decodes the documented legacy 100-byte Trade fields and emits 13,614 selected-instrument prints.
2. Python uses chronological 60% training, 20% validation, and 20% final test windows.
3. Observations within each horizon are non-overlapping.
4. The momentum sign is fixed before evaluation; no threshold or horizon is selected from the results.
5. The majority-direction baseline is learned only from the training window.
6. Gross signed return is mark-to-trade-price and excludes every trading cost.

| Horizon | Window | Samples | Nonzero labels | Hit rate | Majority baseline | IC | Gross signed return (bps) |
|---:|---|---:|---:|---:|---:|---:|---:|
| 10 trades | Validation | 271 | 123 | 43.90% | 51.22% | -0.1550 | -0.4790 |
| 10 trades | Test | 271 | 126 | 38.10% | 46.03% | -0.2538 | -0.6162 |
| 50 trades | Validation | 53 | 34 | 47.06% | 47.06% | -0.0021 | +0.3720 |
| 50 trades | Test | 53 | 34 | 41.18% | 50.00% | -0.1892 | -0.8294 |
| 200 trades | Validation | 12 | 9 | 66.67% | 55.56% | +0.2759 | +2.3835 |
| 200 trades | Test | 12 | 12 | 50.00% | 50.00% | -0.2309 | -3.6623 |

**Outcome: THE PREDECLARED MOMENTUM HYPOTHESIS FAILS.** In the final chronological window it underperforms the majority baseline at 10 and 50 trades, reaches only the baseline at 200 trades, has negative information coefficient at all three horizons, and has negative gross signed return before costs at all three horizons.

The negative test IC and momentum returns mechanically point toward short-horizon reversal, but that is a post-hoc observation, not an accepted signal. On trade prints it can be caused by bid/ask bounce, and any reversal strategy must cross or earn the spread. This single sample provides no cost, fill, or independent-day evidence with which to distinguish those explanations.

## Decision and next gate

This pilot is a useful negative result:

- the existing deterministic components run on the entire supplied legacy files;
- the order-only book is not semantically healthy enough for alpha work;
- the simplest trade-price momentum rule does not survive the final chronological window;
- no Phase 2 alpha claim, simulator work, model escalation, or capital decision is justified.

Do not mine additional rules from the same final window and call them out-of-sample. The next credible alpha test requires representative current paired Order/Trade data, validated book semantics, multiple trading days, and a predeclared signal/label/cost protocol.

## Reproduction boundary

The raw files remain outside Git. Reproduction uses:

- `cargo run --release --example legacy_alpha_snapshot_export` for the Order/book diagnostic;
- `cargo run --release --example legacy_trade_alpha_export` for canonical trade-print export;
- `python -m jft.research.trade_alpha_pilot` for the chronological momentum evaluation.

Generated CSV files are ignored under `tmp/`. The exact small JSON result is tracked in [`legacy_trade_alpha_report.json`](legacy_trade_alpha_report.json).
